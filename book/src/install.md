# Install

icedtea is a Rust library. Add it to a desktop app and take widgets,
tokens, and chrome from it.

Toolchain: Rust 1.89 or newer. `rust-toolchain.toml` pins that and
installs rustfmt, clippy, and llvm-tools.

```toml
[dependencies]
icedtea = "0.1"
```

Until the first crates.io publish, use git:

```toml
[dependencies]
icedtea = { git = "https://github.com/indynull/icedtea" }
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
cargo run -p icedtea-gallery
```

`just check` runs format, clippy (`-D warnings`), tests, rustdoc, and
coverage fail-under on the `icedtea` package.

## Publish

Home: <https://github.com/indynull/icedtea>.

1. Set `version` in `Cargo.toml` and add a `CHANGELOG.md` section.
2. Push `master`. GitHub Actions runs `just check` on Linux, macOS, and
   Windows, and the test suite on Ubuntu `stable` and `beta`.
3. First version: `cargo publish -p icedtea --locked` on your machine
   (`cargo login`). Then on the crate: Settings, Trusted Publishing,
   GitHub. Owner `indynull`, repository `icedtea`, workflow
   `publish.yml`. No environment name.
4. Later versions: tag `vX.Y.Z` (same numbers as `version`) and push
   the tag.
5. GitHub Actions job `crates-io` checks the tag, runs `just check`,
   exchanges an identity token for a short-lived crates.io token, and
   `cargo publish -p icedtea --locked`. No repository secret.

The gallery crate sets `publish = false`; only `icedtea` goes to
crates.io. Docs build on [docs.rs](https://docs.rs/icedtea) from that
publish.
