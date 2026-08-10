# First window

`icedtea::run!` starts the window. `Boot` sets title and theme:

```rust,ignore
use icedtea::widget;
use icedtea::{Boot, Element, Task};

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
        match message {
            Message::Inc => self.n += 1,
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        widget::themed_button(
            format!("Count {}", self.n),
            Some(Message::Inc),
            icedtea::theme::named("dark").tokens,
            icedtea::variant::Variant::Primary,
            icedtea::a11y::A11y::button("inc"),
        )
    }

    fn theme(&self) -> icedtea::iced::Theme {
        icedtea::theme::iced_theme("dark", icedtea::theme::named("dark").tokens)
    }
}

fn main() -> icedtea::iced::Result {
    icedtea::run!(
        Boot::new("Hello", "dev.example.hello"),
        Hello::new,
        Hello::update,
        Hello::view,
        Hello::theme
    )
}
```

`Boot` loads tokens, locale, and window settings. Text uses the
platform sans; code uses the platform mono. Load a named family on
the iced application if you want a specific face.

A compact tool sets size on `Boot` (`.size(380.0, 560.0).min_size(...)`)
instead of calling iced window resize. See [Compact tools](compact-tools.md).

The README pad is a compact tool (sized `Boot`, display reading,
density tiles, typed keys). `icedtea-gallery` is the living catalog:
markdown shows a full document; the code page picks a language and
highlights it. `bootstrap(&boot)` is the same path without opening a
window — use it in tests.
