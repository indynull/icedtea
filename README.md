# icedtea

[![check](https://github.com/indynull/icedtea/actions/workflows/ci.yml/badge.svg)](https://github.com/indynull/icedtea/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/icedtea.svg)](https://crates.io/crates/icedtea)
[![docs.rs](https://docs.rs/icedtea/badge.svg)](https://docs.rs/icedtea)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

Reusable widgets and chrome for [iced](https://iced.rs/) desktop
applications.

`icedtea::run!` boots fonts and theme and starts your window. Controls
return iced `Element`s and emit your messages. Tokens, layouts, and
chrome are Rust values.

The name is iced plus tea.

## Features

- Semantic color tokens, named colorways, and live theme switch
- One `Action` for menus, toolbars, shortcuts, and the command palette
- Layout recipes: dock, split, clamp, form, overlay, breakpoints
- Application, dialog, and overlay window kinds
- Widget catalog with a running gallery

## Example

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
        if matches!(message, Message::Inc) {
            self.n += 1;
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

## Install

Rust 1.89 or newer.

```toml
[dependencies]
icedtea = { git = "https://github.com/indynull/icedtea" }
```

Linux needs `libxkbcommon-dev` and `libwayland-dev`. macOS needs the
Xcode command-line tools. Windows needs the MSVC build tools.

```bash
cargo run -p icedtea-gallery
```

Guide: [`book/`](book/src/SUMMARY.md) · API: [docs.rs/icedtea](https://docs.rs/icedtea)

## License

MIT OR Apache-2.0.
