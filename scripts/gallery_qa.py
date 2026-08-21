#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# ///
"""Gallery QA: tour the icedtea gallery, capture shots, optional interact.

Default Xephyr + metacity. Tour protocol + optional inject scripts.
Writes shots/, steps.jsonl, timings.json, CAPTURE.md under --out.
With --locale ar|ur also writes SCORE.md and exits non-zero if a
Firefox/Microsoft direction beat is broken (see
.grok/skills/gallery-qa/references/rtl.md). Leftover-English is
one row, not the bar.
Does not commit. Does not invent screenshots.

  just gallery-qa
  just gallery-qa --interact --beats 0,8
  just gallery-qa --live-clip
  just gallery-gif     # live pointer demo into assets/ + book/
  just book-stills     # recapture handbook stills into book/src/images/
"""

from __future__ import annotations

# Operator-facing CLI: stderr prints are intentional.
# ruff: noqa: T201
import argparse
import json
import os
import signal
import subprocess
import sys
import time
from datetime import UTC, datetime
from pathlib import Path


def _repo_root() -> Path:
    # …/icedtea/.grok/skills/gallery-visual-walkthrough/scripts/this.py
    return Path(__file__).resolve().parents[1]


def _which(*names: str) -> str | None:
    import shutil

    for n in names:
        p = shutil.which(n)
        if p:
            return p
    return None


def _run(
    cmd: list[str],
    *,
    check: bool = False,
    env: dict[str, str] | None = None,
    timeout: float | None = None,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        check=check,
        text=True,
        capture_output=True,
        env=env,
        timeout=timeout,
    )


def _utc_stamp() -> str:
    return datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")


def _now_ms() -> int:
    return int(time.monotonic() * 1000)


def _kill_display_server(display: str) -> None:
    marker = f" {display} "
    for name in ("Xephyr", "Xvfb"):
        for pid_s in _run(["pgrep", "-x", name]).stdout.split():
            try:
                pid = int(pid_s)
                cmd = (
                    Path(f"/proc/{pid}/cmdline")
                    .read_bytes()
                    .replace(b"\0", b" ")
                    .decode()
                )
            except (ValueError, OSError):
                continue
            if marker in f" {cmd} " or cmd.rstrip().endswith(display):
                try:
                    os.kill(pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
    time.sleep(0.25)


def free_display_num(start: int = 3, end: int = 20) -> int:
    for n in range(start, end + 1):
        if not Path(f"/tmp/.X11-unix/X{n}").exists():
            return n
    raise SystemExit("no free X display in :3-:20")


class NestedDisplay:
    def __init__(
        self,
        *,
        backend: str,
        width: int,
        height: int,
        display_num: int | None,
        host_display: str | None,
    ) -> None:
        self.backend = backend.casefold()
        self.width = width
        self.height = height
        self.display_num = (
            display_num if display_num is not None else free_display_num()
        )
        self.display = f":{self.display_num}"
        self.host_display = host_display or os.environ.get("DISPLAY") or ":0"
        self.proc: subprocess.Popen[bytes] | None = None
        self.wm_proc: subprocess.Popen[bytes] | None = None

    def start(self) -> str:
        if self.backend == "host":
            if not os.environ.get("DISPLAY"):
                raise SystemExit("host backend needs DISPLAY")
            return os.environ["DISPLAY"]

        if self.backend == "xephyr":
            binary = _which("Xephyr")
            if not binary:
                raise SystemExit("Xephyr not found (apt install xserver-xephyr)")
            _kill_display_server(self.display)
            env = os.environ.copy()
            env["DISPLAY"] = self.host_display
            print(
                f"starting Xephyr {self.display} ({self.width}x{self.height}) "
                f"on host {self.host_display}",
                file=sys.stderr,
            )
            self.proc = subprocess.Popen(
                [
                    binary,
                    self.display,
                    "-screen",
                    f"{self.width}x{self.height}",
                    "-ac",
                    "-nolisten",
                    "tcp",
                    "+extension",
                    "RANDR",
                    "+extension",
                    "COMPOSITE",
                    "+extension",
                    "XFIXES",
                    "-title",
                    "icedtea-gallery-qa",
                ],
                env=env,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                start_new_session=True,
            )
        elif self.backend == "xvfb":
            binary = _which("Xvfb")
            if not binary:
                raise SystemExit("Xvfb not found (apt install xvfb)")
            _kill_display_server(self.display)
            print(
                f"starting Xvfb {self.display} ({self.width}x{self.height})",
                file=sys.stderr,
            )
            self.proc = subprocess.Popen(
                [
                    binary,
                    self.display,
                    "-screen",
                    "0",
                    f"{self.width}x{self.height}x24",
                    "-ac",
                    "+extension",
                    "GLX",
                    "-nolisten",
                    "tcp",
                ],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                start_new_session=True,
            )
        else:
            raise SystemExit(f"unknown backend: {self.backend}")

        ready = False
        for _ in range(50):
            if self.proc.poll() is not None:
                raise SystemExit(
                    f"{self.backend} exited early (code {self.proc.returncode})"
                )
            r = _run(["xdpyinfo", "-display", self.display], timeout=2.0)
            if r.returncode == 0:
                ready = True
                break
            time.sleep(0.15)
        if not ready:
            raise SystemExit(f"{self.backend} {self.display} did not become ready")

        if _which("metacity"):
            print(f"starting metacity on {self.display}", file=sys.stderr)
            env = os.environ.copy()
            env["DISPLAY"] = self.display
            env.pop("WAYLAND_DISPLAY", None)
            self.wm_proc = subprocess.Popen(
                ["metacity", "--sm-disable"],
                env=env,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                start_new_session=True,
            )
            time.sleep(0.4)
            if _which("xsetroot"):
                _run(["xsetroot", "-solid", "#1a1a1a"], env=env, check=False)
        else:
            print(
                "warning: metacity missing; nested capture may fail",
                file=sys.stderr,
            )
        return self.display

    def stop(self) -> None:
        for p in (self.wm_proc, self.proc):
            if p is None:
                continue
            try:
                p.terminate()
                p.wait(timeout=2)
            except Exception:
                try:
                    p.kill()
                except Exception:
                    pass
        if self.backend != "host":
            _kill_display_server(self.display)


def resolve_binary(root: Path, prefer_release: bool) -> Path:
    release = root / "target" / "release" / "icedtea-gallery"
    debug = root / "target" / "debug" / "icedtea-gallery"
    if prefer_release and release.is_file():
        return release
    if debug.is_file():
        return debug
    if release.is_file():
        return release
    raise SystemExit(
        "icedtea-gallery binary missing; run: cargo build -p icedtea-gallery"
    )


def build_gallery(root: Path, release: bool) -> None:
    cmd = ["cargo", "build", "-p", "icedtea-gallery"]
    if release:
        cmd.append("--release")
    print(" ".join(cmd), file=sys.stderr)
    r = subprocess.run(cmd, cwd=root, check=False)
    if r.returncode != 0:
        raise SystemExit(f"cargo build failed ({r.returncode})")


def wait_file(path: Path, predicate, timeout_s: float = 12.0) -> bool:
    deadline = time.monotonic() + timeout_s
    while time.monotonic() < deadline:
        if path.is_file():
            try:
                if predicate(path.read_text(encoding="utf-8", errors="replace")):
                    return True
            except OSError:
                pass
        time.sleep(0.05)
    return False


def find_window_id(display: str, pid: int, timeout_s: float = 20.0) -> str:
    env = os.environ.copy()
    env["DISPLAY"] = display
    deadline = time.monotonic() + timeout_s
    while time.monotonic() < deadline:
        r = _run(["wmctrl", "-lp"], env=env, timeout=3.0)
        for line in r.stdout.splitlines():
            parts = line.split()
            if len(parts) >= 3 and parts[2] == str(pid):
                return parts[0]
        time.sleep(0.25)
    raise SystemExit(f"timed out waiting for window (pid={pid})")


def place_window(display: str, wid: str, x: int, y: int, w: int, h: int) -> None:
    env = os.environ.copy()
    env["DISPLAY"] = display
    _run(
        ["wmctrl", "-i", "-r", wid, "-b", "remove,maximized_vert,maximized_horz"],
        env=env,
    )
    _run(["wmctrl", "-i", "-r", wid, "-e", f"0,{x},{y},{w},{h}"], env=env)
    time.sleep(0.15)
    _run(["wmctrl", "-i", "-a", wid], env=env)


def pointer_clear_hover(display: str) -> None:
    """Park the pointer on the root so no control stays in Hovered style."""
    env = os.environ.copy()
    env["DISPLAY"] = display
    try:
        _run(["xdotool", "mousemove", "8", "8"], env=env, timeout=1.0)
    except (subprocess.TimeoutExpired, FileNotFoundError, OSError):
        pass


def window_frame(display: str, wid: str) -> tuple[int, int, int, int]:
    """Absolute X, Y, width, height of the managed frame."""
    env = os.environ.copy()
    env["DISPLAY"] = display
    geo = _run(["xwininfo", "-id", wid], env=env, timeout=3.0)
    abs_x = abs_y = width = height = None
    for line in geo.stdout.splitlines():
        s = line.strip()
        if s.startswith("Absolute upper-left X:"):
            abs_x = int(s.split(":")[-1].strip())
        elif s.startswith("Absolute upper-left Y:"):
            abs_y = int(s.split(":")[-1].strip())
        elif s.startswith("Width:"):
            width = int(s.split(":")[-1].strip())
        elif s.startswith("Height:"):
            height = int(s.split(":")[-1].strip())
    if None in (abs_x, abs_y, width, height):
        raise RuntimeError(f"xwininfo failed for {wid}: {geo.stdout[:200]}")
    return abs_x, abs_y, width, height


def xdotool(display: str, *args: str, timeout: float = 4.0) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env["DISPLAY"] = display
    env.pop("WAYLAND_DISPLAY", None)
    return _run(["xdotool", *args], env=env, timeout=timeout)


def wheel_at(
    display: str,
    x: int,
    y: int,
    *,
    clicks: int = 16,
    down: bool = True,
    delay_ms: int = 30,
) -> None:
    """Send a real mouse wheel. Button 5 is down, 4 is up.

    Move onto the pane first. A nested scroller under the pointer takes
    the event; inject or `update` is not wheel proof.
    """
    button = "5" if down else "4"
    xdotool(display, "mousemove", str(x), str(y))
    xdotool(
        display,
        "click",
        "--repeat",
        str(clicks),
        "--delay",
        str(delay_ms),
        button,
    )


def click_at(display: str, x: int, y: int) -> None:
    xdotool(display, "mousemove", str(x), str(y))
    xdotool(display, "click", "1")


def key_repeat(display: str, key: str, times: int, delay_ms: int = 40) -> None:
    xdotool(
        display,
        "key",
        "--repeat",
        str(times),
        "--delay",
        str(delay_ms),
        key,
    )


def capture_window(display: str, wid: str, dest: Path) -> dict:
    env = os.environ.copy()
    env["DISPLAY"] = display
    dest.parent.mkdir(parents=True, exist_ok=True)
    t0 = _now_ms()
    # Root crop of the managed frame (title bar included) — stable on nested X.
    geo = _run(["xwininfo", "-id", wid], env=env, timeout=3.0)
    abs_x = abs_y = width = height = None
    for line in geo.stdout.splitlines():
        s = line.strip()
        if s.startswith("Absolute upper-left X:"):
            abs_x = int(s.split(":")[-1].strip())
        elif s.startswith("Absolute upper-left Y:"):
            abs_y = int(s.split(":")[-1].strip())
        elif s.startswith("Width:"):
            width = int(s.split(":")[-1].strip())
        elif s.startswith("Height:"):
            height = int(s.split(":")[-1].strip())
    if None in (abs_x, abs_y, width, height):
        raise RuntimeError(f"xwininfo failed for {wid}: {geo.stdout[:200]}")
    # Include chrome: title bar is above client; use root crop with frame.
    # Prefer import -window when it works; fall back to root crop.
    r = _run(
        ["import", "-window", wid, str(dest)],
        env=env,
        timeout=10.0,
    )
    if r.returncode != 0 or not dest.is_file() or dest.stat().st_size < 1000:
        crop = f"{width}x{height}+{abs_x}+{abs_y}"
        r2 = _run(
            ["import", "-window", "root", "-crop", crop, str(dest)],
            env=env,
            timeout=10.0,
        )
        if r2.returncode != 0 or not dest.is_file():
            raise RuntimeError(
                f"import failed: {r.stderr or r.stdout} / {r2.stderr or r2.stdout}"
            )
    ms = _now_ms() - t0
    wh = _run(["identify", "-format", "%wx%h", str(dest)], env=env, timeout=5.0)
    return {
        "ms": ms,
        "size": dest.stat().st_size,
        "geometry": wh.stdout.strip() if wh.returncode == 0 else None,
        "wid": wid,
    }


# Caption prefix (before ':') → dest under book/src/images/.
# One representative idle frame per visual handbook group.
BOOK_STILLS: dict[str, str] = {
    "controls": "controls.png",
    "fields": "fields.png",
    "readout": "readout.png",
    "type": "content.png",
    "list": "collections.png",
    "chrome": "chrome.png",
    "list-and-detail": "patterns.png",
}
BOOK_HELLO_STILL = "first-window.png"
PNG_MAGIC = b"\x89PNG\r\n\x1a\n"


def require_png(path: Path, *, min_bytes: int = 1000) -> None:
    if not path.is_file():
        raise SystemExit(f"book still missing: {path}")
    data = path.read_bytes()
    if len(data) < min_bytes or not data.startswith(PNG_MAGIC):
        raise SystemExit(f"book still is not a painted PNG: {path}")


def publish_book_still(src: Path, dest: Path) -> None:
    require_png(src)
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_bytes(src.read_bytes())
    print(f"book still {dest}", file=sys.stderr)


def hello_binary(root: Path, release: bool) -> Path:
    kind = "release" if release else "debug"
    return root / "target" / kind / "examples" / "hello"


def build_hello(root: Path, release: bool) -> Path:
    cmd = ["cargo", "build", "--example", "hello"]
    if release:
        cmd.append("--release")
    print(" ".join(cmd), file=sys.stderr)
    r = subprocess.run(cmd, cwd=root, check=False)
    if r.returncode != 0:
        raise SystemExit(f"cargo build --example hello failed ({r.returncode})")
    path = hello_binary(root, release)
    if not path.is_file():
        raise SystemExit(f"hello example missing after build: {path}")
    return path


def capture_hello(
    root: Path,
    display: str,
    dest: Path,
    *,
    release: bool,
    no_build: bool,
    width: int = 480,
    height: int = 640,
) -> dict:
    if no_build and hello_binary(root, release).is_file():
        binary = hello_binary(root, release)
    else:
        binary = build_hello(root, release)
    env = os.environ.copy()
    env["DISPLAY"] = display
    env.pop("WAYLAND_DISPLAY", None)
    print(f"starting {binary}", file=sys.stderr)
    proc = subprocess.Popen(
        [str(binary)],
        cwd=root,
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    try:
        wid = find_window_id(display, proc.pid)
        place_window(display, wid, 40, 48, width, height)
        pointer_clear_hover(display)
        time.sleep(0.5)
        cap = capture_window(display, wid, dest)
        require_png(dest)
        print(f"book still {dest}", file=sys.stderr)
        return cap
    finally:
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=3)
            except subprocess.TimeoutExpired:
                proc.kill()


def slug(s: str) -> str:
    out = []
    for c in s.lower():
        if c.isalnum():
            out.append(c)
        elif c in " -_/":
            out.append("-")
    slug_s = "".join(out).strip("-")
    while "--" in slug_s:
        slug_s = slug_s.replace("--", "-")
    return slug_s or "step"


# Built-in interactions keyed by tour caption prefix (case-insensitive).
# Script is drained by the gallery via ICEDTEA_GALLERY_INJECT (not xdotool).
DEFAULT_INTERACT: list[dict[str, str]] = [
    {
        "match": "controls:",
        "name": "controls-toggles",
        "script": "check true\nswitch true\nradio 1\nslide 0.75\nsegment 1\nrange 15 90\ngroup 1\n",
        "expect": "Outlined and Elevated faces; icon-plus-label; toggle-icon; vertical slider; Accept and Notify on",
    },
    {
        "match": "controls:",
        "name": "controls-shape-pill",
        "script": "shape Pill\n",
        "expect": "buttons stadium; exclusive segments joined rectangles, not independent pills",
    },
    {
        "match": "Tabs, accordion",
        "name": "sections-shape-pill",
        "script": "shape Pill\n",
        "expect": "tab labels rectangular with underbar; not stadiums",
    },
    {
        "match": "fields:",
        "name": "search-submit",
        "script": "query icedtea\nsearch-go\n",
        "expect": "search field submitted icedtea; status shows Submitted",
    },
    {
        "match": "code:",
        "name": "code-wrap-off",
        "script": "code-wrap false\n",
        "expect": "wrap off; long source line stays on one row",
    },
    {
        "match": "navigation stack",
        "name": "nav-rail-select",
        "script": "rail 1\n",
        "expect": "Sent rail row selected",
    },
    {
        "match": "list:",
        "name": "list-select",
        "script": "list 1\nface card\n",
        "expect": "second list row selected; card face",
    },
    {
        "match": "list:",
        "name": "list-expand-card",
        "script": "expand-card 1\n",
        "expect": "second expand-card row open (virtual_column)",
        "page_hint": "virtual",  # optional; gallery list page hosts both
    },
    {
        "match": "accordion",
        "name": "sections-expand",
        # Idle starts with Files open (acc 0). Toggle Appearance so the
        # open section changes; expander starts closed.
        "script": "expand true\nacc 1\n",
        "expect": "expander open; Appearance accordion open",
    },
    {
        "match": "tree:",
        "name": "tree-select-leaf",
        # Select a leaf under the open src branch. Do not toggle root closed —
        # that empties the tree for later pages that share the model.
        "script": "tree-face files\ntree-sel 3\n",
        "expect": "Files face; lib.rs leaf selected; folders still open",
    },
    {
        "match": "item grid",
        "name": "grid-pick",
        "script": "grid 2\n",
        "expect": "grid tile 2 selected",
    },
    {
        "match": "marks:",
        "name": "filter-chips-toggle",
        "script": "filter 0\nfilter 1\n",
        "expect": "filter chips toggled; selected vs outline idle visible",
    },
    {
        "match": "dialogs:",
        "name": "side-sheet-open",
        "script": "sheet true\n",
        "expect": "side sheet open over dim scene",
    },
    {
        "match": "Motion: fade and slide",
        "name": "motion-play",
        "script": "dialog false\nbounce-in\npulse true\nshake\n",
        "expect": "overlay closed; bounce card visible; pulse on",
    },
    {
        "match": "Expand motion",
        "name": "expand-open",
        "script": "expand true\n",
        "expect": "expand notes body open",
    },
    {
        "match": "markdown:",
        "name": "markdown-span",
        "script": "md-move 0\nmd-press\nmd-move 800\nmd-release\n",
        "expect": "selection wash on heading through later blocks",
    },
    {
        "match": "table:",
        "name": "table-sort",
        "script": "sort 0\n",
        "expect": "Name column sorted; checks moved with their rows",
    },
    {
        "match": "command palette",
        "name": "palette-omit",
        "script": "pal-omit true\n",
        "expect": "field-only idle; hits region omitted",
    },
    {
        "match": "command palette",
        "name": "palette-spotlight",
        "script": "pal-query save\n",
        "expect": "Save hit after omit idle",
    },
]


def _caption_hits(caption: str, match: str) -> bool:
    """Token match: `table:` must not fire on `Selectable:`."""
    i = caption.find(match)
    if i < 0:
        return False
    return i == 0 or not caption[i - 1].isalpha()


def interactions_for_caption(caption: str) -> list[dict[str, str]]:
    c = caption.casefold()
    return [x for x in DEFAULT_INTERACT if _caption_hits(c, x["match"].casefold())]


def inject_script(inject_path: Path, script: str, timeout_s: float = 6.0) -> int:
    """Write inject script; wait for gallery to clear and ack applied count."""
    ack = inject_path.with_suffix(".ack")
    if ack.exists():
        ack.unlink()
    lines = [
        ln
        for ln in script.splitlines()
        if ln.strip() and not ln.strip().startswith("#")
    ]
    inject_path.write_text(
        script if script.endswith("\n") else script + "\n", encoding="utf-8"
    )
    deadline = time.monotonic() + timeout_s
    while time.monotonic() < deadline:
        if ack.is_file():
            try:
                n = int(ack.read_text(encoding="utf-8").strip() or "0")
                if n >= 0:
                    return n
            except ValueError:
                pass
        # also succeed if inject file was emptied
        try:
            if (
                inject_path.is_file()
                and not inject_path.read_text(encoding="utf-8").strip()
            ):
                if ack.is_file():
                    return int(ack.read_text(encoding="utf-8").strip() or "0")
        except OSError:
            pass
        time.sleep(0.05)
    raise TimeoutError(f"inject not acknowledged ({len(lines)} lines): {script!r}")


def write_capture_md(
    out: Path,
    *,
    meta: dict,
    steps: list[dict],
    timings: dict,
) -> None:
    lines = [
        "# Gallery visual capture",
        "",
        "Auto-generated by `scripts/gallery_qa.py`. Full protocol:",
        "`.grok/skills/gallery-qa/` (SKILL + rubric + manual-pass).",
        "",
        "Goal: find defects, fix source, recapture until ready — not a report.",
        "Full cut = shot pass + live pass. Score only to prioritize fixes.",
        "",
        "## Environment",
        "",
        f"- backend: `{meta.get('backend')}`",
        f"- display: `{meta.get('display')}`",
        f"- binary: `{meta.get('binary')}`",
        f"- tour_len: {meta.get('tour_len')}",
        f"- settle_ms: {meta.get('settle_ms')}",
        f"- git: `{meta.get('git')}`",
        "",
        "## Timings (wall)",
        "",
        f"- boot_ms: {timings.get('boot_ms')}",
        f"- total_ms: {timings.get('total_ms')}",
        f"- mean_step_ms: {timings.get('mean_step_ms')}",
        "",
        "## Shots",
        "",
        "| # | beat | kind | page | caption | file | inject | expect |",
        "| --- | --- | --- | --- | --- | --- | --- | --- |",
    ]
    for s in steps:
        if s.get("shot") is None:
            continue
        lines.append(
            f"| {s['index']} | {s.get('beat', '')} | {s.get('kind', '')} | "
            f"{s.get('page', '')} | {s.get('caption', '').replace('|', '/')} | "
            f"`{s['shot']}` | "
            f"`{(s.get('inject') or '').replace('|', '/').replace(chr(10), ';')}` | "
            f"{(s.get('expect') or '').replace('|', '/')} |"
        )
    lines.extend(
        [
            "",
            "## Agent checklist",
            "",
            "1. `read_file` every `shots/*.png` (multimodal — not filename-only).",
            "2. Score with `.grok/skills/gallery-qa/references/rubric.md`.",
            "3. After-interact: **expect** visible vs idle on the same beat.",
            "4. Hunt: clip/bleed, empty first paint, Fill collapse, dead demos,",
            "   cross-talk, hint lies, stub demos, chrome atoms, platform type.",
            "5. Full cut: live pass per `references/manual-pass.md` (pointer).",
            "6. Fix source (library first), recapture, re-read shots (max 3 cycles).",
            "7. Ready when no broken remains; short chat status, not a report file.",
            "8. Never `image_gen` the UI; inject ≠ pointer proof.",
            "",
        ]
    )
    (out / "CAPTURE.md").write_text("\n".join(lines), encoding="utf-8")


RTL_RAIL_PAGES = frozenset({"list", "tree", "log", "feedback"})
RTL_ALIGN_PAGES = frozenset({"list", "tree", "sections", "tabs-accordion-expander"})

# Painted demo / chrome English that must go through catalog fill.
# Matched in icedtea-gallery/src/main.rs before the test module.
LEFTOVER_ENGLISH = (
    "Reveal the token, then copy it.",
    "Show, then Copy.",
    "Suggest on any field. Pick fills the query.",
    "Enter a valid address.",
    "We never share your email.",
    "Labeled value with a shared form gutter. Select, then Copy.",
    "Primary row opens a submenu flyout.",
    "Saved notes.txt",
    "Inspector rows share a form label gutter. Copy posts the first selection.",
    "Drag or double-click a range. Copy takes that text. Copy all posts the source.",
    "Drag to select. Language + UI colorway",
    'Action::new("edit.copy-all", "Copy all"',
    "Primary action plus a chevron menu. Idle and disabled.",
    "Name is pinned. Role, Status, and Path follow horizontal scroll.",
    "A document card with tags, and an empty neighbour.",
    "Press a filter chip, or dismiss a tag with ×.",
    "Update available",
    "Fields in this group stay read-only.",
    "Last saved just now. Use File",
    "Edit pane. Tabs above switch Edit and Terminal",
    "Local drafts and attachments.",
    "Thanks for the notes. I will follow up after lunch.",
    'Cell::new("Day")',
    'Cell::from("Week")',
    'Cell::from("Month")',
    'password_input(\n                "Secret"',
    'secret_field(\n                    "Token"',
    'themed_text_input(\n                        "Email"',
    'suggest_field(\n                    "Command"',
    'Action::new("file.open", "Open"',
    'Action::new("file.recent", "Recent"',
    'named("find"',
    '("Home".into()',
    '("Gallery".into()',
    "Last saved just now.",
    "Overwrite notes.txt?",
    "Don't save",
    "Open dialog",
    "Washes and text-on colors from the active colorway.",
    "Type a letter, or Enter, Escape, an arrow, or a function key.",
    "Contain, cover, loading, and error. The application owns the bytes.",
    "Write the buffer to disk.",
    "Open the inspector sheet for properties.",
    "Close a tab with the ×. Selecting another tab swaps this body.",
    "Type to filter the action table. Pick a row, or choose Go to line for a parameter.",
    "Move terminal beside explorer",
    "Select a message",
    "Received this morning.",
    "Library sources.",
    "Crate root.",
    "Hide files",
    "Show files",
    "Editor — resize the window or hide the files rail.",
    "Filter shortcuts",
    "Fade and a short slide from progress 0 to 1.",
    "Reduce motion",
    "File, Edit, and View live in this window. Open a menu, then Save.",
    "Accent on",
    "Accent idle",
    'format!("{v:?}")',
    "text on canvas",
    "primary lighten",
    "input cursor",
    "Type a command",
)


def _column_midgray(im, x0: int, x1: int, y0: int, y1: int) -> int:
    """Count mid-luminance pixels (rail / thumb) in [x0, x1) × [y0, y1)."""
    px = im.load()
    n = 0
    for x in range(x0, x1):
        for y in range(y0, y1):
            r, g, b = px[x, y]
            lum = 0.299 * r + 0.587 * g + 0.114 * b
            if 48.0 <= lum <= 130.0:
                n += 1
    return n


def rail_side(path: Path) -> str:
    """Return left, right, or none for the vertical rail in a window shot."""
    from PIL import Image

    im = Image.open(path).convert("RGB")
    w, h = im.size
    if w < 200 or h < 200:
        return "none"
    # Drop the look strip / title and the status bar; drop the RTL nav
    # column on the right (~320px).
    y0, y1 = min(170, h // 3), h - 36
    nav = 340 if w >= 1200 else 0
    # Drop the card's far edge next to the nav; that hairline is not the rail.
    edge = 80 if w >= 1200 else 0
    x_left, x_right = 36, max(37, w - nav - edge)
    if y1 - y0 < 80 or x_right - x_left < 80:
        return "none"
    span = 12
    best_x = x_left
    best_n = -1
    x = x_left
    while x + span <= x_right:
        n = _column_midgray(im, x, x + span, y0, y1)
        if n > best_n:
            best_n = n
            best_x = x
        x += 4
    if best_n <= 0:
        return "none"
    width = x_right - x_left
    rel = (best_x - x_left) / width
    if rel < 0.28:
        return "left"
    if rel > 0.72:
        return "right"
    return "none"


def _production_src(path: Path) -> str:
    """Library or gallery source before the test module."""
    text = path.read_text(encoding="utf-8")
    cut = text.find("#[cfg(test)]")
    return text if cut < 0 else text[:cut]


_PHYSICAL_ALIGN = (
    "Alignment::Left",
    "Alignment::Right",
    "Horizontal::Left",
    "Horizontal::Right",
)


def physical_needles(body: str) -> list[str]:
    return [n for n in _PHYSICAL_ALIGN if n in body]


def physical_align_hits(root: Path) -> list[str]:
    """Physical left/right in chrome constructors (Firefox start/end)."""
    hits: list[str] = []
    paths = [
        root / "src" / "widget.rs",
        root / "src" / "pattern.rs",
        root / "src" / "scroll.rs",
        *sorted((root / "src" / "layout").glob("*.rs")),
    ]
    for path in paths:
        if not path.is_file():
            continue
        for needle in physical_needles(_production_src(path)):
            hits.append(f"{path.relative_to(root)}:{needle}")
    return hits


def ltr_island_hits(root: Path) -> list[str]:
    """Code constructors must stay left-to-right (Firefox LTR islands)."""
    hits: list[str] = []
    src = _production_src(root / "src" / "widget.rs")
    for fn, nxt in (
        ("pub fn code_block", "pub fn hyperlink"),
        ("pub fn highlighted_code", "fn editor_frame"),
    ):
        if fn not in src or nxt not in src:
            hits.append(f"missing {fn}")
            continue
        body = src.split(fn, 1)[1].split(nxt, 1)[0]
        for needle in ("align_start", "align_end", "i18n::order"):
            if needle in body:
                hits.append(f"{fn}:{needle}")
    return hits


def eastern_digit_problems(body: str) -> list[str]:
    hits: list[str] = []
    if "Direction::Rtl" not in body:
        hits.append("clock_digits ignores Direction::Rtl")
    if "'٠'" not in body or "'٩'" not in body:
        hits.append("clock_digits missing Eastern Arabic digits")
    return hits


def eastern_digit_hits(root: Path) -> list[str]:
    """Rtl clocks map Western digits to Eastern Arabic (Firefox ar/ur/fa)."""
    src = _production_src(root / "src" / "widget.rs")
    if "fn clock_digits" not in src or "fn time_colon" not in src:
        return ["missing clock_digits"]
    body = src.split("fn clock_digits", 1)[1].split("fn time_colon", 1)[0]
    return eastern_digit_problems(body)


def leftover_english_in_gallery(root: Path) -> list[str]:
    """Return leftover painted English literals still in gallery or library constructors."""
    src = (root / "icedtea-gallery" / "src" / "main.rs").read_text(encoding="utf-8")
    product = src.split("fn handled_ids()")[0]
    hits = [phrase for phrase in LEFTOVER_ENGLISH if phrase in product]
    lib = (root / "src" / "pattern.rs").read_text(encoding="utf-8")
    pal = (
        lib.split("pub fn command_palette_view")[1].split("pub fn status_page")[0]
        if "pub fn command_palette_view" in lib
        else ""
    )
    tool = (
        lib.split("pub fn tool_panel")[1].split("pub fn drawer")[0]
        if "pub fn tool_panel" in lib
        else ""
    )
    if "Type a command" in pal:
        hits.append("Type a command")
    if '"Dock"' in tool:
        hits.append("Dock")
    cm = (
        lib.split("pub fn context_menu")[1].split("pub fn inspector")[0]
        if "pub fn context_menu" in lib
        else ""
    )
    if "text(a.title" in cm:
        title_only = cm.split("text(a.title")[1].split(")")[0]
        if "Length::Fill" in title_only:
            hits.append("context_menu Fill+align text")
    return hits


def _src_row(name: str, title: str, hits: list[str]) -> dict:
    return {
        "name": name,
        "title": title,
        "score": "ok" if not hits else "broken",
        "detail": "none" if not hits else "found: " + "; ".join(hits)[:200],
    }


def run_rtl_source_checks(root: Path) -> list[dict]:
    """Drive Firefox/Microsoft direction checks. One row per beat."""
    leftover = leftover_english_in_gallery(root)
    rows: list[dict] = [
        _src_row(
            "leftover-src",
            "leftover English in gallery source",
            leftover,
        ),
        _src_row(
            "physical-align",
            "chrome uses start/end (no physical left/right)",
            physical_align_hits(root),
        ),
        _src_row(
            "ltr-islands",
            "code stays left-to-right",
            ltr_island_hits(root),
        ),
        _src_row(
            "digits-eastern",
            "Arabic/Urdu clocks use Eastern digits",
            eastern_digit_hits(root),
        ),
    ]
    checks = [
        (
            "layout-rails",
            ["cargo", "test", "-p", "icedtea", "--lib", "rtl_rails"],
            "list/scroll rails follow direction",
        ),
        (
            "layout-align",
            ["cargo", "test", "-p", "icedtea", "--lib", "rtl_tree"],
            "tree start-align and closed mark",
        ),
        (
            "layout-chevron",
            ["cargo", "test", "-p", "icedtea", "--lib", "rtl_pick"],
            "pick/disclosure mark on the end",
        ),
        (
            "layout-controls",
            ["cargo", "test", "-p", "icedtea", "--lib", "rtl_checkbox"],
            "checkbox and button-group follow direction",
        ),
        (
            "layout-button-face",
            ["cargo", "test", "-p", "icedtea", "--lib", "rtl_themed_button"],
            "themed button keeps a right-to-left title",
        ),
        (
            "copy",
            [
                "cargo",
                "test",
                "-p",
                "icedtea-gallery",
                "--bin",
                "icedtea-gallery",
                "painted_gallery",
            ],
            "gallery leftover-English completeness",
        ),
        (
            "copy-keys",
            [
                "cargo",
                "test",
                "-p",
                "icedtea-gallery",
                "--bin",
                "icedtea-gallery",
                "every_locale",
            ],
            "six-locale catalog key fill",
        ),
    ]
    for name, cmd, title in checks:
        r = subprocess.run(
            cmd,
            cwd=root,
            check=False,
            text=True,
            capture_output=True,
        )
        ok = r.returncode == 0
        rows.append(
            {
                "name": name,
                "title": title,
                "score": "ok" if ok else "broken",
                "detail": title if ok else (r.stderr[-400:] or r.stdout[-400:]),
            }
        )
    return rows


def text_mass_side(path: Path) -> str:
    """Return left, right, or none for light text mass in the mid content band."""
    from PIL import Image

    im = Image.open(path).convert("RGB")
    w, h = im.size
    if w < 200 or h < 200:
        return "none"
    px = im.load()
    y0, y1 = int(h * 0.35), int(h * 0.65)
    nav = 320 if w >= 1200 else 0
    x_left, x_right = 48, max(49, w - nav)
    if y1 - y0 < 40 or x_right - x_left < 80:
        return "none"
    third = (x_right - x_left) // 3
    left_n = right_n = 0
    for x in range(x_left, x_left + third):
        for y in range(y0, y1):
            r, g, b = px[x, y]
            if 0.299 * r + 0.587 * g + 0.114 * b >= 150.0:
                left_n += 1
    for x in range(x_right - third, x_right):
        for y in range(y0, y1):
            r, g, b = px[x, y]
            if 0.299 * r + 0.587 * g + 0.114 * b >= 150.0:
                right_n += 1
    if right_n > left_n * 1.15:
        return "right"
    if left_n > right_n * 1.15:
        return "left"
    return "none"


def _button_pads(
    px, x0: int, x1: int, y0: int, y1: int
) -> list[tuple[int, int, int, int]]:
    """Saturated rectangles sized like filled buttons, not 16px checks."""

    def lum(c: tuple[int, int, int]) -> float:
        return 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]

    def sat(c: tuple[int, int, int]) -> bool:
        r, g, b = c
        mx, mn = max(r, g, b), min(r, g, b)
        return (mx - mn) >= 50 and 35.0 <= lum(c) <= 190.0

    def pad_pixel(c: tuple[int, int, int]) -> bool:
        # Light ink sits inside the pad; do not split the rectangle.
        return sat(c) or lum(c) >= 180.0

    runs_by_y: list[list[tuple[int, int]]] = []
    for y in range(y0, y1):
        runs: list[tuple[int, int]] = []
        x = x0
        while x < x1:
            if pad_pixel(px[x, y]):
                x2 = x + 1
                while x2 < x1 and pad_pixel(px[x2, y]):
                    x2 += 1
                if x2 - x >= 36:
                    runs.append((x, x2))
                x = x2
            else:
                x += 1
        runs_by_y.append(runs)

    used = [[False] * len(runs) for runs in runs_by_y]
    pads: list[tuple[int, int, int, int]] = []
    for yi, runs in enumerate(runs_by_y):
        for ri, (xa, xb) in enumerate(runs):
            if used[yi][ri]:
                continue
            top = y0 + yi
            bot = top + 1
            left, right = xa, xb
            used[yi][ri] = True
            yj = yi + 1
            while yj < len(runs_by_y):
                hit: tuple[int, int, int] | None = None
                for rj, (ca, cb) in enumerate(runs_by_y[yj]):
                    if used[yj][rj]:
                        continue
                    if min(right, cb) - max(left, ca) >= 28:
                        hit = (rj, ca, cb)
                        break
                if hit is None:
                    break
                rj, ca, cb = hit
                used[yj][rj] = True
                left = min(left, ca)
                right = max(right, cb)
                bot += 1
                yj += 1
            pw, ph = right - left, bot - top
            if 36 <= pw <= 220 and 20 <= ph <= 44:
                pads.append((left, top, pw, ph))
    return pads


def control_faces_have_label_ink(path: Path) -> bool:
    """True when filled button pads have light ink inside, not beside a checkbox."""
    from PIL import Image

    im = Image.open(path).convert("RGB")
    w, h = im.size
    if w < 200 or h < 200:
        return False
    px = im.load()
    nav = 340 if w >= 1200 else 0
    x0, x1 = 48, max(49, w - nav)
    y0, y1 = min(170, h // 3), h - 48
    if y1 - y0 < 40 or x1 - x0 < 80:
        return False
    pads = _button_pads(px, x0, x1, y0, y1)
    if not pads:
        return False

    def lum(c: tuple[int, int, int]) -> float:
        return 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]

    labeled = 0
    for left, top, pw, ph in pads:
        ink = 0
        inset = 3
        for x in range(left + inset, left + pw - inset):
            for y in range(top + inset, top + ph - inset):
                if lum(px[x, y]) >= 180.0:
                    ink += 1
        if ink >= 8:
            labeled += 1
    return labeled >= 1 and labeled * 2 >= len(pads)


def score_rtl_shots(steps: list[dict], out: Path) -> list[dict]:
    """Rail on the left; list/tree/sections titles start-align on the right."""
    rows: list[dict] = []
    for s in steps:
        if s.get("kind") != "idle" or s.get("shot") is None:
            continue
        page = s.get("page") or ""
        shot = out / s["shot"]
        if page in RTL_RAIL_PAGES:
            if not shot.is_file():
                rows.append(
                    {
                        "name": f"rail-{page}",
                        "title": f"{page} rail",
                        "score": "broken",
                        "detail": f"missing {s['shot']}",
                    }
                )
            else:
                side = rail_side(shot)
                rows.append(
                    {
                        "name": f"rail-{page}",
                        "title": f"{page} rail on the end (left)",
                        "score": "ok" if side == "left" else "broken",
                        "detail": f"detected {side}",
                    }
                )
        if page == "controls":
            if not shot.is_file():
                rows.append(
                    {
                        "name": "faces-controls",
                        "title": "controls faces carry label ink",
                        "score": "broken",
                        "detail": f"missing {s['shot']}",
                    }
                )
            else:
                ink = control_faces_have_label_ink(shot)
                rows.append(
                    {
                        "name": "faces-controls",
                        "title": "controls faces carry label ink",
                        "score": "ok" if ink else "broken",
                        "detail": "label ink on filled pads"
                        if ink
                        else "filled pads have no label ink",
                    }
                )
        if page in RTL_ALIGN_PAGES:
            if not shot.is_file():
                rows.append(
                    {
                        "name": f"align-{page}",
                        "title": f"{page} start-align (right)",
                        "score": "broken",
                        "detail": f"missing {s['shot']}",
                    }
                )
            else:
                side = text_mass_side(shot)
                rows.append(
                    {
                        "name": f"align-{page}",
                        "title": f"{page} titles on the start (right)",
                        "score": "ok" if side == "right" else "broken",
                        "detail": f"detected {side}",
                    }
                )
    return rows


def write_rtl_score(out: Path, rows: list[dict]) -> bool:
    """Write SCORE.md. Return True when no row is broken."""
    broken = [r for r in rows if r["score"] == "broken"]
    lines = [
        "# Direction gallery QA score",
        "",
        "Bar: `.grok/skills/gallery-qa/references/rtl.md`",
        "(Firefox RTL Guidelines + Microsoft bidirectional / FlowDirection).",
        "broken = fail the command.",
        "",
        "| Check | Score | Detail |",
        "| --- | --- | --- |",
    ]
    for r in rows:
        detail = (r.get("detail") or "").replace("|", "/").replace("\n", " ")[:160]
        lines.append(f"| {r['title']} | **{r['score']}** | {detail} |")
    lines.extend(
        [
            "",
            f"broken: {len(broken)}",
            "",
        ]
    )
    (out / "SCORE.md").write_text("\n".join(lines), encoding="utf-8")
    return not broken


def live_clip_pass(
    root: Path,
    out: Path,
    *,
    release: bool,
    no_build: bool,
    backend: str,
    display_num: int | None,
    screen_w: int,
    screen_h: int,
    client_w: int,
    client_h: int,
) -> int:
    """Wheel and key the List and Table clips. Shots under out/shots/."""
    shots = out / "shots"
    work = out / "work"
    shots.mkdir(parents=True, exist_ok=True)
    work.mkdir(parents=True, exist_ok=True)
    if not no_build:
        build_gallery(root, release)
    binary = resolve_binary(root, release)
    nested = NestedDisplay(
        backend=backend,
        width=screen_w,
        height=screen_h,
        display_num=display_num,
        host_display=os.environ.get("DISPLAY"),
    )
    display = nested.start()
    env = os.environ.copy()
    env["DISPLAY"] = display
    env.pop("WAYLAND_DISPLAY", None)
    env["ICEDTEA_GALLERY_TOUR"] = "1"
    cmdfile = work / "tour.cmd"
    ackfile = work / "tour.ack"
    inject = work / "inject"
    env["ICEDTEA_GALLERY_TOUR_CMD"] = str(cmdfile)
    env["ICEDTEA_GALLERY_TOUR_ACK"] = str(ackfile)
    env["ICEDTEA_GALLERY_INJECT"] = str(inject)
    for p in (cmdfile, ackfile, inject):
        if p.exists():
            p.unlink()
    proc = subprocess.Popen(
        [str(binary)],
        cwd=root,
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )
    try:
        def goto(beat: int) -> None:
            cmdfile.write_text(f"{beat}\n", encoding="utf-8")
            if not wait_file(ackfile, lambda t: t.strip() == str(beat), timeout_s=20.0):
                raise SystemExit(f"gallery did not ack beat {beat}")
            time.sleep(0.5)

        goto(9)
        wid = find_window_id(display, proc.pid)
        place_window(display, wid, 40, 48, client_w, client_h)
        time.sleep(0.3)
        xdotool(display, "windowactivate", wid)
        fx, fy, _fw, _fh = window_frame(display, wid)
        # Content right of the nav (~360). Virtual column is upper;
        # list_view sits under the in-page Search / All radios.
        vc_x, vc_y = fx + 920, fy + 360
        list_x, list_y = fx + 920, fy + 700
        list_search_x, list_search_y = fx + 560, fy + 580
        table_x, table_y = fx + 920, fy + 360

        capture_window(display, wid, shots / "list-00-idle.png")
        wheel_at(display, vc_x, vc_y, clicks=18, down=True)
        time.sleep(0.45)
        capture_window(display, wid, shots / "list-01-vc-wheel.png")
        wheel_at(display, list_x, list_y, clicks=20, down=True)
        time.sleep(0.45)
        capture_window(display, wid, shots / "list-02-list-wheel.png")
        click_at(display, list_x, list_y)
        time.sleep(0.1)
        key_repeat(display, "Down", 16)
        time.sleep(0.45)
        capture_window(display, wid, shots / "list-03-arrow-down.png")
        click_at(display, list_search_x, list_search_y)
        time.sleep(0.15)
        xdotool(display, "type", "--delay", "20", "xyzzy")
        time.sleep(0.55)
        capture_window(display, wid, shots / "list-04-filter.png")

        goto(12)
        time.sleep(0.4)
        capture_window(display, wid, shots / "table-00-idle.png")
        click_at(display, table_x, table_y)
        time.sleep(0.15)
        wheel_at(display, table_x, table_y, clicks=20, down=True)
        time.sleep(0.45)
        capture_window(display, wid, shots / "table-01-wheel.png")
        click_at(display, table_x, table_y)
        time.sleep(0.1)
        key_repeat(display, "Down", 16)
        time.sleep(0.45)
        capture_window(display, wid, shots / "table-02-arrow-down.png")
        print(f"live-clip shots {shots}", file=sys.stderr)
        return 0
    finally:
        if proc.poll() is None:
            proc.send_signal(signal.SIGTERM)
            try:
                proc.wait(timeout=3)
            except subprocess.TimeoutExpired:
                proc.kill()
        nested.stop()


# Catalog page order plus the extra tour beats in icedtea-gallery.
_TOUR_PAGES = (
    "controls",
    "fields",
    "readout",
    "type",
    "markdown",
    "code",
    "image",
    "selectable",
    "list",
    "log",
    "grid",
    "table",
    "tree",
    "sections",
    "theme",
    "colors",
    "keys",
    "marks",
    "chrome-rows",
    "feedback",
    "dialogs",
    "list-detail",
    "inspector",
    "workspace",
    "navigation",
    "tab-view",
    "preferences",
    "about",
    "status-page",
    "palette",
    "main-window",
    "motion",
    "expand-motion",
)
_TOUR_EXTRAS = {"code": 1, "motion": 3, "expand-motion": 1}


def tour_index(*, page: str | None = None, light: bool = False) -> int:
    """0-based tour beat. Matches `tour_beat` in icedtea-gallery."""
    n = 0
    light_at = _TOUR_PAGES.index("theme") + 1
    for i in range(len(_TOUR_PAGES) + 1):
        if i == light_at:
            cur = "theme"
            is_light = True
        else:
            cur = _TOUR_PAGES[i] if i < light_at else _TOUR_PAGES[i - 1]
            is_light = False
        if light and is_light:
            return n
        if page is not None and cur == page and not is_light:
            return n
        n += 1 + _TOUR_EXTRAS.get(cur, 0)
    raise KeyError(page or "light")


def _assert_tour_index() -> None:
    if tour_index(page="list") != 9:
        raise SystemExit(f"list beat drifted: {tour_index(page='list')}")
    if tour_index(page="table") != 12:
        raise SystemExit(f"table beat drifted: {tour_index(page='table')}")


def _srt_stamp(seconds: float) -> str:
    h = int(seconds // 3600)
    m = int((seconds % 3600) // 60)
    sec = seconds - h * 3600 - m * 60
    return f"{h:02d}:{m:02d}:{sec:06.3f}".replace(".", ",")


def _mouse_xy(display: str) -> tuple[int, int]:
    r = xdotool(display, "getmouselocation", "--shell")
    x = y = 0
    for line in r.stdout.splitlines():
        if line.startswith("X="):
            x = int(line.split("=", 1)[1])
        elif line.startswith("Y="):
            y = int(line.split("=", 1)[1])
    return x, y


def glide_to(display: str, x: int, y: int, *, ms: int = 280) -> None:
    """Move the pointer in steps so the live grab shows the path."""
    x0, y0 = _mouse_xy(display)
    steps = max(6, ms // 20)
    for i in range(1, steps + 1):
        xi = int(x0 + (x - x0) * i / steps)
        yi = int(y0 + (y - y0) * i / steps)
        xdotool(display, "mousemove", str(xi), str(yi))
        time.sleep(0.02)


class _Captions:
    def __init__(self, path: Path, t0: float) -> None:
        self.path = path
        self.t0 = t0
        self.n = 0
        self.last_t = 0.0
        self.last_cap = ""
        self.path.write_text("", encoding="utf-8")

    def _now(self) -> float:
        return time.time() - self.t0

    def _append(self, start: float, end: float, text: str) -> None:
        self.n += 1
        block = (
            f"{self.n}\n{_srt_stamp(start)} --> {_srt_stamp(end)}\n{text}\n\n"
        )
        with self.path.open("a", encoding="utf-8") as fh:
            fh.write(block)

    def set(self, text: str) -> None:
        now = self._now()
        if self.last_cap:
            self._append(self.last_t, now, self.last_cap)
            self.last_t = now
        else:
            self.last_t = 0.0
        self.last_cap = text

    def close(self) -> None:
        now = max(self._now(), self.last_t + 0.2)
        if self.last_cap:
            self._append(self.last_t, now, self.last_cap)
            self.last_cap = ""


def record_gif_demo(
    display: str,
    wid: str,
    cmdfile: Path,
    ackfile: Path,
    srt: Path,
    t0: float,
    inject: Path | None = None,
) -> int:
    """Live pointer demo for `just gallery-gif`. Gallery must already be up."""
    _assert_tour_index()
    xdotool(display, "windowactivate", wid)
    time.sleep(0.15)
    fx, fy, _fw, _fh = window_frame(display, wid)
    # Client: look strip + page title sit above hosts. Nav is ~500 wide.
    primary_x, primary_y = fx + 560, fy + 240
    search_x, search_y = fx + 640, fy + 280
    vc_x, vc_y = fx + 920, fy + 400
    list_x, list_y = fx + 920, fy + 760
    table_x, table_y = fx + 920, fy + 400
    pal_x, pal_y = fx + 900, fy + 370
    caps = _Captions(srt, t0)

    def goto(page: str | None = None, *, light: bool = False) -> None:
        beat = tour_index(page=page, light=light)
        cmdfile.write_text(f"{beat}\n", encoding="utf-8")
        if not wait_file(ackfile, lambda t: t.strip() == str(beat), timeout_s=20.0):
            raise SystemExit(f"gallery did not ack beat {beat}")
        time.sleep(0.7)

    def send_inject(script: str) -> None:
        if inject is None:
            raise SystemExit("demo inject file is missing")
        ack = inject.with_suffix(".ack")
        if ack.exists():
            ack.unlink()
        inject.write_text(script, encoding="utf-8")
        if not wait_file(ack, lambda t: t.strip() != "", timeout_s=3.0):
            raise SystemExit(f"gallery did not ack inject {script!r}")

    def show(text: str) -> None:
        caps.set(text)
        time.sleep(0.8)

    try:
        goto(page="controls")
        send_inject("note Primary\n")
        show("Primary writes Primary on the status bar")
        glide_to(display, primary_x, primary_y, ms=420)
        click_at(display, primary_x, primary_y)
        time.sleep(1.4)

        goto(page="fields")
        send_inject("query in\nsearch-go\n")
        show("Search filters Inbox as you type")
        glide_to(display, search_x, search_y, ms=420)
        click_at(display, search_x, search_y)
        time.sleep(0.2)
        xdotool(display, "type", "--delay", "80", "in")
        time.sleep(1.6)

        goto(page="list")
        send_inject("list 4\n")
        show("A virtual list selects a row in a thousand")
        glide_to(display, vc_x, vc_y, ms=420)
        wheel_at(display, vc_x, vc_y, clicks=10, down=True, delay_ms=50)
        time.sleep(0.6)
        glide_to(display, list_x, list_y, ms=380)
        click_at(display, list_x, list_y)
        time.sleep(1.4)

        goto(page="table")
        send_inject("table 3\n")
        show("The table keeps Name in view while you scroll")
        glide_to(display, table_x, table_y, ms=420)
        click_at(display, table_x, table_y)
        time.sleep(0.2)
        wheel_at(display, table_x, table_y, clicks=12, down=True, delay_ms=50)
        time.sleep(1.5)

        goto(page="palette")
        send_inject("pal-query save\n")
        show("The command palette filters the Action table")
        glide_to(display, pal_x, pal_y, ms=420)
        click_at(display, pal_x, pal_y)
        time.sleep(0.2)
        xdotool(display, "type", "--delay", "80", "save")
        time.sleep(1.8)

        goto(light=True)
        send_inject("appearance light\n")
        show("Light and dark share the same color roles")
        time.sleep(2.0)
        caps.close()
    except Exception:
        caps.close()
        raise
    print(f"gallery-gif: wrote {srt} ({caps.n} captions)", file=sys.stderr)
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--out",
        type=Path,
        default=None,
        help="Output directory (default: tmp/gallery-qa/<timestamp>/)",
    )
    ap.add_argument(
        "--backend",
        choices=("xephyr", "host", "xvfb"),
        default="xephyr",
    )
    ap.add_argument("--display-num", type=int, default=None)
    ap.add_argument("--settle-ms", type=int, default=450)
    ap.add_argument("--release", action="store_true", help="Build/use release binary")
    ap.add_argument("--no-build", action="store_true")
    ap.add_argument(
        "--beats",
        default="all",
        help="all | N | start-end | comma list (0-based tour beats)",
    )
    ap.add_argument(
        "--gif",
        type=Path,
        default=None,
        help="Optional ffmpeg GIF from shots (demo package)",
    )
    ap.add_argument(
        "--interact",
        action="store_true",
        help="After each beat, run built-in inject scripts and capture after-state shots",
    )
    ap.add_argument("--client-w", type=int, default=1600)
    ap.add_argument("--client-h", type=int, default=900)
    ap.add_argument("--screen-w", type=int, default=1720)
    ap.add_argument("--screen-h", type=int, default=1080)
    ap.add_argument(
        "--book",
        action="store_true",
        help="Write handbook stills under book/src/images/ from idle constructor frames",
    )
    ap.add_argument(
        "--locale",
        default=None,
        help="Inject language LANG before shots (ar, ur for right-to-left)",
    )
    ap.add_argument(
        "--live-clip",
        action="store_true",
        help="Xephyr wheel/key pass on List and Table (real xdotool wheel)",
    )
    ap.add_argument(
        "--record-demo",
        action="store_true",
        help="Drive the live pointer demo (gallery already running)",
    )
    ap.add_argument("--wid", default=None, help="Window id for --record-demo")
    ap.add_argument("--cmd", type=Path, default=None, help="Tour cmd file")
    ap.add_argument("--ack", type=Path, default=None, help="Tour ack file")
    ap.add_argument("--srt", type=Path, default=None, help="Caption file to write")
    ap.add_argument("--t0", type=float, default=None, help="Epoch start of the live grab")
    ap.add_argument("--inject", type=Path, default=None, help="Inject file for --record-demo")
    args = ap.parse_args()

    if args.record_demo:
        if not _which("xdotool"):
            raise SystemExit("missing xdotool (needed for --record-demo)")
        display = os.environ.get("DISPLAY")
        if not display:
            raise SystemExit("DISPLAY is not set")
        if args.wid is None or args.cmd is None or args.ack is None:
            raise SystemExit("--record-demo needs --wid --cmd --ack --srt --t0")
        if args.srt is None or args.t0 is None:
            raise SystemExit("--record-demo needs --wid --cmd --ack --srt --t0")
        return record_gif_demo(
            display,
            args.wid,
            args.cmd,
            args.ack,
            args.srt,
            args.t0,
            inject=args.inject,
        )

    for cmd in ("wmctrl", "import", "xwininfo", "identify"):
        if not _which(cmd):
            raise SystemExit(f"missing {cmd}")

    root = _repo_root()
    out = args.out
    if out is None:
        out = root / "tmp" / "gallery-qa" / _utc_stamp()
    out = out.resolve()
    shots = out / "shots"
    shots.mkdir(parents=True, exist_ok=True)
    work = out / "work"
    work.mkdir(parents=True, exist_ok=True)

    if args.live_clip:
        if not _which("xdotool"):
            raise SystemExit("missing xdotool (needed for --live-clip wheel)")
        return live_clip_pass(
            root,
            out,
            release=args.release,
            no_build=args.no_build,
            backend=args.backend,
            display_num=args.display_num,
            screen_w=args.screen_w,
            screen_h=args.screen_h,
            client_w=args.client_w,
            client_h=args.client_h,
        )

    if not args.no_build:
        build_gallery(root, args.release)
    binary = resolve_binary(root, args.release)

    git = _run(["git", "-C", str(root), "rev-parse", "--short", "HEAD"]).stdout.strip()

    nested = NestedDisplay(
        backend=args.backend,
        width=args.screen_w,
        height=args.screen_h,
        display_num=args.display_num,
        host_display=os.environ.get("DISPLAY"),
    )
    t_all0 = _now_ms()
    display = nested.start()
    env = os.environ.copy()
    env["DISPLAY"] = display
    env.pop("WAYLAND_DISPLAY", None)

    # Kill stray gallery on this display if any
    _run(["pkill", "-x", "icedtea-gallery"], check=False)
    time.sleep(0.2)

    lenfile = work / "tour_len"
    cmdfile = work / "cmd"
    ackfile = work / "ack"
    injectfile = work / "inject"
    cmdfile.write_text("0\n", encoding="utf-8")
    injectfile.write_text("", encoding="utf-8")
    for p in (lenfile, ackfile, injectfile.with_suffix(".ack")):
        if p.exists():
            p.unlink()

    env["ICEDTEA_GALLERY_TOUR"] = "1"
    env["ICEDTEA_GALLERY_TOUR_LEN_FILE"] = str(lenfile)
    env["ICEDTEA_GALLERY_TOUR_CMD"] = str(cmdfile)
    env["ICEDTEA_GALLERY_TOUR_ACK"] = str(ackfile)
    env["ICEDTEA_GALLERY_INJECT"] = str(injectfile)
    if args.backend != "host":
        env["ICEDTEA_GALLERY_NESTED"] = "1"

    t_boot0 = _now_ms()
    print(f"starting {binary}", file=sys.stderr)
    gallery = subprocess.Popen(
        [str(binary)],
        cwd=root,
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )

    def cleanup() -> None:
        if gallery.poll() is None:
            gallery.terminate()
            try:
                gallery.wait(timeout=3)
            except subprocess.TimeoutExpired:
                gallery.kill()
        nested.stop()

    try:
        if not wait_file(
            lenfile,
            lambda t: t.strip().isdigit() and int(t.strip()) > 1,
            timeout_s=15.0,
        ):
            raise SystemExit("gallery did not report tour length")
        tour_len = int(lenfile.read_text(encoding="utf-8").strip())
        print(f"tour_len={tour_len}", file=sys.stderr)

        wid = find_window_id(display, gallery.pid)
        place_window(display, wid, 40, 48, args.client_w, args.client_h)

        if not wait_file(ackfile, lambda t: t.strip() == "0", timeout_s=10.0):
            raise SystemExit("gallery did not acknowledge beat 0")
        if args.locale:
            print(f"locale {args.locale}", file=sys.stderr)
            try:
                inject_script(injectfile, f"language {args.locale}\n")
            except TimeoutError as exc:
                raise SystemExit(f"locale inject failed: {exc}") from exc
            time.sleep(max(0.45, args.settle_ms / 1000.0))
        boot_ms = _now_ms() - t_boot0

        # Parse --beats
        if args.beats.strip().casefold() == "all":
            beats = list(range(tour_len))
        elif "-" in args.beats and "," not in args.beats:
            a, b = args.beats.split("-", 1)
            beats = list(range(int(a), int(b) + 1))
        elif "," in args.beats:
            beats = [int(x.strip()) for x in args.beats.split(",") if x.strip()]
        else:
            beats = [int(args.beats.strip())]
        for b in beats:
            if b < 0 or b >= tour_len:
                raise SystemExit(f"beat {b} out of range 0..{tour_len - 1}")

        steps: list[dict] = []
        steps_path = out / "steps.jsonl"
        settle = max(0, args.settle_ms) / 1000.0
        shot_i = 0

        def record_shot(
            *,
            beat: int,
            caption: str,
            face: str,
            page: str,
            kind: str,
            name_extra: str,
            t0: int,
            inject: str | None = None,
            expect: str | None = None,
        ) -> None:
            nonlocal shot_i
            place_window(display, wid, 40, 48, args.client_w, args.client_h)
            pointer_clear_hover(display)
            time.sleep(0.05)
            name = f"{shot_i:02d}-beat{beat:02d}-{kind}-{slug(name_extra)[:40]}"
            shot_rel = f"shots/{name}.png"
            shot_path = out / shot_rel
            try:
                cap = capture_window(display, wid, shot_path)
                err = None
            except Exception as exc:
                cap = {"ms": None, "size": 0, "geometry": None}
                err = str(exc)
                print(f"capture failed: {exc}", file=sys.stderr)
            step = {
                "index": shot_i,
                "beat": beat,
                "page": page,
                "caption": caption,
                "theme": face,
                "kind": kind,
                "shot": shot_rel,
                "capture_ms": cap.get("ms"),
                "geometry": cap.get("geometry"),
                "step_ms": _now_ms() - t0,
                "inject": inject,
                "expect": expect,
                "error": err,
            }
            steps.append(step)
            with steps_path.open("a", encoding="utf-8") as f:
                f.write(json.dumps(step) + "\n")
            print(
                f"[{shot_i}] beat={beat} {kind} {caption!r} -> {shot_rel}",
                file=sys.stderr,
            )
            shot_i += 1

        for beat in beats:
            cmdfile.write_text(f"{beat}\n", encoding="utf-8")
            t_step0 = _now_ms()
            if not wait_file(
                ackfile,
                lambda t, b=beat: t.strip() == str(b),
                timeout_s=10.0,
            ):
                raise SystemExit(f"gallery did not acknowledge beat {beat}")
            caption = ""
            face = ""
            cap_path = ackfile.with_suffix(".caption")
            face_path = ackfile.with_suffix(".face")
            if cap_path.is_file():
                caption = cap_path.read_text(encoding="utf-8", errors="replace").strip()
            if face_path.is_file():
                face = face_path.read_text(encoding="utf-8", errors="replace").strip()
            page = (
                slug(caption.split(":", 1)[0])
                if ":" in caption
                else (slug(caption) if caption else f"beat-{beat}")
            )

            if args.book and page not in BOOK_STILLS:
                continue

            time.sleep(settle)
            record_shot(
                beat=beat,
                caption=caption,
                face=face,
                page=page,
                kind="idle",
                name_extra=caption or page,
                t0=t_step0,
            )

            if args.interact:
                for ix in interactions_for_caption(caption):
                    t_ix = _now_ms()
                    try:
                        applied = inject_script(injectfile, ix["script"])
                    except TimeoutError as exc:
                        print(f"inject failed: {exc}", file=sys.stderr)
                        steps.append(
                            {
                                "index": shot_i,
                                "beat": beat,
                                "kind": "inject-error",
                                "error": str(exc),
                                "inject": ix["script"],
                                "name": ix["name"],
                            }
                        )
                        shot_i += 1
                        continue
                    time.sleep(settle)
                    record_shot(
                        beat=beat,
                        caption=caption,
                        face=face,
                        page=page,
                        kind="after-interact",
                        name_extra=ix["name"],
                        t0=t_ix,
                        inject=ix["script"].strip(),
                        expect=ix.get("expect"),
                    )
                    print(
                        f"  interact {ix['name']}: applied={applied} expect={ix.get('expect')!r}",
                        file=sys.stderr,
                    )

        total_ms = _now_ms() - t_all0
        step_ms = [s["step_ms"] for s in steps if s.get("step_ms") is not None]
        timings = {
            "boot_ms": boot_ms,
            "total_ms": total_ms,
            "mean_step_ms": round(sum(step_ms) / len(step_ms), 1) if step_ms else None,
            "settle_ms": args.settle_ms,
            "steps": len(steps),
        }
        meta = {
            "backend": args.backend,
            "display": display,
            "binary": str(binary),
            "tour_len": tour_len,
            "settle_ms": args.settle_ms,
            "git": git,
            "out": str(out),
            "client": f"{args.client_w}x{args.client_h}",
            "locale": args.locale,
        }
        (out / "timings.json").write_text(
            json.dumps(timings, indent=2) + "\n", encoding="utf-8"
        )
        (out / "meta.json").write_text(
            json.dumps(meta, indent=2) + "\n", encoding="utf-8"
        )
        write_capture_md(out, meta=meta, steps=steps, timings=timings)

        if args.book:
            images = root / "book" / "src" / "images"
            written: set[str] = set()
            for s in steps:
                if s.get("kind") != "idle" or s.get("shot") is None:
                    continue
                dest_name = BOOK_STILLS.get(s.get("page", ""))
                if dest_name is None:
                    continue
                publish_book_still(out / s["shot"], images / dest_name)
                written.add(dest_name)
            missing = sorted(set(BOOK_STILLS.values()) - written)
            if missing:
                raise SystemExit(
                    "book stills missing after tour: " + ", ".join(missing)
                )
            if gallery.poll() is None:
                gallery.terminate()
                try:
                    gallery.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    gallery.kill()
            capture_hello(
                root,
                display,
                images / BOOK_HELLO_STILL,
                release=args.release,
                no_build=args.no_build,
            )

        if args.gif is not None:
            gif = args.gif if args.gif.is_absolute() else out / args.gif
            pngs = sorted(shots.glob("*.png"))
            if pngs and _which("ffmpeg"):
                # Short hold per frame for demo playback
                list_file = work / "ffmpeg_list.txt"
                lines = []
                for p in pngs:
                    lines.append(f"file '{p}'")
                    lines.append("duration 1.2")
                if pngs:
                    lines.append(f"file '{pngs[-1]}'")
                list_file.write_text("\n".join(lines) + "\n", encoding="utf-8")
                fr = _run(
                    [
                        "ffmpeg",
                        "-y",
                        "-f",
                        "concat",
                        "-safe",
                        "0",
                        "-i",
                        str(list_file),
                        "-vf",
                        "fps=8,scale=960:-1:flags=lanczos",
                        "-loop",
                        "0",
                        str(gif),
                    ],
                    timeout=120.0,
                )
                if fr.returncode != 0:
                    print(f"ffmpeg gif failed: {fr.stderr[-400:]}", file=sys.stderr)
                else:
                    print(f"wrote {gif}", file=sys.stderr)
            else:
                print("skip --gif (no shots or ffmpeg)", file=sys.stderr)

        print(f"out_dir={out}", file=sys.stderr)
        print(
            f"display={display} backend={args.backend} tour_len={tour_len}",
            file=sys.stderr,
        )
        print(json.dumps({"out": str(out), "timings": timings, "meta": meta}))
        capture_ok = all(s.get("error") is None for s in steps)
        locale = (args.locale or "").split("-", 1)[0].casefold()
        if locale:
            print(
                "scoring direction beats (references/rtl.md)"
                + (
                    ", rails, start-align, faces"
                    if locale in {"ar", "ur", "fa", "he"}
                    else ""
                ),
                file=sys.stderr,
            )
            score_rows = run_rtl_source_checks(root)
            if locale in {"ar", "ur", "fa", "he"}:
                score_rows.extend(score_rtl_shots(steps, out))
            score_ok = write_rtl_score(out, score_rows)
            print(f"wrote {out / 'SCORE.md'} ok={score_ok}", file=sys.stderr)
            if not score_ok:
                return 3
        return 0 if capture_ok else 2
    finally:
        cleanup()


def _self_check() -> None:
    names = {x["name"] for x in interactions_for_caption("Selectable: drag to copy")}
    if "table-sort" in names:
        raise SystemExit("table: must not match Selectable:")
    names = {
        x["name"] for x in interactions_for_caption("Table: frozen leading columns")
    }
    if "table-sort" not in names:
        raise SystemExit("table: must match Table:")
    from PIL import Image

    tmp = Path(os.environ.get("TMPDIR", "/tmp")) / "icedtea-qa-rail-selfcheck"
    tmp.mkdir(parents=True, exist_ok=True)
    left = tmp / "rail-left.png"
    right = tmp / "rail-right.png"
    img = Image.new("RGB", (400, 300), (20, 20, 20))
    for x in range(40, 52):
        for y in range(80, 260):
            img.putpixel((x, y), (90, 90, 90))
    img.save(left)
    img = Image.new("RGB", (400, 300), (20, 20, 20))
    for x in range(330, 342):
        for y in range(80, 260):
            img.putpixel((x, y), (90, 90, 90))
    img.save(right)
    if rail_side(left) != "left":
        raise SystemExit(f"rail_side left fixture -> {rail_side(left)}")
    if rail_side(right) != "right":
        raise SystemExit(f"rail_side right fixture -> {rail_side(right)}")
    align_r = tmp / "align-right.png"
    img = Image.new("RGB", (800, 500), (20, 20, 20))
    for x in range(520, 700):
        for y in range(180, 320):
            img.putpixel((x, y), (220, 220, 220))
    img.save(align_r)
    if text_mass_side(align_r) != "right":
        raise SystemExit(f"text_mass_side right fixture -> {text_mass_side(align_r)}")
    blank = tmp / "faces-blank.png"
    img = Image.new("RGB", (800, 500), (20, 20, 20))
    for x in range(80, 180):
        for y in range(200, 236):
            img.putpixel((x, y), (50, 90, 200))
    img.save(blank)
    if control_faces_have_label_ink(blank):
        raise SystemExit("control_faces_have_label_ink blank pad should fail")
    labeled = tmp / "faces-labeled.png"
    img = Image.new("RGB", (800, 500), (20, 20, 20))
    for x in range(80, 180):
        for y in range(200, 236):
            img.putpixel((x, y), (50, 90, 200))
    for x in range(100, 160):
        for y in range(212, 220):
            img.putpixel((x, y), (240, 240, 245))
    img.save(labeled)
    if not control_faces_have_label_ink(labeled):
        raise SystemExit("control_faces_have_label_ink labeled pad should pass")
    check = tmp / "faces-checkbox.png"
    img = Image.new("RGB", (800, 500), (20, 20, 20))
    for x in range(120, 136):
        for y in range(210, 226):
            img.putpixel((x, y), (50, 90, 200))
    for x in range(124, 132):
        for y in range(214, 222):
            img.putpixel((x, y), (240, 240, 245))
    img.save(check)
    if control_faces_have_label_ink(check):
        raise SystemExit("control_faces_have_label_ink checkbox-only should fail")
    if not physical_needles("align_x(Alignment::Left)"):
        raise SystemExit("physical_needles must flag Alignment::Left")
    if physical_needles("align_x(crate::i18n::align_start(tok.direction))"):
        raise SystemExit("physical_needles false positive on align_start")
    if not eastern_digit_problems("if dir != Direction::Ltr { western }"):
        raise SystemExit("eastern_digit_problems must fail without Rtl map")
    if eastern_digit_problems("Direction::Rtl '٠' '٩'"):
        raise SystemExit("eastern_digit_problems false positive")
    root = Path(__file__).resolve().parents[1]
    hits = leftover_english_in_gallery(root)
    if hits:
        raise SystemExit(f"leftover English in gallery source: {hits}")
    phys = physical_align_hits(root)
    if phys:
        raise SystemExit(f"physical left/right in chrome: {phys}")
    islands = ltr_island_hits(root)
    if islands:
        raise SystemExit(f"code LTR island: {islands}")
    digits = eastern_digit_hits(root)
    if digits:
        raise SystemExit(f"Eastern digits: {digits}")


if __name__ == "__main__":
    try:
        _self_check()
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
