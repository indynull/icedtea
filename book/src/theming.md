# Theming

Colorways seed **Material Design 3** schemes (`m3::Scheme`). See [Material Design 3 foundations](./m3-foundations.md).


Styling is Rust: semantic tokens plus mixing rules.

Short token fields map onto M3 roles (`Tokens::scheme()` for the full
scheme). Aliases: `canvas`/`surface`/`panel`/`text`/`muted`/`primary`/
`accent`/`danger`/`border`/`selection`. Baseline light/dark keep exact
M3 containers; community colorways sync containers from aliases and
recompute solid-fill `on_*` roles (`on_primary` and the rest) for
contrast on those fills.
Washes use scheme state layers (`hover_fill` / `pressed_fill`). Desktop
control corners are M3 shape **None** (see [foundations](./m3-foundations.md)).
Constructors take `Tokens`.

Built-in names are 40 palettes: `dark`, `light`, `high-contrast`, and
community colorways (Solarized, Gruvbox, Catppuccin, Nord, Tokyo Night,
Dracula, Everforest, Kanagawa, Ayu, GitHub, and others).
[`theme::named`](https://docs.rs/icedtea/latest/icedtea/theme/fn.named.html)
and `theme::code_highlight` pick UI tokens and the iced highlighter
face together. Register more on `ThemeCatalog`. `Boot.theme` is a
concrete name and defaults to `dark`. `light` and `dark` are a
neutral desktop pair (not a community skin). `markdown_view` paints
inline code and links from `Tokens::scheme()` (`on_surface`,
`surface_container_high`, `primary`). Truncation is slicing the source
before `MarkdownDoc::parse`.

```rust
let mut cat = icedtea::theme::ThemeCatalog::new();
cat.register("brand", icedtea::theme::named("dark").tokens, true);
let tokens = cat.resolve("brand");
let _ = tokens.primary;
```

Live switch: store a theme name on state and return
`icedtea::theme::iced_theme(&name, tokens)` from the theme function.

Families are explicit pairs (`github` → `github-dark` / `github-light`,
and the other real couples). A follow-OS preference selects the light
or dark member of one family (default family: `light` / `dark`).
High-contrast is its own name. Names without a pair do not follow the
OS. Persist stores `theme` plus optional `family` and `follow_os`
(`follow_os` defaults on). Mode changes come from iced
(`system::theme` / `theme_changes`).

## Follow-OS chrome

**Default:** a named colorway only. Set `follow_os` to false, or pass
[`OsChrome::empty`](https://docs.rs/icedtea/latest/icedtea/theme/struct.OsChrome.html),
and no desktop colors are applied.

**Opt-in:** with `follow_os` true:

1. Resolve the family light/dark member from OS appearance (as above).
2. Read host chrome via [`theme::os_chrome`](https://docs.rs/icedtea/latest/icedtea/theme/fn.os_chrome.html)
   (boot, main thread on macOS) or
   [`theme::listen_os_chrome`](https://docs.rs/icedtea/latest/icedtea/theme/fn.listen_os_chrome.html)
   (live updates when accent or color-scheme changes).
3. Apply with
   [`theme::apply_os_chrome`](https://docs.rs/icedtea/latest/icedtea/theme/fn.apply_os_chrome.html).
   Each `Some` field overwrites the matching token; the rest of the
   colorway stays. Selection is rebuilt from primary + canvas.

What the host fills (missing fields stay on the colorway):

| Token | macOS | Windows | Linux (portal, Wayland or X11) |
| --- | --- | --- | --- |
| `primary` | control accent | system accent | portal accent when published |
| `canvas` | window background (system gray, not paper white) | `COLOR_WINDOW` | — |
| `surface` | text background (often white in light mode) | button face | — |
| `panel` | control background | button face | — |
| `text` | label | window text | — |
| `muted` | secondary label | gray text | — |
| `border` | separator | button shadow | — |

Success, warning, and danger always stay on the colorway. Decorated
windows keep the native title bar. Turn follow-OS off for high-contrast
or a fixed brand palette.

```rust
use icedtea::theme::{self, OsChrome};

let base = theme::named("dark").tokens;
// Opt out: colorway only
let pure = theme::apply_os_chrome(base, false, OsChrome::empty());
assert_eq!(pure, base);

// Opt in: layer whatever the host reported
let live = theme::os_chrome();
let tok = theme::apply_os_chrome(base, true, live);
let _ = tok.canvas;
```

- [`theme`](https://docs.rs/icedtea/latest/icedtea/theme/index.html)
- [source](https://github.com/indynull/icedtea/blob/master/src/theme.rs)
- [crates.io](https://crates.io/crates/icedtea)
