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
data and message flow (Textual-like for iced). This skill only checks
that the **shipped constructors and gallery demos** still look and
behave that way.

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

## Agent loop

1. `just gallery-qa` (add `--interact` when state changes matter).
2. Open every shot with `read_file`.
3. **Broken / clear ugly:** fix **library or pure gallery demos**
   (`src/`, `icedtea-gallery/`), not the PNG. Match `AGENTS.md`
   (tokens, 4px grid, A11y, one path, no dual-pane fakes).
4. Re-run QA on affected beats (or full). Cap **3** fix→rewalk cycles
   per defect; then document residual.
5. `cargo test -p icedtea --lib` / `cargo test -p icedtea-gallery` /
   `just check` as appropriate.
6. If public gallery chrome shipped: `just gallery-gif`.
7. Write a short `VISUAL_REPORT.md` under the out dir when doing a full pass.

## Knobs (exceptional only)

| Flag | Use when |
|------|----------|
| `--interact` | Need before/after control state (inject messages) |
| `--beats N` / `a-b` / `a,b` | One page or a slice while iterating a fix |
| `--backend host` | Xephyr unavailable (dirties the live display) |
| `--release` | Demo timing; snappier paint |
| `--no-build` | Binary already current |
| `--gif path` | Scratch demo package under out dir (not ship assets) |
| `--settle-ms N` | Slow GPU / flaky settle |

Default is enough for ordinary QA. Prefer defaults over knobs.

## Inject (only with `--interact`)

Gallery reads `ICEDTEA_GALLERY_INJECT` (one command per line). Harness
drives it. Common: `check true`, `switch true`, `list 2`, `expand-card 1`,
`expand true`. Full table is in `scripts/gallery_qa.py` (`DEFAULT_INTERACT`).

This is **message inject** (same as app `update`), not pointer hit-tests.

## Not this skill

- App business logic or domain widgets
- Inventing screenshots or dual gallery paths
- Replacing `just check` or unit tests
- Shipping crates / tags

## Related

- Harness: `scripts/gallery_qa.py`
- Ship GIF: `scripts/gallery-gif.sh` → `just gallery-gif`
- Rubric: `references/rubric.md`
- Contract: `AGENTS.md`
