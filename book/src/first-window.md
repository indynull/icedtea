# First window

`icedtea::run!` starts the window. One `Action` feeds the toolbar.
`ctrl+i` or the Count row increments the same counter.

The program is [`examples/hello.rs`](https://github.com/indynull/icedtea/blob/master/examples/hello.rs)
in the repository. `cargo run --example hello`.

```rust
{{#include ../../examples/hello.rs}}
```

`Boot` loads tokens, locale, and window settings. Text uses the
platform sans; code uses the platform mono. Load a named family on
the iced application if you want a specific face.

A compact tool sets size on `Boot` (`.size(380.0, 560.0).min_size(...)`)
instead of calling iced window resize. See [Compact tools](compact-tools.md).

`bootstrap(&boot)` is the same path without opening a window — use it
in tests. Crate docs: [`run!`](https://docs.rs/icedtea/latest/icedtea/macro.run.html),
[`Boot`](https://docs.rs/icedtea/latest/icedtea/app/struct.Boot.html).
