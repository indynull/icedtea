# TODO

Work left on icedtea. The public surface is `catalog::ENTRIES` and the
book. One path per item. Gallery page and `just check` in the same
change. Measure before claiming smoothness.

## Order

1. `lazy` constructor if a heavy sibling still remounts on list scroll

## Consider

After the list above, or as a thin iced pass-through.

- **`widget::lazy`.** Thin constructor over iced `lazy`. The
  application owns the key. Do not auto-lazy `list_detail`.
- **Estimated-height rows on the same list.** One collection path.
  Only if uniform rows plus `lazy` are not enough.
- **Secret row.** Mask, reveal, and a copy `Action` for settings.
  Password stays the field.
- **Navigation width.** Document the one resize subscription
  (`Subscription::map` non-capturing; convert in `update`).
  `navigation_view` keeps taking width.
- **Overlay pop-out.** Helpers to retarget window settings
  (decorated application). The summon / hide / pop-out loop stays in
  the application.
- **Card meta row.** Compose `group_box` plus chips. New filter type
  only if `Tabs` or radio cannot do exclusive filters.
- **OS accent as `primary`.** When follow-OS is on, mundy's accent
  color can fill `Tokens.primary`. Canvas and text stay the family's
  tokens. Decorated windows keep the native title bar.
