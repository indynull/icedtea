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
so fills and ink fade with the slide. `Slide::None` is fade only:
same constructor, no translate.

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

## Bounce, pulse, and shake

[`bounce_out`](https://docs.rs/icedtea/latest/icedtea/motion/fn.bounce_out.html),
[`pulse`](https://docs.rs/icedtea/latest/icedtea/motion/fn.pulse.html),
and [`shake`](https://docs.rs/icedtea/latest/icedtea/motion/fn.shake.html)
are curves, like [`Ease::sample`](https://docs.rs/icedtea/latest/icedtea/m3/enum.Ease.html#method.sample).
Store `from`, `to`, and the start instant. In `view`, turn elapsed
time into `u` in 0..=1, sample the curve, lerp, and pass that into
`overlay`, `expand`, or `Tokens::fade`.

```
use icedtea::m3::DurationStep;
use icedtea::motion::{self, Slide};

let dur = motion::duration(DurationStep::Long2, tok.reduced_motion);
let u = (elapsed.as_secs_f32() / dur.as_secs_f32()).clamp(0.0, 1.0);
let t = from + (to - from) * motion::bounce_out(u);
let paint = tok.fade(t);
let card = motion::overlay(body, t, Slide::Up, tok, a11y);
```

Expand and contract are [`motion::expand`](https://docs.rs/icedtea/latest/icedtea/motion/fn.expand.html):
height from a peek to the open size. Pulse loops `t` (hold 1 when
reduced motion is on). Shake is a decaying wiggle that starts and
ends at 0; multiply by pixels and shift padding.

Named steps cover overlay, sheet, toast, expand, and progress. Pick
one of those, or pick a `DurationStep` and a curve and drive the
constructor yourself.

See [Material Design 3](m3-foundations.md) for the token names.
Overlay and expand are listed under [Chrome](reference/chrome.md).
Progress is under [Readout](reference/readout.md).
