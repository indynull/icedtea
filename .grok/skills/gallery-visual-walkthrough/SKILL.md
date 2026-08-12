---
name: gallery-visual-walkthrough
description: >
  Launch icedtea-gallery in isolated Xephyr, drive the built-in tour
  beats, capture timed screenshots per page, optionally build a demo
  GIF, then visually review each grab for polish, layout, and broken
  state using multimodal image inspection. Use when the user runs
  /gallery-visual-walkthrough, asks for gallery screenshot review,
  pixel perfection, visual QA, automated gallery demo, or "is the
  gallery ugly / broken".
metadata:
  short-description: "Gallery timed tour + visual QA"
---

# Gallery visual walkthrough

End-to-end **visual** review of the icedtea gallery (every catalog
tour beat), plus an optional demo GIF. Product eyes on real pixels —
not a unit suite alone.

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

1. Open with the **read_file** tool (image path). You **must** inspect
   pixels with multimodal vision — **filename-only scoring fails**.
2. Score against `references/rubric.md`.
3. Human-usefulness bar:
   - Correct page selected (nav + title match caption)
   - Readable type; primary controls not clipped or buried
   - Catalog page not an empty stub
   - No overlapping labels or unusable density
4. Write one short note per shot: **ok / ugly / broken** + one sentence why.

Do **not** use `image_gen` to “fix” the UI.

### 4. Report

Write `out_dir/VISUAL_REPORT.md` with:

1. **Environment** — git SHA, backend, display, release vs debug.
2. **Timings** — from `timings.json`.
3. **Shot review** — ordered table: beat · page · file · verdict · notes.
4. **Verdict** — `SHIPPABLE` / `POLISH` / `BROKEN`.
5. **Top fixes** — concrete product/UI asks.
6. **Demo** — if `--gif` was used, path to the GIF and whether frames are clean enough to show.

Paste a short summary into chat. Keep the full report on disk.

### 5. Do not

- Commit screenshots, casts, or `tmp/gallery-walk/**` unless the user asks.
- Claim pixel perfection without reading every shot.
- Skip recapture after gallery shell/page behavior changes when the user
  asked for visual proof (AGENTS: recapture tour when public UI ships).

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
