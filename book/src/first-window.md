# First window

This is the smallest icedtea program: a notes window. `icedtea::run!`
opens it. One `Action` named Save sits on the toolbar. `ctrl+s` or
that row writes the text length into the status line. The notes live
only in memory until you build [Keep a task list](cookbook/tasks.md).

The program is [`examples/hello.rs`](https://github.com/indynull/icedtea/blob/main/examples/hello.rs)
in the repository. `cargo run --example hello`.

![Notes: Save on the toolbar and a filling editor](images/first-window.png)

```rust
{{#include ../../examples/hello.rs}}
```

`Message` implements `From<keyboard::Event>` so `run!` can subscribe
`key::listen`. `view` wraps the column in `focus::cycle` so Tab
reaches the editor. `seed_quit` inserts `app.quit` on `ctrl+q`.
`key::handle` matches Save and Quit after the editor owns typing.

`Boot` is the window name, size, colors, and look (density, type
scale, corners, elevation). `run!` (and `daemon!`)
also pick an installed sans face for UI text (normal and bold) and a
fixed face for code. Load a named family on the iced application if
you want a specific font.

A compact tool sets size on `Boot` (`.size(380.0, 560.0).min_size(...)`)
instead of calling iced window resize. See [Compact tools](compact-tools.md).

When you want the list to survive quitting, go to
[Keep a task list](cookbook/tasks.md).

`bootstrap(&boot)` is the same path without opening a window — use it
in tests. An overlay that hides and pops out uses [`daemon!`](https://docs.rs/icedtea/latest/icedtea/macro.daemon.html)
plus [`Prepared::open`](https://docs.rs/icedtea/latest/icedtea/app/struct.Prepared.html).
Crate docs: [`run!`](https://docs.rs/icedtea/latest/icedtea/macro.run.html),
[`Boot`](https://docs.rs/icedtea/latest/icedtea/app/struct.Boot.html).
