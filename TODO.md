# TODO

## Do

## Bugs

Material Design 3 shape map leftovers. `badge` now uses
`Component::Badge`; these still pick the wrong family or omit a scale
step.

- Badge sizes: M3 small is a 6 dp circle with no text; large is 16 dp
  tall, max 16×34. Ours are caption pads (`BadgeSize` Small / Large).
- `toast` and tooltip use Card corners. M3 snackbar and tooltip are
  extra-small (4 dp).
- `banner` uses Card corners. M3 banner is flush (0 dp).
- Search uses Field. M3 search is extra-large or full.
- Switch track, slider rail, and progress use Button. M3 those tracks
  are Full.
- `m3::Shape` has no Large Increased (20 dp), Extra Large Increased
  (32 dp), or Extra Extra Large (48 dp).

## Consider

- Plugin surfaces and extension host chrome.
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
