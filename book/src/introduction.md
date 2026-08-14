# icedtea

Native desktop widgets and chrome for [iced](https://iced.rs/).
Paint and layout follow [Material Design 3](https://m3.material.io/)
roles (`m3` module and `Tokens::scheme()`). Desktop chrome stays
rectangular (M3 shape None).

`icedtea::run!` starts a themed window. An `ActionTable` feeds the
toolbar, menus, and shortcuts. Constructors return `Element`s and emit
your messages. Color, layout, and chrome are Rust values.

![A themed icedtea window](gallery.gif)

[First window](first-window.md) is the shortest path: one `Action`, a
toolbar, and a notes editor. [Install](install.md) has the crate line
and host libraries. Compose covers architecture through compact tools.
The [cookbook](cookbook/save.md) walks four jobs. The
[reference](widgets.md) lists every public constructor.

- [Crate docs](https://docs.rs/icedtea)
- [crates.io](https://crates.io/crates/icedtea)
- [Source](https://github.com/indynull/icedtea)
