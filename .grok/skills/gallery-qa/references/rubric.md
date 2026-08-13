## Material Design 3

- Spacing on a **4 dp grid**; default density 8 dp gap, 48 dp touch targets.
- Controls show M3 states: enabled, disabled, hovered, focused, pressed, selected/error where defined.
- Surfaces use token roles only (no one-off hex).
- Type hierarchy: label < body < title (M3 type scale).

# Gallery QA rubric

Score every shot (and every live page) **ok / ugly / broken**. Prefer
one primary class per defect. Severity first, then taste.

| Tag | Meaning | Ship bar |
|-----|---------|----------|
| **ok** | Intentional hierarchy, honest demo, states clear | Allowed |
| **ugly** | Usable but weak contrast, density, alignment, or chrome | Full cut: fix or residual |
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

## 2. Alignment and geometry

- Shared leading edges for labels/fields in a column
- Buttons in a row share height and baseline; split primary + overflow same height
- Icons optically centered in hit boxes; chevrons are chevrons, not dots
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

Busy, toast, jobs, progress: motion or value must be **perceivable** in
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
- First useful frame shows content (empty list until scroll message = broken)
- Sticky/frozen columns stay put; horizontal scroll does not orphan them
- Overlays (menu, context, palette, dialog) sit above content with clear
  z-order; flyouts align to parent row and do not needlessly overlap

## 7. Interaction integrity

- Inject after-shot ≠ idle in the way `expect` describes
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

## 8. Platform

| Host | Proves |
|------|--------|
| Linux Xephyr (default) | Layout, tokens, most demos, inject |
| Host Linux display | Same + real window manager chrome |
| macOS / Windows host | UI font cascade, bold proportional, OS accent/chrome if follow-OS |

Do not claim macOS type quality from Xephyr. When `host_font` / `typo` /
follow-OS change: host or remote Mac pass required for **SHIP** on those
paths.

## 9. Performance feel (light check)

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
