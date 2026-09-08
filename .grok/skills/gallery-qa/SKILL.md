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
**Material:** `references/material/INDEX.md` (`just material-snapshot`).

```bash
just gallery-qa --interact                   # full tour + inject
just gallery-qa --interact --beats 0,8       # iterate a fix
just gallery-qa --live-clip                  # real mouse wheel on List + Table
just gallery-qa --locale all                 # every fill language (en vi ja zh ar ur he)
just gallery-qa --locale ar --beats 8,9,12,19,20  # one language while iterating
just gallery-qa --backend xvfb --locale all   # headless nested; does not take the seat
just gallery-gif                             # live pointer demo into tmp/gallery.gif
just gallery-gif persist                     # assets/ + handbook; version tag only
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
6. **No fake evidence.** Never call `image_gen` during gallery QA
   (not for UI, not for animals, not for anything). Score only the
   book still in `visual.md` and the captured `shots/*.png`. No
   hand-edited PNGs.
7. **Drive the shipped constructor.** Tests call the public fn under
   review from a real start state. Do not assert only a private helper,
   and do not start past the widget.
8. **Score the local tour** after a paint change (`just gallery-gif`
   writes `tmp/gallery.gif`; handbook stills stay in-tree). A source
   fix whose recaptured GIF still shows the old mark is **broken**.
   Persist the GIF on a version tag, not in the fix commit.
9. **Trailing marks.** Score the painted size against
   `references/m3-trailing-icon.md`. A body-sized or 4 dp-flush
   chevron is not residual.
10. **Idle must show the constructor under review.** Shared pages pack
    unpublished or changed hosts above the fold (`pack_at` / catalog
    order). A look-strip neighbor is not a Fields/select score.
11. **Compare to the still.** For every idle shot, `read_file` the
    still in `references/visual.md`, then the QA shot. Skip no page
    class (readout, motion, chrome). Locale shots use the same still
    plus `rtl.md`. A `SCORE.md` row or a constructor-source substring
    is **not** this compare. The bar is the pixels.
12. **New or changed paint updates the still map.** Adding a catalog
    constructor or page, or changing how one looks, recaptures that
    book still and rewrites the `visual.md` must-show in the same
    change. When a shot miss was not in the must-show, add it there
    (and `rtl.md` / `rubric.md` if it is directional). Never add a
    constructor-body grep as the proof that page looks right.
13. **Score the size a human opens.** Tour and QA resize to 1600×900.
    The gallery default is 1280×800 (floor 1100×700) so the look
    strip and menu stay one line. A wrap that only appears at that
    default is still **broken**. A 1600-wide SCORE ok is not proof
    the first page is clean.
14. **Score Fill + shrink together.** A shrink button beside a Fill
    toast or scroller snaps to physical left when the column is not
    Fill + `align_start`. Score the button after a toast is visible,
    not only the idle empty queue.
15. **Locale density must match English.** Open the English shot, then
    the locale shot of the same page. Hints, pick captions, form
    labels, and image-slot captions sit under the heading (or slot)
    on the **start** edge. Form fields put placeholder and caret on
    start (right in RTL). A page whose widgets exist but look empty
    is **broken**. Do not score “constructors present” as clean.
16. **Catalog fill is not leftover English.** Score the still’s
    must-show (`visual.md` + `rtl.md`). Colorway ids, face ids, icon
    slugs, filename samples, and key chords stay as they are. Leftover
    Latin is broken only where the still names it. Hebrew painted
    numbers stay Western; ar/ur/fa use Eastern (including `%` → `٪`).
    The confirm card stays centered on the dim wash.
17. **Material numbers** come from `references/material/` and `src/m3/`.
    Read the snapshot page when scoring elevation, shape, type,
    spacing, motion, or trailing icons. Do not invent values. A
    documented desktop approximation in `src/m3` is not a miss.
    Score the painted drop against `Component::elevation()`; the
    assignment check is `style::tests::resting_elevation_matches_material_table`.
18. **Composite focus.** Score the item face (row wash, outlined tile).
    A Field-radius stadium around the pane, two cards, or the tile
    wall is **broken**.
19. **Icon fields.** After changing pad, icon size, or field height,
    `read_file` the still. Search glass and placeholder share the bar
    midline. A layout test is not that look.

## Loop (do this)

1. Capture: `just gallery-qa --interact` (release if paint is slow).
   Locale cut: `just gallery-qa --locale all` — every gallery fill
   language (`en`, `vi`, `ja`, `zh`, `ar`, `ur`, `he`). Never a subset.
   Capture is Xephyr (nested). Never `--backend host` from an agent
   session: that activates the window and moves the pointer on their
   seat. Host is only when they asked to watch fonts on the live display.
2. For **every** idle shot: `read_file` the `visual.md` still, then
   the QA `shots/*.png`. Score with the rubric and the still’s
   must-show line. After-inject shots too. Do not stop at nav/list.
   A shot that is the previous page’s pixels is a recapture fail.
   A locale shot whose look strip is still English is a recapture fail.
   After `group` / `rail` / `list` / `grid` inject, the status/job note is the
   locale fill plus mapped digits. Leftover English `Group 2` or
   `Opened tile 2` on `ar` / `ur` / `he` is **broken**. Leftover Latin `Enter` / `Escape` on
   Keys, `Contain, cover` on Image, or a raw `ok` on Main window
   status is leftover English — score the pixels, do not grep
   constructor bodies.
3. For each **broken** / clear **ugly**: implement the fix, run cheap
   tests on touched modules, recapture affected beats, re-read shots.
4. Full cut / pre-release: also run the **live pass**
   (`references/manual-pass.md`) on collections, overlays, and pages
   you touched. Fix as you go.
5. When no broken remains and every known ugly is fixed: `just check`
   (or targeted tests + full check before release claim).
6. Recapture the tour (`just gallery-gif` → `tmp/gallery.gif`) after
   a gallery layout, nav-clip, or public-chrome change. That file is
   a short live pointer demo, not a catalog walk. Captions are Fira
   or Fura 32 bold (Gruvbox cream, outline). Each line names the
   widget, the action, and the result on screen at the end of the
   beat. Score **that file** (status/job note, filtered hits, selected
   row, light canvas), not only `tmp/gallery-qa/` shots. A source fix
   whose recaptured tour still shows the old paint is **broken**. Do
   not persist or commit `assets/gallery.gif` / `book/src/gallery.gif`
   except when tagging a version (`just gallery-gif persist`).
7. Commit fixes. Leave the tracked tour GIFs untouched. Working tree
   clean for the work you finished.

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
| `--backend xephyr` | Nested display; default. Does not take the seat |
| `--backend host` | Only when they asked to watch live fonts; activates their window |
| `--release` | Snappier paint |
| `--no-build` | Binary already current |
| `--settle-ms` | Slow GPU |
| `--locale all` | Every fill language; each writes `SCORE.md`. `all` is the locale bar. One `LANG` or a comma list is for iterating a fix |

Inject table: `DEFAULT_INTERACT` in `scripts/gallery_qa.py`. After-inject
state must be **visible on screen**. A match is a **token**, not a
substring of another word (`table:` must not fire on `Selectable:`).

Hash consecutive shot files. A pair that is byte-identical across
**different** tour pages is not evidence — the grab landed before the
page painted. Recapture those beats (`--settle-ms` if paint is slow).
Only if a second grab is still the previous page is the gallery stuck.

Do not invent beat numbers from the page name. Tour beats follow
gallery pages: every catalog page, extra Theme and Colors pages
(not catalog ids) before Keys, a Light flip on Theme, plus extras
on Code and Motion. Layout sits after Keys: idle first screen is
pack (Find, filling search, Go) then wrap chips and min-width
tiles. Score start/end on RTL; two parent widths are the library
tests, not one still.
An idle-only cut writes
one file per beat (`00-beat00-idle-…`). An `--interact` cut
interleaves after-inject frames, so file prefixes are sequential
across idle *and* after (`00` idle controls, `01` after-toggles,
`03` idle fields). `ls` the shot dir before `read_file`. A miss
on `07-selectable` in an idle-only cut is looking at Image; in
`--interact` that prefix is a later beat.

Motion extras: idle `Motion: pick what opens and play` is the
must-show (Opens / Switch / Attention / Expand two-up, overlay
closed). The four extras apply `dialog true`, `switch-face axis`
plus `step-next`, `field-check`, and `expand true`. Score the
open overlay on the first extra, not on idle.

Jumping `--beats N` from a cold gallery can grab the previous page.
Walk in from the neighbor (`--beats N-1,N --settle-ms 700`) and
re-read the shot before scoring. A controls grab for markdown is
not evidence. Palette beat 31 is the same class: an empty
filename, a status-page grab, or a previous-page twin is a
recapture fail — walk `--beats 30,31`. A start-align or pack
fix (colors hint, grid hint, cheatsheet title/shortcut) is not
done after one locale — recapture that beat for every fill
language and re-read those shots before claiming clean.

Score the pixels, not a constructor-body grep:

- Cheatsheet: title on start, shortcut on end. LTR title-left on
  `ar` / `ur` / `he` is broken.
- Badge / expander / tree selected id: Eastern on ar/ur (`٢` / `٩` /
  `٣`). Hebrew stays Western.
- Closable tabs (idle 14 / 27): icon, title, badge, close on the end
  (left in RTL). The close mark is the proof.
- Job lines: painted caption. A physical left or right after the
  layout mirrors is leftover.
- Side sheet after `sheet true`: docks on the end; close on the
  sheet end. The confirm card stays centered.

## Not this skill

- Writing reports as the product of the work
- App domain logic, shipping tags, replacing unit tests
- Dual gallery paths or generative UI fakes

## Related

- `references/visual.md` — still + must-show per page  
- `references/rubric.md` — what counts as broken/ugly  
- `references/material/INDEX.md` — Material spec snapshot (`just material-snapshot`)
- `references/m3-trailing-icon.md` — trailing-mark sizes to score
- `references/rtl.md` — SCORE map  
- `references/firefox-rtl.md` / `ms-bidi.md` / `ms-flowdirection.md`
- `references/manual-pass.md` — pointer / live protocol  
- `scripts/gallery_qa.py` — capture harness  
- Constructor rustdoc and `src/m3` — library law (do not copy it here)
