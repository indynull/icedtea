# icedtea

[![Documentation](https://img.shields.io/docsrs/icedtea/latest)][documentation]
[![Crates.io](https://img.shields.io/crates/v/icedtea.svg)][crates-io]
[![License](https://img.shields.io/crates/l/icedtea)][license]
[![check](https://github.com/indynull/icedtea/actions/workflows/ci.yml/badge.svg)][actions]
[![Coverage](https://codecov.io/gh/indynull/icedtea/graph/badge.svg)][codecov]

A Rust library that draws a native desktop window: buttons, lists,
menus, and the chrome around them, on [iced](https://iced.rs/).
You keep the data. Color and type follow
[Material Design 3](https://m3.material.io/get-started) roles
(`m3` module). Desktop chrome is rectangular (M3 shape None).

`icedtea::run!` opens the window. An `Action` is a command you write
once. The same row feeds the menu, the toolbar, shortcuts, the
context menu, footer hints, and the command palette. A constructor
is a function that draws one control and sends your message.

Click focuses a control. Tab walks those targets. A focused list,
tree, slider, or pick owns arrows, Enter, and Space. Escape closes
an open menu, pick, context menu, drawer, or cancel dialog.
`run!` listens for keys. `ActionTable::seed_quit` adds `ctrl+q`.
Lists, tables, and trees virtualize. Markdown, code, and fields
select and copy through the same `select` contract.

Layout is Rust (`pack`, `split_view`, `form`). Patterns dock a list
beside detail, a dialog sheet, a drawer, or a workspace of splits.
Chrome follows the window direction. Tokens carry density and follow
the host light/dark pair. Every drawing constructor takes `A11y`.

![A themed icedtea window](https://github.com/indynull/icedtea/raw/main/assets/gallery.gif)

## First window

`cargo add icedtea`. It tracks iced 0.14. The crates.io badge above is
the crate version.

The program is [`examples/hello.rs`](examples/hello.rs): a notes buffer,
Save on `ctrl+s`, Quit on `ctrl+q`, a toolbar, Tab into the editor,
and a status line.

```rust,ignore
icedtea::run!(
    Boot::new("Notes", "dev.example.hello"),
    Hello::new,
    Hello::update,
    Hello::view,
    Hello::theme,
)
```

`cargo run --example hello` from a checkout.

## Where to look

- [Guide](https://indynull.github.io/icedtea/) — first window, then a
  one-day task list that writes a SQLite file, then every control
- [Crate docs](https://docs.rs/icedtea) — `widget`, `theme`, `action`,
  `layout`, `window`, `pattern`
- [crates.io](https://crates.io/crates/icedtea) ·
  [source](https://github.com/indynull/icedtea)

Linux needs `libxkbcommon-dev` and `libwayland-dev`. macOS needs the
Xcode command-line tools. Windows needs the MSVC build tools.

[documentation]: https://docs.rs/icedtea
[crates-io]: https://crates.io/crates/icedtea
[license]: https://github.com/indynull/icedtea/blob/main/LICENSE
[actions]: https://github.com/indynull/icedtea/actions
[codecov]: https://app.codecov.io/gh/indynull/icedtea
