# icedtea

Native desktop widgets and chrome for [iced](https://iced.rs/).

`icedtea::run!` starts a themed window. Constructors return `Element`s
and emit your messages. Color, layout, and chrome are Rust values.

![The icedtea gallery](gallery.gif)

The [gallery](https://github.com/indynull/icedtea/tree/master/icedtea-gallery)
shows every `catalog::ENTRIES` id. Each id has one constructor.
Related controls share a page:

```bash
cargo run -p icedtea-gallery
```

[First window](first-window.md) is the shortest path. [Install](install.md)
has the crate line and host libraries. [Widgets](widgets.md) covers
constructors, time, and virtual lists.
