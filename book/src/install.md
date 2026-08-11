# Install

icedtea is a Rust library. Add it to a desktop app and take widgets,
tokens, and chrome from it. This guide is also at
<https://indynull.github.io/icedtea/>.

Toolchain: Rust 1.89 or newer.

```toml
[dependencies]
iced = "0.14"
icedtea = "0.2"
```

[Crate docs](https://docs.rs/icedtea) ·
[crates.io](https://crates.io/crates/icedtea) ·
[source](https://github.com/indynull/icedtea)

## Host libraries

- **Linux:** `libxkbcommon-dev` and `libwayland-dev` (Debian/Ubuntu
  names). Same set iced needs for a window.
- **macOS:** Xcode command-line tools (`xcode-select --install`).
- **Windows:** MSVC build tools.

## For contributors

This section is for people working in the icedtea repository, not
for application authors.

Install [`just`](https://github.com/casey/just) and
[`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov), then:

```bash
cargo install just cargo-llvm-cov
just check
just clean
cargo run -p icedtea-gallery
```

`just check` is format, clippy (`-D warnings`), tests, rustdoc, and
line coverage fail-under 99 on the `icedtea` package (host glue
ignored). A passing coverage run deletes the instrumented
`target/llvm-cov-target` tree. `just clean` is `cargo clean`.

Home: <https://github.com/indynull/icedtea>. Tag `vX.Y.Z` (same
numbers as `Cargo.toml` `version`) publishes `icedtea` to crates.io.
The gallery crate sets `publish = false`. This guide builds on every
`master` push and is served at <https://indynull.github.io/icedtea/>.
