# TODO

## Do

- `list_view` from `visible_range_var` (variable row heights).
- Paint `CommandPalette::prompt` in `command_palette_view`.

## Consider

- Plugin surfaces and extension host chrome.
- Multi-monitor overlay pin beyond `window::place` / `Boot::displays`.
- Estimated-height virtualization from a measure callback (today the
  application supplies heights).
- Touch density as a fourth named preset beyond compact / default /
  comfortable.

## discard

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
