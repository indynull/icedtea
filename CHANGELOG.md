# Changelog

## Unreleased

First product release. Widgets and chrome for iced 0.14 desktop
applications. `icedtea::run!` boots theme and starts the window.
Constructors return `Element`s and emit the application's messages.

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
- Gallery pages every `catalog::ENTRIES` id. Guide:
  <https://indynull.github.io/icedtea/>. API docs:
  <https://docs.rs/icedtea>.

## 0.1.0

crates.io publish check. Tag and package path for `icedtea` 0.1.
