# List and detail

A sidebar list beside a filling detail pane. The application owns
which row is selected.

```rust
use icedtea::a11y::{A11y, Role};
use icedtea::collection::{Selection, VecList, VisibleWindow};
use icedtea::i18n::Direction;
use icedtea::pattern;
use icedtea::theme;
use icedtea::widget;
use icedtea::{Element, Task};

struct App {
    list: VecList,
    sel: Selection,
    window: VisibleWindow,
}

#[derive(Clone)]
enum Message {
    Select(usize),
    Scroll(VisibleWindow),
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Select(i) => self.sel.select_single(i),
            Message::Scroll(w) => self.window = w,
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let tok = theme::named("dark").tokens;
        let side = widget::list_view(
            &self.list,
            &self.sel,
            Message::Select,
            tok,
            self.window,
            32.0,
            2,
            Message::Scroll,
            "No rows",
            |_| tok.muted,
            None,
            icedtea::collection::RowFace::FLUSH,
            Message::Select,
            A11y::new("files", Role::List),
        );
        let title = self
            .sel
            .primary()
            .and_then(|i| self.list.items.get(i).map(|r| r.title.clone()))
            .unwrap_or_else(|| "Nothing selected".into());
        let detail = widget::label(title, tok, A11y::new("detail", Role::Status));
        pattern::list_detail(side, detail, icedtea::layout::fixed(260.0), tok)
    }
}
```

`layout::fixed` sizes the sidebar. The detail child fills. Selection
stays on indices. Pass `RowHeights::PerRow` when rows are not one
height. `RowFace::Card` wraps the title on a surface; `Flush` is one
clipped line.
