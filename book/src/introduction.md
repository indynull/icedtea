# icedtea

Native desktop widgets and chrome for [iced](https://iced.rs/).

`icedtea::run!` starts a themed window. One `Action` feeds the toolbar,
menus, and shortcuts. Constructors return `Element`s and emit your
messages. Color, layout, and chrome are Rust values.

![A themed icedtea window](gallery.gif)

[First window](first-window.md) is the shortest path: one `Action`, a
toolbar, and a button. [Install](install.md) has the crate line and
host libraries. The [reference](widgets.md) lists every public
constructor.

- [Crate docs](https://docs.rs/icedtea)
- [crates.io](https://crates.io/crates/icedtea)
- [Source](https://github.com/indynull/icedtea)
