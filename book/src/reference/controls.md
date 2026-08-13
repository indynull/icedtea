# Controls

Buttons, toggles, and a slider. Faces paint from M3 roles and
`ControlState` (enabled, disabled, hovered, pressed, selected) via
`style` and `Tokens::scheme()`.
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/index.html) ·
[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

### Button

**`button`** — Press a labeled control to send a message.

Constructor: [`widget::themed_button`](https://docs.rs/icedtea/latest/icedtea/widget/fn.themed_button.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`title` is the face. `msg` is `None` when there is nothing to send.
`A11y::button(name).with_disabled(true)` drops the press handler.
`Variant` picks the token wash (`Primary`, `Quiet`, `Ghost`).

### Split button

**`split-button`** — A primary press plus a more menu.

Constructor: [`widget::split_button`](https://docs.rs/icedtea/latest/icedtea/widget/fn.split_button.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`primary` is the main message. `more` opens the overflow. Disabled
drops both.

### Toggle button

**`toggle-button`** — A button that stays pressed while on.

Constructor: [`widget::toggle_button`](https://docs.rs/icedtea/latest/icedtea/widget/fn.toggle_button.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Pass the current on/off state. The message fires on press. Disabled
keeps the face and drops the handler.

### Checkbox

**`checkbox`** — Check or clear a boolean.

Constructor: [`widget::themed_checkbox`](https://docs.rs/icedtea/latest/icedtea/widget/fn.themed_checkbox.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

The application owns the bool. The message carries the next value.
Disabled keeps the box visible and ignores clicks.

### Radio

**`radio`** — Pick one value from a small set.

Constructor: [`widget::themed_radio`](https://docs.rs/icedtea/latest/icedtea/widget/fn.themed_radio.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Compare the selected value to this option. Disabled rows stay in the
group and do not change the selection.

### Switch

**`switch`** — A sliding on/off control.

Constructor: [`widget::themed_switch`](https://docs.rs/icedtea/latest/icedtea/widget/fn.themed_switch.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Same contract as checkbox: the application owns the bool. Disabled
freezes the thumb.

### Slider

**`slider`** — Pick a number on a range.

Constructor: [`widget::themed_slider`](https://docs.rs/icedtea/latest/icedtea/widget/fn.themed_slider.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Pass min, max, and the current value. The message is the new value
while the thumb moves. Disabled ignores drag.
