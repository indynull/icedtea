# TODO

## Do

- `pattern::workspace` paints application content only in the first
  leaf (depth-first). Other leaves show their title. The public
  constructor takes content per leaf.

## Consider

- `list_view` from `visible_range_var` (variable row heights). Fixed
  `row_h` is the shipped contract. The application supplies heights;
  do not add a second collection widget.
- Frozen leading columns on `data_table` (`ColumnLayout` order stays
  the scroll order; pin the first *n* so they stay in view).
- A parameter field on the command palette (go to line, rename) that
  `command_palette_view` paints.
- Plugin surfaces and extension host chrome.
- Multi-monitor overlay pin beyond `window::place` / `Boot::displays`.
- Touch density as a fourth named preset beyond compact / default /
  comfortable.

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
