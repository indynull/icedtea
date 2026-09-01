# Sortable table

A virtualized table. The clip owns the pixel offset. Sort writes
application state. `on_scroll` reports the mounted range. Jump with
`scroll_to` on `scroll_id`. `on_h_scroll` moves the unfrozen strip.
A focused scroll pane moves with arrows, Page, Home, and End.

```rust
use icedtea::a11y::{A11y, Role};
use icedtea::collection::{ColumnLayout, ItemClick, Selection, TableModel, VisibleWindow};
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
    Cell(ItemClick, usize),
    Sort(usize),
    Scroll(VisibleWindow),
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Cell(click, _col) => self.sel.apply_item_click(click),
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
            None,
            |_| (),
            tok,
            A11y::new("files", Role::Table),
        )
    }
}
```

`ColumnLayout` holds widths, display order, and `frozen` leading
columns. `on_sort` and `on_h_scroll` land in `update`. `on_scroll`
stores `start..end` for the next `view`. Pass `scroll_id` when the
application calls `scroll_to`. Change `scroll_id` when the row set
is a different table. Selecting a row outside the viewport moves
the clip.
