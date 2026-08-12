# icedtea gallery visual rubric

Score each screenshot. Prefer evidence over taste essays.

## Verdicts per shot

| Tag | Meaning |
|-----|---------|
| **ok** | Usable, hierarchy clear, no obvious defect |
| **ugly** | Usable but dense, uneven gaps, weak contrast, or chrome noise |
| **broken** | Wrong state, empty when content expected, clipped text, overlapping controls, unreadable, all-black |

## Pixel / layout checks

1. **Edges** — content inset from card/panel edge (~12–16px chrome rhythm). No text flush to border.
2. **Clipping** — labels, chips, buttons, or list rows cut off; body not under status bar.
3. **Alignment** — nav rail vs main; filter/tool rows baseline-aligned where expected.
4. **Density** — 4/8px grid; more space between sections than within a card.
5. **Contrast** — muted meta still readable; danger/status meaningful.
6. **Focus** — active nav item and page title match the step caption; primary content is obvious.

## Product-state checks (validity)

1. **Page match** — caption/step says Controls/Markdown/List/… and the nav + title agree.
2. **Theme** — dark vs light beat matches chrome (light theme step must look light).
3. **Catalog content** — not an empty stub: controls show variants, list has rows, markdown has headings, code has syntax paint.
4. **Select surfaces** — selectable/code/markdown show the public constructors (fields with Copy where designed), not label-only paint-only fakes.
5. **Broken signals** — panic dialog, zero-size window, all-black frame, missing window chrome = **broken**.

## Demo bar

Polished demo frames: calm density, one clear hierarchy (nav → title → body), consistent chips/buttons, quiet footer. Flag **ugly** if it reads like a debug dump or uneven prototype.

## Timing interpretation

| Class | Guidance |
|-------|----------|
| boot_ms &lt; 3s | Fine for debug build on nested X |
| boot_ms &gt; 8s | Note cold compile or GPU thrash |
| step_ms &gt; 2s after settle | Suspect layout/scroll cost |
| mean_step_ms includes settle_ms | Subtract settle when comparing absolute numbers |
