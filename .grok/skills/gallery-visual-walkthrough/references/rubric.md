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

## Usability / accessibility (visual)

Score from the frame; then confirm in code when fixing.

| Check | Broken / ugly signal |
|-------|----------------------|
| Target size | Controls look crushed or labels touch edges |
| Hierarchy | Primary action not distinct from Quiet/Ghost |
| Disabled | Disabled rows/buttons same weight as enabled |
| Selection | Selected list/table row or field has no visible selection |
| Empty state | Blank void without meta/status copy when page should explain |
| Motion of focus | Active nav page does not match caption (wrong beat paint) |

## Interaction shots (`kind=after-interact`)

Compare to the preceding **idle** shot on the same beat.

| Tag | Meaning |
|-----|---------|
| **ok** | Expected state change is obvious in the frame |
| **ugly** | State changed but hard to see (weak selection, no checked paint) |
| **broken** | Inject applied (or claimed) but paint matches idle / wrong control |

## Auto-fix eligibility

| Fix now (in source) | Defer / report only |
|---------------------|---------------------|
| Padding, spacing, token contrast, clipped layout in a constructor | Upstream iced paint bugs with no icedtea lever |
| Gallery dual-path / stub content | One-line fixture taste without product impact |
| Missing disabled/selection styling | Continuous cross-block markdown select (TODO) |
| Density grid violations in chrome rows | Host font rasterization differences |

Never “fix” by regenerating the screenshot without a code change.

## Timing interpretation

| Class | Guidance |
|-------|----------|
| boot_ms &lt; 3s | Fine for debug build on nested X |
| boot_ms &gt; 8s | Note cold compile or GPU thrash |
| step_ms &gt; 2s after settle | Suspect layout/scroll cost |
| mean_step_ms includes settle_ms | Subtract settle when comparing absolute numbers |
