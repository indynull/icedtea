# TODO

## Do

Shipped or shipping in this cut as library constructors and models:

- Nested/tabbed docking with save/restore, perspectives, min/max and
  ratio constraints, panel move between docks, breakpoint drawers.
  Float-or-dock is an overlay tool panel, not an in-frame document mosaic.
- Variable-height virtualization by extending list. Multi-column tables
  with sticky headers, resize/reorder, frozen columns. Range selection
  and keyboard movement that stay virtualized. Cross-list drag of
  indices. Lazy mount of heavy rows.
- Namespaced commands with conflict detection, sequential key
  sequences, context-sensitive maps. Palette recent/favorites and
  parameter prompts. Searchable keyboard cheatsheet.
- Roving tabindex, spatial arrow navigation, modal focus traps,
  landmarks and live regions, hierarchical breadcrumb.
- Master-detail plus inspector kept in sync. Document tabs with dirty
  state and close confirm (application owns the document). Status jobs
  and toasts. Runtime density and user theme/accent/scale persistence.
- Gallery demos handle the messages their widgets emit (outline jump,
  dismiss, places).

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
