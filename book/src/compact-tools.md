# Compact tools

A tool-sized window sets size on `Boot`. Density, type scale, corners,
and elevation are the same `Boot` / `Tokens` fields as a full window.
Tiles use `button` with `ButtonOpts` and `layout::pad`. Large values use
`Tokens::display()` (M3 Display Small times `font_scale`) with platform
bold. Keys use `key::press` so Shift+8 is `*`.

```rust,ignore
use icedtea::density::Density;
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

// In view: large reading with typo::DISPLAY, keypad with layout::pad
// layout::pad(tiles, 4, Density::default().space)
```

A four-function keypad on `layout::pad`, a large reading for the value,
and `key::press` for digits and Shift+8 as `*` is the compact tool
shape. The application owns the arithmetic. The first window is
[`examples/hello.rs`](https://github.com/indynull/icedtea/blob/main/examples/hello.rs).

The caption on a tile can be a glyph; the accessible name stays
readable (`⌫` on the face, name `Backspace`).

## Field focus

`text_input` takes an optional iced `Id`. After the window
shows, focus that field with `iced::widget::operation::focus`.

## Chrome compose

There is no dedicated shell constructor. Compose search,
`list_detail`, `Tabs { closable: false }`, and a footer
(`status_bar`). `style::shell` uses M3 Component radius (desktop None)
for a flush rectangular card.

```rust
// Pinned tabs
let tabs = icedtea::collection::Tabs::new(["Read", "Write"]);
assert!(!tabs.closable);

// Focus after show
let _ = iced::widget::Id::new("query");
```

## Card plus chips

Compose `group_box` with chips for tags and meta. Exclusive filters
are `Tabs` or radio.

## Secret field

`secret_field` is a settings row: `password_input` (mask or reveal)
plus a copy `Action`. The application owns the copy message
(`icedtea::copy_text`).

- [`layout::pad`](https://docs.rs/icedtea/latest/icedtea/layout/fn.pad.html)
- [source](https://github.com/indynull/icedtea/blob/main/src/layout/recipes.rs)
- [crates.io](https://crates.io/crates/icedtea)
