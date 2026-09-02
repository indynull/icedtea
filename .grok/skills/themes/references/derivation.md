# How other kits derive final colors

Survey for icedtea catalog work. The shipping rule is in
`../SKILL.md` and `theme::auto_ink`. This file is the source list.

## Textual (the catalog dump)

Community colorways in `catalog.json` started as Textual theme hex.
Textual body copy is **not** the `foreground` field.

- `$text` = `Color.automatic(87%)` (`auto 87%`)
- `$text-muted` = `Color.automatic(60%)`
- `Color.brightness` is Rec. 601: `(299r + 587g + 114b) / 1000` on
  0..1 channels.
- If brightness of the paper `< 0.5`, ink is white; else black.
- That ink is then mixed toward the paper by the percentage
  (`blend` / alpha).

`Color.automatic` only stores `auto=True` and an alpha. The white /
black pick happens when the color is resolved against a background
(`get_contrast_text`).

Source: [textual/src/textual/color.py](https://github.com/Textualize/textual/blob/main/src/textual/color.py)
(`automatic`, `brightness`, `get_contrast_text`). CSS:
[textual.textualize.io/styles/color](https://textual.textualize.io/styles/color/).

icedtea `auto_ink` is that resolution, fixed to the **canvas**.

## COSMIC / libcosmic

Pop!_OS COSMIC is an iced desktop. `cosmic-theme` does not import
terminal palettes as body ink. The operator picks a background (and
optional accent, text tint, neutral tint). Everything else is derived.

- Convert the seed to OKLCH.
- Build a 100-step ramp by walking lightness 0..1 at that hue/chroma
  (`steps`). Invalid sRGB chroma is binary-searched down.
- Paper index = lightness of the seed, scaled to 0..99.
- `get_text`: if paper index `< 60` treat as dark. Take the ramp color
  **70** steps toward the other end; if that walks off the ramp, try
  **50**; else snap to white or black. A `text_tint` replaces the ramp
  so ink keeps the tint's hue.
- Surfaces step a shorter distance on the same ramp.
  `get_small_widget_color` also caps chroma at 3%.

Source: [cosmic-theme/src/steps.rs](https://github.com/pop-os/libcosmic/blob/master/cosmic-theme/src/steps.rs).
Design note: [system76.com/blog customizing COSMIC](https://system76.com/blog/post/customizing-cosmic-theming-and-applications).

`cosmic-term` ships explicit scheme files (Solarized, Gruvbox, …)
with their own `foreground` / `background`. Those do not drive
libcosmic window chrome.

## Base16 / Tinted Theming

Sixteen named slots. No math at apply time.

| Slot | Job |
| --- | --- |
| `base00` | default background |
| `base01` | status / lighter background |
| `base02` | selection background |
| `base03` | comments |
| `base04` | status foreground |
| `base05` | default foreground |
| `base06` / `base07` | lighter foregrounds |
| `base08`–`base0F` | syntax / ANSI |

Dark schemes run `base00`→`base07` dark to light; light schemes reverse
that span. Solarized `base05` (`#839496`) on `base00` (`#002b36`) is
the authored terminal default. It is comment-weight as desktop body.

Sources: [tinted-theming/home styling.md](https://github.com/tinted-theming/home/blob/main/styling.md),
[base24/styling.md](https://github.com/tinted-theming/base24/blob/master/styling.md).

Tinted templates that target a desktop (Claude Code, GTK, …) map
`text ← base05` unless the template author overrides it.

## VS Code / xterm.js

The workbench theme JSON sets `editor.foreground` and
`terminal.foreground` explicitly. After that, the **terminal** can
rewrite cell foregrounds:

- `terminal.integrated.minimumContrastRatio` (xterm.js
  `minimumContrastRatio`), default 4.5 in VS Code.
- WCAG contrast `(L1 + 0.05) / (L2 + 0.05)`.
- If the cell is short, walk the foreground's luminance toward white
  or black until the ratio holds. If one direction hits the end,
  try the other (`ensureContrastRatio`).
- Same-color fg/bg is left alone.
- Side effect: dim ANSI (Solarized comments, Copilot gray) washes
  toward white or black.

Sources: [VS Code terminal appearance](https://code.visualstudio.com/docs/terminal/appearance),
[xterm.js Color.ts `ensureContrastRatio`](https://github.com/xtermjs/xterm.js/blob/master/src/common/Color.ts).

## WezTerm

Same lift, **off** unless set:

```lua
config.text_min_contrast_ratio = 4.5
```

Docs: luminance of the cell foreground is adjusted up or down.
Identical fg/bg is treated as deliberate. WCAG AA 4.5 is the
suggested value.

Source: [wezterm.org text_min_contrast_ratio](https://wezterm.org/config/lua/config/text_min_contrast_ratio.html).
`wezterm.color.load_base16_scheme` maps Base16 slots; it does not
recompute body ink.

## Ghostty

`background` and `foreground` are required hex (or X11 names).
The 256-color **palette** can be filled from the first 16:

- `palette-generate` (since 1.3): indices 16–255 from the base 16
  plus fg/bg. Explicit `palette = N=…` is never overwritten.
- 6×6×6 cube: trilinear interpolation in Lab, corners are the eight
  base hues with **fg/bg in place of black/white**.
- Gray ramp 232–255: interpolate paper → ink.
- `palette-harmonious`: invert the generated span so light themes
  stay semantically ordered.

This is palette fill, not GUI body ink.

Sources: [ghostty.org palette-generate](https://ghostty.org/docs/config/reference/),
[jake-stewart gist](https://gist.github.com/jake-stewart/0a8ea46159a7da2c808e5be2177e1783).
Kitty and SwiftTerm shipped the same generator.

## themer

One authored set (`shade0`–`shade7`, `accent0`–`accent7`) templates
out to terminals, editors, and wallpapers.

- `shade0` background
- `shade6` / `shade7` foreground text
- `shade3` comments

No contrast pass. [github.com/themerdev/themer](https://github.com/themerdev/themer).

## Tabby

Desktop terminal. Tab chrome from a single base:

- `DeriveTextColor(bg)`: white or black, WCAG AA **large-text** 3:1
  (prefer white on a colored tab).
- `EnsureContrast(fg, bg, min)`: WCAG 4.5 / 7.0 lift.

Not a widget kit; same “pick ink from paper” job.

## Terminal.Gui

`Scheme.Normal` is required. Unset **Focus** swaps that fg/bg.
Other roles (`HotNormal`, `Disabled`, …) brighten or dim with
dark/light paper awareness (`GetBrighterColor` / `GetDimmerColor`).
`DeriveAccent` builds an opaque Normal from a base scheme.

Source: [Terminal.Gui Scheme.cs](https://github.com/gui-cs/Terminal.Gui/blob/v2_develop/Terminal.Gui/Drawing/Scheme.cs).

## Material 3 (icedtea roles)

HCT from a seed; each role is a tone on that hue. icedtea
`Tokens::scheme()` maps short fields onto those roles
(`canvas` → `surface`, `text` → `on_surface`, `muted` →
`on_surface_variant`). The catalog picks paper and accents as hex,
then `auto_ink` supplies `on_surface` / `on_surface_variant`.
Do not invent a second HCT pass for community colorways.

## What icedtea takes from this

1. **Textual auto** for catalog body and mute (matches the TUI the
   colorways came from).
2. **COSMIC** as the iced desktop that derives ink from paper in
   OKLCH. Use that if a later change needs tinted ink, not gray mix.
3. **VS Code / WezTerm lift** only as an app-level opt-in, never the
   catalog default (it erases Solarized).
4. **Ghostty generate** only if we ever ship a 256 ANSI table.
   Body ink stays explicit-or-auto, not interpolated cube.
5. **Base16 slots** for mapping a `.yaml` import: `canvas ← base00`,
   `primary ← base0D`, then `auto_ink` for text. Do not assign
   `text ← base05`.
