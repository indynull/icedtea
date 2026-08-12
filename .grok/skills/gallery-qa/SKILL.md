---
name: gallery-qa
description: >
  Run the icedtea gallery QA harness (tour shots, optional control
  injects), inspect PNGs with read_file, and fix real constructor/demo
  defects. Use for gallery visual QA, demo capture, pixel polish, or
  "is the gallery broken".
metadata:
  short-description: "Gallery QA for icedtea polish"
---

# Gallery QA

**Product goal:** icedtea is chrome and widgets so apps can stay on
data and message flow (Textual-like for iced). This skill checks that
**shipped constructors and gallery demos** still look and behave that
way — including defects only a human would notice on a live window.

**One tool:** `scripts/gallery_qa.py` via `just gallery-qa`.

```bash
just gallery-qa                              # full tour, idle shots
just gallery-qa --interact                   # + control state changes
just gallery-qa --beats 0,8 --interact       # exceptional: subset
just gallery-gif                             # ship README/book GIF only
```

Output: `tmp/gallery-qa/<timestamp>/` (or `--out DIR`) with `shots/`,
`steps.jsonl`, `timings.json`, `CAPTURE.md`.

## How to look

Use **`read_file` on each `shots/*.png`**. That is multimodal vision.
Do not score by filename. Do not use `image_gen` to “fix” UI.

Score: **ok / ugly / broken** — see `references/rubric.md`.

Idle shots alone miss many defects. Always read the rubric’s **Live
behavior**, **Demo honesty**, and **Clip / stack** sections. Prefer
`--interact` on a full pass so after-shots exist for toggles, list
select, expand, tree, grid.

### Defect classes (manual QA → automated checklist)

When scoring, actively hunt these (they keep recurring):

| Class | Look for |
|-------|----------|
| **Clip / bleed** | Virtual list/table rows paint **over** filters, chrome, or siblings when scrolled; overscan not scissored |
| **Empty first paint** | Collection pane blank on idle (viewport 0 / Fill fight) until a later message |
| **Fill collapse** | Progress, slider, list, or table is a hairline or missing bar |
| **Dead demo** | Filters, pagination, split overflow, menus, or “more” present but do nothing |
| **Cross-talk** | One control’s message flips unrelated state (e.g. toggle checks a different checkbox) |
| **Hint lies** | Page/widget meta names a control that is not on that page |
| **Stub demo** | Grey boxes, one-line stubs, or duplicate fields instead of a product story |
| **Chrome atoms** | Chevrons as dots, tabs as bare text+×, text flush to card edges, uneven page padding |
| **Platform type** | Titles/logo mono or “1990s” bold (macOS cascade); Xephyr only proves Linux |
| **Catalog sense** | Wrong group/page for a control; two copies of the same control on one page |

**Inject ≠ pointer proof.** Inject drives `update` (checkbox, list index).
It does **not** prove slider drag, wheel scroll clip, split overflow
open, or context flyout placement. When those matter: fix from code
paths you can unit-test, or run host gallery and scroll/click, or ask
the human for a live pass on the suspect page.

**Library first.** Broken constructor → fix `src/`. Misleading demo or
dead binding → fix gallery **and** make the control actually drive
state. Do not hide a library clip bug by shrinking the demo.

## Agent loop

1. `just gallery-qa --interact` for a full pass (idle + inject pairs).
2. Open **every** shot with `read_file`. Score with the rubric tables.
3. **Broken / clear ugly:** fix library or pure gallery demos
   (`src/`, `icedtea-gallery/`), not the PNG. Match `AGENTS.md`
   (tokens, 4px grid, A11y, one path, live demos not `Nop`).
4. For multi-host pages (list + virtual column, long Controls): confirm
   each host has a usable height above the fold **or** page scrolls and
   inject targets stay visible in after-shots.
5. Re-run QA on affected beats (or full). Cap **3** fix→rewalk cycles
   per defect; then document residual.
6. `cargo test -p icedtea --lib` / `cargo test -p icedtea-gallery` for
   the modules you touched; `just check` before claiming a full cut.
7. If public gallery chrome shipped: `just gallery-gif`.
8. Write `VISUAL_REPORT.md` under the out dir (environment, shot table,
   fix log, verdict). Residual pointer-only risks go under Defer with a
   concrete next step.

## Knobs (exceptional only)

| Flag | Use when |
|------|----------|
| `--interact` | Need before/after control state (default for full polish) |
| `--beats N` / `a-b` / `a,b` | One page or a slice while iterating a fix |
| `--backend host` | Xephyr unavailable, or you need the real display (dirties host) |
| `--release` | Demo timing; snappier paint |
| `--no-build` | Binary already current |
| `--gif path` | Scratch demo package under out dir (not ship assets) |
| `--settle-ms N` | Slow GPU / flaky settle |

Prefer defaults over knobs. Use `--backend host` when macOS fonts or
OS chrome are in doubt; Linux Xephyr does not prove them.

## Inject (only with `--interact`)

Gallery reads `ICEDTEA_GALLERY_INJECT` (one command per line). Harness
drives it. Common: `check true`, `switch true`, `list 2`, `expand-card 1`,
`expand true`, `acc 1`. Full table: `scripts/gallery_qa.py`
(`DEFAULT_INTERACT`).

Message inject only — same as app `update`, not pointer hit-tests.

After-interact shot must show the **expect** string visibly. Same paint
as idle → broken inject, off-screen target, or broken style. Reorder
demos or give fixed heights so inject targets sit above the fold.

## Not this skill

- App business logic or domain widgets
- Inventing screenshots or dual gallery paths
- Replacing `just check` or unit tests
- Shipping crates / tags
- OS-wide theme plumbing beyond what the gallery already exercises

## Related

- Harness: `scripts/gallery_qa.py`
- Ship GIF: `scripts/gallery-gif.sh` → `just gallery-gif`
- Rubric: `references/rubric.md`
- Contract: `AGENTS.md`
