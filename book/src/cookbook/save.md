# Save a buffer

Save on `ctrl+s` through the toolbar is the first window.

[First window](../first-window.md) is that program: `file.save`,
`pattern::toolbar`, `widget::textarea`, `pattern::status_bar`,
`key::listen`, `key::handle`, `focus::cycle`, and
`ActionTable::seed_quit`.

An open modal consumes keys first. Otherwise focused text owns
unmodified typing. Otherwise `key::handle` matches the action table,
so Save still fires while the caret is in the editor. `seed_quit`
adds Quit on `ctrl+q`. `focus::cycle` wraps `view` so Tab reaches
the editor.

```rust
if let Some(next) = key::handle(KeyContext::default(), &self.table, &ev) {
    return self.update(next);
}
```

[Keep a task list](tasks.md) is the next job: a list, Add, mark done,
and a SQLite file on disk.
