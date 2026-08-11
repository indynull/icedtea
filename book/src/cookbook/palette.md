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
            tok,
        )
    }
}
```

`CommandPalette` owns the query and the highlight. Empty query lists
favorites, then recent. An overlay window uses `Boot` with an overlay
kind; the constructor itself is the card.
