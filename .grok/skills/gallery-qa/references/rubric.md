# Gallery QA rubric

| Tag | Meaning |
|-----|---------|
| **ok** | Usable, clear hierarchy, no obvious defect |
| **ugly** | Usable but dense, weak contrast, uneven gaps, weak dim |
| **broken** | Wrong page, empty stub, clipped UI, paint bleed, dead control, invisible selection/disabled, all-black, first paint empty |

## Layout

- ~12–16px inset; 4/8px spacing grid; pages share the same rhythm
- No clipped labels/buttons; body not under status bar
- Nav selection matches page title/caption
- Multi-host pages: each fill widget has a real height (not one row + hole)
- Text and icons clear card edges (no flush title/meta)

## Clip / stack

- Virtualized lists/tables do not paint rows **over** filters, headers, or sibling demos when scrolled
- Soft `container::clip` is not enough if card backgrounds fill their layout box — library must scissor (e.g. layer) when overscan exists
- Idle list/table is not blank; first useful frame shows rows without a prior scroll message

## Live behavior

- Selection, disabled, open expander, and selected row are visible
- Inject after-shot differs from idle in the way `expect` describes
- Filters, pagination, split overflow, and menus that appear must **drive state** (dead chrome is broken)
- One control must not flip unrelated widgets (message cross-talk)
- Progress/slider/list have non-zero width/height (Fill collapse is broken)

Inject proves `update` paths. It does **not** prove slider drag, wheel
scroll clip, or menu hit-tests — call those out if only inject is green.

## Demo honesty

- Catalog page shows real demo content (not empty, not one-line stub)
- Select surfaces use real constructors (not label-only fakes)
- Demos teach the job of the widget (not grey boxes or random duplicates)
- Page/widget job text matches what is actually on the page (no “slider on this page” when there is none)
- Light theme beat looks light; dark looks dark
- Related controls share a page; full-window screens (About, Preferences) are not dumped under an unrelated “Patterns” pile without a clear group

## Chrome atoms

- Chevrons and icons are recognizable shapes at control size (not dots or reinvented stubs)
- Tabs/document tabs have a clear face (border/shell), not bare text + close
- Split button: primary and overflow same height; overflow opens a real menu
- Modal sheets sit on a visible dim wash
- Selectable value rows read as body text, not heavy editor slabs

## Platform

- Default harness is Linux Xephyr. Titles and logo must not look mono there.
- macOS bold/UI cascade and OS chrome colors need a **host** (or remote Mac) pass when those areas changed — do not claim them from Xephyr alone.

## Interact pairs (`idle` → `after-interact`)

Expected state change must be **visible**. Same paint as idle → broken
inject, target below the fold, or broken style.

## Fix

Change **source**, re-capture. Never edit PNGs or `image_gen` the UI.
Prefer library fix for clip, Fill, icons, and control behavior; gallery
only for pure demo packing, seed data, and page grouping.
