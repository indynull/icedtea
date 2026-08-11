# Widgets

Every public constructor returns an iced `Element` and emits the
application's messages. It takes `Tokens` and `A11y` (name, role,
disabled, checked). The application owns state. `A11y` name is the
node identity; live values go in `with_value`, not the name. Disabled
drops handlers. Composite fields use `A11y::child` so the inner editor
keeps that name and disabled flag.

The gallery (`cargo run -p icedtea-gallery`) pages every catalog id.
Related controls share a page. This reference uses the same groups.

| Group | Page |
| --- | --- |
| Controls | [Controls](reference/controls.md) |
| Fields | [Fields](reference/fields.md) |
| Readout | [Readout](reference/readout.md) |
| Content | [Content](reference/content.md) |
| Collections | [Collections](reference/collections.md) |
| Chrome | [Chrome](reference/chrome.md) |
| Patterns | [Patterns](reference/patterns.md) |

Each entry names the job, the shipped constructor, and links to
[rustdoc](https://docs.rs/icedtea),
[source](https://github.com/indynull/icedtea), and
[crates.io](https://crates.io/crates/icedtea).

## Time

`TimeValue` stores hour, minute, and second on a 24-hour clock.
`TimeClock` is display only: 12-hour or 24-hour, with optional seconds.
`time_picker` steps one `TimeField` (hour, minute, second, period).
See [`time`](reference/fields.md#time).

## Lists, tables, and logs

`VisibleWindow.scroll` is the only list and table offset. The rail and
the wheel write it. Variable-height rows use `row_offsets` /
`visible_range_var`. `Selection::select_range` and `move_primary` stay
on indices. `ColumnLayout` orders, freezes, and resizes table columns.
See [Collections](reference/collections.md).
