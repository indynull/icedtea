---
name: themes
description: >
  Use when adding or editing icedtea catalog colorways, deriving body
  or mute ink from a terminal palette, importing Base16 / Textual /
  Ghostty / COSMIC colors, scoring named theme paint, or changing
  theme::auto_ink or assets/themes/catalog.json.
metadata:
  short-description: "Catalog colorways and derived body ink"
---

# Themes

**Objective:** keep named colorways readable on their own paper.
Paper, accents, and status stay hex. Body and mute ink are derived.

The function is [`theme::auto_ink`](../../../src/theme.rs).
`catalog_from_json` ignores JSON `text` / `muted` / `foreground`.
`high-contrast` is defined in Rust next to the catalog.

```rust
// Rec. 601 brightness of canvas < 0.5 → white, else black.
// Mix that ink toward the paper.
theme::auto_ink(canvas, 0.87) // Tokens.text   (Textual $text)
theme::auto_ink(canvas, 0.60) // Tokens.muted  (Textual $text-muted)
```

Check: `theme::tests::catalog_body_and_mute_follow_auto_ink`.

## One path

| Field | Source |
| --- | --- |
| `canvas` / `surface` / `panel` | catalog hex (paper) |
| `primary` / `accent` / status / `border` | catalog hex |
| `text` / `muted` | `auto_ink` on **canvas** |
| `selection` | `mix(primary, canvas, 0.28)` |
| `high-contrast` | Rust table, not JSON |

A new colorway is a hex record in `assets/themes/catalog.json` plus
the `dark` flag. Do not write body ink into that file. Do not lift
to a WCAG ratio on the catalog path (that washes Solarized / ANSI
the way VS Code's terminal contrast lift does).

`Tokens::from_aliases` still takes caller ink. Apps that want a
different body color pass it there. Follow-OS host `text` /
`muted` still overwrite when `follow_os` is on.

## Other kits (how they get final ink)

Full notes: `references/derivation.md`.

| Kit | Body ink |
| --- | --- |
| **Textual** | `$text` / `$text-muted` = `auto 87%` / `auto 60%` on the widget paper. Rec. 601, then mix white or black. Catalog `foreground` is a different token. |
| **COSMIC** (`cosmic-theme`) | 100-step OKLCH lightness ramp from the seed. Text is 70 steps from paper (fallback 50). Optional `text_tint` swaps the ramp. Closest iced desktop sibling. |
| **Base16 / Tinted** | Explicit slots. `base00` paper, `base05` body, `base03` comments. Solarized `base05` is mid-gray on purpose. |
| **VS Code / xterm.js** | Keep the cell foreground. Lift luminance to WCAG 4.5:1 (`minimumContrastRatio`). Washes ANSI. |
| **WezTerm** | Same lift, opt-in (`text_min_contrast_ratio`). |
| **Ghostty** | `foreground` and `background` are explicit. `palette-generate` interpolates indices 16–255 in Lab from the base 16 plus fg/bg. Does not invent body ink. |
| **themer** | `shade0` paper, `shade6`/`shade7` body. Explicit. |
| **Tabby** | White or black from WCAG 3:1 on the tab paper. |
| **Terminal.Gui** | Focus swaps Normal fg/bg; other roles brighten or dim. |
| **Material 3** | HCT tone from a seed. icedtea maps aliases onto those roles **after** ink is chosen. |

## Common mistakes

| Excuse | Reality |
| --- | --- |
| Dump Textual / kitty `foreground` | That is ANSI / syntax gray. Textual body is `auto`, not `$foreground`. |
| Use Base16 `base05` as desktop body | Solarized `base05` on `base00` is a comment-weight gray in a GUI. |
| WCAG-lift the catalog | Same wash as VS Code's terminal contrast. Leave lift to a later opt-in if an app asks. |
| Derive mute from the dumped fg | Mute is `auto_ink(canvas, 0.60)`, same paper, lower mix. |
| Second constructor for "TUI ink" | One catalog load path. `auto_ink` is public for apps that build tokens themselves. |

## Related

- `src/theme.rs` — `auto_ink`, `catalog_from_json`, `named`
- `assets/themes/README.md` — catalog fields
- `book/src/theming.md` — reader path
- `references/derivation.md` — sources and algorithms
- gallery-qa — score the theme page still, then the shot
