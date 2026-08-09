# Changelog

## 0.1.0

- GitHub check runs `just check` on Linux, macOS, and Windows at Rust
  1.89, and the test suite on Ubuntu `stable` and `beta`. Tag `vX.Y.Z`
  publishes `icedtea` to crates.io.
- Built-in colorways: Solarized, Gruvbox, Catppuccin, Nord, Tokyo Night,
  Dracula, Everforest, Kanagawa, Ayu, and more. Gallery theme page and
  header select switch chrome and code highlighting together.
- Book covers install, `run!`, actions, layout, theming, navigation,
  and overlay windows.

- Initial public crate: design system, layouts, actions, command palette,
  window kinds, widget catalog, persistence, undo, internationalization,
  gallery, and book.
- Split sash drag uses window-space pointer events (`listen_sash`).
- Every public widget constructor takes `A11y` (name, role, value, disabled,
  checked) and attaches it to the iced node id.
- Menu, toolbar, and status bar follow `Boot` locale direction.
- UI copy uses the platform sans. Applications may load their own UI
  family. JetBrains Mono is bundled for code.
- Gallery markdown page is a full document; code page highlights many
  languages behind a language select. Catalog search sits on the sidebar.
- `examples/consumer` is a themed window (menu, list/detail, toasts, live
  theme).
- Menu bar is File / Edit / View titles; each opens an overlay list of
  that group's actions (with shortcuts). Dialogs page runs native
  Open/Save/Folder and an in-app save sheet. The gallery window uses
  that bar, a theme select, searchable sidebar, content, and status.
