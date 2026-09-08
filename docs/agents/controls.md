# Controls

Generated from `catalog::ENTRIES`, constructor rustdoc, and
`m3::mapping`. Compose starts at `examples/hello.rs` (see
[llms.txt](llms.txt)).

## button

Press a labeled control to send a message.

`title` is the face. `msg` is `None` when there is nothing to send. `A11y::button` plus `with_disabled(true)` drops the handler. `variant` picks the token wash.

Constructor: `widget::button`
Material: Button
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.button.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## toggle-icon-button

Icon button that stays pressed while on.

Same wash as `toggle_button`. Disabled keeps the face.

Constructor: `widget::icon_button_toggle`
Material: Icon button (toggle)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.icon_button_toggle.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## slider

Pick a number on a range.

Pass min, max, and the current value. The message is the new value while the thumb moves. Wheel over the control steps by `slider_step`. Focused arrows, Home, and End step the same way. Disabled ignores drag, wheel, and keys. `marks` paints ticks and end labels when set; min sits on start, max on end. The rail fills from start. Rail corners follow `Tokens::shape` (`crate::m3::shape::Component::Track`).

Constructor: `widget::slider`
Material: Slider
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.slider.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## button-group

Related actions in one strip (M3 button group). Not exclusive.

Each label sends its index. Empty labels paint an empty row. Disabled drops every press. The outline hugs the cells. The strip has no selected cell: focused Enter, Space, and Home send index 0; arrows send 1 when a second cell exists; End sends the last index.

Constructor: `widget::button_group`
Material: Button groups
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.button_group.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## segmented-button

Exclusive choice among labeled segments (M3 segmented button).

The application owns the selected index. Press emits the new index. Disabled freezes all segments. Compact is the in-pane strip (`tab_bar` stays the pane chrome).

Constructor: `widget::segmented_button`
Material: Segmented button
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.segmented_button.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## checkbox

Check, clear, or mark a partial selection.

The application owns `CheckState`. Press cycles through `CheckState::toggle`. Disabled keeps the box and ignores clicks. An empty label is the box only (shrink width) so it can sit in a row next to a title.

Constructor: `widget::checkbox`
Material: Checkbox
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.checkbox.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## radio

Pick one value from a small set.

Compare the selected value to this option. Disabled rows stay in the group and do not change the selection.

Constructor: `widget::radio`
Material: Radio button
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.radio.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## switch

A sliding on/off control.

Same contract as checkbox: the application owns the bool. Disabled freezes the thumb. Track corners follow `Tokens::shape` (`crate::m3::shape::Component::Track`). `SwitchOpts::FORM` is a settings row (track then caption, Fill). `SwitchOpts::BAR` is a toolbar control (caption then track, shrink) at `ControlSize` height so it sits next to a compact pick.

Constructor: `widget::switch`
Material: Switch
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.switch.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## range-slider

Inclusive low/high pair on one range (M3 range slider as two linked thumbs).

The application owns `low` and `high`. Messages are the clamped pair with `low <= high`. Disabled freezes both thumbs.

Constructor: `widget::range_slider`
Material: Slider (range)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.range_slider.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## icon-button

Icon-only press control (toolbar density).

Same variant wash as labeled buttons. Disabled drops the press.

Constructor: `widget::icon_button`
Material: Icon button
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.icon_button.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## split-button

A primary press plus a more menu.

`primary` is the main message. `more` opens the overflow. Disabled drops both.

Constructor: `widget::split_button`
Material: Button (split)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.split_button.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## toggle-button

A button that stays pressed while on.

Pass the current on/off state. The message fires on press. Disabled keeps the face and drops the handler.

Constructor: `widget::toggle_button`
Material: Icon button / toggle
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.toggle_button.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)
