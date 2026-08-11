# Command palette

The same action table powers the toolbar and the palette. Type to
rank; Enter invokes.

```rust
use icedtea::action::ActionTable;
use icedtea::palette::CommandPalette;
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
                if let Some(action) = self.palette.results(&self.table).get(i) {
                    if let Some(next) = action.invoke() {
                        return self.update(next);
                    }
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
        pattern::command_palette_view(
            self.palette.query(),
            &hits,
            self.palette.selected(),
            Message::Query,
            Message::Pick,
            self.palette.prompt.as_ref(),
            Message::Prompt,
            None,
            tok,
        )
    }
}
```

`CommandPalette` owns the query and the highlight. Empty query lists
favorites, then recent. `ask` opens a parameter field that
`command_palette_view` paints. An overlay window uses `Boot` with an
overlay kind; `window::place_pinned` keeps it on a chosen display.
