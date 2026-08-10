# Public check: format, clippy, tests, docs, coverage.
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    @just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo test --workspace --all-features

doc:
    cargo doc --package icedtea --no-deps --document-private-items

cov:
    cargo llvm-cov --package icedtea --all-features --fail-under-lines 99 --ignore-filename-regex 'src[/\\]host'

check: fmt-check clippy test doc cov

# Dry-run the published package (needs network for registry).
publish-dry:
    cargo publish -p icedtea --dry-run --locked

# HTML guide (needs mdbook).
book:
    mdbook build

book-serve:
    mdbook serve
