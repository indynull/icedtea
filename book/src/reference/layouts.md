# Layouts

Multi-pane and window structure. Regions the application fills with
its own content.
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/index.html) ·
[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

### Main window

**`main-window`** — Menu, toolbar, center, and status docked together.

Constructor: [`pattern::main_window`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.main_window.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Pass the four regions as `Element`s. [First window](../first-window.md)
is the smaller form: toolbar plus one control.

### Navigation view

**`navigation`** — Sidebar beside content, or a stack with Back.

Constructor: [`pattern::navigation_view`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.navigation_view.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

`NavStack` is push / pop / replace. Pass window width.
`Breakpoint::from_width` picks beside vs stacked. See
[Navigation](../navigation.md).

### Tab view

**`tab-view`** — Tabs plus a filling body.

Constructor: [`pattern::tab_view`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.tab_view.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Select and close messages. The application paints the body for the
active tab.

### List/detail

**`list-detail`** — A sidebar list beside a filling detail pane.

Constructor: [`pattern::list_detail`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.list_detail.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

`sidebar` is `layout::fixed` or `layout::FILL`. The list pane is a
padded panel; a hairline splits the panes; the detail pane is inset
12px on every side.

### Inspector

**`inspector`** — Master, detail, and a side inspector in one row.

Constructor: [`pattern::inspector`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.inspector.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Three panes. The application owns selection in the list.

### Workspace

**`workspace`** — Nested dock tree: splits, sash, tab groups, leaf chrome.

Constructor: [`pattern::workspace`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.workspace.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

`DockNode` is the layout tree (save, restore, `move_panel`).
`pane` is called with each leaf id so every leaf can hold application
content.

### Drawer

**`drawer`** — A compact-width side pane beside content.

Constructor: [`pattern::drawer`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.drawer.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

`open` is `list_detail` with a fixed pane. Closed paints content only.

### Tool panel

**`tool-panel`** — Title chrome plus a Dock control.

Constructor: [`pattern::tool_panel`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.tool_panel.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Title plus body. `on_dock` is the Dock control. Empty body is title
chrome only.
