# Navigation

Keep page identity in application **state**. `NavStack` is push / pop /
replace with `can_back` when depth > 1. `view` calls
`pattern::navigation_view`, which places a sidebar beside content on
medium and expanded widths, and stacks with a back **message** on
compact. `navigation_view` still takes that width as an argument.

One resize subscription feeds it. `Subscription::map` must be
non-capturing; convert the size in `update`.

```rust,ignore
fn window_width((_id, size): (iced::window::Id, iced::Size)) -> Message {
    Message::WindowSize(size.width)
}

fn subscription(_state: &App) -> iced::Subscription<Message> {
    iced::window::resize_events().map(window_width)
}
```

List/detail, tab view, preferences, about, and status page are in
`icedtea::pattern` — they return `Element`s.
