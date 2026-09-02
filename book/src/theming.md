# Theming

Styling is Rust: semantic tokens plus mixing rules. Short token fields
map onto Material Design 3 roles (`Tokens::scheme()` for the full
set). See [Material Design 3 foundations](./m3-foundations.md).

Aliases: `canvas`/`surface`/`panel`/`text`/`muted`/`primary`/
`accent`/`danger`/`border`/`selection`.
[`Tokens::from_aliases`](https://docs.rs/icedtea/latest/icedtea/theme/struct.Tokens.html#method.from_aliases)
builds a scheme from those fields. Colorways sync containers from
those aliases and recompute solid-fill `on_*` roles for contrast.
Washes use scheme state layers (`hover_fill` / `pressed_fill`). Desktop
control corners are M3 shape **None** unless
[`Tokens::with_shape`](https://docs.rs/icedtea/latest/icedtea/theme/struct.Tokens.html#method.with_shape)
selects Tight, Soft, Pill, or the Material component map
(see [foundations](./m3-foundations.md)). Tabs, app bars, banners,
and exclusive segments stay flush under every policy.
Type size is the M3 scale times
[`Tokens::font_scale`](https://docs.rs/icedtea/latest/icedtea/theme/struct.Tokens.html#method.with_font_scale)
(1.0 by default). Shadows follow
[`Tokens::with_elevation`](https://docs.rs/icedtea/latest/icedtea/theme/struct.Tokens.html#method.with_elevation).
Constructors take `Tokens`. `Tokens.density` (Compact / Default /
Comfortable) sets control pad (`pad`), inter-item gap (`space`), and
container inset. A `*Face` enum on the same constructor picks the
painted look (`RowFace`, `CardFace`, `FieldFace`, `TreeFace`).
`ControlSize` Compact and Comfortable stay per-control overrides;
Default follows window density.

Built-in names are 40 palettes: `dark`, `light`, `high-contrast`, and
community colorways (Solarized, Gruvbox, Catppuccin, Nord, Tokyo Night,
Dracula, Everforest, Kanagawa, Ayu, GitHub, and others).
[`theme::named`](https://docs.rs/icedtea/latest/icedtea/theme/fn.named.html)
and `theme::code_highlight` pick UI tokens and the iced highlighter
face together. Catalog paper, accents, and status stay hex. Body and
mute ink are
[`theme::auto_ink`](https://docs.rs/icedtea/latest/icedtea/theme/fn.auto_ink.html)
on the canvas (87% / 60%). A dumped terminal foreground is not body
copy. `high-contrast` is defined in Rust. Register more on
`ThemeCatalog`. `Boot.theme` is a concrete name and defaults to `dark`.
`Boot::transparent` asks iced for an ARGB buffer.
`Boot::decorations` sets the client title bar. `light` and `dark` are a
neutral desktop pair. Persist defaults `follow_os` on, so host chrome
layers onto that pair; pick another catalog name to choose a colorway.
`markdown_view` paints inline code and links from `Tokens::scheme()`
(`on_surface`, `surface_container_high`, `primary`). Truncation is
slicing the source before `MarkdownDoc::parse`.

```rust
let mut cat = icedtea::theme::ThemeCatalog::new();
cat.register("brand", icedtea::theme::named("dark").tokens, true);
let tokens = cat.resolve("brand")
    .with_font_scale(1.125)
    .with_shape(icedtea::m3::ShapePolicy::Material);
let _ = tokens.primary;
```

Persist stores the same fields on
[`UiState`](https://docs.rs/icedtea/latest/icedtea/persist/struct.UiState.html)
(`density`, `font_scale`, `shape`, `elevation`). Restore with
`ui.look(tokens)`. `Boot` has the same setters so a window starts
on that look.

Chrome labels come from
[`Catalog::for_locale`](https://docs.rs/icedtea/latest/icedtea/i18n/struct.Catalog.html).
English, Vietnamese, Japanese, Chinese, Arabic, and Urdu are built
in. Direction is
[`direction_for`](https://docs.rs/icedtea/latest/icedtea/i18n/fn.direction_for.html)
on the locale (Arabic and Urdu are right-to-left). `Tokens.direction`
flips pick-list chevrons, disclosure marks, tree twisties, search
icons, button leading/trailing icons, list slots, and the scroll rail
(end side). Menu and toolbar rows take
that direction and call
[`order`](https://docs.rs/icedtea/latest/icedtea/i18n/fn.order.html).

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

Persist defaults `follow_os` on. The desktop pair (`light` / `dark`)
follows OS appearance; host colors layer on top. A named colorway is a
choice: set `theme` to that name, and set `follow_os` to false to keep
the palette as authored (or pass
[`OsChrome::empty`](https://docs.rs/icedtea/latest/icedtea/theme/struct.OsChrome.html)).

With `follow_os` true:

1. Resolve the family light/dark member from OS appearance (as above).
2. Read host chrome via [`theme::os_chrome`](https://docs.rs/icedtea/latest/icedtea/theme/fn.os_chrome.html)
   (boot, main thread on macOS) or
   [`theme::listen_os_chrome`](https://docs.rs/icedtea/latest/icedtea/theme/fn.listen_os_chrome.html)
   (live updates when accent or color-scheme changes).
3. Apply with
   [`theme::apply_os_chrome`](https://docs.rs/icedtea/latest/icedtea/theme/fn.apply_os_chrome.html).
   Each `Some` field overwrites the matching token; the rest of the
   pair (or chosen colorway) stays. Selection is rebuilt from primary + canvas.

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
- [source](https://github.com/indynull/icedtea/blob/main/src/theme.rs)
- [crates.io](https://crates.io/crates/icedtea)
