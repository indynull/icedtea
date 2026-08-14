# Patterns

Composed chrome.
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/index.html) ·
[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

### Dialogs

**`dialogs`** — A confirm / message / save sheet.

Constructor: [`pattern::dialog_sheet`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.dialog_sheet.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Primary and optional cancel messages. Native file dialogs go through
`icedtea::native_dialog`. In-window modals sit on `pattern::modal_card`.

### List/detail

**`list-detail`** — A sidebar list beside a filling detail pane.

Constructor: [`pattern::list_detail`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.list_detail.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

`sidebar` is `layout::fixed` or `layout::FILL`. Children fill their
panes.

### Inspector

**`inspector`** — Master, detail, and a side inspector in one row.

Constructor: [`pattern::inspector`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.inspector.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Three panes. The application owns selection in the list.

### Drawer

**`drawer`** — A compact-width side pane beside content.

Constructor: [`pattern::drawer`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.drawer.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

`open` is `list_detail` with a fixed pane. Closed paints content only.

### Workspace

**`workspace`** — Nested dock tree: splits, sash, tab groups, leaf chrome.

Constructor: [`pattern::workspace`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.workspace.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

`DockNode` is the layout tree (save, restore, `move_panel`).
`pane` is called with each leaf id so every leaf can hold application
content.

### Tool panel

**`tool-panel`** — Title chrome plus a Dock control.

Constructor: [`pattern::tool_panel`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.tool_panel.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Title plus body. `on_dock` is the Dock control. Empty body is title
chrome only.

### Navigation rail

**`nav-rail`** — Compact destination list beside content.

Constructor: [`pattern::nav_rail`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.nav_rail.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Selected row uses the rail wash. Press emits the destination index.
`expanded` is the wide labeled rail; compact shows the first letter
in a 72 px column. Empty items paint an empty column.

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

### Preferences

**`preferences`** — Searchable preference groups.

Constructor: [`pattern::preferences_page`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.preferences_page.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

`PrefGroup` is a title plus key/value rows. Empty query shows every
group.

### About

**`about`** — Name, version, license, and credits.

Constructor: [`pattern::about_page`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.about_page.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Strings are the application's. Catalog supplies chrome labels.

### Status page

**`status-page`** — Centered empty or error state.

Constructor: [`pattern::status_page`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.status_page.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Title, body, and an optional action. Use when a list has no rows.

### Command palette

**`palette`** — Fuzzy find over the action table.

Constructor: [`pattern::command_palette_view`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.command_palette_view.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

`CommandPalette` owns the query and hits. Empty query can show
recent and favorites. See [Overlay windows](../overlay-windows.md).

### Main window

**`main-window`** — Menu, toolbar, center, and status docked together.

Constructor: [`pattern::main_window`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.main_window.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Pass the four regions as `Element`s. [First window](../first-window.md)
is the smaller form: toolbar plus one control.

### Side sheet

**`side-sheet`** — Docked supporting pane over a dimmed scene.

Constructor: [`pattern::side_sheet`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.side_sheet.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`end` docks the trailing edge. Optional dismiss closes the sheet.
