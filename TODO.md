# TODO

## Do

## Consider

- Plugin surfaces and extension host chrome.
- `segmented_button` takes `ControlSize` for in-pane exclusive choice
  (Prompt / Assistant). `tab_bar` stays the pane strip (overflow, close,
  underbar).
- `markdown_view` page vs inset type map: inset uses title for H1 and
  meta for body. `font_scale` stays the user scale.

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
