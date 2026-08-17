#!/usr/bin/env python3
"""Fail when counted lcov DA records are unhit.

Same bar as codecov.yml (project and patch target 100, threshold 0):
a DA:line,0 on a counted src file. Ignores host glue, the gallery,
examples, and the guide. Does not use llvm-cov --fail-under-lines.
"""

from __future__ import annotations

import sys
from pathlib import Path

# Match codecov.yml ignore.
_IGNORE_STARTS = (
    "src/host",
    "icedtea-gallery/",
    "examples/",
    "book/",
)


def repo_rel(sf: str) -> str:
    p = sf.replace("\\", "/")
    for key in ("icedtea-gallery/", "examples/", "book/", "src/"):
        i = p.find(key)
        if i != -1:
            return p[i:]
    return Path(p).name


def counted(rel: str) -> bool:
    return not any(rel.startswith(p) for p in _IGNORE_STARTS)


def missed(text: str) -> list[tuple[str, int]]:
    sf: str | None = None
    out: list[tuple[str, int]] = []
    for raw in text.splitlines():
        if raw.startswith("SF:"):
            sf = repo_rel(raw[3:])
        elif raw.startswith("DA:") and sf is not None and counted(sf):
            line_s, hits_s = raw[3:].split(",", 1)
            if int(hits_s) == 0:
                out.append((sf, int(line_s)))
        elif raw == "end_of_record":
            sf = None
    return out


def main(argv: list[str]) -> int:
    if argv == ["--self-test"]:
        _self_test()
        return 0
    if len(argv) != 1:
        print("usage: check_lcov.py LCOV.info", file=sys.stderr)
        return 2
    path = Path(argv[0])
    hits = missed(path.read_text())
    if not hits:
        return 0
    print("counted lcov DA,0 (codecov.yml source-line 100):", file=sys.stderr)
    for rel, line in hits:
        print(f"  {rel}:{line}", file=sys.stderr)
    return 1


def _self_test() -> None:
    sample = """
SF:/repo/src/widget.rs
DA:10,1
DA:11,0
end_of_record
SF:/repo/src/host.rs
DA:1,0
end_of_record
SF:/repo/src/host_canvas.rs
DA:2,0
end_of_record
SF:/repo/icedtea-gallery/src/main.rs
DA:3,0
end_of_record
SF:/repo/examples/hello.rs
DA:4,0
end_of_record
SF:C:\\repo\\book\\src\\intro.md
DA:5,0
end_of_record
SF:C:\\repo\\src\\collection.rs
DA:20,0
end_of_record
"""
    got = missed(sample)
    assert got == [("src/widget.rs", 11), ("src/collection.rs", 20)], got
    assert missed("SF:src/a11y.rs\nDA:1,3\nend_of_record") == []
    assert counted("src/widget.rs")
    assert not counted("src/host.rs")
    assert not counted("src/host_dialog.rs")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
