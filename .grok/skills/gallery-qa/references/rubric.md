## Material Design 3

- Spacing on a **4 dp grid**; default density 8 dp gap, 48 dp touch targets.
- Controls show M3 states: enabled, disabled, hovered, focused, pressed, selected/error where defined.
- Surfaces use token roles only (`Tokens::scheme()`); no one-off hex.
- Desktop chrome is **rectangular** (M3 shape None / 0 dp). Rounded pills are ugly unless intentional geometry (slider thumb).
- Type hierarchy: label < body < title (M3 type scale via `typo`).
- Data table: selected row ≠ zebra stripe; focused cell is outline on selection wash.

# Gallery QA rubric

Score every shot (and every live page) **ok / ugly / broken**. Prefer
one primary class per defect. Severity first, then taste.

| Tag | Meaning | Ship bar |
|-----|---------|----------|
| **ok** | Intentional hierarchy, honest demo, states clear | Allowed |
| **ugly** | Usable but weak contrast, density, alignment, or chrome | Full cut: fix. Residual only if the fix is unclear |
| **broken** | Wrong, empty, dead, bleed, cross-talk, unreadable, first-paint fail | Must fix |

Industry design-QA and usability heuristics (consistency, status,
affordance, hierarchy, contrast, states) map onto the classes below.
This is a **desktop widget library gallery**, not a marketing site:
prefer control correctness and catalog honesty over pixel-diff thrash.

---

## 1. Hierarchy and density

- Page title → section title → control → meta is readable at a glance
- One primary focus per section; no equal-weight wall of controls
- Gaps on the **4px grid** (default density 8px); uneven “random” air is ugly
- Related items grouped; orphan controls or two copies of the same idea are broken catalog sense
- Multi-host pages: each host has usable height (not one row + empty Fill hole)
- A constructor this pass changed must be **on the idle first screen**
  of its page. Below-fold only is not a score for that widget.

## 2. Alignment and geometry

- Shared leading edges for labels/fields in a column
- Buttons in a row share height and baseline; split primary + overflow same height
- Icons optically centered in hit boxes; chevrons are chevrons, not dots
- Trailing pick / menu / list icons: **24 dp** (20 dp Compact),
  **12 dp** from the trailing edge at default density
  (`references/m3-trailing-icon.md`). A body-sized or 4 dp-flush
  chevron is **ugly**. A disc or missing mark is **broken**.
- Text and chrome clear card edges (no flush titles); ~12–16px page inset
- Body not under status/menu; nav selection matches the open page

## 3. Color, type, contrast

- Text readable on its surface (body, meta, muted, disabled all distinct)
- Selection and focus use tokens (`selection`, primary), not a mystery wash
- Disabled ≠ idle (opacity or muted), but still legible
- Light beat is light; dark is dark; high-contrast still hierarchical
- Type: UI for chrome, mono only for code/values that need it — titles/logo never mono by accident
- Modal: sheet above a **visible** dim wash; weak dim is ugly

## 4. States and feedback (usability: system status)

For each interactive control, the gallery should show or inject:

| State | Fail if |
|-------|---------|
| Idle | Missing or collapsed |
| Selected / on / open | No visual change vs idle |
| Disabled | Looks enabled or invisible |
| Empty | No empty copy or layout collapse |
| Loading | Slot collapses or static blank with no spinner/progress |
| Error | No error face when the constructor has one |

Busy, toast, progress: motion or value must be **perceivable** in
the shot (not a zero-width bar).

## 5. Affordance and honesty

- Looks pressable → works (dead split-overflow, dead filters, dead
  pagination = **broken**)
- Job / meta text matches the page (hint lies = broken)
- Demos teach the **job** of the widget (grey stubs, random duplicates,
  label-only “select” fakes = broken or ugly)
- Select/copy pages use real select constructors
- Catalog grouping is sensible (lists with pagination; overlays vs
  full-window screens; not a junk “Patterns” drawer)

## 6. Clip, scroll, stack

- Virtualized rows do not paint **over** filters, headers, or siblings
- Soft container clip is not enough when backgrounds fill layout boxes —
  library needs a real scissor/layer for overscan
- `themed_scroll` and any sibling scroller clip **below** sticky chrome.
  Section titles and rows must not paint through a Search field or other
  header that sits above the list (opaque header + scissor, not hope)
- First useful frame shows content (empty list until scroll message = broken)
- Sticky/frozen columns stay put; horizontal scroll does not orphan them
- Overlays (menu, context, palette, dialog) sit above content with clear
  z-order; flyouts align to parent row and do not needlessly overlap
- Published tour (`assets/gallery.gif`, `book/src/gallery.gif`) is a
  ship artifact. After a layout, nav-clip, or public-chrome commit,
  recapture it in the same change and score **the published frames**
  (sticky header at scrolled sidebar beats). A layout-source commit
  whose tour still shows the pre-fix paint is **broken**. Encoder
  input is a live window grab, not a still sequence.

## 7. Interaction integrity

- A shot that is byte-identical to the previous beat's file is not
  a score for this page when the beats are different pages. Recapture.
  Same pixels on a same-page extra beat (already-open) can be honest.
- Inject after-shot ≠ idle in the way `expect` describes
- Inject match is a token, not a substring of another word
- One message must not flip unrelated widgets (**cross-talk** = broken)
- Progress/slider/list/table have non-zero size (**Fill collapse** = broken)
- Keyboard story where the library owns it: focus order, Escape on
  modal/menu (live pass; note if unproven)

### Pointer-only (shot pass cannot fully prove)

Call out if only inject is green:

- Slider / sash drag
- Wheel scroll + clip
- Split overflow open, context submenu flyout placement
- Hover/pressed faces
- Text selection drag

## 8. Direction and locale

Bar: `references/rtl.md` (Firefox RTL Guidelines + Microsoft
bidirectional / FlowDirection). Mixed strings: Unicode UAX #9.

Score on a right-to-left locale beat (`just gallery-qa --locale ar`
or `ur`). That command writes `SCORE.md` and **exits non-zero** if
any row is **broken**. Leftover-English greps are one row, not the
bar. Do not walk languages by eye.

- Window is one direction. Chrome uses start/end (`i18n::order`,
  `align_start`). Physical `Alignment::Left` / `Right` in constructors
  is **broken** (`physical-align`).
- Vertical rail sits on the **end** side: left in RTL, right in LTR.
  `layout-rails` / `rtl_rails` plus `rail-*` on list, tree, log,
  feedback idle shots.
- List, tree, and section titles start-align. `layout-align` /
  `rtl_tree` plus mid-band text mass (`align-*`, right in RTL).
- Closed disclosure / pick chevron on the end (`▸` LTR, `◂` RTL).
  `layout-chevron` / `rtl_pick`. Button groups and checks:
  `layout-controls` / `rtl_checkbox`.
- Filled control faces carry label ink. Empty colored pads
  (Fill+align inside iced `button` drops RTL glyphs) are **broken**.
  `layout-button-face` / `rtl_themed_button`; `faces-controls`.
- Arabic/Urdu clocks use Eastern digits (`digits-eastern`). Hebrew
  keeps 123. Code, paths, and URLs stay left-to-right (`ltr-islands`).
- Painted chrome is the locale fill. Leftover English on those
  labels is **broken** (`leftover-src`, `copy`, `copy-keys`).
- Chrome stays one line (menu, look strip, toolbar, nav, buttons,
  chips, tabs). Mid-word wrap is **broken** (`wrap-chrome`).
  Markdown, code wrap-on, card titles, expand / job / hint, and
  status copy wrap on purpose.
- Linear progress and time motion fill from the **start** edge.
  A left-growing bar on `ar` / `ur` / `he` is **broken**.
- Arabic/Urdu/Persian painted numbers (progress, clocks, ranges)
  use Eastern digits. Hardcoded `25%` / `60%` on those locales is
  **broken**. Hebrew keeps 123.
- Control rows that mirror use `i18n::order`. A physical
  left-to-right `row!` of actions on an RTL page is **broken**.
- Tab/arrow order and mixed-string islands in live fields: live
  pass (`manual-pass.md`).

## 9. Platform

| Host | Proves |
|------|--------|
| Linux Xephyr (default) | Layout, tokens, most demos, inject |
| Host Linux display | Same + real window manager chrome |
| macOS / Windows host | UI font cascade, bold proportional, OS accent/chrome if follow-OS |

Do not claim macOS type quality from Xephyr. When `host_font` / `typo` /
follow-OS change: host or remote Mac pass required for **SHIP** on those
paths.

## 10. Performance feel (light check)

- First paint not empty for virtual collections
- Scrolling large lists stays usable (live); no full-list mount panic
- No obvious thrash (spinner stuck, scroll jump every frame) in a short live try

## Interact pairs

`idle` → `after-interact`: expected change **visible on screen**. Same
as idle → broken inject, target below fold, or broken style. Reorder or
fixed heights so inject targets sit above the fold.

## Fix layer

| Symptom | Prefer |
|---------|--------|
| Clip, Fill, icons, control API, tokens, type bind | `src/` |
| Packing, seeds, page group, job copy, multi-host heights | `icedtea-gallery/` |
| Both | Library for mechanism, gallery for story |

**Outcome:** patch source, recapture, confirm the defect is gone. Scoring
is only to prioritize work — not a report-writing exercise.

Never edit PNGs. Never generative-fake the UI.
