# icedtea

[![Documentation](https://docs.rs/icedtea/badge.svg)][documentation]
[![Crates.io](https://img.shields.io/crates/v/icedtea.svg)][crates-io]
[![License](https://img.shields.io/crates/l/icedtea.svg)][license]
[![check](https://github.com/indynull/icedtea/actions/workflows/ci.yml/badge.svg)][actions]

Reusable widgets and chrome for [iced](https://iced.rs/) desktop
applications.

`icedtea::run!` boots fonts and theme and starts your window. Controls
return iced `Element`s and emit your messages. Tokens, layouts, and
chrome are Rust values.

You may be looking for:

- [The guide](book/src/SUMMARY.md)
- [API documentation][documentation]
- [The gallery](icedtea-gallery/)
- [Release notes](CHANGELOG.md)

## Usage

```toml
[dependencies]
iced = "0.14"
icedtea = { git = "https://github.com/indynull/icedtea" }
```

| iced | icedtea |
| --- | --- |
| 0.14 | 0.1 |

Linux needs `libxkbcommon-dev` and `libwayland-dev`. macOS needs the
Xcode command-line tools. Windows needs the MSVC build tools.

## Overview

```rust,ignore
use icedtea::widget;
use icedtea::{Boot, Element, Task};

struct Counter {
    value: i32,
}

#[derive(Clone)]
enum Message {
    Increment,
}

impl Counter {
    fn new() -> (Self, Task<Message>) {
        (Self { value: 0 }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => self.value += 1,
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        widget::themed_button(
            format!("Count {}", self.value),
            Some(Message::Increment),
            icedtea::theme::named("dark").tokens,
            icedtea::variant::Variant::Primary,
            icedtea::a11y::A11y::button("increment"),
        )
    }

    fn theme(&self) -> icedtea::iced::Theme {
        icedtea::theme::iced_theme("dark", icedtea::theme::named("dark").tokens)
    }
}

fn main() -> icedtea::iced::Result {
    icedtea::run!(
        Boot::new("Counter", "dev.example.counter"),
        Counter::new,
        Counter::update,
        Counter::view,
        Counter::theme
    )
}
```

## Features

- Semantic color tokens, named colorways, and live theme switch
- One `Action` for menus, toolbars, shortcuts, and the command palette
- Layout recipes: dock, split, clamp, form, overlay, breakpoints
- Application, dialog, and overlay window kinds
- Widget catalog with a running gallery

```bash
cargo run -p icedtea-gallery
```

[documentation]: https://docs.rs/icedtea
[crates-io]: https://crates.io/crates/icedtea
[license]: LICENSE-MIT
[actions]: https://github.com/indynull/icedtea/actions

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in icedtea by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
