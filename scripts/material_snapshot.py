#!/usr/bin/env python3
"""Snapshot Material Design 3 spec pages used by gallery QA.

The public site is an Angular app. Page bodies live at
``/_dsm/content/m3/<carbonVersion>/<fileId>.json``. This script finds
the current bundle, fetches a closed slug list, and writes markdown
plus ``index.json`` for the QA pass.

    just material-snapshot
    just material-snapshot --check
    python3 scripts/material_snapshot.py --self-test

Does not edit ``src/m3``. Applying a new number is a human edit after
reading the snapshot.
"""

from __future__ import annotations

import argparse
import hashlib
import html
import json
import re
import sys
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

SITE = "https://m3.material.io"
UA = "icedtea-material-snapshot/1"
# Closed list: styles and components icedtea maps in src/m3.
PAGES: tuple[str, ...] = (
    "styles/elevation",
    "styles/shape",
    "styles/spacing",
    "styles/typography",
    "styles/color/roles",
    "styles/motion/overview",
    "styles/motion/easing-and-duration",
    "foundations/design-tokens",
    "foundations/interaction/states",
    "foundations/layout/grids-spacing",
    "foundations/layout/bidirectionality-rtl",
    "components/lists",
    "components/menus",
    "components/search",
    "components/text-fields",
    "components/buttons",
    "components/dialogs",
)
DEFAULT_OUT = Path(".grok/skills/gallery-qa/references/material")


def http_get(url: str, timeout: int = 30) -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return resp.read()


def discover_bundle() -> tuple[str, str, dict[str, str]]:
    home = http_get(f"{SITE}/").decode("utf-8", "replace")
    match = re.search(r"/static/angular/main\.([a-f0-9]+)\.js", home)
    if not match:
        raise RuntimeError("homepage has no main.*.js (Material site layout changed)")
    js_url = f"{SITE}/static/angular/main.{match.group(1)}.js"
    js = http_get(js_url).decode("utf-8", "replace")
    ver = re.search(r'carbonVersion:"([^"]+)"', js)
    if not ver:
        raise RuntimeError("main.js has no carbonVersion")
    routes = dict(re.findall(r'"slug":"([^"]+)".{0,80}"exportedCarbonFileId":"([^"]+)"', js))
    missing = [slug for slug in PAGES if slug not in routes]
    if missing:
        raise RuntimeError(f"bundle missing slugs: {', '.join(missing)}")
    return js_url, ver.group(1), routes


def fetch_page(version: str, file_id: str) -> dict:
    url = f"{SITE}/_dsm/content/m3/{version}/{file_id}"
    return json.loads(http_get(url))


def html_to_text(raw: str) -> str:
    text = re.sub(r"(?i)<br\s*/?>", "\n", raw)
    text = re.sub(r"(?i)</(p|div|tr|h[1-6]|li|table|thead|tbody)>", "\n", text)
    text = re.sub(r"(?i)<h([1-6])[^>]*>", r"\n\n", text)
    text = re.sub(r"(?i)<li[^>]*>", "- ", text)
    text = re.sub(r"(?i)<th[^>]*>", "| ", text)
    text = re.sub(r"(?i)<td[^>]*>", "| ", text)
    text = re.sub(r"<[^>]+>", "", text)
    text = html.unescape(text)
    text = re.sub(r"[ \t]+\n", "\n", text)
    text = re.sub(r"\n{3,}", "\n\n", text)
    return text.strip()


def flatten_page(page: dict) -> str:
    lines: list[str] = []
    title = str(page.get("headerTitle") or page.get("title") or "Untitled")
    desc = str(page.get("description") or "").strip()
    lines.append(f"# {title}")
    if desc:
        lines.append("")
        lines.append(desc)
    for section in page.get("sections") or []:
        if not isinstance(section, dict):
            continue
        name = str(section.get("name") or "").strip()
        if name:
            lines.append("")
            lines.append(f"## {name}")
        for block in section.get("contentBlocks") or []:
            if not isinstance(block, dict):
                continue
            for chunk in block.get("contentChunks") or []:
                if not isinstance(chunk, dict):
                    continue
                html_val = chunk.get("htmlValue")
                if isinstance(html_val, str) and html_val.strip():
                    converted = html_to_text(html_val)
                    if converted:
                        lines.append("")
                        lines.append(converted)
                alt = chunk.get("altText")
                if isinstance(alt, str) and alt.strip():
                    lines.append("")
                    lines.append(f"*{alt.strip()}*")
    return "\n".join(lines).strip() + "\n"


def page_stem(slug: str) -> str:
    return slug.replace("/", "__")


def write_snapshot(out: Path, js_url: str, version: str, pages: dict[str, dict]) -> None:
    out.mkdir(parents=True, exist_ok=True)
    pages_dir = out / "pages"
    pages_dir.mkdir(exist_ok=True)
    fetched = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    index_pages: list[dict] = []
    index_md = [
        "# Material Design 3 spec snapshot",
        "",
        f"Fetched **{fetched}**. Carbon version `{version}`.",
        f"Bundle: `{js_url}`.",
        "",
        "Source of what Material says for gallery QA. icedtea desktop",
        "map is `src/m3/`. Refresh with `just material-snapshot`.",
        "Do not treat a desktop approximation as a miss when `src/m3`",
        "documents it.",
        "",
        "| Slug | File |",
        "| --- | --- |",
    ]
    for slug, body in pages.items():
        stem = page_stem(slug)
        text = flatten_page(body)
        md_path = pages_dir / f"{stem}.md"
        md_path.write_text(text, encoding="utf-8")
        digest = hashlib.sha256(text.encode()).hexdigest()
        public = f"{SITE}/{slug}"
        index_pages.append(
            {
                "slug": slug,
                "url": public,
                "file": f"pages/{stem}.md",
                "sha256": digest,
                "title": body.get("headerTitle") or body.get("title"),
                "updated": body.get("updatedTimestamp"),
            }
        )
        index_md.append(f"| `{slug}` | [pages/{stem}.md](pages/{stem}.md) |")
    index = {
        "fetched_at": fetched,
        "carbon_version": version,
        "bundle": js_url,
        "site": SITE,
        "pages": index_pages,
    }
    (out / "index.json").write_text(json.dumps(index, indent=2) + "\n", encoding="utf-8")
    (out / "INDEX.md").write_text("\n".join(index_md) + "\n", encoding="utf-8")


def refresh(out: Path) -> None:
    js_url, version, routes = discover_bundle()
    pages: dict[str, dict] = {}
    for slug in PAGES:
        print(f"fetch {slug}", file=sys.stderr)
        pages[slug] = fetch_page(version, routes[slug])
    write_snapshot(out, js_url, version, pages)
    print(f"wrote {out / 'INDEX.md'}", file=sys.stderr)


def check(out: Path) -> None:
    index_path = out / "index.json"
    if not index_path.is_file():
        raise SystemExit(f"missing {index_path}; run just material-snapshot")
    index = json.loads(index_path.read_text(encoding="utf-8"))
    pages = index.get("pages")
    if not isinstance(pages, list) or not pages:
        raise SystemExit(f"{index_path} has no pages")
    for row in pages:
        rel = row.get("file")
        if not isinstance(rel, str):
            raise SystemExit("index row missing file")
        path = out / rel
        if not path.is_file() or path.stat().st_size == 0:
            raise SystemExit(f"missing snapshot {path}")
        text = path.read_text(encoding="utf-8")
        want = row.get("sha256")
        if isinstance(want, str) and hashlib.sha256(text.encode()).hexdigest() != want:
            raise SystemExit(f"hash mismatch {path}")
    print(f"ok {len(pages)} pages, carbon {index.get('carbon_version')}")


def self_test() -> None:
    sample = "<p>Hello</p><ul><li>One</li><li>Two</li></ul><table><tr><th>A</th><td>2 dp</td></tr></table>"
    text = html_to_text(sample)
    assert "Hello" in text
    assert "- One" in text
    assert "2 dp" in text
    page = {
        "headerTitle": "Elevation",
        "description": "Distance on the z-axis",
        "sections": [
            {
                "name": "Overview",
                "contentBlocks": [
                    {
                        "contentChunks": [
                            {"htmlValue": "<p>Measured in dps.</p>"},
                            {"altText": "Five levels"},
                        ]
                    }
                ],
            }
        ],
    }
    md = flatten_page(page)
    assert md.startswith("# Elevation")
    assert "## Overview" in md
    assert "Measured in dps." in md
    assert "Five levels" in md
    print("material_snapshot self-test ok")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--out",
        type=Path,
        default=DEFAULT_OUT,
        help="snapshot directory (default: gallery-qa references/material)",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="validate an existing snapshot (no network)",
    )
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    if args.check:
        check(args.out)
        return
    refresh(args.out)


if __name__ == "__main__":
    main()
