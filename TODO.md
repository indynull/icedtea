# TODO

## Do

- `list_view`: per-row indent on `ListModel` (pixels or depth). `RowSlot`
  grows a text / badge member on lead and trail. Same constructor;
  `ListModel` stays title, meta, id, length, and check.
- `expander`: optional trailing `Element` on the title row (count
  badge, meta). String title stays the a11y name. One constructor.
- Public `Tokens` constructor from the short color aliases (canvas,
  surface, panel, text, muted, primary, accent, success, warning,
  danger, border) that builds the scheme. Seat color stays in the
  application.

## Bugs

- `themed_scroll` `update` and `mouse_interaction`: pointer events and
  the pointer cursor stay inside the pane bounds. Keyboard still
  reaches the child. Paint already scissors.
- `themed_checkbox` with an empty label is shrink-width (the box
  only). `labeled_control` Fill is for a named row.

## Consider

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
- FAB, extended FAB, FAB menu, bottom navigation, pull-to-refresh.
- Carousel and marketing hero sheets.
- Snackbar as a second path next to `toast`.
- Bottom sheets (mobile).
- Loading indicator as M3 Expressive shape-morph.
- Floating toolbar as a pill / expressive float.
- Compact `tab_bar` as a second size. In-pane exclusive tabs are
  `segmented_button`.
