# Collections

Lists, tables, trees, tabs, expanders, and pages.
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/index.html) ·
[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`VisibleWindow.scroll` is the only list and table offset. The rail and
the wheel write it. Variable-height rows use `row_offsets` /
`visible_range_var`. `scroll_id` names the list clip pane.

### List

**`list`** — A virtualized row list.

Constructor: [`widget::list_view`](https://docs.rs/icedtea/latest/icedtea/widget/fn.list_view.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`empty` is the copy when the model has no rows. Selection stays on
indices. Disabled drops row messages. `on_scroll` reports the window
after a wheel or rail move.

### Log

**`log`** — Append-only lines that stick to the end.

Constructor: [`widget::log_view`](https://docs.rs/icedtea/latest/icedtea/widget/fn.log_view.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Virtualizes long logs. Iced's end anchor reports offset 0, so the log
reads the reversed offset and mounts the tail. Empty lines show
“No lines”.

### Item grid

**`grid`** — Tiles that share the row width.

Constructor: [`widget::item_grid`](https://docs.rs/icedtea/latest/icedtea/widget/fn.item_grid.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Pass titles. Click sends the index. Empty grid is an empty column.

### Data table

**`table`** — A virtualized table. Last column fills.

Constructor: [`widget::data_table`](https://docs.rs/icedtea/latest/icedtea/widget/fn.data_table.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`TableModel` holds headers and rows. `on_cell` is (row, column).
`on_sort` is the header click. `ColumnLayout` orders, freezes, and
resizes columns. Empty rows still paint headers.

### Tree

**`tree`** — An expandable outline.

Constructor: [`widget::tree_view`](https://docs.rs/icedtea/latest/icedtea/widget/fn.tree_view.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

The application owns expand state. Leaf rows have no twisty. Empty
tree is an empty column.

### Document tabs

**`document-tabs`** — Closable document titles; dirty titles get a bullet.

Constructor: [`pattern::document_tabs`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.document_tabs.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Close confirm is the application's. `DocumentTabs` holds titles and
dirty flags.

### Tabs

**`tabs`** — A tab bar over a body the application paints.

Constructor: [`widget::tab_bar`](https://docs.rs/icedtea/latest/icedtea/widget/fn.tab_bar.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`Tabs { closable: false }` is pinned sections. Select sends the
index. See also [`pattern::tab_view`](patterns.md#tab-view).

### Accordion

**`accordion`** — An open row shows a body under the header.

Constructor: [`widget::accordion_view`](https://docs.rs/icedtea/latest/icedtea/widget/fn.accordion_view.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

The application owns which row is open. Closed rows are headers
only. The chevron sits on the trailing edge.

### Expander

**`expander`** — A card that clips its child until the header opens it.

Constructor: [`widget::expander`](https://docs.rs/icedtea/latest/icedtea/widget/fn.expander.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

The application owns `open`. Closed shows a `Peek` of the child
(pixels, or whole body lines with room for the last descent) and
fades the cut. Open paints the full child. Title starts; the
chevron sits on the trailing edge. Accordion is many headers;
this is one card.

### Pagination

**`pagination`** — Page through a long list.

Constructor: [`widget::pagination`](https://docs.rs/icedtea/latest/icedtea/widget/fn.pagination.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Pass page count and the current page. Messages are previous, next,
and jump. One page hides the control or disables the arrows.
