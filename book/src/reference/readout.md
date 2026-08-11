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

Pass the fraction. Values outside 0..=1 clamp. No message; it is a
readout.

### Progress ring

**`progress-ring`** — A determinate arc from 0 to 1.

Constructor: [`widget::progress_ring`](https://docs.rs/icedtea/latest/icedtea/widget/fn.progress_ring.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Same fraction contract as the bar, drawn as a ring.

### Sparkline

**`sparkline`** — A tiny series chart.

Constructor: [`widget::sparkline`](https://docs.rs/icedtea/latest/icedtea/widget/fn.sparkline.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Pass a slice of numbers. Empty data paints an empty box.

### Spinner

**`spinner`** — An indeterminate quarter-arc.

Constructor: [`widget::spinner`](https://docs.rs/icedtea/latest/icedtea/widget/fn.spinner.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`phase` is 0..=1 and comes from application time. Advance it each
frame while work is running.

### Display reading

**`display`** — A large value for a compact tool.

Constructor: [`widget::display_reading`](https://docs.rs/icedtea/latest/icedtea/widget/fn.display_reading.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Use on a calculator or meter. Empty string is a blank reading.
`display_line` is the smaller caption above it.
