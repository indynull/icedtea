# Navigation

Keep page identity in application **state**. `NavStack` is push / pop /
replace with `can_back` when depth > 1. `view` calls
[`pattern::navigation_view`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.navigation_view.html),
which places a sidebar beside content on medium and expanded widths,
and stacks with a back **message** on compact. `navigation_view` still
takes that width as an argument.

One resize subscription feeds it. `Subscription::map` must be
non-capturing; convert the size in `update`.

```rust
fn window_width((_id, size): (iced::window::Id, iced::Size)) -> u16 {
    size.width as u16
}

fn subscription() -> iced::Subscription<u16> {
    iced::window::resize_events().map(window_width)
}
```

List/detail and tab view are [Layouts](reference/layouts.md).
Preferences, about, and status page are [Screens](reference/screens.md).
They live in
[`icedtea::pattern`](https://docs.rs/icedtea/latest/icedtea/pattern/index.html)
and return `Element`s.

- [`NavStack`](https://docs.rs/icedtea/latest/icedtea/nav/struct.NavStack.html)
- [source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs)
- [crates.io](https://crates.io/crates/icedtea)
