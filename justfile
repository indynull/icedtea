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

# Gallery QA (Xephyr tour shots). Skill: .grok/skills/gallery-qa/SKILL.md
#   just gallery-qa
#   just gallery-qa --interact --beats 0,8
gallery-qa *args:
    python3 scripts/gallery_qa.py {{args}}

# Ship README/book tour GIF only (captioned frames into assets/ + book/).
# ICEDTEA_GALLERY_ISOLATED=0 uses the current display.
gallery-gif:
    bash scripts/gallery-gif.sh
