# Actions

An `Action` carries the application's **message**. One definition feeds
menus, toolbars, shortcuts, footer hints, and the command palette.
`update` decides what happens when that message arrives.

```rust
use icedtea::action::{Action, ActionTable};
use icedtea::shortcut::Shortcut;

let mut table = ActionTable::new();
table.insert(
    Action::new("file.save", "Save", "saved")
        .with_shortcut(Shortcut::parse("ctrl+s").unwrap()),
);
assert_eq!(table.invoke("file.save"), Some("saved"));
```

Write `ctrl+s` once. icedtea stores Command on macOS and Control on
Linux and Windows; menus print that host form.

[First window](first-window.md) uses `count.inc` the same way: the
toolbar row and `ctrl+i` both send `Message::Inc`.

`ActionTable::conflicts` reports duplicate shortcuts in the same
context. `with_sequence` plus `key::SequenceBuffer` is `ctrl+k` then
`g`. `handle_in` takes a keymap context. The palette keeps recent and
favorites and can open a `Prompt`. `pattern::cheatsheet` lists the
table.

`key::handle` uses `KeyContext`: an open modal consumes (even if a
field is focused); otherwise focused text owns unmodified typing;
otherwise chords hit the action table, so Save works while the caret
is in an editor. `dispatch` / `KeyLayer` are for an application that
builds its own stack. `key::typed` and `key::press` read what the user
typed: Shift+8 is `*`, not `8`. F1-F24 are `Press::Function`. Control,
alt, and logo chords return `None` from `typed` / `press` so `handle`
still owns them.

`pattern::context_menu` places the same `Action` list under the
pointer. The application stores the point (`layout::listen_cursor`)
and whether the menu is open. Click-away and Escape close it. Editors
enable Cut/Copy from `text_editor::Content::selection`. Lists select
on right-click, then the application opens the menu.

Subscribe with `key::listen` and map events into `update`, as
`examples/hello.rs` does.

- [`Action`](https://docs.rs/icedtea/latest/icedtea/action/struct.Action.html)
- [`ActionTable`](https://docs.rs/icedtea/latest/icedtea/action/struct.ActionTable.html)
- [`key::handle`](https://docs.rs/icedtea/latest/icedtea/key/fn.handle.html)
- [`key::listen`](https://docs.rs/icedtea/latest/icedtea/key/fn.listen.html)
- [source](https://github.com/indynull/icedtea/blob/master/src/action.rs)
- [crates.io](https://crates.io/crates/icedtea)
