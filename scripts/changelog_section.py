#!/usr/bin/env python3
"""Print the CHANGELOG.md body for a version (for example 0.8.0)."""

from __future__ import annotations

import sys
from pathlib import Path


def section(text: str, version: str) -> str:
    heading = f"## {version}"
    lines = text.splitlines()
    start = None
    for i, line in enumerate(lines):
        if line == heading or line.startswith(f"{heading} "):
            start = i + 1
            break
    if start is None:
        raise SystemExit(f"no CHANGELOG section for {version}")
    end = len(lines)
    for i in range(start, len(lines)):
        if lines[i].startswith("## "):
            end = i
            break
    body = "\n".join(lines[start:end]).strip()
    if not body:
        raise SystemExit(f"empty CHANGELOG section for {version}")
    return f"{body}\n"


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: changelog_section.py VERSION")
    path = Path("CHANGELOG.md")
    sys.stdout.write(section(path.read_text(), sys.argv[1]))


if __name__ == "__main__":
    main()
