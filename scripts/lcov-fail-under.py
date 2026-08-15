#!/usr/bin/env python3
"""Fail unless every lcov DA record ran (Codecov source-line hits)."""

from __future__ import annotations

import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: lcov-fail-under.py LCOV", file=sys.stderr)
        return 2
    text = Path(sys.argv[1]).read_text()
    total = 0
    missed: list[tuple[str, int]] = []
    current = ""
    for line in text.splitlines():
        if line.startswith("SF:"):
            current = line[3:]
        elif line.startswith("DA:"):
            num, _, rest = line[3:].partition(",")
            hits = rest.split(",", 1)[0]
            total += 1
            if int(hits) == 0:
                missed.append((current, int(num)))
    if total == 0:
        print("no DA records", file=sys.stderr)
        return 1
    covered = total - len(missed)
    print(f"lcov lines {total} missed {len(missed)} {100.0 * covered / total:.2f}%")
    for path, n in missed:
        print(f"  {path}:{n}")
    return 0 if not missed else 1


if __name__ == "__main__":
    raise SystemExit(main())
