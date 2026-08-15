# Install

icedtea is a Rust library. You add it to a desktop program and call
its functions to draw the window. This guide is also at
<https://indynull.github.io/icedtea/>.

You need [Rust](https://rustup.rs/) 1.89 or newer (`rustup`). icedtea
tracks iced 0.14.

```bash
cargo add icedtea
```

That line is enough for [First window](first-window.md). A later job
that keeps tasks in a file also adds `rusqlite` — see
[Keep a task list](cookbook/tasks.md).

[Crate docs](https://docs.rs/icedtea) ·
[crates.io](https://crates.io/crates/icedtea) ·
[source](https://github.com/indynull/icedtea)

## Host libraries

These are the same libraries iced needs to open a window.

- **Linux:** `libxkbcommon-dev` and `libwayland-dev` (Debian/Ubuntu
  names).
- **macOS:** Xcode command-line tools (`xcode-select --install`).
- **Windows:** MSVC build tools.
