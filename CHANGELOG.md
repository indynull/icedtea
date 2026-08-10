# Changelog

## 0.1.0

- `textarea` and `highlighted_code` take a height (`layout::FILL` or
  `layout::fixed`). `layout::dock` fills its parent so a main-window
  center gets leftover height. `tree_view` distinguishes disclosure
  toggle from row select. `list_detail` takes a sidebar size and fills
  both panes. `row_box` / `column_box` take width and height.
  `key::handle` invokes modifier-chord actions while text is focused.
- README pad is a compact four-function tool (sized `Boot`, display
  reading, density tiles, typed keys). The gallery is the in-tree
  window.

- GitHub check runs `just check` on Linux, macOS, and Windows at Rust
  1.89, and the test suite on Ubuntu `stable` and `beta`. Tag `vX.Y.Z`
  publishes `icedtea` to crates.io.
- Built-in colorways: 40 named palettes (dark, light, high-contrast,
  plus community sets such as Solarized, Gruvbox, Catppuccin, Nord,
  Tokyo Night, Dracula, Everforest, Kanagawa, Ayu). Gallery theme page
  and header select switch chrome and code highlighting together.
- Book covers install, `run!`, actions, layout, theming, navigation,
  and overlay windows.

- Initial public crate: design system, layouts, actions, command palette,
  window kinds, widget catalog, persistence, internationalization,
  gallery, and book.
- Split sash drag uses window-space pointer events (`listen_sash`).
- Every public widget constructor takes `A11y` (name, role, value, disabled,
  checked) and attaches it to the iced node id.
- Menu, toolbar, and status bar follow `Boot` locale direction.
- UI copy uses the platform sans; code uses the platform mono.
  Applications that want a named family load it themselves.
- `widget::image` takes an application `Handle` and size. Gallery
  fixtures (markdown document, highlighted languages) live in
  `icedtea-gallery`, not the library. Document undo is not a library
  type.
- List and table use an icedtea scroll rail with a 24px minimum handle.
  iced's own scroller still floors at 2px on `themed_scroll`.
- Gallery markdown page is a full document; code page highlights many
  languages behind a language select. Catalog search sits on the sidebar.
- Compact-tool seams: `Boot::size` / `min_size`, `themed_button_sized`,
  `layout::pad` equal-fill tiles, `typo::DISPLAY` and display reading
  widgets, `key::typed` / `key::press` (Shift+8 is `*`), accessible
  name no longer replaces the visible caption. Book page Compact tools.
  The README pad uses those constructors.
- Menu bar is File / Edit / View titles; each opens an overlay list of
  that group's actions (with shortcuts). Dialogs page runs native
  Open/Save/Folder and an in-app save sheet. The gallery window uses
  that bar, a theme select, searchable sidebar, content, and status.
