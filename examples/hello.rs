//! First window: Save writes the buffer into the status line.

use icedtea::a11y::{A11y, Role};
use icedtea::action::{Action, ActionTable};
use icedtea::i18n::Direction;
use icedtea::iced::widget::text_editor::Content;
use icedtea::key::{self, KeyContext};
use icedtea::pattern;
use icedtea::shortcut::Shortcut;
use icedtea::theme;
use icedtea::widget;
use icedtea::{Boot, Element, Task};

fn main() -> icedtea::iced::Result {
    icedtea::run!(
        Boot::new("Notes", "dev.example.hello"),
        Hello::new,
        Hello::update,
        Hello::view,
        Hello::theme,
        Hello::subscription
    )
}

struct Hello {
    doc: Content,
    status: String,
    table: ActionTable<Message>,
}

#[derive(Clone)]
enum Message {
    Edit(icedtea::iced::widget::text_editor::Action),
    Save,
    Key(icedtea::iced::keyboard::Event),
}

impl Hello {
    fn new() -> (Self, Task<Message>) {
        let mut table = ActionTable::new();
        table.insert(
            Action::new("file.save", "Save", Message::Save)
                .with_shortcut(Shortcut::parse("ctrl+s").unwrap()),
        );
        (
            Self {
                doc: Content::with_text("Notes"),
                status: "Ready".into(),
                table,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => self.doc.perform(action),
            Message::Save => {
                self.status = format!("Saved ({} chars)", self.doc.text().chars().count());
            }
            Message::Key(ev) => {
                if let Some(next) = key::handle(KeyContext::default(), &self.table, &ev) {
                    return self.update(next);
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let tok = theme::named("dark").tokens;
        icedtea::iced::widget::column![
            pattern::toolbar(self.table.iter(), tok, Direction::Ltr),
            widget::textarea(
                &self.doc,
                Message::Edit,
                tok,
                icedtea::layout::FILL,
                A11y::new("notes", Role::TextBox),
            ),
            pattern::status_bar(&self.status, &self.table, tok, Direction::Ltr),
        ]
        .spacing(8)
        .padding(12)
        .width(icedtea::layout::FILL)
        .height(icedtea::layout::FILL)
        .into()
    }

    fn theme(&self) -> icedtea::iced::Theme {
        theme::iced_theme("dark", theme::named("dark").tokens)
    }

    fn subscription(&self) -> icedtea::iced::Subscription<Message> {
        key::listen().map(Message::Key)
    }
}
