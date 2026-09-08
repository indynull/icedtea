# Readout

Generated from `catalog::ENTRIES`, constructor rustdoc, and
`m3::mapping`. Compose starts at `examples/hello.rs` (see
[llms.txt](llms.txt)).

## progress

A determinate bar from 0 to 1.

Values outside the range clamp. No message; it is a readout. Track corners follow `Tokens::shape` (`crate::m3::shape::Component::Track`). Interpolate `value` with `crate::motion::value_animation` so the fill eases when the fraction changes. `indeterminate` paints a traveling chunk; pass a looping phase (0..=1) as `value`.

Constructor: `widget::progress`
Material: Progress indicator
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.progress.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## progress-ring

Circular progress: arc sweep follows `value`. A determinate arc from 0 to 1.

Same fraction contract as `progress`, drawn as a ring. Interpolate `value` with `crate::motion::value_animation`.

Constructor: `widget::progress_ring`
Material: Progress indicator
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.progress_ring.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## spinner

Eight dots around a circle. `phase` (0..=1) lights them in turn.

Advance `phase` each frame while work is running.

Constructor: `widget::spinner`
Material: Progress indicator
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.spinner.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)
