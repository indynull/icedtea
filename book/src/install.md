# Install

icedtea is a Rust library. Add it to a desktop app and take widgets,
tokens, and chrome from it. This guide is also at
<https://indynull.github.io/icedtea/>.

Toolchain: Rust 1.89 or newer.

```toml
[dependencies]
iced = "0.14"
icedtea = "0.4"
```

[Crate docs](https://docs.rs/icedtea) ·
[crates.io](https://crates.io/crates/icedtea) ·
[source](https://github.com/indynull/icedtea)

## Host libraries

- **Linux:** `libxkbcommon-dev` and `libwayland-dev` (Debian/Ubuntu
  names). Same set iced needs for a window.
- **macOS:** Xcode command-line tools (`xcode-select --install`).
- **Windows:** MSVC build tools.

Repository checks and publish steps live in `AGENTS.md` at the crate
root.
