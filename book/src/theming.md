# Theming

Styling is Rust: semantic tokens plus mixing rules.

Tokens: `canvas`, `surface`, `panel`, `text`, `muted`, `primary`,
`accent`, `success`, `warning`, `danger`, `border`, `selection`,
`selection-text`. Washes (hover, pressed, chip) come from
[`theme::mix`](https://docs.rs/icedtea/latest/icedtea/theme/fn.mix.html).
`Tokens::faces` adds lighten/darken, text-on-wash, scrollbar, input,
link, and focus. Constructors still take `Tokens`.

Built-in names are 40 palettes: `dark`, `light`, `high-contrast`, and
community colorways (Solarized, Gruvbox, Catppuccin, Nord, Tokyo Night,
Dracula, Everforest, Kanagawa, Ayu, GitHub, and others).
[`theme::named`](https://docs.rs/icedtea/latest/icedtea/theme/fn.named.html)
and `theme::code_highlight` pick UI tokens and the iced highlighter
face together. Register more on `ThemeCatalog`. `Boot.theme` is a
concrete name and defaults to `dark`. `light` and `dark` are a
neutral desktop pair (not a community skin). `markdown_view` paints inline
code and links from `Tokens` (`text`, `panel`, `accent`). Truncation
is slicing the source before `MarkdownDoc::parse`.

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
When follow-OS is on, `theme::os_accent` / `theme::listen_os_accent`
read the desktop accent (settings portal, Windows accent, macOS
control accent). `theme::apply_os_accent` puts that color in
`Tokens.primary` and rebuilds selection. Canvas and text stay the
family's tokens. Decorated windows keep the native title bar.

- [`theme`](https://docs.rs/icedtea/latest/icedtea/theme/index.html)
- [source](https://github.com/indynull/icedtea/blob/master/src/theme.rs)
- [crates.io](https://crates.io/crates/icedtea)
