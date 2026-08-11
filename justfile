# Public check: format, clippy, tests, docs, coverage.
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    @just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    CARGO_INCREMENTAL=0 cargo test --workspace --all-features

doc:
    CARGO_INCREMENTAL=0 cargo doc --package icedtea --no-deps --document-private-items

# Drop target/llvm-cov-target after a passing report.
cov:
    CARGO_INCREMENTAL=0 cargo llvm-cov --package icedtea --all-features --fail-under-lines 99 --ignore-filename-regex 'src[/\\]host'
    rm -rf target/llvm-cov-target target/llvm-cov

check: fmt-check clippy test doc cov

clean:
    cargo clean

# Dry-run the published package (needs network for registry).
publish-dry:
    cargo publish -p icedtea --dry-run --locked

# HTML guide (needs mdbook).
book:
    mdbook build

book-serve:
    mdbook serve

# Record the gallery tour into assets/gallery.gif and book/src/gallery.gif.
# Needs a display, ffmpeg, xwininfo, wmctrl, import, python3.
# Floats a tiled window and places it before capture.
gallery-gif:
    bash scripts/gallery-gif.sh
