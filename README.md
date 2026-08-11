# icedtea

Native desktop widgets and chrome for [iced](https://iced.rs/).

`icedtea::run!` starts a themed window. One `Action` feeds the toolbar,
menus, and shortcuts. Constructors return iced `Element`s and emit
your messages.

![A themed icedtea window](https://github.com/indynull/icedtea/raw/master/assets/gallery.gif)

## First window

```toml
[dependencies]
iced = "0.14"
icedtea = "0.2"
```

Same program: [`examples/hello.rs`](examples/hello.rs). `ctrl+i` or
the toolbar increments the count.

```rust,ignore
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
```

`cargo run --example hello` from a checkout.

## Where to look

- [Guide](https://indynull.github.io/icedtea/) — first window, actions,
  layout, theming, and a reference for every control
- [Crate docs](https://docs.rs/icedtea) — `widget`, `theme`, `action`,
  `layout`, `window`, `pattern`
- [crates.io](https://crates.io/crates/icedtea) ·
  [source](https://github.com/indynull/icedtea)

Linux needs `libxkbcommon-dev` and `libwayland-dev`. macOS needs the
Xcode command-line tools. Windows needs the MSVC build tools.

| iced | icedtea |
| --- | --- |
| 0.14 | 0.2 |
