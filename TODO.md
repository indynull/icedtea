# TODO

## Do

- `split_view`: paint the existing 6 px sash (hairline plus one short
  centered handle, both axes). Constructor takes `Tokens` so the line
  uses the scheme outline. Grip still emits `SashEvent::Press` only;
  Move and Release stay on `listen_sash`. Keep the 6 px thickness and
  the resize cursor (empty `Space` over the paint: iced 0.14
  `mouse_area.interaction` only wins when the child reports
  `Interaction::None`). Same catalog id. Recapture gallery / book
  stills that show the sash.

- `tree_view`: optional trailing `RowSlot` on `TreeNode` (same slot as
  `list_view`; `Text` is a badge). `TreeFace` stays Outline / Files.
  The application supplies the title strings.

## Bugs

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
