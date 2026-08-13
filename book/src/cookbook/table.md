# Sortable table

A virtualized table. `VisibleWindow.scroll` is the only offset. Sort
and scroll write application state.

```rust
use icedtea::a11y::{A11y, Role};
use icedtea::collection::{ColumnLayout, Selection, TableModel, VisibleWindow};
use icedtea::theme;
use icedtea::widget;
use icedtea::{Element, Task};

struct App {
    table: TableModel,
    cols: ColumnLayout,
    sel: Selection,
    window: VisibleWindow,
}

#[derive(Clone)]
enum Message {
    Cell(usize, usize),
    Sort(usize),
    Scroll(VisibleWindow),
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Cell(row, _col) => self.sel.select_single(row),
            Message::Sort(col) => self.table.sort(col),
            Message::Scroll(w) => self.window = w,
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let tok = theme::named("dark").tokens;
        widget::data_table(
            &self.table,
            &self.sel,
            None,
            &self.cols,
            true,
            self.window,
            32.0,
            2,
            Message::Cell,
            Message::Sort,
            Message::Scroll,
            |_| 0.0,
            |_| (),
            tok,
            A11y::new("files", Role::Table),
        )
    }
}
```

`ColumnLayout` holds widths, display order, and `frozen` leading
columns. `on_sort`, `on_scroll`, and `on_h_scroll` must land in
`update` or the table will not move.
