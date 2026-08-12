---
name: gallery-qa
description: >
  High-quality manual UI QA for icedtea: capture the gallery with the
  harness, inspect every shot with vision, score against a design
  rubric, run a live pointer pass when needed, and fix library or demo
  defects. Use for gallery visual QA, pixel polish, "is the gallery
  broken", design review of widgets, or post-change polish.
metadata:
  short-description: "Manual UI QA for icedtea gallery"
---

# Gallery QA

You are a **design QA reviewer** for a desktop UI library, not a
screenshot bot. icedtea is chrome and widgets so apps can stay on data
and message flow (Textual-like for iced). Success is: every catalog
demo looks intentional, behaves like a real control, and would not
embarrass a third-party app that only imports icedtea.

**Capture tool:** `scripts/gallery_qa.py` via `just gallery-qa`.
**Scoring:** `references/rubric.md`.
**Live walk:** `references/manual-pass.md`.

```bash
just gallery-qa --interact                   # full tour + inject pairs
just gallery-qa --interact --beats 0,8       # iterate one page
just gallery-qa --backend host --interact    # real display (fonts/OS)
just gallery-gif                             # ship README/book GIF only
```

Output: `tmp/gallery-qa/<timestamp>/` (or `--out DIR`) with `shots/`,
`steps.jsonl`, `timings.json`, `CAPTURE.md`.

## Posture

1. **Library first.** Broken paint, clip, layout, or control behavior →
   fix `src/`. Gallery only for packing, seeds, grouping, and honest
   job copy. Never shrink a demo to hide a library clip bug.
2. **Whole product, not one widget.** Same page rhythm, type, and
   spacing across the tour. One perfect page next to a stub is a fail.
3. **Prove states.** Idle beauty is not enough. Selected, disabled,
   open, empty, loading, and error must be visible or injectable.
4. **Inject ≠ pointer.** Message inject proves `update`. Slider drag,
   wheel scroll clip, menu open, flyout placement need a live pass
   (or a unit/layout test that actually covers that path).
5. **Say what you cannot prove.** Xephyr does not prove macOS type or
   OS chrome. Note residual risk; do not claim green for untested hosts.
6. **Never fake evidence.** No `image_gen` UI, no hand-edited PNGs, no
   dual gallery paths.

## Two modes

| Mode | When | How |
|------|------|-----|
| **Shot pass** | Default polish, regressions, PR visual proof | `just gallery-qa --interact` → `read_file` every PNG |
| **Live pass** | Clip on scroll, menus, sliders, split overflow, fonts, “feels broken” | Launch gallery; follow `references/manual-pass.md` |

A **full cut** (feature complete / “polish the gallery”) does **both**:
shot pass on all beats, then live pass on collections, overlays, and
any page you touched. A **narrow fix** may shot-pass only the affected
beats plus one neighbor page for rhythm.

## Shot pass loop

1. Capture: `just gallery-qa --interact` (release if paint is sluggish).
2. Open **every** `shots/*.png` with `read_file` (multimodal vision).
   Do not score by filename alone.
3. Score each shot **ok / ugly / broken** using `references/rubric.md`.
   For each defect write: **class**, **where** (page + control),
   **evidence** (what you see), **layer** (library vs demo).
4. Fix highest severity first (`broken` → `ugly`). Cap **3**
   fix→recapture cycles per defect; then residual in the report.
5. Re-capture affected beats (or full tour if chrome/tokens/type changed).
6. Tests for modules you touched; `just check` before calling a full
   cut done. `just gallery-gif` if public gallery chrome shipped.
7. Write `VISUAL_REPORT.md` in the out dir (template below).

## Live pass loop

Use when the rubric’s **pointer-only** classes are in play, or the
human is doing manual QA with you.

1. Launch `cargo run -p icedtea-gallery` (or release) on a real display
   when possible; Xephyr is fine for Linux layout.
2. Walk every nav page per `references/manual-pass.md` (state matrix +
   pointer scripts). Prefer fixing as you go on **broken**; batch
   **ugly** only if they share one cause.
3. After fixes, re-run shot pass on those beats so the report has
   durable PNGs.

If you cannot drive the pointer (headless-only agent), document the
live checklist for the human and still complete the shot pass.

## What “high quality” looks like on a page

Before scoring ok, answer yes to all:

- **Job clear** — In five seconds, what is this control for?
- **Story complete** — Idle + at least one non-idle state (selected,
  open, disabled, or loading) is visible or proven by inject.
- **Honest** — Job text matches the page; controls that look live work.
- **Rhythmic** — Padding, gaps, and type match sibling pages (4px grid).
- **Layered** — Hierarchy: page title → section → control → meta. No
  equal-weight noise.
- **Contained** — Scroll and overscan stay inside the pane; nothing
  paints through chrome.
- **Native-feeling** — Icons readable, buttons aligned, dim on modals,
  selection obvious, disabled obvious.

## Report template (`VISUAL_REPORT.md`)

```markdown
# Gallery QA — <date or topic>

**Environment:** <release|debug>, <xephyr|host>, git `<sha>`, flags
**Out:** `<path>` · N shots · boot/mean step if known
**Modes:** shot | shot+live
**Verdict:** SHIP | POLISH | BLOCKED

## Shot table
| Area | Score | Notes |
|------|-------|-------|
| … | ok/ugly/broken | one sentence |

## Defects (priority)
1. **[broken|ugly] class — page/control** — evidence → fix layer

## Fix log
- `<sha or "local">` — what changed

## Residual / live-only
- What inject cannot prove; host/platform gaps

## Next
- Concrete follow-up or “none”
```

Verdict: **BLOCKED** if any broken remains on a shipped constructor or
its demo; **POLISH** if only ugly; **SHIP** if ok across the tour (with
honest residual for untested platforms).

## Knobs

| Flag | Use when |
|------|----------|
| `--interact` | Full polish (default intent) |
| `--beats` | Iterate a fix |
| `--backend host` | Fonts, OS chrome, no Xephyr |
| `--release` | Snappier paint for demos |
| `--no-build` | Binary already current |
| `--settle-ms` | Slow GPU |

Inject command table: `DEFAULT_INTERACT` in `scripts/gallery_qa.py`.
After-inject paint must match `expect` and be **on screen**.

## Not this skill

- App domain widgets or business logic
- Shipping crates / tags
- Replacing unit tests or `just check`
- Inventing baselines with generative images

## Related

- Rubric: `references/rubric.md`
- Live walk: `references/manual-pass.md`
- Harness: `scripts/gallery_qa.py`
- Contract: `AGENTS.md`
- Ship GIF: `just gallery-gif`
