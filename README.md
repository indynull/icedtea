# icedtea

[![Documentation](https://img.shields.io/docsrs/icedtea/latest)][documentation]
[![Crates.io](https://img.shields.io/crates/v/icedtea.svg)][crates-io]
[![License](https://img.shields.io/crates/l/icedtea)][license]
[![check](https://github.com/indynull/icedtea/actions/workflows/ci.yml/badge.svg)][actions]

Reusable widgets and chrome for [iced](https://iced.rs/) desktop
applications.

`icedtea::run!` boots theme and starts your window. Controls
return iced `Element`s and emit your messages. Tokens, layouts, and
chrome are Rust values.

You may be looking for:

- [The guide](https://indynull.github.io/icedtea/)
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

## Example

A compact pad: sized `Boot`, display reading, density tiles, typed
keys. The application owns the arithmetic.

```rust,ignore
use icedtea::a11y::{A11y, Role};
use icedtea::density::Density;
use icedtea::iced::keyboard::Event as KeyEvent;
use icedtea::iced::{Length, Subscription, Theme};
use icedtea::key::{self, Press};
use icedtea::layout;
use icedtea::theme::{self, Tokens};
use icedtea::variant::Variant;
use icedtea::widget;
use icedtea::{Boot, Element, Task};

fn main() -> icedtea::iced::Result {
    icedtea::run!(
        Boot::new("Pad", "dev.example.pad")
            .theme("light")
            .size(320.0, 420.0)
            .min_size(300.0, 400.0),
        App::new,
        App::update,
        App::view,
        App::theme,
        App::subscription
    )
}

#[derive(Clone, Copy)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone)]
enum Message {
    Digit(u8),
    Op(Op),
    Eq,
    Clear,
    Key(KeyEvent),
}

struct App {
    tokens: Tokens,
    shown: String,
    acc: Option<f64>,
    op: Option<Op>,
    typed: bool,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                tokens: theme::named("light").tokens,
                shown: "0".into(),
                acc: None,
                op: None,
                typed: false,
            },
            Task::none(),
        )
    }

    fn theme(&self) -> Theme {
        theme::iced_theme("light", self.tokens)
    }

    fn subscription(&self) -> Subscription<Message> {
        key::listen().map(Message::Key)
    }

    fn value(&self) -> f64 {
        self.shown.parse().unwrap_or(0.0)
    }

    fn show(&mut self, n: f64) {
        self.shown = if n.fract() == 0.0 {
            format!("{n:.0}")
        } else {
            n.to_string()
        };
    }

    fn apply(&mut self) {
        let rhs = self.value();
        let Some((acc, op)) = self.acc.zip(self.op) else {
            self.acc = Some(rhs);
            self.typed = false;
            return;
        };
        let n = match op {
            Op::Add => acc + rhs,
            Op::Sub => acc - rhs,
            Op::Mul => acc * rhs,
            Op::Div if rhs != 0.0 => acc / rhs,
            Op::Div => {
                self.shown = "Error".into();
                self.acc = None;
                self.op = None;
                self.typed = false;
                return;
            }
        };
        self.show(n);
        self.acc = Some(n);
        self.typed = false;
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Digit(d) => {
                if !self.typed || self.shown == "0" || self.shown == "Error" {
                    self.shown = d.to_string();
                } else {
                    self.shown.push(char::from(b'0' + d));
                }
                self.typed = true;
            }
            Message::Op(op) => {
                if self.typed {
                    self.apply();
                }
                self.op = Some(op);
                self.typed = false;
            }
            Message::Eq => {
                self.apply();
                self.op = None;
            }
            Message::Clear => {
                *self = App::new().0;
            }
            Message::Key(ev) => {
                if let Some(msg) = match key::press(&ev) {
                    Some(Press::Character(s)) => match s.as_str() {
                        "+" => Some(Message::Op(Op::Add)),
                        "-" => Some(Message::Op(Op::Sub)),
                        "*" => Some(Message::Op(Op::Mul)),
                        "/" => Some(Message::Op(Op::Div)),
                        d => d.parse::<u8>().ok().filter(|n| *n <= 9).map(Message::Digit),
                    },
                    Some(Press::Enter) => Some(Message::Eq),
                    Some(Press::Escape) => Some(Message::Clear),
                    _ => None,
                } {
                    return self.update(msg);
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let tok = self.tokens;
        let density = Density::default();
        let h = Length::Fixed(density.tile() as f32);
        let tile = |title: &'static str, msg: Message, variant: Variant| {
            widget::themed_button_sized(
                title,
                Some(msg),
                tok,
                variant,
                Length::Fill,
                h,
                A11y::button(title),
            )
        };
        let q = Variant::Quiet;
        let chip = Variant::Chip;
        icedtea::iced::widget::column![
            widget::display_reading(
                self.shown.clone(),
                tok,
                A11y::new(self.shown.clone(), Role::Status),
            ),
            layout::pad(
                vec![
                    tile("C", Message::Clear, q),
                    tile("÷", Message::Op(Op::Div), chip),
                    tile("×", Message::Op(Op::Mul), chip),
                    tile("−", Message::Op(Op::Sub), chip),
                    tile("7", Message::Digit(7), q),
                    tile("8", Message::Digit(8), q),
                    tile("9", Message::Digit(9), q),
                    tile("+", Message::Op(Op::Add), chip),
                    tile("4", Message::Digit(4), q),
                    tile("5", Message::Digit(5), q),
                    tile("6", Message::Digit(6), q),
                    tile("=", Message::Eq, Variant::Primary),
                    tile("1", Message::Digit(1), q),
                    tile("2", Message::Digit(2), q),
                    tile("3", Message::Digit(3), q),
                    tile("0", Message::Digit(0), q),
                ],
                4,
                density.space,
            ),
        ]
        .spacing(density.space)
        .padding(12)
        .into()
    }
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
[license]: LICENSE
[actions]: https://github.com/indynull/icedtea/actions
