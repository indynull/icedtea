//! Personal task list. Rows live in a SQLite file next to the process.

use icedtea::a11y::{A11y, Role};
use icedtea::action::{Action, ActionTable};
use icedtea::i18n::Direction;
use icedtea::key::{self, KeyContext};
use icedtea::pattern;
use icedtea::shortcut::Shortcut;
use icedtea::theme;
use icedtea::widget;
use icedtea::{Boot, Element, Task};
use rusqlite::Connection;

fn main() -> icedtea::iced::Result {
    icedtea::run!(
        Boot::new("Tasks", "dev.example.tasks"),
        Tasks::new,
        Tasks::update,
        Tasks::view,
        Tasks::theme,
    )
}

struct TaskRow {
    id: i64,
    title: String,
    done: bool,
}

struct Tasks {
    db: Connection,
    rows: Vec<TaskRow>,
    draft: String,
    status: String,
    table: ActionTable<Message>,
}

#[derive(Clone)]
enum Message {
    Draft(String),
    Add,
    Toggle(i64, bool),
    Quit,
    Key(icedtea::iced::keyboard::Event),
}

impl From<icedtea::iced::keyboard::Event> for Message {
    fn from(ev: icedtea::iced::keyboard::Event) -> Self {
        Self::Key(ev)
    }
}

const DB_FILE: &str = "tasks.db";

impl Tasks {
    fn new() -> (Self, Task<Message>) {
        let mut table = ActionTable::new();
        table.insert(
            Action::new("task.add", "Add", Message::Add)
                .with_shortcut(Shortcut::parse("ctrl+n").unwrap()),
        );
        table.seed_quit(Message::Quit);
        let db = open_db(DB_FILE);
        let rows = load_rows(&db);
        let n = rows.len();
        (
            Self {
                db,
                rows,
                draft: String::new(),
                status: format!("{n} tasks · {DB_FILE}"),
                table,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Draft(s) => self.draft = s,
            Message::Add => {
                let title = self.draft.trim().to_string();
                if title.is_empty() {
                    self.status = "Type a task first".into();
                } else if let Err(e) = insert_row(&self.db, &title) {
                    self.status = format!("Could not save: {e}");
                } else {
                    self.draft.clear();
                    self.reload("Added");
                }
            }
            Message::Toggle(id, done) => {
                if let Err(e) = set_done(&self.db, id, done) {
                    self.status = format!("Could not update: {e}");
                } else {
                    self.reload(if done { "Done" } else { "Not done" });
                }
            }
            Message::Quit => return icedtea::iced::exit(),
            Message::Key(ev) => {
                if let Some(next) = key::handle(KeyContext::default(), &self.table, &ev) {
                    return self.update(next);
                }
            }
        }
        Task::none()
    }

    fn reload(&mut self, note: &str) {
        match try_load_rows(&self.db) {
            Ok(rows) => {
                let n = rows.len();
                self.rows = rows;
                self.status = format!("{note} · {n} tasks · {DB_FILE}");
            }
            Err(e) => self.status = format!("Could not read: {e}"),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let tok = theme::named("dark").tokens;
        let mut list = icedtea::iced::widget::column![].spacing(8);
        if self.rows.is_empty() {
            list = list.push(widget::meta(
                "No tasks yet. Type one and press Add.",
                tok,
                A11y::new("empty", Role::Status),
            ));
        } else {
            for row in &self.rows {
                let id = row.id;
                list = list.push(widget::checkbox(
                    row.title.clone(),
                    row.done,
                    move |done| Message::Toggle(id, done),
                    tok,
                    A11y::new(row.title.clone(), Role::Checkbox).with_checked(row.done),
                ));
            }
        }
        icedtea::focus::cycle(
            icedtea::iced::widget::column![
                pattern::toolbar(self.table.iter(), tok, Direction::Ltr),
                widget::text_input(
                    "New task",
                    &self.draft,
                    Message::Draft,
                    Some(Message::Add),
                    widget::FieldOpts::NONE,
                    tok,
                    A11y::new("new-task", Role::TextBox),
                    None,
                ),
                widget::scroll(
                    list.into(),
                    tok,
                    A11y::new("tasks", Role::List),
                    false,
                    None,
                    None::<fn(_) -> Message>,
                ),
                pattern::status_bar(&self.status, None, None, &self.table, tok, Direction::Ltr),
            ]
            .spacing(8)
            .padding(12)
            .width(icedtea::layout::FILL)
            .height(icedtea::layout::FILL)
            .into(),
            None,
        )
    }

    fn theme(&self) -> icedtea::iced::Theme {
        theme::iced_theme("dark", theme::named("dark").tokens)
    }
}

fn open_db(path: &str) -> Connection {
    let db = Connection::open(path).expect("open tasks.db");
    db.execute(
        "CREATE TABLE IF NOT EXISTS task (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            done INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )
    .expect("create task table");
    db
}

fn load_rows(db: &Connection) -> Vec<TaskRow> {
    try_load_rows(db).unwrap_or_default()
}

fn try_load_rows(db: &Connection) -> rusqlite::Result<Vec<TaskRow>> {
    let mut stmt = db.prepare("SELECT id, title, done FROM task ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok(TaskRow {
            id: row.get(0)?,
            title: row.get(1)?,
            done: row.get::<_, i64>(2)? != 0,
        })
    })?;
    rows.collect()
}

fn insert_row(db: &Connection, title: &str) -> rusqlite::Result<()> {
    db.execute("INSERT INTO task (title, done) VALUES (?1, 0)", [title])?;
    Ok(())
}

fn set_done(db: &Connection, id: i64, done: bool) -> rusqlite::Result<()> {
    db.execute(
        "UPDATE task SET done = ?1 WHERE id = ?2",
        rusqlite::params![i64::from(done), id],
    )?;
    Ok(())
}
