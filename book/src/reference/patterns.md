# Patterns

Composed chrome. Gallery pages `dialogs`, `list-detail`, `workspace`,
`navigation`, `tab-view`, `preferences`, `about`, `status-page`,
`palette`, `main-window`.
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/index.html) ·
[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

### Dialogs

**`dialogs`** — A confirm / message / save sheet.

Constructor: [`pattern::dialog_sheet`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.dialog_sheet.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Primary and optional cancel messages. Native file dialogs go through
`icedtea::native_dialog`. In-window modals sit on `pattern::modal_card`.

### List/detail

**`list-detail`** — A sidebar list beside a filling detail pane.

Constructor: [`pattern::list_detail`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.list_detail.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`sidebar` is `layout::fixed` or `layout::FILL`. Children fill their
panes.

### Inspector

**`inspector`** — Master, detail, and a side inspector in one row.

Constructor: [`pattern::inspector`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.inspector.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Three panes. The application owns selection in the list.

### Drawer

**`drawer`** — A compact-width slide-over for a collapsed dock.

Constructor: [`pattern::drawer`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.drawer.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`open` shows the pane over `content`. Closed paints content only.

### Workspace

**`workspace`** — Nested dock slots as a labeled strip plus center.

Constructor: [`pattern::workspace`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.workspace.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`DockNode` is the layout tree (save, restore, `move_panel`).
Applications own panel content.

### Tool panel

**`tool-panel`** — Overlay chrome for a floatable tool window.

Constructor: [`pattern::tool_panel`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.tool_panel.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Title plus body. `on_dock` is the Dock control. Empty body is title
chrome only.

### Navigation view

**`navigation`** — Sidebar beside content, or a stack with Back.

Constructor: [`pattern::navigation_view`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.navigation_view.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`NavStack` is push / pop / replace. Pass window width.
`Breakpoint::from_width` picks beside vs stacked. See
[Navigation](../navigation.md).

### Tab view

**`tab-view`** — Tabs plus a filling body.

Constructor: [`pattern::tab_view`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.tab_view.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Select and close messages. The application paints the body for the
active tab.

### Preferences

**`preferences`** — Searchable preference groups.

Constructor: [`pattern::preferences_page`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.preferences_page.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`PrefGroup` is a title plus key/value rows. Empty query shows every
group.

### About

**`about`** — Name, version, license, and credits.

Constructor: [`pattern::about_page`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.about_page.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Strings are the application's. Catalog supplies chrome labels.

### Status page

**`status-page`** — Centered empty or error state.

Constructor: [`pattern::status_page`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.status_page.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Title, body, and an optional action. Use when a list has no rows.

### Command palette

**`palette`** — Fuzzy find over the action table.

Constructor: [`pattern::command_palette_view`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.command_palette_view.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`CommandPalette` owns the query and hits. Empty query can show
recent and favorites. See [Overlay windows](../overlay-windows.md).

### Main window

**`main-window`** — Menu, toolbar, center, and status docked together.

Constructor: [`pattern::main_window`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.main_window.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Pass the four regions as `Element`s. [First window](../first-window.md)
is the smaller form: toolbar plus one control.
