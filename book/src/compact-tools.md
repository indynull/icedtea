# Compact tools

A tool-sized window sets size on `Boot`. Tiles use
`themed_button_sized` and `layout::pad`. The large reading uses
`widget::display_reading`. Keys use `key::press` so Shift+8 is `*`.

```rust,ignore
use icedtea::density::Density;
use icedtea::iced::Length;
use icedtea::key::{self, Press};
use icedtea::layout;
use icedtea::widget;
use icedtea::{Boot, Element};

fn main() -> icedtea::iced::Result {
    icedtea::run!(
        Boot::new("Tool", "dev.example.tool")
            .theme("light")
            .size(380.0, 560.0)
            .min_size(360.0, 480.0),
        App::new,
        App::update,
        App::view,
        App::theme,
        App::subscription
    )
}

// In update, from key::listen:
// match key::press(&event) { Some(Press::Character(s)) => ..., Some(Press::Enter) => ... }
// Ctrl+S still goes through key::handle and Action shortcuts.

// In view:
// widget::display_line(expr, tok, a11y)
// widget::display_reading(value, tok, a11y)
// layout::pad(tiles, 4, Density::default().space)
```

The README pad is that window: a four-function keypad on
`layout::pad`, `display_reading` for the value, `key::press` for
digits and Shift+8 as `*`. The application owns the arithmetic.

The caption on a tile can be a glyph; the accessible name stays
readable (`⌫` on the face, name `Backspace`).
