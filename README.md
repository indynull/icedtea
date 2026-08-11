# icedtea

Native desktop widgets and chrome for [iced](https://iced.rs/).

`icedtea::run!` starts a themed window. One `Action` feeds the toolbar,
menus, and shortcuts. Constructors return iced `Element`s and emit
your messages.

![A themed icedtea window](https://github.com/indynull/icedtea/raw/master/assets/gallery.gif)

## First window

```toml
[dependencies]
iced = "0.14"
icedtea = "0.2"
```

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

| iced | icedtea |
| --- | --- |
| 0.14 | 0.2 |
