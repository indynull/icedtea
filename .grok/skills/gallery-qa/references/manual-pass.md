# Live / manual gallery pass

Use with the rubric. This is the **pointer and judgment** layer the
harness cannot replace. Pair with shot recapture after fixes.

## Setup

```bash
cargo run -p icedtea-gallery
# or: cargo run -p icedtea-gallery --release
```

- Window large enough to show sidebar + content (~1600×900 matches tour).
- Start on **dark**; flip **light** once on Theme (or tour light beat).
- Prefer the host display for type/OS; Xephyr is OK for layout-only.

Agent: if you cannot click, run the shot pass fully and give the human
this checklist page-by-page; still fix anything vision already proves.

## Pace

For each nav page:

1. **Land** — title, job line, first screenful: job clear? rhythm OK?
2. **Inventory** — list every control/section; any stub or dead chrome?
3. **States** — exercise the matrix below for interactive pieces.
4. **Scroll** — if content scrolls, scroll to top/mid/end; watch clip.
5. **Notes** — broken first, then ugly; one line each with class name.

Do not “approve” a page until states and scroll are done.

## State matrix (per interactive control)

| Action | Pass |
|--------|------|
| Click / toggle primary | Visible state change; status/note if gallery records it |
| Disable face (if shown) | Clearly not idle; not invisible |
| Keyboard where owned | Focus visible; Enter/Space activate; Escape dismisses overlay |
| Empty / zero data | Empty copy or honest blank, no collapse of the pane |
| Loading (image, busy) | Slot keeps size; spinner/progress moves or phase advances |

**Cross-talk check:** after acting on control A, scan the rest of the
page — nothing else should have changed unless the demo says so.

## Pointer scripts (high-risk pages)

### Controls

- Split **overflow** chevron opens a real menu; items invoke; menu dismisses.
- Primary and overflow **same height**, aligned.
- Slider drags across the range (not stuck at 0).
- Toggle buttons do not flip unrelated checkboxes.

### Fields / selectable

- Focus a field; type if enabled; select-all + copy path if demo claims it.
- Value fields: no heavy slab; labels align.

### Collections (list, table, log, tree, virtual column)

- Idle shows **rows** (not empty first paint).
- Scroll with wheel: rows stay **inside** the list pane; no paint over
  filters, headers, or sibling demos.
- Select a row: selection visible; keyboard if supported.
- Filters / pagination: change **which data** is shown (dead chrome = fail).
- Expand card / expander: open body text visible, height grows.

### Sections / tabs

- Tab select changes body content (not empty strip).
- Document tabs: clear face (shell/border), dirty/close readable.
- Accordion/expander: padding inside the face; chevron readable.

### Overlays (dialogs, palette, context)

- Dialog: dim wash obvious; primary/cancel clear; focus in sheet.
- Context: open via demo path; nested flyout **row-aligned**, gap from
  parent, not covering the parent title by accident.
- Palette: filter narrows list; highlight moves.

### Workspace / list-detail / inspector

- Sash drag resizes without fighting the cursor.
- List-detail: list and detail padding; rail not kissing the hairline.
- Drawer/dock: open/close; content not clipped under chrome.

### Theme / type

- Flip light/dark: canvas, text, selection still contrast.
- Titles and “icedtea” logo: proportional UI face, not mono bold cascade
  (macOS: confirm on Mac host).

## Scroll and clip protocol

On every scroller (virtual list **or** `themed_scroll` nav/page):

1. Note first row title at rest.
2. Scroll down several pages; first rows must disappear **into** the clip,
   not draw on top of chrome above the list.
3. Check sticky siblings (Search, section titles, filters): scrolled
   labels must not show through those fields.
4. Scroll to end; last rows fully visible; rail handle usable.
5. Scroll back to top; no permanent wrong offset.

If bleed occurs: library scissor/layer issue until proven otherwise.

## Direction (Arabic / Urdu)

Use `references/rtl.md`. On a right-to-left session:

- Tab moves from start to end. Left/right arrows follow start.
- Localizable punctuation (`.`, `:`, `…`, `?`, `!`) sits on the start
  side of the run.
- Code, paths, and URLs read left-to-right (semicolon on the right of
  `padding: 20px`).
- Media transport, checkmarks, logos, and `1920x1080` stay unflipped.

## Catalog and copy pass (once per full cut)

Walk the **sidebar order** without opening every page:

- Groups match mental model (controls, fields, collections, chrome,
  layouts, overlays, screens — names as in current catalog).
- No page hosts contradictory demos (e.g. two paginations, one dead).
- Spot-check job strings against the open page for lies.

## When to stop and fix immediately

Stop the walk and fix **now** (do not finish all pages first):

- Paint bleed / empty first paint / cross-talk / dead primary control
- Unreadable type (mono titles, white-on-white)
- Crash or panic

Batch pure density/alignment uglies after the broken set is green.

## After the live pass

1. Recapture shot pass for pages you changed (`--beats` or full).
2. Confirm each fixed defect is gone in the new shots (or live recheck).
3. Commit library/demo fixes; do not commit harness PNGs. Recapture the
   published tour with `just gallery-gif` when gallery layout, nav clip,
   or public chrome changed. Score that file for sticky-header bleed
   (Search and other headers above a scrolled list), not only the
   `tmp/gallery-qa/` shots.
