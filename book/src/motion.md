# Motion

The application owns [`iced::Animation`](https://docs.rs/iced/latest/iced/struct.Animation.html)
and the clock. Constructors paint one frame from a 0–1 progress (or
an interpolated value). `Tokens::with_reduced_motion(true)` collapses
every [`m3::DurationStep`](https://docs.rs/icedtea/latest/icedtea/m3/enum.DurationStep.html)
to 0 ms so progress snaps to the target.

Duration and easing live in [`m3::motion`](https://docs.rs/icedtea/latest/icedtea/m3/motion/index.html).
Paint helpers live in [`motion`](https://docs.rs/icedtea/latest/icedtea/motion/index.html).

## Drive a constructor

Hold an animation. On open or close, `go_mut` the target. While
`is_animating`, subscribe to frames. In `view`, interpolate and pass
the number into the constructor.

```
use icedtea::motion::{self, Slide};
use icedtea::theme::Tokens;

// in the application
let mut anim = motion::overlay_animation(false, tok.reduced_motion);
anim.go_mut(true, iced::time::Instant::now());
let t = anim.interpolate(0.0, 1.0, iced::time::Instant::now());
let paint = tok.fade(t);
let sheet = motion::overlay(body, t, Slide::End, tok, a11y);
```

[`motion::overlay`](https://docs.rs/icedtea/latest/icedtea/motion/fn.overlay.html)
slides. Build the child with [`Tokens::fade`](https://docs.rs/icedtea/latest/icedtea/theme/struct.Tokens.html#method.fade)
so fills and ink fade with the slide.
[`motion::expand`](https://docs.rs/icedtea/latest/icedtea/motion/fn.expand.html)
clips height between a peek and the open size.
[`motion::value_animation`](https://docs.rs/icedtea/latest/icedtea/motion/fn.value_animation.html)
eases a determinate 0–1 for [`progress`](https://docs.rs/icedtea/latest/icedtea/widget/fn.progress.html)
and [`progress_ring`](https://docs.rs/icedtea/latest/icedtea/widget/fn.progress_ring.html).
[`motion::progress_run`](https://docs.rs/icedtea/latest/icedtea/motion/fn.progress_run.html)
is the linear busy-bar phase.

`modal_card`, `side_sheet`, `command_palette_view`, `context_menu`,
`cascade_menu`, `drawer`, `expander`, `accordion_view`, and `tree_view`
take that same 0–1 progress (`tree_view` as `animating: Option<(id, progress)>`).

## Mix several jobs

Each surface has its own `Animation`. A sheet, an expander, and a
progress value can run together. `view` samples each one and passes
the numbers in. The subscription fires while any of them reports
`is_animating` (or while toasts still have TTL).

## Write a number yourself

`Ease::sample(t)` is the cubic at `t` in 0..=1. `duration(step, reduced)`
is the length. A custom job is: store `from`, `to`, and the start
instant; in `view` compute `t` from elapsed time; sample the ease;
lerp `from` toward `to`; pass the result into `overlay`, `expand`,
`fade`, or your own paint. That is the same path the helpers use.

Named steps cover overlay, sheet, toast, expand, and progress. Pick
one of those, or pick a `DurationStep` and an `Ease` and drive the
constructor yourself.

See [Material Design 3](m3-foundations.md) for the token names.
Overlay and expand are listed under [Chrome](reference/chrome.md).
Progress is under [Readout](reference/readout.md).
