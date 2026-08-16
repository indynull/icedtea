# Controls

Buttons, toggles, and a slider. Faces paint from M3 roles and
`ControlState` (enabled, disabled, hovered, pressed, selected) via
`style` and `Tokens::scheme()`.
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/index.html) ·
[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

![Buttons, checks, radios, and sliders](../images/controls.png)

Each constructor takes `A11y` unless noted. iced 0.14 publishes the widget id only.

### Button

**`button`** — Press a labeled control to send a message.

Constructor: [`widget::themed_button`](https://docs.rs/icedtea/latest/icedtea/widget/fn.themed_button.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`title` is the face. `msg` is `None` when there is nothing to send.
`A11y::button(name).with_disabled(true)` drops the press handler.
`Variant` picks the token wash (`Primary` filled, `Quiet` tonal,
`Outlined`, `Elevated`, `Ghost` text). `Icons` is leading and trailing
chrome.

### Split button

**`split-button`** — A primary press plus a more menu.

Constructor: [`widget::split_button`](https://docs.rs/icedtea/latest/icedtea/widget/fn.split_button.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`primary` is the main message. `more` opens the overflow. Disabled
drops both.

Pass `A11y`.

### Toggle button

**`toggle-button`** — A button that stays pressed while on.

Constructor: [`widget::toggle_button`](https://docs.rs/icedtea/latest/icedtea/widget/fn.toggle_button.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Pass the current on/off state. The message fires on press. Disabled
keeps the face and drops the handler.

Pass `A11y`.

### Checkbox

**`checkbox`** — Check or clear a boolean.

Constructor: [`widget::themed_checkbox`](https://docs.rs/icedtea/latest/icedtea/widget/fn.themed_checkbox.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

The application owns the bool. The message carries the next value.
Disabled keeps the box visible and ignores clicks.

Pass `A11y`.

### Radio

**`radio`** — Pick one value from a small set.

Constructor: [`widget::themed_radio`](https://docs.rs/icedtea/latest/icedtea/widget/fn.themed_radio.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Compare the selected value to this option. Disabled rows stay in the
group and do not change the selection.

Pass `A11y`.

### Switch

**`switch`** — A sliding on/off control.

Constructor: [`widget::themed_switch`](https://docs.rs/icedtea/latest/icedtea/widget/fn.themed_switch.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Same contract as checkbox: the application owns the bool. Disabled
freezes the thumb.

Pass `A11y`.

### Slider

**`slider`** — Pick a number on a range.

Constructor: [`widget::themed_slider`](https://docs.rs/icedtea/latest/icedtea/widget/fn.themed_slider.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Pass min, max, and the current value. `SliderMarks` paints ticks and
end labels, a `vertical` rail, and an optional thumb value label. The
message is the new value while the thumb moves. Disabled ignores drag.

Pass `A11y`.

### Segmented button

**`segmented-button`** — Exclusive choice among labeled segments.

Constructor: [`widget::segmented_button`](https://docs.rs/icedtea/latest/icedtea/widget/fn.segmented_button.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

The application owns the selected index. Cells may carry an icon. Press emits the new index. Disabled freezes every segment.

Pass `A11y`.

### Button group

**`button-group`** — Related actions in one strip.

Constructor: [`widget::button_group`](https://docs.rs/icedtea/latest/icedtea/widget/fn.button_group.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Each cell is a label plus optional icon (`Cell`). Press sends the index.
Empty cells paint an empty row. Disabled drops every press.

Pass `A11y`.

### Icon button

**`icon-button`** — Icon-only press control for dense toolbars.

Constructor: [`widget::icon_button`](https://docs.rs/icedtea/latest/icedtea/widget/fn.icon_button.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`Variant` picks the wash. `ControlSize` is the hit box. Pass a shipped
`Icon` or `Glyph::Bytes`. Disabled drops the press.

Pass `A11y`.

### Toggle icon button

**`toggle-icon-button`** — Icon button that stays pressed while on.

Constructor: [`widget::icon_button_toggle`](https://docs.rs/icedtea/latest/icedtea/widget/fn.icon_button_toggle.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Same wash as toggle button. Disabled keeps the face.

Pass `A11y`.

### Range slider

**`range-slider`** — Inclusive low and high values on one range.

Constructor: [`widget::range_slider`](https://docs.rs/icedtea/latest/icedtea/widget/fn.range_slider.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Messages are the clamped pair with `low <= high`. Disabled freezes both thumbs.

Pass `A11y`.

### Indeterminate checkbox

**`checkbox-indeterminate`** — Three-state checkbox including partial selection.

Constructor: [`widget::checkbox_indeterminate`](https://docs.rs/icedtea/latest/icedtea/widget/fn.checkbox_indeterminate.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Uses [`CheckState`](https://docs.rs/icedtea/latest/icedtea/widget/enum.CheckState.html). Press follows `CheckState::toggle`.

Pass `A11y`.

