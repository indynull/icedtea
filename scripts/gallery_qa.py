#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# ///
"""Gallery QA: tour the icedtea gallery, capture shots, optional interact.

Default Xephyr + metacity. Tour protocol + optional inject scripts.
Writes shots/, steps.jsonl, timings.json, CAPTURE.md under --out.
Does not commit. Does not invent screenshots.

  just gallery-qa
  just gallery-qa --interact --beats 0,8
  just gallery-gif   # ship assets/gallery.gif for README/book only
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
    raise SystemExit(f"timed out waiting for gallery window (pid={pid})")


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
        "script": "check true\nswitch true\nsounds true\nradio 1\nslide 0.75\nsegment 1\nrange 15 90\ngroup 1\n",
        "expect": "Button group visible; Accept and Sounds on; Option B; status Group 1",
    },
    {
        "match": "fields:",
        "name": "search-view-filter",
        "script": "query in\n",
        "expect": "search view hits filter to Inbox",
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
        "script": "list 2\nface card\n",
        "expect": "third list row selected; card face",
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
        "script": "tree-sel 3\n",
        "expect": "lib.rs leaf selected; folders still open",
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
]


def interactions_for_caption(caption: str) -> list[dict[str, str]]:
    c = caption.casefold()
    return [x for x in DEFAULT_INTERACT if x["match"].casefold() in c]


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
    args = ap.parse_args()

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
        }
        (out / "timings.json").write_text(
            json.dumps(timings, indent=2) + "\n", encoding="utf-8"
        )
        (out / "meta.json").write_text(
            json.dumps(meta, indent=2) + "\n", encoding="utf-8"
        )
        write_capture_md(out, meta=meta, steps=steps, timings=timings)

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
        return 0 if all(s.get("error") is None for s in steps) else 2
    finally:
        cleanup()


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
