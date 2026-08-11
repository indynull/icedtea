# Architecture

icedtea is the design system, layouts, chrome, and widgets for a
desktop window. `icedtea::run!` loads theme, then starts the
window from `Boot` and your `new`, `update`, `view`, and `theme`
functions. `bootstrap` is the same path without opening a window.

## Boot

`Boot` sets title, application id, theme name, locale, density, and
window kind: application, dialog, or overlay.

## Theme

`theme::named` and `theme::mix` produce `Tokens`. Widgets take tokens
and a `Variant`. Register more colorways on `ThemeCatalog`. Code
highlighting follows the active colorway (`theme::code_highlight`).

## Actions

One `Action` feeds the menu bar, toolbar, shortcuts, context menus,
footer hints, and the command palette. The action carries your message
type. Write `ctrl+s` once: Command on macOS, Control on Linux and
Windows.

## Layout and chrome

Recipes (`dock`, `split`, `clamp`, `form`, `overlay`) compose rows and
columns. Patterns (`list_detail`, `inspector`, `workspace`, `drawer`,
`document_tabs`, `navigation_view`, `preferences`, `about`) combine
recipes with widgets. `workspace::DockNode` is the nested dock tree.

## Catalog

`catalog::ENTRIES` is the public surface. Each id has one constructor
and appears on a gallery page. Related atoms share a page. The
constructor takes `A11y` and tokens; rustdoc on that function is the
intended call. Constructors emit the application's messages; the
application owns `update`. Constructors, time, and virtual lists are
in [Widgets](widgets.md).
