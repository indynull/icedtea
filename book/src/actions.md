# Actions

An `Action` carries the application's **message**. One definition feeds
menus, toolbars, shortcuts, footer hints, and the command palette.
`update` decides what happens when that message arrives.

```rust,ignore
use icedtea::action::{Action, ActionTable};
use icedtea::shortcut::Shortcut;

let mut table = ActionTable::new();
table.insert(
    Action::new("file.save", "Save", Message::Save)
        .with_shortcut(Shortcut::parse("ctrl+s").unwrap()),
);
assert_eq!(table.invoke("file.save"), Some(Message::Save));
```

Key order is focused text input, then modal, then window, then
application (`icedtea::key::dispatch`).

`key::handle` matches **logical** shortcuts (`ctrl+s`). Focused text
still owns unmodified typing; Ctrl/Cmd/Alt chords still invoke the
action table, so Save works while the caret is in an editor.
`key::typed` and `key::press` read what the user typed: Shift+8 is
`*`, not `8`. Control, alt, and logo chords return `None` from
`typed` / `press` so `handle` still owns them.
