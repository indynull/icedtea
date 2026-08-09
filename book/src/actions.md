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
