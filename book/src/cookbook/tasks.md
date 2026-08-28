# Keep a task list

This is a one-day job after [First window](../first-window.md). You
will build a small desktop window that:

- lists your tasks
- lets you type a new one and add it
- lets you mark one done
- remembers them in a real database file on disk (`tasks.db`)

icedtea draws the window, the toolbar, the field, the checks, and the
status line. **You** own the list and the file. icedtea does not store
your tasks.

The finished program is about 200 lines. The same source lives in the
repository as
[`examples/tasks.rs`](https://github.com/indynull/icedtea/blob/main/examples/tasks.rs).
`cargo run --example tasks` from a checkout.

## What the words mean

A **message** is a note about something the user did: they typed, they
pressed Add, they ticked a box. You invent the notes.

**`update`** reads the note and changes your data (and the file).

**`view`** looks at the data and draws the window. It does not write
the file.

An **`Action`** is a command you declare once. The same Add command
can sit on the toolbar and on `ctrl+n`.

A **constructor** is an icedtea function that draws one control and
sends your message when the user uses it.

## New crate

You need a Rust toolchain (`rustup`). In an empty folder:

```bash
cargo new tasks
cd tasks
cargo add icedtea
cargo add rusqlite --features bundled
```

`bundled` means the database engine is compiled with the crate. You
do not install SQLite yourself.

After those two commands, `Cargo.toml` looks like this. The published
guide fills the crate versions from this repository's `Cargo.toml`
when the book builds.

```toml
[package]
name = "tasks"
version = "0.1.0"
edition = "2021"

[dependencies]
icedtea = "{{ICEDTEA_VERSION}}"
rusqlite = { version = "{{RUSQLITE_VERSION}}", features = ["bundled"] }
```

Replace `src/main.rs` with the program below.

## The program

```rust
{{#include ../../../examples/tasks.rs}}
```

## What each part does

`Boot::new("Tasks", "dev.example.tasks")` names the window. The second
string is an application id the operating system can use.

`open_db` creates `tasks.db` in the folder where you launched the
program, then creates a `task` table if it is missing. That file is
the database.

`load_rows` reads every row. `insert_row` and `set_done` write. The
window never keeps a second copy that can drift from the file: after
a successful write, `reload` reads the table again.

`Message::Draft` is each keystroke in the field. `Message::Add` is the
toolbar, `ctrl+n`, or Enter in the field. `Message::Toggle` is a
checkbox. Empty titles are not saved.

`scroll` lets a long list move. `A11y` is the name a screen
reader uses; every icedtea constructor takes one.

## Run it

```bash
cargo run
```

From this repository instead:

```bash
cargo run --example tasks
```

Type a line, press Add or Enter, tick the box. Quit and run again:
the same rows come back from `tasks.db`.

If the window does not start on Linux, install the host libraries on
[Install](../install.md).

## Next

[List and detail](list-detail.md) is the same idea with a sidebar
list and a filling pane. The [reference](../widgets.md) names every
constructor.
