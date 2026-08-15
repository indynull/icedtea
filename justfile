# Public check: lint, tests, docs, coverage.
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    @just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Fast style gate. No tests, docs, or coverage.
lint: fmt-check clippy

test:
    cargo test --workspace --all-features

doc:
    cargo doc --package icedtea --no-deps --document-private-items

# Isolated coverage tree. Incremental off only here so rustc flags do not
# poison target/debug. Delete the tree after a passing local report.
# The coverage job leaves the tree so rust-cache can reuse it.
# Fail-under is lcov/Codecov source-line hits (100), not llvm-cov macros.
cov:
    CARGO_INCREMENTAL=0 cargo llvm-cov --package icedtea --all-features --ignore-filename-regex 'src[/\\]host' --lcov --output-path target/lcov.info
    python3 scripts/lcov-fail-under.py target/lcov.info
    rm -rf target/llvm-cov-target target/llvm-cov target/lcov.info

check: lint test doc cov

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

# Recapture handbook constructor stills into book/src/images/.
# Same Xephyr path as gallery-qa. Does not invent screenshots.
book-stills *args:
    python3 scripts/gallery_qa.py --book {{args}}
