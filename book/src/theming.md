# Theming

Styling is Rust: semantic tokens plus mixing rules.

Tokens: `canvas`, `surface`, `panel`, `text`, `muted`, `primary`,
`accent`, `success`, `warning`, `danger`, `border`, `selection`,
`selection-text`. Washes (hover, pressed, chip) come from `theme::mix`.

Built-in names are 40 palettes: `dark`, `light`, `high-contrast`, and
community colorways (Solarized, Gruvbox, Catppuccin, Nord, Tokyo Night,
Dracula, Everforest, Kanagawa, Ayu, GitHub, and others). `theme::named`
and `theme::code_highlight` pick UI tokens and the iced highlighter
face together. Register more on `ThemeCatalog`.

```rust,ignore
let mut cat = icedtea::theme::ThemeCatalog::new();
cat.register("brand", icedtea::theme::named("dark").tokens, true);
let tokens = cat.resolve("brand");
```

Live switch: store a theme name on state and return
`icedtea::theme::iced_theme(&name, tokens)` from the theme function.
