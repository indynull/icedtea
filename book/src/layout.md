# Layout

Layout is Rust functions that return iced `Element`s. Recipes live in
`icedtea::layout`: `dock`, `split_view`, `clamp`, `form`, `grid`,
`overlay_center`, `scroll_y`, plus size policy and breakpoints.

```rust,ignore
use icedtea::layout::{Breakpoint, SizePolicy, distribute};

let sizes = distribute(100.0, &[SizePolicy::fixed(20.0), SizePolicy::expand(1.0)]);
assert_eq!(sizes[0], 20.0);
let mode = Breakpoint::from_width(width);
```

Split ratios persist through `UiState::set_split`. The sash grip emits
press; while pressed, `listen_sash` feeds window-space pointer move and
release into `SashDrag::apply`.
