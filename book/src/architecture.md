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
type.

## Layout and chrome

Recipes (`dock`, `split`, `clamp`, `form`, `overlay`) compose rows and
columns. Patterns (`list_detail`, `nav_view`, `preferences`, `about`)
combine recipes with widgets.

## Catalog

`catalog::ENTRIES` is the public surface. Each id has a gallery page.
