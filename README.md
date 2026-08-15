# icedtea

[![Documentation](https://img.shields.io/docsrs/icedtea/latest)][documentation]
[![Crates.io](https://img.shields.io/crates/v/icedtea.svg)][crates-io]
[![License](https://img.shields.io/crates/l/icedtea)][license]
[![check](https://github.com/indynull/icedtea/actions/workflows/ci.yml/badge.svg)][actions]
[![Coverage](https://codecov.io/gh/indynull/icedtea/graph/badge.svg)][codecov]

Native desktop widgets and chrome for [iced](https://iced.rs/).
Built on [Material Design 3](https://m3.material.io/get-started)
foundations (`m3` module). Desktop chrome is rectangular (M3 shape None).

`icedtea::run!` starts a themed window. One `Action` feeds the toolbar,
menus, and shortcuts. Constructors return iced `Element`s and emit
your messages.

![A themed icedtea window](https://github.com/indynull/icedtea/raw/master/assets/gallery.gif)

## First window

`cargo add icedtea`. It tracks iced 0.14. The crates.io badge above is
the crate version.

The program is [`examples/hello.rs`](examples/hello.rs): a notes buffer,
Save on `ctrl+s`, a toolbar, and a status line.

```rust,ignore
icedtea::run!(
    Boot::new("Notes", "dev.example.hello"),
    Hello::new,
    Hello::update,
    Hello::view,
    Hello::theme,
    Hello::subscription
)
```

`cargo run --example hello` from a checkout.

## Where to look

- [Guide](https://indynull.github.io/icedtea/) — first window, actions,
  cookbook jobs, and a reference for every control
- [Crate docs](https://docs.rs/icedtea) — `widget`, `theme`, `action`,
  `layout`, `window`, `pattern`
- [crates.io](https://crates.io/crates/icedtea) ·
  [source](https://github.com/indynull/icedtea)

Linux needs `libxkbcommon-dev` and `libwayland-dev`. macOS needs the
Xcode command-line tools. Windows needs the MSVC build tools.

[documentation]: https://docs.rs/icedtea
[crates-io]: https://crates.io/crates/icedtea
[license]: https://github.com/indynull/icedtea/blob/master/LICENSE
[actions]: https://github.com/indynull/icedtea/actions
[codecov]: https://app.codecov.io/gh/indynull/icedtea
