# Command palette

The same action table powers the toolbar and the palette. Type to
rank; Enter invokes.

```rust
use icedtea::action::ActionTable;
use icedtea::palette::{CommandPalette, PaletteOpts};
use icedtea::pattern;
use icedtea::theme;
use icedtea::{Element, Task};

struct App {
    table: ActionTable<Message>,
    palette: CommandPalette,
}

#[derive(Clone)]
enum Message {
    Query(String),
    Pick(usize),
    Prompt(String),
    Save,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Query(q) => self.palette.set_query(&self.table, q),
            Message::Pick(i) => {
                if let Some(next) = self.palette.activate(&self.table, i) {
                    return self.update(next);
                }
            }
            Message::Prompt(s) => {
                if let Some(p) = self.palette.prompt.as_mut() {
                    p.value = s;
                }
            }
            Message::Save => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let tok = theme::named("dark").tokens;
        let hits = self.palette.results(&self.table);
        let mut opts = PaletteOpts::new();
        opts.page = self.palette.page();
        opts.favorite_count = self.palette.favorite_hit_count();
        pattern::command_palette_view(
            self.palette.query(),
            "Type a command",
            &hits,
            self.palette.selected(),
            Message::Query,
            Message::Pick,
            self.palette.prompt.as_ref(),
            Message::Prompt,
            None,
            1.0,
            opts,
            tok,
        )
    }
}
```

`CommandPalette` owns the query and the highlight. Empty query lists
favorites, then recent. Type to rank the table. Enter on the field
invokes the highlighted row, or opens a child page when the action
has `children`. `PaletteOpts` sets grouping, row face, empty copy
(including omit for a field-only idle), panel size, and highlight.
The query field stays visible when a page or `ask` parameter is
showing. An overlay window uses `Boot` with an overlay kind;
`window::place_pinned` keeps it on a chosen display.
