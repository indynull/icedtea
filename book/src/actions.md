# Actions

An `Action` carries the application's **message**. One definition feeds
menus, toolbars, shortcuts, footer hints, and the command palette.
`update` decides what happens when that message arrives.

`key::handle` plus `KeyContext` is the default path. An open modal
consumes (even if a field is focused). Otherwise focused text owns
unmodified typing. Otherwise chords hit the action table, so Save
works while the caret is in an editor.

```rust
use icedtea::action::{Action, ActionTable};
use icedtea::key::{handle, KeyContext};
use icedtea::shortcut::Shortcut;

let mut table = ActionTable::new();
table.insert(
    Action::new("file.save", "Save", "saved")
        .with_shortcut(Shortcut::parse("ctrl+s").unwrap()),
);
assert_eq!(table.invoke("file.save"), Some("saved"));
let _ = handle;
let _ = KeyContext::default();
```

Write `ctrl+s` once. icedtea stores Command on macOS and Control on
Linux and Windows; menus print that host form.

[First window](first-window.md) is that Save path: the toolbar row and
`ctrl+s` both send `Message::Save`.

`key::typed` and `key::press` read what the user typed: Shift+8 is
`*`, not `8`. F1-F24 are `Press::Function`. Control, alt, and logo
chords return `None` from `typed` / `press` so `handle` still owns
them.

`ActionTable::conflicts` reports duplicate shortcuts in the same
context. `with_sequence` plus `key::SequenceBuffer` is `ctrl+k` then
`g`. `handle_in` takes a keymap context. The palette keeps recent and
favorites. `pattern::cheatsheet` lists the
table.

## Own stack

`dispatch` / `KeyLayer` are for an application that builds its own
layer list. Most windows only need `key::handle` and `KeyContext`.

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
