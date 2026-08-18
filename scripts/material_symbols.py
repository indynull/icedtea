#!/usr/bin/env python3
"""Fetch Material Symbols Sharp FILL 1 SVGs for icedtea icon_svg.

Writes filled-black SVGs (and optionally a Rust module of byte slices)
so an application can pass Glyph::Bytes. Does not grow icedtea::icon::Icon.

    python3 scripts/material_symbols.py close folder save --out src/icons
    python3 scripts/material_symbols.py close --rs src/icons.rs
    python3 scripts/material_symbols.py --self-test
"""

from __future__ import annotations

import argparse
import re
import urllib.error
import urllib.request
from pathlib import Path

SOURCE = (
    "https://fonts.gstatic.com/s/i/short-term/release/"
    "materialsymbolssharp/{name}/fill1/24px.svg"
)
COMMENT = """<!-- Material Symbols Sharp `{name}` FILL 1, 24 dp.
     Copyright Google LLC. Apache License 2.0.
     https://github.com/google/material-design-icons -->
"""


def adapt_material_svg(svg: str) -> str:
    """Set root fill to #000 and drop currentColor. Same job as icon::adapt_material_svg."""
    text = svg.strip()
    if not text:
        raise ValueError("svg is empty")
    text = text.replace("currentColor", "#000")
    start = text.find("<svg")
    if start < 0:
        raise ValueError("not an svg document")
    rel_end = text.find(">", start)
    if rel_end < 0:
        raise ValueError("not an svg document")
    tag = text[start:rel_end]
    fill = tag.find("fill=")
    if fill < 0:
        new_tag = f'{tag} fill="#000"'
    else:
        value_at = fill + 5
        if value_at >= len(tag):
            raise ValueError("not an svg document")
        quote = tag[value_at]
        if quote not in {'"', "'"}:
            raise ValueError("not an svg document")
        close = tag.find(quote, value_at + 1)
        if close < 0:
            raise ValueError("not an svg document")
        new_tag = f'{tag[:fill]}fill="#000"{tag[close + 1 :]}'
    return f"{text[:start]}{new_tag}{text[rel_end:]}"


def rust_ident(name: str) -> str:
    ident = re.sub(r"[^0-9A-Za-z_]", "_", name).upper()
    if ident and ident[0].isdigit():
        ident = f"ICON_{ident}"
    return ident or "ICON"


def fetch(name: str) -> str:
    url = SOURCE.format(name=name)
    req = urllib.request.Request(
        url, headers={"User-Agent": "icedtea-material-symbols/1"}
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            raw = resp.read().decode("utf-8")
    except urllib.error.HTTPError as exc:
        raise SystemExit(f"{name}: HTTP {exc.code} from {url}") from exc
    except urllib.error.URLError as exc:
        raise SystemExit(f"{name}: {exc.reason}") from exc
    return f"{COMMENT.format(name=name)}{adapt_material_svg(raw)}\n"


def write_svg(out: Path, name: str, body: str) -> Path:
    out.mkdir(parents=True, exist_ok=True)
    path = out / f"{name}.svg"
    path.write_text(body, encoding="utf-8")
    return path


def write_rs(path: Path, names: list[str], bodies: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "// Generated from Material Symbols Sharp FILL 1.",
        "// Copyright Google LLC. Apache License 2.0.",
        "",
    ]
    for name, body in zip(names, bodies, strict=True):
        ident = rust_ident(name)
        lines.append(f'pub const {ident}: &[u8] = br###"{body.strip()}"###;')
        lines.append("")
    path.write_text("\n".join(lines), encoding="utf-8")


def self_test() -> None:
    raw = (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960">'
        '<path fill="currentColor" d="M80 80h80v80H80z"/></svg>'
    )
    out = adapt_material_svg(raw)
    assert 'fill="#000"' in out, out
    assert "currentColor" not in out, out
    painted = adapt_material_svg(
        '<svg fill="red" viewBox="0 0 24 24"><path d="M0 0h24v24H0z"/></svg>'
    )
    assert painted.startswith('<svg fill="#000" '), painted
    quoted = adapt_material_svg("<svg fill='#fff'><path d=\"M0 0h8v8H0z\"/></svg>")
    assert 'fill="#000"' in quoted, quoted
    for bad in ("", "png", "<svg fill=#000", "<svg fill=#000>"):
        try:
            adapt_material_svg(bad)
        except ValueError:
            continue
        raise SystemExit(f"expected reject: {bad!r}")
    print("material_symbols self-test ok")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "names", nargs="*", help="Material Symbols names (close, arrow_back, …)"
    )
    parser.add_argument(
        "--out", type=Path, default=Path("."), help="directory for .svg files"
    )
    parser.add_argument("--rs", type=Path, help="optional Rust module of byte slices")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    if not args.names:
        parser.error("pass one or more symbol names, or --self-test")
    bodies = [fetch(name) for name in args.names]
    for name, body in zip(args.names, bodies, strict=True):
        path = write_svg(args.out, name, body)
        print(path)
    if args.rs is not None:
        write_rs(args.rs, args.names, bodies)
        print(args.rs)


if __name__ == "__main__":
    main()
