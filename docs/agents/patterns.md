# Patterns

Generated from `catalog::ENTRIES`, constructor rustdoc, and
`m3::mapping`. Compose starts at `examples/hello.rs` (see
[llms.txt](llms.txt)).

## side-sheet

Docked supporting pane (M3 side sheet) over a dimmed scene.

`end` true docks the trailing edge (right in LTR, left in RTL). Body is application content; dismiss is optional. `progress` is 0 (gone) to 1 (rest).

Constructor: `pattern::side_sheet`
Material: Side sheets
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.side_sheet.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## dialogs

An in-window confirm / message / save sheet (`DialogOpts`). Native file pick is `crate::native_dialog`.

Primary accept is required. Cancel, extra actions, header icon, and a dim backdrop live on `DialogOpts`.

Constructor: `pattern::dialog_sheet`
Material: Dialogs
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.dialog_sheet.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## list-detail

A sidebar list beside a filling detail pane.

`sidebar` is `crate::layout::fixed` or `crate::layout::FILL`. Children fill their panes.

Constructor: `pattern::list_detail`
Desktop: Supporting pane
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.list_detail.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## inspector

Master, detail, and a side inspector stay in one row.

The list sits on the start edge. The application owns selection in the list.

Constructor: `pattern::inspector`
Desktop: Supporting pane
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.inspector.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## drawer

Compact-width side pane beside `content`.

`open` is the committed pane. `progress` is 0 (gone) to 1 (220 dp). Closed at 0 paints `content` only.

Constructor: `pattern::drawer`
Material: Navigation drawer
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.drawer.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## workspace

Nested dock tree: splits with a sash, tab groups, and leaf chrome.

`pane` is called with each leaf id (and the active tab id). `header` is trailing chrome on that leaf (kind badge, close). Pass `|_| None` when the leaf has no header extra. `on_sash` is the split index then the grip event. `on_tab` is the depth-first tab-group index, then the selected tab (`DockNode::select_tab_group`).

Constructor: `pattern::workspace`
Desktop: Layout (dock)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.workspace.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## tool-panel

Title chrome plus a Dock control.

Title plus body. `on_dock` is the Dock button message.

Constructor: `pattern::tool_panel`
Desktop: Supporting pane
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.tool_panel.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## nav-rail

Compact destination rail beside content (desktop map of M3 navigation rail).

Selected row uses `style::nav_rail`. Press emits the destination index. Empty `items` is an empty column. `expanded` is the labeled 220 px face.

Constructor: `pattern::nav_rail`
Material: Navigation rail
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.nav_rail.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## navigation

Sidebar beside content, or a stack with Back.

`width` is the window inner width. Subscribe with `iced::window::resize_events` and a non-capturing `Subscription::map`; store the width in `update`.

Constructor: `pattern::navigation_view`
Material: Navigation
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.navigation_view.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## tab-view

Tabs plus a filling body.

Select and close messages. The application paints the body for the active tab.

Constructor: `pattern::tab_view`
Material: Tabs
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.tab_view.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## preferences

Searchable preference groups.

`PrefGroup` is a title plus key/value rows. Empty query shows every group.

Constructor: `pattern::preferences_page`
Desktop: Lists (settings)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.preferences_page.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## about

Name, version, license, and credits.

The card is a fixed width so the credits line wraps inside the group box. `Length::Fill` plus word wrap still measures as one line under mixed bidi (Hebrew/Arabic plus `iced 0.14`), so the credits text gets a definite inner width.

Constructor: `pattern::about_page`
Material: Dialogs
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.about_page.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## status-page

Centered empty or error state.

Title, body, and an optional action. Use when a list has no rows.

Constructor: `pattern::status_page`
Material: Empty states
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.status_page.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## palette

Fuzzy find over the action table (`crate::palette::CommandPalette` results, `crate::palette::PaletteOpts`; stack over the window).

Pass `CommandPalette::results` and `crate::palette::PaletteOpts`. An empty query lists favorites, then recent. The query field stays up when a nested page or `ask` parameter is showing. Enter on the query invokes the highlighted row (`on_pick(selected)`).

Constructor: `pattern::command_palette_view`
Desktop: Menus (command)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.command_palette_view.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## main-window

Menu, toolbar, center, and status docked together.

Pass the four regions as `Element`s.

Constructor: `pattern::main_window`
Desktop: App bars (desktop)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.main_window.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)
