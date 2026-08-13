# TODO

## Do

- Webpage-like continuous select in `markdown_view`: drag a range across
  headings, paragraphs, and lists (not only within one block). Keep real
  markdown layout (lists, code frames, spacing). Do not ship a single
  mixed-size `Rich` flatten — that broke layout and multi-line selection
  paint. Full-document copy already uses `copy_text` on
  `MarkdownDoc::source`. Code/fields (`text_editor` + `select_only`) stay
  the clean multi-line highlight reference.

Desktop Material leftovers (first batch already shipped: segmented and
icon buttons, range slider, indeterminate checkbox, field supporting
text, filter chips, sectioned/cascade menus, side sheet, search clear):

- Navigation rail as a catalog constructor. `style::nav_rail` exists;
  `navigation_view` is still list/detail, not a compact rail.
- List leading and trailing slots (icon, checkbox) on `list_view`. Same
  model and virtualization; richer row face, not a second list.
- Slider ticks and labeled ends on the existing slider path.
- Tabs badge and overflow when the strip does not fit.
- Progress buffer and labeled remaining as constructor arguments.
- Data table row checkbox column (sort headers already ship).

## Consider

- Plugin surfaces and extension host chrome.

## Discard

Not library API. Applications own these, or they contradict icedtea
Non-goals:

- Language-service hooks, editors' language servers.
- Timeline, audio, or video engines; CAD kernels; live telemetry daemons.
- Document undo/redo.
- Multiple-document-interface window mosaics.
- A second collection widget for variable-height cards.
- System-wide hotkeys or host focus steal.
- A stylesheet, markup language, or second renderer.
- Threading views as a mail-specific widget (compose from list + detail).
- Offline indicators as a domain widget (status/toasts cover the chrome).
- FAB, extended FAB, bottom navigation, pull-to-refresh.
- Carousel and marketing hero sheets.
- Snackbar as a second path next to `toast`.
