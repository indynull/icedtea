# Collections

Generated from `catalog::ENTRIES`, constructor rustdoc, and
`m3::mapping`. Compose starts at `examples/hello.rs` (see
[llms.txt](llms.txt)).

## virtual-column

A virtualized column of app-built rows with known heights.

Use for expand cards and other free-form faces: pass `collection::expand_card_heights` (or any per-row slice), keep a `VisibleWindow`, and build each mounted index. The clip keeps the pixel offset; `on_scroll` fires when the mounted range changes or the scroll arrives at 0 or max. `on_click` is the same `ItemClick` as `list_view`: primary on release (select only), secondary on press. Activate stays Enter or an application message. Pass `window.start..window.end` so `view` can build the last published range. Jump with `scroll_to` on `scroll_id`. A new `cover` index that sits outside the viewport moves the clip so that row is in view. Change `scroll_id` when the row set is a different list so the clip remounts at 0. This reuses list windowing (overscan, rail, wheel); it is not a second list model — title/meta lists stay on `list_view`.

Constructor: `widget::virtual_column`
Desktop: Lists (variable height)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.virtual_column.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## list

A virtualized row list.

`empty` is the copy when `model` has no rows. `meta_color` paints the second line. The clip keeps the live pixel offset; the rail and the wheel write it. Uniform rows sit at `i * row_h - scroll`. Variable rows use `RowHeights::PerRow` and `crate::collection::visible_window_var`. `face` is `RowFace::Flush` (clipped line) or `RowFace::Card` (wrapped title, 2px gap, optional meter). The clip keeps the pixel offset; `on_scroll` fires when the mounted range changes or the scroll arrives at 0 or max. A wheel while parked at an end does not publish. Pass `window.start..window.end` so `view` can build the last published range. Jump with `scroll_to` on `scroll_id`. A new selection that sits outside the viewport moves the clip so that row is in view. A later wheel with the same selection leaves the offset alone. Change `scroll_id` when the row set is a different list (filter, page, session) so the clip remounts at 0. `scroll_id` names the clip pane. The 24px rail sits beside it. `ListModel::indent` insets the row from start. `RowSlot::Text` paints a small badge (`RowSlot::text` is Quiet).

Constructor: `widget::list_view`
Material: Lists
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.list_view.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## pagination

Page through a long list.

Pass page count and the current page. Messages are previous, next, and jump.

Constructor: `widget::pagination`
Desktop: Lists (paging)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.pagination.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## log

Append-only lines. Sticks to the end. `window` virtualizes long logs. Append-only lines that stick to the end.

Virtualizes long logs. Empty lines show “No lines”.

Constructor: `widget::log_view`
Desktop: Lists (log)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.log_view.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## grid

Tiles that share the row width.

Each tile is control height. Click sends the index. Empty grid is an empty column.

Constructor: `widget::item_grid`
Material: Lists (grid)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.item_grid.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## table

A virtualized table. Last column fills.

`on_cell` is an `ItemClick` (row) plus the column. `on_sort` is the header click. Empty rows still paint headers. `columns.frozen` stays in view; `on_h_scroll` is the unfrozen strip. The clip owns the vertical offset. Pass `window.start..window.end` so `view` can build the last published range. `scroll_id` names the clip for `scroll_to`. A new selection that sits outside the viewport moves the clip so that row is in view. Change `scroll_id` when the row set is a different table.

Constructor: `widget::data_table`
Desktop: Data table (desktop)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.data_table.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## tree

Heading or file tree (`TreeNode`, `tree_toggle`, `TreeFace`, `crate::collection::ItemClick`). The disclosure control emits `on_toggle`; the row label emits `on_select`. `selected` is the app-owned id.

`TreeFace::Outline` is a tight heading tree (no marks). `TreeFace::Files` is an explorer (folder and file marks from `dir`). Both paint the selected wash across the full row (indent through trailing slot). The twisty stays its own press. Density scales pad, gap, and indent. The application owns expand state. Leaf rows have no twisty. `animating` is the branch that is opening or closing and its 0–1 height progress. `None` paints the committed tree. `TreeNode::trailing` is the same `crate::collection::RowSlot` as `list_view`; `crate::collection::RowSlot::Text` is a badge (`RowSlot::text` is Quiet). Outline wraps the title inside its slot; Files stays one line. The title clips so a trailing badge stays on the end, clear of the name.

Constructor: `widget::tree_view`
Desktop: Lists (tree)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.tree_view.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## accordion

An open row shows a body under the header.

The application owns which row is open. Closed rows are headers only. The chevron sits on the trailing edge.

Constructor: `widget::accordion_view`
Material: Lists (expand)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.accordion_view.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## expander

A card that clips its child until opened.

The application owns `open` and `progress` (0 peek, 1 full). The header toggles. Closed shows a `Peek` of the child. Title and body share the card inset. The chevron sits on the trailing edge. `trail` sits after the title (count badge, meta) and before the mark.

Constructor: `widget::expander`
Material: Lists (expand)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.expander.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## tabs

A tab bar over a body the application paints.

`Tabs { closable: false }` is pinned sections. Titles use meta type. `with_disabled` freezes one tab. Select sends the index.

Constructor: `widget::tab_bar`
Material: Tabs
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.tab_bar.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)
