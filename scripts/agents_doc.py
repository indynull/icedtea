#!/usr/bin/env python3
"""Emit the app-authoring agent pack from catalog rustdoc.

    just agents-doc
    just agents-doc --check
    python3 scripts/agents_doc.py --self-test

Writes ``docs/agents/llms.txt`` and one markdown file per catalog
group. Do not hand-edit those files.
"""

from __future__ import annotations

import argparse
import re
from dataclasses import dataclass
from pathlib import Path
from tempfile import TemporaryDirectory


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUT = ROOT / "docs" / "agents" / "llms.txt"
MODULE_FILES = {
    "widget": Path("src/widget.rs"),
    "pattern": Path("src/pattern.rs"),
    "layout": Path("src/layout/flow.rs"),
    "motion": Path("src/motion.rs"),
}
ENTRY_RE = re.compile(
    r'Entry \{ id: "([^"]+)", title: "([^"]+)", group: "([^"]+)", page: "([^"]+)" \}'
)
CTOR_RE = re.compile(r'"([^"]+)" => \("([^"]+)", "([^"]+)"\)')
MAP_RE = re.compile(
    r'id: "([^"]+)",\s*m3: "([^"]+)",\s*fate: Fate::(Map|Desktop|Delete)',
    re.S,
)


@dataclass(frozen=True)
class Entry:
    id: str
    title: str
    group: str
    page: str


@dataclass(frozen=True)
class MapInfo:
    m3: str
    fate: str


def parse_entries(text: str) -> list[Entry]:
    start = text.find("pub const ENTRIES")
    if start < 0:
        raise SystemExit("catalog.rs has no ENTRIES")
    body = text[start:]
    end = body.find("];")
    if end < 0:
        raise SystemExit("catalog.rs ENTRIES is not closed")
    rows = [
        Entry(m.group(1), m.group(2), m.group(3), m.group(4))
        for m in ENTRY_RE.finditer(body[:end])
    ]
    if not rows:
        raise SystemExit("catalog.rs ENTRIES is empty")
    return rows


def parse_constructors(text: str) -> dict[str, tuple[str, str]]:
    start = text.find("pub fn constructor")
    if start < 0:
        raise SystemExit("catalog.rs has no constructor()")
    body = text[start:]
    end = body.find("\n}")
    if end < 0:
        raise SystemExit("catalog.rs constructor() is not closed")
    return {m.group(1): (m.group(2), m.group(3)) for m in CTOR_RE.finditer(body[:end])}


def parse_mapping(text: str) -> dict[str, MapInfo]:
    start = text.find("pub const MAP")
    if start < 0:
        raise SystemExit("mapping.rs has no MAP")
    body = text[start:]
    out: dict[str, MapInfo] = {}
    for m in MAP_RE.finditer(body):
        if m.group(3) == "Delete":
            continue
        out[m.group(1)] = MapInfo(m.group(2), m.group(3))
    if not out:
        raise SystemExit("mapping.rs MAP has no live rows")
    return out


def group_file(group: str) -> str:
    return f"{group.lower().replace(' ', '-')}.md"


def find_pub_fn(src: str, name: str) -> int | None:
    pat = f"pub fn {name}"
    start = 0
    while True:
        at = src.find(pat, start)
        if at < 0:
            return None
        after = src[at + len(pat) : at + len(pat) + 1]
        if after in ("(", "<"):
            return at
        start = at + len(pat)


def rustdoc_block_above(before: str) -> list[str]:
    docs: list[str] = []
    for line in reversed(before.splitlines()):
        t = line.lstrip()
        if t == "":
            if not docs:
                continue
            break
        if t.startswith("///") or t.startswith("#["):
            docs.append(t)
            continue
        break
    docs.reverse()
    return docs


def rustdoc_lines(src: str, name: str) -> list[str]:
    at = find_pub_fn(src, name)
    if at is None:
        raise SystemExit(f"missing pub fn {name}")
    lines: list[str] = []
    for raw in rustdoc_block_above(src[:at]):
        if not raw.startswith("///"):
            continue
        text = raw[3:]
        if text.startswith(" "):
            text = text[1:]
        if text == "":
            if lines:
                break
            continue
        if text.startswith("```"):
            break
        lines.append(text)
    if not lines:
        raise SystemExit(f"pub fn {name} has no rustdoc paragraph")
    return lines


def rustdoc_first_paragraph(src: str, name: str) -> str:
    return first_sentence(plain_rustdoc(" ".join(rustdoc_lines(src, name))))


def rustdoc_paragraph(src: str, name: str) -> str:
    return rustdoc_prose(src, name)


def rustdoc_prose(src: str, name: str) -> str:
    at = find_pub_fn(src, name)
    if at is None:
        raise SystemExit(f"missing pub fn {name}")
    paras: list[str] = []
    cur: list[str] = []
    for raw in rustdoc_block_above(src[:at]):
        if not raw.startswith("///"):
            continue
        text = raw[3:]
        if text.startswith(" "):
            text = text[1:]
        if text.startswith("```"):
            break
        if text == "":
            if cur:
                paras.append(" ".join(cur))
                cur = []
            continue
        cur.append(text)
    if cur:
        paras.append(" ".join(cur))
    if not paras:
        raise SystemExit(f"pub fn {name} has no rustdoc paragraph")
    return "\n\n".join(plain_rustdoc(p) for p in paras)


def plain_rustdoc(text: str) -> str:
    return re.sub(r"\[`([^`]+)`\](?:\([^)]*\))?", r"`\1`", text)


def first_sentence(paragraph: str) -> str:
    out: list[str] = []
    in_tick = False
    for i, ch in enumerate(paragraph):
        if ch == "`":
            in_tick = not in_tick
            out.append(ch)
            continue
        nxt = paragraph[i + 1] if i + 1 < len(paragraph) else ""
        if ch == "." and not in_tick and nxt in ("", " "):
            out.append(".")
            return "".join(out)
        out.append(ch)
    return paragraph


def rustdoc_url(module: str, name: str) -> str:
    return f"https://docs.rs/icedtea/latest/icedtea/{module}/fn.{name}.html"


def source_url(module: str) -> str:
    rel = MODULE_FILES.get(module)
    if rel is None:
        raise SystemExit(f"unknown constructor module {module}")
    return f"https://github.com/indynull/icedtea/blob/main/{rel.as_posix()}"


def render(
    entries: list[Entry],
    constructors: dict[str, tuple[str, str]],
    sources: dict[str, str],
) -> str:
    out: list[str] = [
        "# icedtea",
        "",
        "> Native desktop widgets and chrome for iced. Constructors return",
        "> Elements and emit the application's messages.",
        "",
        "Your program owns the data. One Action table feeds menus, toolbars,",
        "shortcuts, and the command palette. Call a constructor for a job.",
        "Start from `examples/hello.rs`.",
        "",
        "This file is generated from `catalog::ENTRIES` and constructor",
        "rustdoc. Run `just agents-doc` after those change.",
        "",
        "## Compose",
        "",
        "- [First window](examples/hello.rs): `run!`, `Boot`, `ActionTable`,",
        "  `seed_quit`, `From<keyboard::Event>`, `key::handle`, `focus::cycle`",
        "- [Task list](examples/tasks.rs): list plus on-disk SQLite",
        "- [Crate rustdoc](https://docs.rs/icedtea/latest/icedtea/): Boot,",
        "  tokens, Action, `native_dialog`, `theme::named`, `theme::follow`",
        "- Heading jump: `MarkdownDoc::item_offset` then",
        "  `icedtea::iced::widget::operation::scroll_to` with",
        "  `AbsoluteOffset { x: None, y: Some(y) }`",
        "- A path crate should copy this repo's `Cargo.lock` so a fresh",
        "  resolve stays on rustc 1.89",
        "- Keys: set `KeyContext.modal_open` when a palette or menu is open",
        "  so Escape and arrows stay on that layer",
        "- Open a row's rustdoc for the signature, `*Opts`, `*Face`, and the",
        "  compiling example. The one-line job is not the full call",
        "- Native file open/save is `icedtea::native_dialog` + `DialogSpec`",
        "  (crate rustdoc host). Catalog `dialogs` is the in-window sheet",
        "- Palette: keep `CommandPalette` in the app; paint it in an iced",
        "  `stack` over the window",
        "- A path crate next to this repo needs an empty `[workspace]`",
        "  table so Cargo does not treat it as a member",
    ]
    groups: list[str] = []
    for entry in entries:
        if entry.group not in groups:
            groups.append(entry.group)
    for group in groups:
        slug = group_file(group)
        out.extend(["", f"## [{group}]({slug})", ""])
        for entry in entries:
            if entry.group != group:
                continue
            ctor = constructors.get(entry.id)
            if ctor is None:
                raise SystemExit(f"catalog id {entry.id} has no constructor()")
            module, name = ctor
            src = sources.get(module)
            if src is None:
                raise SystemExit(f"{entry.id}: missing source for {module}")
            job = rustdoc_first_paragraph(src, name)
            out.append(
                f"- [{entry.id}]({rustdoc_url(module, name)}): {job} "
                f"[source]({source_url(module)})"
            )
    out.append("")
    return "\n".join(out)


def material_line(info: MapInfo) -> str:
    if info.fate == "Map":
        return f"Material: {info.m3}"
    return f"Desktop: {info.m3}"


def render_group(
    group: str,
    entries: list[Entry],
    constructors: dict[str, tuple[str, str]],
    sources: dict[str, str],
    mapping: dict[str, MapInfo],
) -> str:
    out = [
        f"# {group}",
        "",
        "Generated from `catalog::ENTRIES`, constructor rustdoc, and",
        "`m3::mapping`. Compose starts at `examples/hello.rs` (see",
        "[llms.txt](llms.txt)).",
        "",
    ]
    for entry in entries:
        if entry.group != group:
            continue
        ctor = constructors.get(entry.id)
        if ctor is None:
            raise SystemExit(f"catalog id {entry.id} has no constructor()")
        module, name = ctor
        src = sources.get(module)
        if src is None:
            raise SystemExit(f"{entry.id}: missing source for {module}")
        info = mapping.get(entry.id)
        if info is None:
            raise SystemExit(f"catalog id {entry.id} has no m3::mapping row")
        job = rustdoc_paragraph(src, name)
        out.extend(
            [
                f"## {entry.id}",
                "",
                job,
                "",
                f"Constructor: `{module}::{name}`",
                material_line(info),
                f"[rustdoc]({rustdoc_url(module, name)}) · "
                f"[source]({source_url(module)})",
                "",
            ]
        )
    return "\n".join(out)


def render_pack(
    entries: list[Entry],
    constructors: dict[str, tuple[str, str]],
    sources: dict[str, str],
    mapping: dict[str, MapInfo],
) -> dict[str, str]:
    groups: list[str] = []
    for entry in entries:
        if entry.group not in groups:
            groups.append(entry.group)
    files = {"llms.txt": render(entries, constructors, sources)}
    for group in groups:
        files[group_file(group)] = render_group(
            group, entries, constructors, sources, mapping
        )
    return files


def emit_pack(root: Path) -> dict[str, str]:
    catalog = (root / "src" / "catalog.rs").read_text(encoding="utf-8")
    mapping_src = (root / "src" / "m3" / "mapping.rs").read_text(encoding="utf-8")
    entries = parse_entries(catalog)
    constructors = parse_constructors(catalog)
    mapping = parse_mapping(mapping_src)
    sources = {
        module: (root / rel).read_text(encoding="utf-8")
        for module, rel in MODULE_FILES.items()
    }
    return render_pack(entries, constructors, sources, mapping)


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def check(path: Path, text: str) -> None:
    if not path.is_file():
        raise SystemExit(f"missing {path}; run just agents-doc")
    got = path.read_text(encoding="utf-8")
    if got != text:
        raise SystemExit(f"{path} is stale; run just agents-doc")


def write_pack(out_dir: Path, files: dict[str, str]) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    for name, text in files.items():
        write(out_dir / name, text)
    for path in out_dir.iterdir():
        if path.is_file() and path.name not in files:
            path.unlink()


def check_pack(out_dir: Path, files: dict[str, str]) -> None:
    if not out_dir.is_dir():
        raise SystemExit(f"missing {out_dir}; run just agents-doc")
    for name, text in files.items():
        check(out_dir / name, text)
    extras = sorted(
        p.name for p in out_dir.iterdir() if p.is_file() and p.name not in files
    )
    if extras:
        raise SystemExit(f"extra {', '.join(extras)}; run just agents-doc")


def self_test() -> None:
    catalog = """
pub const ENTRIES: &[Entry] = &[
    Entry { id: "button", title: "Button", group: "Controls", page: "controls" },
    Entry { id: "toolbar", title: "Toolbar", group: "Chrome", page: "chrome-rows" },
];

pub fn constructor(id: &str) -> Option<(&'static str, &'static str)> {
    Some(match id {
        "button" => ("widget", "button"),
        "toolbar" => ("pattern", "toolbar"),
        _ => return None,
    })
}
"""
    entries = parse_entries(catalog)
    assert [(e.id, e.group) for e in entries] == [
        ("button", "Controls"),
        ("toolbar", "Chrome"),
    ]
    ctors = parse_constructors(catalog)
    assert ctors["button"] == ("widget", "button")
    assert ctors["toolbar"] == ("pattern", "toolbar")

    widget = """
/// Press a labeled control to send a message.
///
/// `title` is the face.
///
/// ```
/// let save = ();
/// ```
#[allow(clippy::too_many_arguments)]
pub fn button<'a, M: Clone + 'a>(
    title: impl Into<String>,
)
"""
    assert (
        rustdoc_first_paragraph(widget, "button")
        == "Press a labeled control to send a message."
    )
    assert (
        first_sentence(
            "Related actions in one strip (M3 button group). Not exclusive."
        )
        == "Related actions in one strip (M3 button group)."
    )
    assert first_sentence("Use `1.0` then stop.") == "Use `1.0` then stop."
    assert (
        plain_rustdoc("A line ([`LabelFace`]: Body).")
        == "A line (`LabelFace`: Body)."
    )
    assert (
        first_sentence("Replace incoming from a 0..=1 progress.")
        == "Replace incoming from a 0..=1 progress."
    )

    sources = {
        "widget": widget,
        "pattern": """
/// One Action table as a row of buttons.
pub fn toolbar(
    actions: impl IntoIterator,
)
""",
    }
    text = render(entries, ctors, sources)
    assert text.startswith("# icedtea\n")
    assert "> Native desktop widgets and chrome for iced." in text
    assert "examples/hello.rs" in text
    assert "examples/tasks.rs" in text
    assert "`run!`" in text
    assert "KeyContext.modal_open" in text
    assert "native_dialog" in text
    assert "operation::scroll_to" in text
    assert "Cargo.lock" in text
    assert "## [Controls](controls.md)" in text
    assert "## [Chrome](chrome.md)" in text
    assert (
        "[button](https://docs.rs/icedtea/latest/icedtea/widget/fn.button.html)"
        in text
    )
    assert "Press a labeled control to send a message." in text
    assert (
        "[source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)"
        in text
    )
    assert (
        "[toolbar](https://docs.rs/icedtea/latest/icedtea/pattern/fn.toolbar.html)"
        in text
    )
    assert "One Action table as a row of buttons." in text

    mapping_src = """
pub const MAP: &[MapRow] = &[
    MapRow {
        id: "button",
        m3: "Button",
        fate: Fate::Map,
    },
    MapRow {
        id: "toolbar",
        m3: "Toolbar",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "gone",
        m3: "Old",
        fate: Fate::Delete,
    },
];
"""
    mapping = parse_mapping(mapping_src)
    assert mapping["button"] == MapInfo("Button", "Map")
    assert mapping["toolbar"] == MapInfo("Toolbar", "Desktop")
    assert "gone" not in mapping

    pack = render_pack(entries, ctors, sources, mapping)
    assert set(pack) == {"llms.txt", "controls.md", "chrome.md"}
    assert "](controls.md)" in pack["llms.txt"]
    assert "](chrome.md)" in pack["llms.txt"]
    controls = pack["controls.md"]
    assert controls.startswith("# Controls\n")
    assert "examples/hello.rs" in controls
    assert "## button" in controls
    assert "`widget::button`" in controls
    assert "Material: Button" in controls
    assert "Press a labeled control to send a message." in controls
    assert "`title` is the face." in controls
    chrome = pack["chrome.md"]
    assert "## toolbar" in chrome
    assert "Desktop: Toolbar" in chrome

    with TemporaryDirectory() as tmp:
        path = Path(tmp) / "llms.txt"
        try:
            check(path, text)
        except SystemExit as exc:
            assert "missing" in str(exc)
        else:
            raise AssertionError("check must fail when the file is absent")
        write(path, text)
        check(path, text)
        try:
            check(path, text + "x")
        except SystemExit as exc:
            assert "stale" in str(exc)
        else:
            raise AssertionError("check must fail when the emit changed")
        out = Path(tmp) / "agents"
        write_pack(out, pack)
        check_pack(out, pack)
        (out / "stray.md").write_text("no", encoding="utf-8")
        try:
            check_pack(out, pack)
        except SystemExit as exc:
            assert "extra" in str(exc)
        else:
            raise AssertionError("check_pack must fail on an extra file")

    print("agents_doc self-test ok")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--out",
        type=Path,
        default=DEFAULT_OUT,
        help="index path (default: docs/agents/llms.txt)",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="fail when the on-disk index does not match a fresh emit",
    )
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    files = emit_pack(ROOT)
    out_dir = args.out.parent
    if args.check:
        check_pack(out_dir, files)
        print(f"ok {out_dir} ({len(files)} files)")
        return
    write_pack(out_dir, files)
    print(f"wrote {out_dir} ({len(files)} files)")


if __name__ == "__main__":
    main()
