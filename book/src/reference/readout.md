# Readout

Progress, sparks, and a large reading.
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/index.html) ·
[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

### Progress

**`progress`** — A determinate bar from 0 to 1.

Constructor: [`widget::progress`](https://docs.rs/icedtea/latest/icedtea/widget/fn.progress.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Pass the fraction. Optional buffer is a second fill on the same track.
`indeterminate` paints a linear busy chunk (`value` is the offset).
`progress_label` builds the remaining-time copy. Values outside 0..=1
clamp. No message; it is a readout.

### Progress ring

**`progress-ring`** — A determinate arc from 0 to 1.

Constructor: [`widget::progress_ring`](https://docs.rs/icedtea/latest/icedtea/widget/fn.progress_ring.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Same fraction contract as the bar, drawn as a ring.


### Spinner

**`spinner`** — Eight dots around a circle. Phase lights them in turn.

Constructor: [`widget::spinner`](https://docs.rs/icedtea/latest/icedtea/widget/fn.spinner.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`phase` is 0..=1 and comes from application time. Advance it each
frame while work is running.

