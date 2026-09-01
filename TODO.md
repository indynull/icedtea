# TODO

## Do

### Blocked on iced

- RTL `textarea`: iced 0.14 `text_editor` has no writing direction
  (caret, hit-test, and line origin stay LTR). Do **not** shrink-wrap
  or pin a measured width — that growing right-hand slab puts the
  caret ahead of Urdu/Arabic and splits a new line to the opposite
  edge. Keep a stable Fill field. Replace the editor when iced 0.15
  lands [iced#3294](https://github.com/iced-rs/iced/pull/3294)
  (fixes [iced#2102](https://github.com/iced-rs/iced/issues/2102),
  [iced#1877](https://github.com/iced-rs/iced/issues/1877),
  [iced#1454](https://github.com/iced-rs/iced/issues/1454)).
- Field highlighter bidi: `HighlightField` packs one left-to-right
  paragraph (caret, scroll, span bounds). Arabic and Urdu queries
  do not follow writing direction. Stay on this path until iced
  styles the input paragraph. Do not treat English SQL highlighting
  or the empty-search clear mark as this item.

## Bugs

## Consider

- Typeahead jump in long lists and trees (beyond first-letter menus).
- `spatial_next` when Tab leaves a pane (today it is a helper).
- Horizontal keyboard scroll (`ctrl+Page`) on tables with an
  unfrozen strip.
- Hover-open menubar titles (desktop convention). Click-open stays
  the default.

## Discard

Not library API. Applications own these, or they contradict icedtea
Non-goals:

- A per-widget BINDINGS list that bubbles beside `ActionTable`.
  Grow the one table and the constructors that consume it.
- Library-owned layered dismiss (help → detail → compose → parent
  → hide). `should_hide` is the overlay-window policy only.
- Forcing first-widget focus when `Boot` names an `Id` or none
  (summon search, empty chrome).
- Language-service hooks, editors' language servers.
- Timeline, audio, or video engines; CAD kernels; live telemetry daemons.
  Applications own decode and paint (`iced_video_player`, shader blit).
  `image_slot` is stills.
- In-process web view (WebKitGTK, Servo, CEF, WebView2). Applications
  host the engine as a custom iced element.
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
- Layer-shell via a Smithay client loop in the widget crate. Window
  roles stay iced `window::Settings`. An exclusive-zone panel binds
  `zwlr_layer_shell_v1` in the application until iced can open that
  surface. A later `host*` opener is only if iced still cannot and a
  second app needs the role.
- A foreign Wayland client surface hosted inside a pane. The compositor
  maps that surface; icedtea paints the chrome around it.
- Domain columns, permission cards, mail list/read/compose, or
  host-file chips as catalog widgets. Applications compose those from
  `chip`, field, table, dialog, list, and textarea.
- Loading indicator as M3 Expressive shape-morph.
- Floating toolbar as a pill / expressive float.
- Compact `tab_bar` as a second size. In-pane exclusive tabs are
  `segmented_button`.
