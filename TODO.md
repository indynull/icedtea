# TODO

Work left on icedtea. The public surface is `catalog::ENTRIES` and the
book. One path per item. Gallery page and `just check` in the same
change. Measure before claiming smoothness.

Shipped: fill list/table with `virtual_pads` and range messages,
`on_submit`, list keys, overlay `size` / `place` / `should_hide`,
chip without dismiss, stick-to-end, `MarkdownDoc`, theme families and
follow-OS, large-list gallery measure, overlay hide/place demo.

## Order

1. List: scroll `Id`, usable rail, empty copy, row color
2. Scroll: `Id` and `on_scroll`
3. Overlay: Escape from a focused field
4. Place: center on the pointer's display
5. Overlay to decorated window
6. Text-input `Id`; compact chrome compose
7. `lazy` only if a heavy sibling still remounts after 1–2

## Do

- **List host.** Optional iced `Id` on `list_view` so the application
  can `scroll_to` the highlighted or matched row. Use the existing
  24px `ScrollRail` on that constructor so a long list stays
  grabbable. Optional empty-state string. Optional per-row meta
  color (or a small row-style hook) for success / warning / danger.
  Keep `ListModel` as title + meta + id + len; cover from
  `Selection::primary()`.
- **Scroll host.** Optional `Id` and optional `on_scroll`
  (`scrollable::Viewport` → message) on `themed_scroll` so the
  application can jump and can react when content leaves the
  viewport. Keep `stick`.
- **Overlay Escape.** `in_palette` (or a renamed inside-card flag)
  only suppresses `HideEvent::FocusLoss`. Escape hides the overlay
  even when a text field is focused. Document that iced 0.14
  `text_input` captures Escape and the application must forward that
  captured key into `should_hide`.
- **Centered place.** `place_centered`: display under the pointer
  (else first), center `size` in that rectangle. Keep `place` for
  pointer-origin menus.

## Consider

- **Window retarget.** Helpers to change an overlay into a decorated
  application window (resizable, `Level::Normal`, platform policy
  for Dock / task switcher). The application owns when to summon,
  hide, or pop out.
- **Compact chrome.** Optional `Id` on `themed_text_input` so a field
  can take focus after show. Document `Tabs { closable: false }`,
  and composing search + `list_detail` + tabs + footer. Optional
  `Corner::None` for flush cards. A dedicated shell constructor only
  after a second in-tree consumer needs the same recipe.
- **Markdown in tokens.** Optional `Tokens` faces on inline code and
  links if iced's markdown style allows it. Truncation is slicing
  source before `parse`.
- **`widget::lazy`.** Thin constructor over iced `lazy`. The
  application owns the key. Do not auto-lazy `list_detail`.
- **Variable-height rows.** Same collection path as the uniform
  list. Only if fixed row height plus `lazy` are not enough.
- **Secret field.** Mask, reveal, and a copy `Action` on a settings
  row. `password_input` stays the editor.
- **Navigation width.** Document the one resize subscription
  (`Subscription::map` non-capturing; convert in `update`).
  `navigation_view` keeps taking width.
- **Card plus chips.** Compose `group_box` and chips for filters and
  meta. A new exclusive-filter type only if `Tabs` or radio cannot
  do it.
- **OS accent.** When follow-OS is on, the desktop accent can fill
  `Tokens.primary`. Canvas and text stay the family's tokens.
  Decorated windows keep the native title bar.

Caller-built `Tokens` style every constructor. Named catalog keys
are optional. Host config files map into `Tokens` in the application.
