# icedtea

[![Documentation](https://img.shields.io/docsrs/icedtea/latest)][documentation]
[![Crates.io](https://img.shields.io/crates/v/icedtea.svg)][crates-io]
[![License](https://img.shields.io/crates/l/icedtea)][license]
[![check](https://github.com/indynull/icedtea/actions/workflows/ci.yml/badge.svg)][actions]

Native desktop widgets and chrome for [iced](https://iced.rs/).

`icedtea::run!` starts a themed window. Constructors return iced
`Element`s and emit your messages. Color, layout, and chrome are Rust
values.

![The icedtea gallery](https://github.com/indynull/icedtea/raw/master/assets/gallery.png)

The [gallery](https://github.com/indynull/icedtea/tree/master/icedtea-gallery)
is a running catalog: every [`catalog::ENTRIES`][catalog] id has a
page. Run it from a checkout:

```bash
cargo run -p icedtea-gallery
```

## First window

```toml
[dependencies]
iced = "0.14"
icedtea = { git = "https://github.com/indynull/icedtea" }
```

`0.1` on crates.io is the publish check. This tree is `0.2`. Use git
until the 0.2 tag. After that tag: `icedtea = "0.2"`.

```rust,ignore
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
```

Same program: `cargo run --example hello` in this repository.

Tokens mix in ordinary Rust:

```rust
let tokens = icedtea::theme::named("dark").tokens;
let mixed = icedtea::theme::mix(tokens.primary, tokens.canvas, 0.28);
assert_eq!(mixed, tokens.selection);
```

## Where to look

- [Guide](https://indynull.github.io/icedtea/) — first window, actions,
  layout, theming, overlays
- [API docs][documentation] — `widget`, `theme`, `action`, `layout`,
  `window`, `pattern`
- [Gallery](https://github.com/indynull/icedtea/tree/master/icedtea-gallery)
  — every public control and pattern
- [Release notes](CHANGELOG.md)

One `Action` feeds menus, toolbars, shortcuts, and the command
palette. `theme::named` has forty colorways. Linux needs
`libxkbcommon-dev` and `libwayland-dev`. macOS needs the Xcode
command-line tools. Windows needs the MSVC build tools.

| iced | icedtea |
| --- | --- |
| 0.14 | 0.2 |

[documentation]: https://docs.rs/icedtea
[crates-io]: https://crates.io/crates/icedtea
[license]: https://github.com/indynull/icedtea/blob/master/LICENSE
[actions]: https://github.com/indynull/icedtea/actions
[catalog]: https://docs.rs/icedtea/latest/icedtea/catalog/static.ENTRIES.html
