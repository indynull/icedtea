# Layout

Layout is Rust functions that return iced `Element`s. Recipes live in
[`icedtea::layout`](https://docs.rs/icedtea/latest/icedtea/layout/index.html):
`dock`, `split_view`, `clamp`, `form`, `grid`, `pad` (equal-fill
tiles), `overlay_center`, plus size policy and breakpoints.

`layout::pad(cells, 4, density.space)` shares row width across cells.
Pair it with `widget::themed_button_sized` and `Density::tile()` for a
key pad. Scroll a pane with `widget::themed_scroll`.
`Breakpoint::from_width` picks the stacked or beside sidebar recipe.

`layout::FILL`, `layout::SHRINK`, and `layout::fixed(px)` are the size
language for boxes and editors. `row_box` / `column_box` take width and
height. A fill-height column gives leftover space to children that
themselves fill (a caption above a filling `textarea`).
`pattern::list_detail` takes the sidebar as that same size.
`split_view` and `list_detail` take `Direction`: the first pane is
left-to-right start (the right edge when the locale is Arabic or Urdu).

```rust,ignore
use icedtea::layout::{self, column_box, row_box};

let panes = row_box(
    [source, preview],
    8,
    0,
    layout::FILL,
    layout::FILL,
    icedtea::i18n::Direction::Ltr,
);
let _ = column_box([caption, editor], 4, 8, layout::FILL, layout::FILL);
```

```rust
use icedtea::layout::{Breakpoint, SizePolicy, distribute};

let sizes = distribute(100.0, &[SizePolicy::fixed(20.0), SizePolicy::expand(1.0)]);
assert_eq!(sizes[0], 20.0);
let _ = Breakpoint::from_width(800.0);
```

Split ratios persist through `UiState::set_split`. The sash grip emits
press; while pressed, `listen_sash` feeds window-space pointer move and
release into `SashDrag::apply`. `listen_cursor` is the same window
pointer for a placed context menu.

`workspace::DockNode` is a nested leaf / split / tab tree with JSON
save and restore, ratio clamps, and `move_panel` between docks.
`pattern::workspace` paints that tree with splits, a sash, and tabs.
`pattern::drawer` is a compact side pane beside content; `progress` eases the pane width. `pattern::tool_panel` is title chrome
plus a Dock control. Perspectives are named `DockNode`
snapshots.

- [layout rustdoc](https://docs.rs/icedtea/latest/icedtea/layout/index.html)
- [source](https://github.com/indynull/icedtea/blob/master/src/layout/mod.rs)
- [crates.io](https://crates.io/crates/icedtea)
