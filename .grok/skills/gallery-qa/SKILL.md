---
name: gallery-qa
description: >
  Catch and fix icedtea gallery and constructor defects before release:
  capture shots, inspect with vision, live-pass when needed, fix library
  or demos, re-verify. Use for gallery QA, pixel polish, "is the gallery
  broken", locale or right-to-left scoring, leftover English, empty
  control pads, or pre-release polish.
metadata:
  short-description: "Find and fix icedtea gallery bugs"
---

# Gallery QA

**Objective:** find visual and interaction defects in shipped constructors
and gallery demos, **fix them in source**, and re-verify until the release
bar is met. A markdown report is optional scratch — not the deliverable.
Clean tree + green checks + fixed defects is the deliverable.

icedtea is chrome and widgets so apps can stay on data and message flow.
A third-party app that only imports icedtea should not inherit broken
clip, dead overflow, empty first paint, or stub demos from the gallery.

**Capture:** `scripts/gallery_qa.py` via `just gallery-qa`.  
**Score:** `references/rubric.md`.  
**Direction:** `references/rtl.md` (Firefox + Microsoft).  
**Live walk:** `references/manual-pass.md`.

```bash
just gallery-qa --interact                   # full tour + inject
just gallery-qa --interact --beats 0,8       # iterate a fix
just gallery-qa --locale ar --beats 8,9,12,19,20  # RTL; SCORE.md vs references/rtl.md
just gallery-qa --backend host --interact    # fonts / OS chrome
just gallery-gif                             # after layout, nav clip, or public chrome
```

Shots land under `tmp/gallery-qa/<timestamp>/` (or `--out DIR`).

## Posture

1. **Fix is the goal.** Do not stop at “found ugly list.” Patch `src/`
   or `icedtea-gallery/`, recapture, confirm the defect is gone.
2. **Library first.** Clip, Fill, icons, control behavior, tokens, type →
   `src/`. Packing, seeds, job copy, page group → gallery. Never hide a
   library bug by shrinking the demo.
3. **Severity order.** Fix every **broken** before batching **ugly**.
   Cap **3** fix→recapture cycles per defect. Residual only when the
   fix is genuinely unclear (need a human pointer, other host). A
   known packing, contrast, or clip fix is not residual.
4. **Inject ≠ pointer.** Inject proves `update`. Scroll clip, slider
   drag, menu/flyout need live pass or a real unit/layout test.
5. **Prove what you claim.** Xephyr ≠ macOS type. Do not call a font
   path done without a host/Mac pass when those files changed.
6. **No fake evidence.** No `image_gen` UI, no hand-edited PNGs.

## Loop (do this)

1. Capture: `just gallery-qa --interact` (release if paint is slow).
2. `read_file` **every** `shots/*.png`. Score with the rubric.
3. For each **broken** / clear **ugly**: implement the fix, run cheap
   tests on touched modules, recapture affected beats, re-read shots.
4. Full cut / pre-release: also run the **live pass**
   (`references/manual-pass.md`) on collections, overlays, and pages
   you touched. Fix as you go.
5. When no broken remains and every known ugly is fixed: `just check`
   (or targeted tests + full check before release claim).
6. Recapture the published tour (`just gallery-gif`) after a gallery
   layout, nav-clip, or public-chrome change. Score **that file**
   (sticky Search at scrolled sidebar beats), not only `tmp/gallery-qa/`
   shots. A source fix whose published tour still shows the old paint
   is **broken**.
7. Commit fixes. Working tree clean for the work you finished.

Narrow change: shot-pass affected beats + one neighbor for rhythm; live
only if pointer classes apply.

## Release bar

| Bar | Meaning |
|-----|---------|
| **Ready** | No broken on shipped constructors/demos; known uglies fixed; residual only if the fix is unclear; checks green |
| **Not ready** | Any broken left, or pointer/platform residual on paths you changed without a pass |

Chat the human a **short** status: what you fixed, what residual remains
(if any), command evidence. Do not produce a long `VISUAL_REPORT.md`
unless they ask for a written audit.

## Knobs

| Flag | Use when |
|------|----------|
| `--interact` | Pre-release / full polish |
| `--beats` | Iterate one defect |
| `--backend host` | Fonts, OS chrome, no Xephyr |
| `--release` | Snappier paint |
| `--no-build` | Binary already current |
| `--settle-ms` | Slow GPU |
| `--locale ar` | Arabic/Urdu; `SCORE.md` fails on any `references/rtl.md` beat (not leftover-English only) |

Inject table: `DEFAULT_INTERACT` in `scripts/gallery_qa.py`. After-inject
state must be **visible on screen**. A match is a **token**, not a
substring of another word (`table:` must not fire on `Selectable:`).

Hash consecutive shot files. A pair that is byte-identical across
**different** tour pages is not evidence — the grab landed before the
page painted. Recapture those beats (`--settle-ms` if paint is slow).
Only if a second grab is still the previous page is the gallery stuck.

## Not this skill

- Writing reports as the product of the work
- App domain logic, shipping tags, replacing unit tests
- Dual gallery paths or generative UI fakes

## Related

- `references/rubric.md` — what counts as broken/ugly  
- `references/rtl.md` — Firefox + Microsoft direction beats  
- `references/manual-pass.md` — pointer / live protocol  
- `scripts/gallery_qa.py` — capture harness  
- `AGENTS.md` — library contract  
