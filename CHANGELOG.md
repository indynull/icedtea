# Changelog

## 0.1.0

First release. Widgets and chrome for iced 0.14 desktop applications.
`icedtea::run!` boots theme and starts the window. Constructors return
`Element`s and emit the application's messages.

- One `Action` for menus, toolbars, shortcuts, the command palette, and
  footer hints. Menu bar is File / Edit / View; each title opens that
  group's actions.
- Layout recipes: dock, split, pad, form, overlay. Split sash drag uses
  window-space pointer events (`listen_sash`).
- Semantic tokens and `theme::mix`. Forty named colorways (dark, light,
  high-contrast, Solarized, Gruvbox, Catppuccin, Nord, Tokyo Night,
  Dracula, Everforest, Kanagawa, Ayu, and others). Families pair light
  and dark names; follow-OS picks the member. `ThemeCatalog::register`
  adds application colorways. Code highlighting follows the UI colorway.
- Application, dialog, and overlay window kinds. Overlay `Boot::size` is
  the inner size. `window::place` uses the display under the pointer.
  Hide policy ignores in-palette controls.
- Every public widget takes `A11y`. Lists and tables virtualize with
  `virtual_pads` (overscan and a cover index). Text inputs take
  `on_submit`. `key::press` reports arrows, page, home, and end.
  `themed_scroll` can stick to the end. Chip and badge use `Variant`.
  `MarkdownDoc` parses in the application; `markdown_view` borrows.
- Compact tools: `Boot::size`, `themed_button_sized`, `layout::pad`,
  display reading, typed keys. The README pad is a four-function tool.
- User-facing text uses the platform sans; code uses the platform mono.
  Chrome follows `Boot` locale direction.
- Gallery (`cargo run -p icedtea-gallery`) pages every
  `catalog::ENTRIES` id. Guide: <https://indynull.github.io/icedtea/>.
  API docs: <https://docs.rs/icedtea>.
- `just check` on Linux, macOS, and Windows at Rust 1.89. Tag `vX.Y.Z`
  (same as `Cargo.toml` `version`) publishes `icedtea` to crates.io.
