# Install

icedtea is a Rust library. Add it to a desktop app and take widgets,
tokens, and chrome from it. This guide is also at
<https://indynull.github.io/icedtea/>.

Toolchain: Rust 1.89 or newer. `rust-toolchain.toml` pins that and
installs rustfmt, clippy, and llvm-tools.

```toml
[dependencies]
iced = "0.14"
icedtea = "0.2"
```

## Host libraries

- **Linux:** `libxkbcommon-dev` and `libwayland-dev` (Debian/Ubuntu
  names). Same set iced needs for a window.
- **macOS:** Xcode command-line tools (`xcode-select --install`).
- **Windows:** MSVC build tools.

## This repository

Install [`just`](https://github.com/casey/just) and
[`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov), then:

```bash
cargo install just cargo-llvm-cov
just check
just clean
cargo run -p icedtea-gallery
```

`just check` runs format, clippy (`-D warnings`), tests, rustdoc, and
line coverage fail-under 99 on the `icedtea` package (host glue
ignored; llvm-cov maps some const/macro lines as missed). The tests
require a rustdoc example on each catalog constructor. A passing
coverage run deletes the instrumented `target/llvm-cov-target` tree.
`just clean` is `cargo clean`.

## Publish

Home: <https://github.com/indynull/icedtea>.

1. Set `version` in `Cargo.toml` and move `CHANGELOG.md` Unreleased
   into a version section.
2. Push `master`. GitHub Actions runs `just check` on Linux, macOS, and
   Windows, and the test suite on Ubuntu `stable` and `beta`.
3. Tag `vX.Y.Z` (same numbers as `version`) and push the tag.
4. GitHub Actions job `crates-io` checks the tag, runs `just check`,
   exchanges an identity token for a short-lived crates.io token, and
   `cargo publish -p icedtea --locked`. No repository secret.

The gallery crate sets `publish = false`; only `icedtea` goes to
crates.io. API docs build on [docs.rs](https://docs.rs/icedtea) from
that publish. This guide builds on every `master` push
(`.github/workflows/book.yml`) and is served at
<https://indynull.github.io/icedtea/>.
