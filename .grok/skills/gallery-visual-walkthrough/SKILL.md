---
name: gallery-visual-walkthrough
description: >
  Launch icedtea-gallery in isolated Xephyr, drive the built-in tour
  beats, capture timed screenshots per page, optionally build a demo
  GIF, visually review each grab with multimodal image inspection
  (read_file on PNGs), then auto-fix clear pixel/layout/usability/a11y
  defects in icedtea source and re-walk until shippable. Use when the
  user runs /gallery-visual-walkthrough, asks for gallery screenshot
  review, pixel perfection, visual QA, automated gallery demo, or "is
  the gallery ugly / broken".
metadata:
  short-description: "Gallery tour QA + fix loop"
---

# Gallery visual walkthrough

End-to-end **visual** review of the icedtea gallery (every catalog
tour beat), optional demo GIF, and a **fix loop** when the bar is
pixel-level polish plus usability/accessibility. Product eyes on real
pixels — not a unit suite alone.

## How the agent “looks” at screens

There is **no separate vision skill** and **no** `image_gen` for QA.

| Mechanism | Role |
|-----------|------|
| **`read_file` on a `.png` / `.jpg` path** | Host presents the image to the model’s **multimodal** path. You see layout, color, type, clipping. **This is the only approved way to inspect shots.** |
| Filename / `identify` / file size | Metadata only — **never** a substitute for opening the image. |
| `image_gen` / `image_edit` | **Forbidden** for fixing UI. They invent pixels; they do not change icedtea constructors. |
| Unit tests / `just check` | Necessary after code fixes; **not** visual proof. |

If `read_file` cannot open a path, re-capture; do not invent a description.

## Prerequisites

- Host X11 `DISPLAY` (Xephyr nests on it).
- **`Xephyr`**, **`metacity`**, **`wmctrl`**, **`import`**, **`identify`**
  (ImageMagick), **`xwininfo`**. Optional: **`ffmpeg`** for `--gif`.
- Workspace binary: `cargo build -p icedtea-gallery` (script can build).
- Prefer **release** for snappier walks: `--release`.

## Isolation (default)

Matches `scripts/gallery-gif.sh`:

1. **Xephyr** nested display  
2. **metacity** WM inside it  
3. Tour protocol: `ICEDTEA_GALLERY_TOUR` + cmd/ack files  
4. **wmctrl place** + **import** of the gallery window  

| Backend | Flag | Notes |
|---------|------|--------|
| Xephyr | `--backend xephyr` (default) | Isolated; demo-safe |
| Host | `--backend host` | Interferes with the live desktop |
| Xvfb | `--backend xvfb` | Often black without a managed window |

## Inputs

| Arg / env | Meaning |
|-----------|---------|
| `--out DIR` | Default `tmp/gallery-walk/<timestamp>/` under the repo |
| `--backend` | `xephyr` (default), `host`, or `xvfb` |
| `--display-num N` | Nested display number (else free `:3`–`:20`) |
| `--settle-ms N` | Sleep after beat ack before capture (default 450) |
| `--beats` | `all` (default), `N`, `start-end`, or `0,2,5` |
| `--release` | Build/use `target/release/icedtea-gallery` |
| `--no-build` | Use existing binary only |
| `--gif PATH` | Optional ffmpeg GIF from shots (relative to `--out` unless absolute) |

## Agent procedure

### 1. Run the harness

From the **icedtea** repo root:

```bash
# Prefer release for demos:
#   cargo build -p icedtea-gallery --release

python3 .grok/skills/gallery-visual-walkthrough/scripts/gallery_walkthrough.py \
  --out tmp/gallery-walk/latest \
  --gif demo.gif
```

Read stderr/stdout. Note `out_dir=`, `display=`, `backend=`, `tour_len=`,
and any capture errors.

If the script fails before any screenshots, fix environment (binary, X,
tools) and re-run. **Do not invent screenshots.**

### 2. Timings

`timings.json`: `boot_ms`, `total_ms`, `mean_step_ms`, `settle_ms`.
Report as printed — do not invent. `mean_step_ms` includes settle sleep.

### 3. Visual review (mandatory — every shot)

For **each** `shots/*.png` under `out_dir` in step order:

1. Open with the **`read_file` tool** (image path). Multimodal vision
   only works when the tool actually loads the file — **filename-only
   scoring fails**.
2. Score against `references/rubric.md` (ok / ugly / broken).
3. Human-usefulness + a11y bar (in addition to the rubric):
   - Correct page selected (nav + title match caption)
   - Readable type; primary controls not clipped or buried
   - Catalog page not an empty stub
   - No overlapping labels or unusable density
   - Name/role-worthy controls are visible and distinct (not a sea of
     identical muted blocks); disabled state is visually quieter than
     enabled; focus/selection not invisible
4. Write one short note per shot: **ok / ugly / broken** + one sentence why.

Do **not** use `image_gen` / `image_edit` to “fix” the UI.

### 4. Auto-fix loop (when the goal is pixel perfection)

Default when the user asks for polish / pixel perfection / “make it
demo-ready”: **do not stop at a report**. Fix clear defects in **product
source**, then re-walk. Taste-only nits can wait if nothing is broken.

#### 4a. Classify each defect

| Class | Examples | Action |
|-------|----------|--------|
| **Fix now** | Clipped text, wrong page chrome, empty stub that should have content, broken contrast on body text, overlapping controls, missing disabled styling, selection ring invisible, 4/8px grid blatantly violated, dual paint-only demos | Edit `src/` / gallery demo; prove with re-capture |
| **Defer** | Continuous markdown cross-block select, host font metrics, iced upstream paint limits, one-off content length in a fixture | Note in report + `TODO.md` if library-level |
| **Do not “fix”** | Inventing bitmaps with `image_gen`, screenshot photoshop, dual-path gallery fakes | Refuse |

A defect is **fix now** only when vision shows it **and** you can name
the owning constructor/path (`widget::…`, `pattern::…`, gallery
`demo_widget`, tokens in `theme`/`style`).

#### 4b. Fix in the library (not the PNG)

1. Map shot → catalog page → constructor or gallery branch.
2. Change the **shipped** path (density/padding in `density`/`chrome`,
   `style::*`, constructor layout, gallery seed content). Match
   `AGENTS.md` (4px grid, tokens, A11y on public constructors, pure
   gallery demos).
3. Run cheap gates: `cargo test -p icedtea --lib` (or targeted),
   `cargo test -p icedtea-gallery` when gallery changed, `cargo fmt`.
4. **Re-run the harness** on at least the affected beats (and a full
   walk before calling SHIPPABLE):

```bash
python3 .grok/skills/gallery-visual-walkthrough/scripts/gallery_walkthrough.py \
  --beats <affected> --out tmp/gallery-walk/recheck
```

5. **`read_file` the new shots** for those beats. If still ugly/broken,
   fix again (cap: **three** fix→rewalk cycles on the same defect, then
   stop, document residual, and ask).
6. When shell/pages change for ship: also `just gallery-gif` so
   `assets/gallery.gif` / `book/src/gallery.gif` match (AGENTS).

#### 4c. Usability / accessibility (visual + code)

From pixels, fix when obvious:

- Interactive targets too tight or flush (padding / hit area via density).
- Disabled and enabled look the same (use muted tokens / disabled style).
- Primary action not findable (variant hierarchy Primary vs Quiet).
- Status/error text unreadable on canvas (token contrast).
- Focus/selection missing when a row or field is active.

Always keep **`A11y` on public constructors** (`a11y::attach`); do not
strip names/roles to “clean up” paint. If the shot shows a control that
cannot be distinguished, improve **visual** hierarchy first; a11y
metadata alone does not fix the frame.

#### 4d. Report after fixes

Append a **Fix log** to `VISUAL_REPORT.md`: defect → path → commit/diff
summary → rewalk beat ids → post-fix verdict. Final verdict only after
re-inspection of the new shots.

### 5. Report

Write `out_dir/VISUAL_REPORT.md` with:

1. **Environment** — git SHA, backend, display, release vs debug.
2. **Timings** — from `timings.json`.
3. **Shot review** — ordered table: beat · page · file · verdict · notes
   (from vision, not from filenames alone).
4. **Fix log** — if step 4 ran.
5. **Verdict** — `SHIPPABLE` / `POLISH` / `BROKEN`.
6. **Top fixes** — remaining asks (or “none after rewalk”).
7. **Demo** — if `--gif` was used, path and whether frames are clean.

Paste a short summary into chat. Keep the full report on disk.

### 6. Do not

- Commit screenshots, casts, or `tmp/gallery-walk/**` unless the user asks.
- Claim pixel perfection without reading every shot (before **and** after
  fixes).
- “Fix” by editing PNGs or generating fake UI images.
- Skip rewalk after code changes that affect paint.
- Skip recapture of `just gallery-gif` when public gallery shell/pages
  change and the user expects shippable assets.

## Step map

Beats are the gallery **tour index** (same as `gallery-gif`): every
`catalog::pages()` entry plus the light Theme beat. Captions come from
the gallery (`ack.caption`). Typical order starts with Controls, Fields,
… Markdown, Code, List, Theme (dark then light), Patterns, …

Subset for a quick pass:

```bash
python3 .grok/skills/gallery-visual-walkthrough/scripts/gallery_walkthrough.py \
  --beats 0-8 --out tmp/gallery-walk/smoke
```

## Quality

- Script is plain Python 3.12+; keep **ruff** clean when you edit it
  (`ruff check` + `ruff format` on the script).
- Prefer release binary for demos; document when debug is used.
- Measurement is external (pixels + wall clock) — no product hooks.

## Related

- Rubric: `references/rubric.md`
- Harness: `scripts/gallery_walkthrough.py`
- GIF publisher (assets/book): `scripts/gallery-gif.sh`
- Product tour env: `ICEDTEA_GALLERY_TOUR`, `_CMD`, `_ACK`, `_LEN_FILE`
