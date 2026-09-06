# Motion

Pick a [`Job`](https://docs.rs/icedtea/latest/icedtea/motion/enum.Job.html).
Hold a [`Run`](https://docs.rs/icedtea/latest/icedtea/motion/struct.Run.html).
Pass `progress` into the constructor. The application owns the clock.
`Tokens::with_reduced_motion(true)` collapses every duration to 0 ms
so progress snaps to the target.

| Job | When to use it | Constructor |
| --- | --- | --- |
| `Enter` | Dialog, menu, sheet, toast, tooltip | [`overlay`](https://docs.rs/icedtea/latest/icedtea/motion/fn.overlay.html) |
| `Switch(FadeThrough)` | Tab body, settings group, unrelated destinations | [`switch`](https://docs.rs/icedtea/latest/icedtea/motion/fn.switch.html) |
| `Switch(SharedAxis)` | Next/previous peers (timeline, wizard, list to detail) | [`switch`](https://docs.rs/icedtea/latest/icedtea/motion/fn.switch.html) |
| `Disclose(Block)` | Expander, accordion, tree branch | [`expand`](https://docs.rs/icedtea/latest/icedtea/motion/fn.expand.html) |
| `Disclose(Inline)` | Drawer, folder rail | [`expand`](https://docs.rs/icedtea/latest/icedtea/motion/fn.expand.html) |
| `Attention(Shake)` | Invalid field after a failed check | [`attention`](https://docs.rs/icedtea/latest/icedtea/motion/fn.attention.html) |
| `Attention(Pulse)` | Live or recording mark | [`attention`](https://docs.rs/icedtea/latest/icedtea/motion/fn.attention.html) |
| `Value` | Determinate progress or ring | [`value_animation`](https://docs.rs/icedtea/latest/icedtea/motion/fn.value_animation.html) |

Enter decelerates in; exit is shorter and accelerates.
`Run::lasting` holds a custom enter length and keeps that exit ratio
(the gallery duration slider does this). Shared axis
travels 12 dp: incoming uses `progress` on that slide, leaving uses
`1 - progress` on the opposite slide. Fade through finishes the
leaving fade before the incoming fade starts.

## Drive a constructor

```
use icedtea::motion::{self, Enter, Job, Slide};

let mut run = motion::run(Job::Enter(Enter::Dialog), false, tok.reduced_motion);
run.go(true, iced::time::Instant::now());
let t = run.progress(iced::time::Instant::now());
let paint = tok.fade(t);
let sheet = motion::overlay(body, t, Slide::End, tok, a11y);
```

Build overlay and switch children with
[`Tokens::fade`](https://docs.rs/icedtea/latest/icedtea/theme/struct.Tokens.html#method.fade).
For switch, use `SwitchFace::incoming_fade` / `outgoing_fade`.

`modal_card`, `side_sheet`, `command_palette_view`, `context_menu`,
`cascade_menu`, `drawer`, `expander`, `accordion_view`, and `tree_view`
take that same 0–1 progress (`tree_view` as `animating: Option<(id, progress)>`).

## Mix several jobs

Each surface has its own `Run`. A sheet, an expander, and a progress
value can run together. The subscription fires while any of them
reports `is_live` (or while toasts still have TTL).

[`bounce_out`](https://docs.rs/icedtea/latest/icedtea/motion/fn.bounce_out.html)
is a curve for a hop that is not a named job. Sample it like
[`Ease`](https://docs.rs/icedtea/latest/icedtea/m3/enum.Ease.html).

See [Material Design 3](m3-foundations.md) for the token names.
Constructors are listed under [Chrome](reference/chrome.md).
Progress is under [Readout](reference/readout.md).
