# TODO

Work left on icedtea. The public surface is `catalog::ENTRIES` and the
book. One path per item. Gallery page and `just check` in the same
change. Measure before claiming smoothness.

Shipped: fill list/table with `virtual_pads` and range messages,
`on_submit`, list keys, overlay `size` / `place` / `should_hide`,
chip without dismiss, stick-to-end, `MarkdownDoc`, theme families and
follow-OS, gallery 1_000-row measure, palette overlay demo.

## Order

1. List: scroll `Id`, 24px rail, empty copy, row meta color
2. `themed_scroll`: `Id` and `on_scroll`
3. Overlay hide: Escape while a field is focused
4. `place_centered` on the pointer's display
5. Overlay pop-out helpers
6. Text-input `Id`; overlay shell compose
7. `lazy` only if a heavy sibling still remounts after 1–2

## Do

- **List scroll host.** Optional iced `Id` on `list_view` so the
  application can `scroll_to` a selected or matched row (keyboard
  highlight, search). Use the existing 24px `ScrollRail` on the list
  (one constructor) so a tall catalog stays grabbable. Optional
  empty-state string. Optional per-row meta color (or a small
  row-style hook) for success / warning / danger, not only muted.
  Keep `ListModel` as title + meta + id + len; cover from
  `Selection::primary()`.
- **`themed_scroll` id and viewport.** Optional `Id` and optional
  `on_scroll` (`scrollable::Viewport` → message) so a log, transcript,
  or timeline can jump and can tell when a card leaves the pane.
  Keep `stick`.
- **Escape hides the overlay.** `in_palette` only suppresses
  `HideEvent::FocusLoss` (clicking from search to the result list).
  Escape still hides, including while a text field is focused.
  Document that iced 0.14 `text_input` captures Escape and the
  application must forward that captured key into `should_hide`.
- **`place_centered`.** Pick the display under the pointer (else
  first) and center `size` in that rectangle (launcher, palette,
  dialog). Keep `place` for menus that sit on the pointer.

## Consider

- **Overlay pop-out.** Helpers to retarget window settings from an
  undecorated always-on-top card to a decorated application
  (resizable, `Level::Normal`, no override-redirect). macOS accessory
  vs regular policy. The summon / hide / pop-out loop stays in the
  application.
- **Overlay shell compose.** `Tabs { closable: false }` already
  hides close buttons; document that. Optional `Id` on
  `themed_text_input` so search can take focus after show. Document
  composing search + `list_detail` + tabs + footer. Optional
  `Corner::None` for flush overlay cards. Extract a dedicated shell
  constructor only when a second in-tree consumer needs the same
  recipe.
- **Markdown previews.** Application can slice source before `parse`.
  Optional `Tokens.accent` / `primary` on inline code and links if
  iced's markdown style allows it.
- **`widget::lazy`.** Thin constructor over iced `lazy`. The
  application owns the key. Do not auto-lazy `list_detail`.
- **Estimated-height rows on the same list.** One collection path.
  Only if uniform rows plus `lazy` are not enough.
- **Secret row.** Mask, reveal, and a copy `Action` for settings.
  Password stays the field.
- **Navigation width.** Document the one resize subscription
  (`Subscription::map` non-capturing; convert in `update`).
  `navigation_view` keeps taking width.
- **Card meta row.** Compose `group_box` plus chips. New filter type
  only if `Tabs` or radio cannot do exclusive filters.
- **OS accent as `primary`.** When follow-OS is on, mundy's accent
  color can fill `Tokens.primary`. Canvas and text stay the family's
  tokens. Decorated windows keep the native title bar.

Caller-built `Tokens` already style list, tabs, inputs, and scroll.
Named catalog keys are optional. Mapping a host config file onto
`Tokens` stays in the application.
