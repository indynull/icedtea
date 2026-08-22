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
**Stills:** `references/visual.md` (book still + must-show per page).  
**Direction:** `references/rtl.md` (SCORE map). Sources:
`references/firefox-rtl.md`, `references/ms-bidi.md`,
`references/ms-flowdirection.md`.  
**Live walk:** `references/manual-pass.md`.

```bash
just gallery-qa --interact                   # full tour + inject
just gallery-qa --interact --beats 0,8       # iterate a fix
just gallery-qa --live-clip                  # real mouse wheel on List + Table
just gallery-qa --locale ar --beats 8,9,12,19,20  # RTL; SCORE.md vs references/rtl.md
just gallery-qa --locale he --beats 0,1           # Hebrew; Western digits on RTL
just gallery-qa --backend host --interact    # fonts / OS chrome
just gallery-gif                             # live pointer demo (click, type, wheel + inject)
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
   drag, menu/flyout, and pick-list open need live pass or a real
   unit/layout test that calls the shipped constructor.
   **Wheel:** `xdotool mousemove X Y` onto the pane, then
   `xdotool click --repeat N --delay 30 5` (button 5 down, 4 up).
   Do not use `mousemove --sync` on Xephyr (it hangs).
   `just gallery-qa --live-clip` is that pass for List and Table.
   A `ListScroll` inject is not a wheel.
   **Tour GIF:** `record_gif_demo` clicks, then injects the **same**
   message (`note Primary`, `query` / `search-go`, `list`, `table`,
   `pal-query`, `appearance`). A caption that claims a result the
   status, hits, selection, or theme does not show is **broken**.
   Do not put a tour-only Action in the chrome `ActionTable`
   (`menu_groups` adds a menu from the id prefix). Primary is
   `demo_primary_action()`.
5. **Prove what you claim.** Xephyr ≠ macOS type. Do not call a font
   path done without a host/Mac pass when those files changed.
6. **No fake evidence.** No `image_gen` UI, no hand-edited PNGs.
7. **Drive the shipped constructor.** Tests call `themed_pick_list`
   (or the public fn under review) from a real start state. Do not
   assert only a private helper, and do not start past the widget.
8. **Score the published tour** after a paint change (`just gallery-gif`
   / handbook stills). A source fix whose published GIF still shows
   the old mark is **broken**.
9. **Trailing-icon geometry** is `references/m3-trailing-icon.md`
   (24 dp / 12 dp default). Do not accept a body-sized or 4 dp-flush
   chevron as residual when the Material numbers are in-repo.
10. **Idle must show the constructor under review.** Shared pages pack
    unpublished or changed hosts above the fold (`pack_at` / catalog
    order). A look-strip neighbor is not a Fields/select score.
11. **Compare to the still.** For every idle shot, `read_file` the
    still in `references/visual.md`, then the QA shot. Skip no page
    class (readout, motion, chrome). Locale shots use the same still
    plus `rtl.md` (progress from start, Eastern digits on ar/ur, wrap,
    start/end order).
12. **New or changed paint updates the still map.** Adding a catalog
    constructor or page, or changing how one looks, recaptures that
    book still and rewrites the `visual.md` row in the same change.
    SCORE `visual-map` is **broken** if a tour page has no row or the
    still PNG is gone. If SCORE can fail on the new look, add that
    check in `gallery_qa.py` too.

## Loop (do this)

1. Capture: `just gallery-qa --interact` (release if paint is slow).
   Locale cut: also `just gallery-qa --locale ar` and `--locale ur`
   (and `--locale he` when Hebrew chrome changed).
2. For **every** idle shot: `read_file` the `visual.md` still, then
   the QA `shots/*.png`. Score with the rubric and the still’s
   must-show line. After-inject shots too. Do not stop at nav/list.
3. For each **broken** / clear **ugly**: implement the fix, run cheap
   tests on touched modules, recapture affected beats, re-read shots.
4. Full cut / pre-release: also run the **live pass**
   (`references/manual-pass.md`) on collections, overlays, and pages
   you touched. Fix as you go.
5. When no broken remains and every known ugly is fixed: `just check`
   (or targeted tests + full check before release claim).
6. Recapture the published tour (`just gallery-gif`) after a gallery
   layout, nav-clip, or public-chrome change. That file is a short
   live pointer demo, not a catalog walk. Captions are Fira or Fura
   32 bold (Gruvbox cream, outline). Each line names the widget, the
   action, and the result on screen at the end of the beat. Score
   **that file** (status/job note, filtered hits, selected row, light
   canvas), not only `tmp/gallery-qa/` shots. A source fix whose
   published tour still shows the old paint is **broken**.
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
| `--locale ar` | Arabic/Urdu; `SCORE.md` fails on any `references/rtl.md` beat (not leftover-English only), including chrome wrap |

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

- `references/visual.md` — still + must-show per page  
- `references/rubric.md` — what counts as broken/ugly  
- `references/m3-trailing-icon.md` — pick / menu / list trailing mark  
- `references/rtl.md` — SCORE map  
- `references/firefox-rtl.md` — Firefox RTL Guidelines  
- `references/ms-bidi.md` — Microsoft bidirectional design  
- `references/ms-flowdirection.md` — Microsoft FlowDirection / layout  
- `references/manual-pass.md` — pointer / live protocol  
- `scripts/gallery_qa.py` — capture harness  
- `AGENTS.md` — library contract  
