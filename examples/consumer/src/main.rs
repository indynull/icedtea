//! Third-party window: theme, actions, list/detail, toasts, live theme switch.

use icedtea::a11y::{A11y, Role};
use icedtea::action::{Action, ActionTable};
use icedtea::collection::{ListModel, Selection, VecList};
use icedtea::i18n::Direction;
use icedtea::iced::{Length, Theme};
use icedtea::layout::Breakpoint;
use icedtea::pattern;
use icedtea::shortcut::Shortcut;
use icedtea::theme::{self, Tokens};
use icedtea::toast::ToastQueue;
use icedtea::variant::Variant;
use icedtea::widget;
use icedtea::{Boot, Element, Task};

fn main() -> icedtea::iced::Result {
    prove_public_api();
    println!("consumer-ok selection mix and breakpoint asserted");
    icedtea::run!(
        Boot::new("icedtea consumer", "dev.icedtea.consumer"),
        App::new,
        App::update,
        App::view,
        App::theme
    )
}

fn prove_public_api() {
    let tokens = theme::named("dark").tokens;
    let mixed = theme::mix(tokens.primary, tokens.canvas, 0.28);
    assert_eq!(mixed, tokens.selection);
    assert_eq!(theme::named("high-contrast").name, "high-contrast");
    assert!(!theme::named("solarized-light").dark);
    assert!(theme::named("catppuccin-mocha").dark);
    assert_eq!(
        theme::code_highlight("nord"),
        icedtea::iced::highlighter::Theme::Base16Ocean
    );
    let mut save = Action::new("file.save", "Save", 7u32);
    assert_eq!(save.invoke(), Some(7));
    save.enabled = false;
    assert_eq!(save.invoke(), None);
    assert_eq!(Breakpoint::from_width(500.0), Breakpoint::Compact);
    assert!(Breakpoint::from_width(1200.0).sidebar_beside());
}

#[derive(Debug, Clone)]
enum Message {
    Theme(String),
    Select(usize),
    Toast,
    Dismiss(u64),
    Save,
    Scroll,
}

struct App {
    tokens: Tokens,
    theme: String,
    list: VecList,
    sel: Selection,
    toasts: ToastQueue,
    actions: ActionTable<Message>,
    catalog: icedtea::i18n::Catalog,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let mut actions = ActionTable::new();
        actions.insert(Action::new("file.new", "New", Message::Toast));
        actions.insert(Action::new("file.open", "Open…", Message::Toast));
        actions.insert(
            Action::new("file.save", "Save", Message::Save)
                .with_shortcut(Shortcut::parse("ctrl+s").unwrap()),
        );
        actions.insert(Action::new("edit.undo", "Undo", Message::Toast));
        actions.insert(Action::new(
            "view.palette",
            "Command palette",
            Message::Toast,
        ));
        actions.insert(Action::new("help.about", "About", Message::Toast));
        (
            Self {
                tokens: theme::named("dark").tokens,
                theme: "dark".into(),
                list: VecList {
                    items: vec!["Inbox".into(), "Drafts".into(), "Sent".into()],
                },
                sel: Selection::Single(0),
                toasts: ToastQueue::new(),
                actions,
                catalog: icedtea::i18n::Catalog::builtin(),
            },
            Task::none(),
        )
    }

    fn theme(&self) -> Theme {
        theme::iced_theme(&self.theme, self.tokens)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Theme(name) => {
                self.theme = name.clone();
                self.tokens = theme::named(&name).tokens;
            }
            Message::Select(i) => self.sel.select_single(i),
            Message::Toast | Message::Save => {
                self.toasts.push_info("Saved");
            }
            Message::Dismiss(id) => self.toasts.dismiss(id),
            Message::Scroll => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let tok = self.tokens;
        let dir = Direction::Ltr;
        let themes = widget::themed_pick_list(
            icedtea::theme::builtin_names()
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>(),
            Some(self.theme.clone()),
            Message::Theme,
            tok,
            A11y::new(&self.theme, Role::ComboBox),
        );
        let detail = match self.sel.primary() {
            Some(i) => widget::label(
                format!("Detail: {}", self.list.label(i)),
                tok,
                A11y::new("detail", Role::Header),
            ),
            None => widget::meta("Nothing selected", tok, A11y::new("empty", Role::Status)),
        };
        let mut toasts = icedtea::iced::widget::column![].spacing(4);
        for t in self.toasts.iter() {
            toasts = toasts.push(widget::toast_view(
                t,
                Message::Dismiss(t.id),
                tok,
                A11y::new(&t.text, Role::Status),
            ));
        }
        pattern::main_window(
            pattern::menu_bar(&self.actions, tok, dir, &self.catalog),
            pattern::toolbar(self.actions.iter(), tok, dir),
            icedtea::iced::widget::column![
                themes,
                pattern::list_detail(
                    widget::list_view(
                        &self.list,
                        &self.sel,
                        Message::Select,
                        tok,
                        0.0,
                        36.0,
                        160.0,
                        |_| Message::Scroll,
                        A11y::new("mail", Role::List),
                    ),
                    icedtea::iced::widget::column![
                        detail,
                        widget::themed_button(
                            "Toast",
                            Some(Message::Toast),
                            tok,
                            Variant::Primary,
                            A11y::button("Toast"),
                        ),
                        toasts,
                    ]
                    .spacing(12)
                    .into(),
                    tok,
                ),
            ]
            .spacing(12)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
            pattern::status_bar("consumer", &self.actions, tok, dir),
            tok,
        )
    }
}
