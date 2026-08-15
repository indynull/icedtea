# icedtea

A Rust library that draws a native desktop window on
[iced](https://iced.rs/). Buttons, lists, menus, and the chrome around
them come from icedtea. Your program keeps the data (the notes, the
tasks, the file on disk). Paint follows
[Material Design 3](https://m3.material.io/) color roles
(`m3` module and `Tokens::scheme()`). Desktop chrome stays
rectangular (M3 shape None).

`icedtea::run!` opens the window. An `ActionTable` is the list of
commands you declare once; the toolbar, menus, and shortcuts read
that list. A constructor is a function that draws one control and
sends a **message** — a note about what the user did. Color, layout,
and chrome are Rust values.

![A themed icedtea window](gallery.gif)

[First window](first-window.md) is the shortest path: one `Action`, a
toolbar, and a notes editor. [Install](install.md) has the crate line
and host libraries. After that, [Keep a task list](cookbook/tasks.md)
is a one-day job: list, add, mark done, remember rows in a SQLite
file. Compose covers how the pieces fit. The
[cookbook](cookbook/save.md) walks those jobs. The
[reference](widgets.md) lists every public constructor.

- [Crate docs](https://docs.rs/icedtea)
- [crates.io](https://crates.io/crates/icedtea)
- [Source](https://github.com/indynull/icedtea)
