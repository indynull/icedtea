//! Short first window. Same shape as the README example.

use icedtea::a11y::A11y;
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
        Hello::theme
    )
}

struct Hello {
    n: i32,
}

#[derive(Clone)]
enum Message {
    Inc,
}

impl Hello {
    fn new() -> (Self, Task<Message>) {
        (Self { n: 0 }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        if matches!(message, Message::Inc) {
            self.n += 1;
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let tok = theme::named("dark").tokens;
        widget::themed_button(
            format!("Count {}", self.n),
            Some(Message::Inc),
            tok,
            Variant::Primary,
            A11y::button("inc"),
        )
    }

    fn theme(&self) -> icedtea::iced::Theme {
        theme::iced_theme("dark", theme::named("dark").tokens)
    }
}
