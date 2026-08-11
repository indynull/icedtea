# Changelog

## Unreleased

Guide lists every catalog id by gallery group, with rustdoc, source,
and crate links. Crate-root rustdoc is a teaching tour. The first
window uses one `Action` and a toolbar (`examples/hello.rs`).

Workspace docks, drawers, and tool panels. Variable-height lists,
frozen columns, range selection, command contexts and sequences,
palette prompts, cheatsheet, inspector, document tabs, and a job
strip. `MarkdownDoc::item_offset` drives outline jump. Gallery demos
handle the messages their widgets emit.

## 0.2.0 — 2026-08-11

Widgets and chrome for iced 0.14 desktop applications.
`icedtea::run!` boots theme and starts the window. Constructors
return `Element`s and emit the application's messages.

- One `Action` for menus, toolbars, shortcuts, the command palette, and
  footer hints. `ctrl+s` is Command on macOS and Control on Linux and
  Windows. F1-F24 parse and press.
- Layout: dock, split, pad, form, overlay. Split sash drag uses
  window-space pointer events.
- Semantic tokens and `theme::mix`. Named colorways, high-contrast,
  light/dark families. Follow-OS can take the desktop accent.
- Application, dialog, and overlay windows. Overlay placement uses the
  display under the pointer.
- Every public widget takes `A11y`. Lists, tables, and logs virtualize.
- Image slots keep their box. Item-grid tiles share the row. An open
  accordion shows a body under its header.
- One constructor per catalog id. That function takes `A11y` and
  tokens. Rustdoc on the function is the intended call.
  `image_slot`, `key::listen`, `themed_scroll`, and
  `Breakpoint::from_width` are those jobs.
- Gallery pages host every `catalog::ENTRIES` id. Related controls
  share a page. `just gallery-gif` records the README tour. Guide:
  <https://indynull.github.io/icedtea/>. API docs:
  <https://docs.rs/icedtea>.

## 0.1.0

crates.io publish check. Tag and package path for `icedtea` 0.1.
