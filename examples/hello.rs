//! First window: one Action feeds the toolbar and increments the count.

use icedtea::a11y::A11y;
use icedtea::action::{Action, ActionTable};
use icedtea::i18n::Direction;
use icedtea::key::{self, KeyContext};
use icedtea::pattern;
use icedtea::shortcut::Shortcut;
use icedtea::theme;
use icedtea::variant::Variant;
use icedtea::widget;
use icedtea::{Boot, Element, Task};

fn main() -> icedtea::iced::Result {
    icedtea::run!(
        Boot::new("Hello", "dev.example.hello"),
        Hello::new,
        Hello::update,
        Hello::view,
        Hello::theme,
        Hello::subscription
    )
}

struct Hello {
    n: i32,
    table: ActionTable<Message>,
}

#[derive(Clone)]
enum Message {
    Inc,
    Key(icedtea::iced::keyboard::Event),
}

impl Hello {
    fn new() -> (Self, Task<Message>) {
        let mut table = ActionTable::new();
        table.insert(
            Action::new("count.inc", "Count", Message::Inc)
                .with_shortcut(Shortcut::parse("ctrl+i").unwrap()),
        );
        (Self { n: 0, table }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Inc => self.n += 1,
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
            widget::themed_button(
                format!("Count {}", self.n),
                Some(Message::Inc),
                tok,
                Variant::Primary,
                A11y::button("inc"),
            ),
        ]
        .spacing(12)
        .padding(16)
        .into()
    }

    fn theme(&self) -> icedtea::iced::Theme {
        theme::iced_theme("dark", theme::named("dark").tokens)
    }

    fn subscription(&self) -> icedtea::iced::Subscription<Message> {
        key::listen().map(Message::Key)
    }
}
