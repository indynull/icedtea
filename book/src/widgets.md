# Widgets

Every public constructor returns an iced `Element` and emits the
application's messages. It takes `Tokens` and `A11y` (name, role,
disabled, checked). The application owns state. `A11y` name is the
node identity; live values go in `with_value`, not the name. Disabled
drops handlers. Composite fields use `A11y::child` so the inner editor
keeps that name and disabled flag.

The gallery (`cargo run -p icedtea-gallery`) pages every
`catalog::ENTRIES` id. Related controls share a page. Rustdoc on
`icedtea::widget` is the reference.

## Time

`TimeValue` stores hour, minute, and second on a 24-hour clock.
`TimeClock` is display only: 12-hour or 24-hour, with optional seconds.
`time_picker` steps one `TimeField` (hour, minute, second, period).

```rust,ignore
use icedtea::widget::{self, TimeClock, TimeField, TimeValue};

let t = TimeValue::hm(9, 30);
let clock = TimeClock::HOUR12;
let _ = widget::time_picker(
    t,
    clock,
    |field| Message::Time(t.step_field(field, clock)),
    tokens,
    icedtea::a11y::A11y::new("alarm", icedtea::a11y::Role::SpinButton),
);
```

## Lists, tables, and logs

`VisibleWindow.scroll` is the only list and table offset. The rail and
the wheel write it. `list_view` and `data_table` paint row `i` at
`i * row_h - scroll` (list separator rows occupy `row_h`). The pane
height comes from layout; `on_scroll` reports that height after a wheel
or rail move. `scroll_id` names the list clip pane. `log_view` sticks
to the end: iced's end anchor reports offset 0, so the log reads the
reversed offset and mounts the tail.

## Fields

`secret_field` is a settings row: masked `password_input`, reveal, and
a copy `Action`. `masked_input` fills digit slots on a template
(`0000-0000`). `suggest_field` attaches a pick list to any text input.
`themed_text_input` takes an optional id for `focus` after show.
