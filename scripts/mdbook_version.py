#!/usr/bin/env python3
"""mdBook preprocessor: fill {{ICEDTEA_VERSION}} and {{RUSQLITE_VERSION}}.

Values come from the workspace Cargo.toml so cookbook snippets cannot
drift from the crate or the rusqlite pin used by examples/tasks.rs.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CARGO = ROOT / "Cargo.toml"


def versions(text: str) -> tuple[str, str]:
    pkg = re.search(r'(?ms)^\[package\].*?^version = "([^"]+)"', text)
    rus = re.search(
        r'(?ms)^\[dev-dependencies\].*?^rusqlite = \{ version = "([^"]+)"',
        text,
    )
    if pkg is None:
        raise SystemExit("Cargo.toml [package] version is missing")
    if rus is None:
        raise SystemExit("Cargo.toml rusqlite dev-dependency version is missing")
    return pkg.group(1), rus.group(1)


def fill(text: str, icedtea_v: str, rusqlite_v: str) -> str:
    return text.replace("{{ICEDTEA_VERSION}}", icedtea_v).replace(
        "{{RUSQLITE_VERSION}}", rusqlite_v
    )


def walk(item: object, icedtea_v: str, rusqlite_v: str) -> None:
    if not isinstance(item, dict):
        return
    chapter = item.get("Chapter")
    if not isinstance(chapter, dict):
        return
    content = chapter.get("content")
    if isinstance(content, str):
        chapter["content"] = fill(content, icedtea_v, rusqlite_v)
    for child in chapter.get("sub_items", []):
        walk(child, icedtea_v, rusqlite_v)


def main() -> None:
    if len(sys.argv) > 1 and sys.argv[1] == "supports":
        sys.exit(0)
    icedtea_v, rusqlite_v = versions(CARGO.read_text(encoding="utf-8"))
    _ctx, book = json.load(sys.stdin)
    for item in book.get("items", []):
        walk(item, icedtea_v, rusqlite_v)
    json.dump(book, sys.stdout)


if __name__ == "__main__":
    main()
