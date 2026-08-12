# Gallery QA rubric

| Tag | Meaning |
|-----|---------|
| **ok** | Usable, clear hierarchy, no obvious defect |
| **ugly** | Usable but dense, weak contrast, uneven gaps |
| **broken** | Wrong page, empty stub, clipped UI, invisible selection/disabled, all-black |

## Layout

- ~12–16px inset; 4/8px spacing grid
- No clipped labels/buttons; body not under status bar
- Nav selection matches page title/caption

## Product

- Catalog page shows real demo content (not empty)
- Select surfaces use real constructors (not label-only fakes)
- Light theme beat looks light; dark looks dark

## Interact pairs (`idle` → `after-interact`)

Expected state change must be **visible**. Same paint as idle → broken inject or broken style.

## Fix

Change **source**, re-capture. Never edit PNGs or `image_gen` the UI.
