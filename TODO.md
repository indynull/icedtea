# TODO

Work left on icedtea. The public surface is `catalog::ENTRIES` and the
book. One path per item. Gallery page and `just check` in the same
change. Measure before claiming smoothness.

## Order

1. List and table: fill height, two-line rows, range-change scroll,
   access wrap, empty caption
2. Text submit, list and palette keys, chips, image, stick-to-end
3. Overlay size, pointer display, hide policy
4. Markdown document helper; theme catalog in the book and gallery
5. Measure the list and table gallery pages
6. `lazy` constructor if a heavy sibling still remounts on list scroll

## Do

- **Virtual list and table.** `list_view` and `data_table` fill the
  pane (`layout::FILL`). Rows are a primary line and optional meta.
  `ListModel` / table cells borrow; no clone of the whole catalog in
  `view`. `virtual_pads` takes overscan and an optional cover index so
  the selected row stays mounted. A 1_000-row gallery page mounts tens
  of widgets.
- **Scroll messages on range change.** Emit when the visible index
  range or viewport size changes. Pixel motion stays in iced’s
  scrollable. The application stores the range, not every pixel.
- **Access wrap and caption.** `a11y::attach` must not shrink a fill
  child. Empty `themed_text_input` / password / number must not paint
  the access name as the value.
- **Submit on `themed_text_input`.** Optional `on_submit` for Enter.
- **List and palette keys.** `key::press` (or the list/palette path)
  reports arrows, page, home, and end so highlight can move.
- **Overlay window.** `Boot::overlay().size(w, h)` is the inner size
  (no 720×480 cap). Place on the display under the pointer. Hide
  policy (Escape, focus loss) ignores in-palette controls. `should_hide`
  tests cover those cases. Book `overlay-windows.md` matches.
- **Chip and badge.** Chip without a dismiss control. Both take the
  named variants buttons already have.
- **Stick-to-end.** `themed_scroll` uses `layout::stick_to_end` so a
  log or transcript can pin to the latest line.
- **Markdown document.** `MarkdownDoc` owns source hash and parsed
  items. Pure `parse` helper. Application parses in `update` (or a
  `Task`) and `markdown_view` borrows. rustdoc example. Grow the
  existing markdown gallery page.
- **Theme catalog.** Book theming page and gallery show
  `ThemeCatalog::register` for a custom name and a live switch. No new
  API.
- **Measure.** On the list and table gallery pages: large row count, a
  status line that ticks, mounted widget count recorded on the page or
  here. Command: `cargo run -p icedtea-gallery`.

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
