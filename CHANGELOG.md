# Changelog

## Unreleased

`light` and `dark` are a neutral desktop pair. Persist defaults
`follow_os` on. When follow-OS is on, `OsChrome` /
`os_chrome` / `listen_os_chrome` / `apply_os_chrome` layer optional
desktop colors onto the active colorway: accent as `primary` on every
host; `canvas`, `surface`, `panel`, `text`, `muted`, and `border` on
macOS and Windows; Linux (settings portal) supplies accent only.
Unset fields and follow-OS off leave the colorway unchanged. Success,
warning, and danger stay on the colorway.

`run!` and `daemon!` call `typo::install_platform_faces` so SansSerif
and Monospace bind to installed faces. UI prefers a discrete
Regular+Bold family (Helvetica Neue, Lucida Grande, …) and accepts
variable faces when needed; monospaced names never bind as UI.
Apps that start iced without those macros call `install_platform_faces`
before the first frame.

Virtual lists and tables hard-clip overscan with `ClipLayer` (real
scissor, not soft `container.clip`). Paint clamps scroll so a past-end
offset cannot blank a non-empty model. Frozen table columns stay clear
of horizontally scrolled cells. `list_detail` and `inspector` pad the
list pane and separate regions with a hairline. Nested context menus
place the flyout beside the root (no overlap) and top-align to the
parent row; tall flyouts share the root height cap and scroll.
`split_button` takes overflow rows and opens an iced menu from the
chevron.

The catalog groups chrome recipes as Layouts, Overlays, and Screens
(replacing Patterns). Guide reference pages match those groups.

The spinner is eight dots around a circle. `chip` takes optional press
and dismiss. `Selectables::get` is `Option`; unbound `perform` is a
no-op. `status_bar` takes an optional tone and caption.
`markdown_view` keeps structured markdown layout with paint-side
select within each block. Code and fields stay select-only editors
with clean multi-line highlight. Contract: `select` module. Gallery
content pages always demo those constructors (no paint-only toggle).
The List page pages filtered rows and isolates selection from
list-detail and the table demo.

## 0.4.0 — 2026-08-11

`pattern::workspace` calls `pane` with each leaf id. `list_view`
takes `RowHeights` so rows can be `visible_range_var` heights, and
`RowFace::Card` for a wrapped title on a surface. `data_table` keeps
`ColumnLayout.frozen` leading columns in view; `on_h_scroll` moves
the rest. `command_palette_view` paints a parameter field when
`CommandPalette::ask` is set. `window::place_pinned` clamps an
overlay onto a chosen display. `daemon!` starts `iced::daemon` with
the same `Prepared` settings; `Prepared::open` maps a window and
`Prepared::open_desktop` maps a decorated pop-out. `expander` title
and body share the 12px card inset. `selectable` is body text the
user can drag-select and copy. `field::Selectables` binds those
buffers by id; `value_field` is the labeled row. `highlighted_code`
and `code_block` use the same select-and-copy contract. Markdown
copy posts `MarkdownDoc.source`.

## 0.3.0 — 2026-08-11

The first window is a Save tool (`examples/hello.rs`). Crate-root
rustdoc walks compose, boot, keys, tokens, a widget, a pattern, and
scope. The guide has a four-job cookbook. Constructor rustdoc names
the message. The reference lists every public constructor by group.

`pattern::workspace` paints a `DockNode` (splits, sash, tabs).
`center` fills the first leaf. `move_panel` takes a leaf out of a
split by collapsing onto the sibling. `data_table` follows
`ColumnLayout` order. Empty-query palette lists favorites, then
recent. `context_menu` clamps to the card size. `DropAccept::Text`
is text only. `widget::expander` is a card that clips its child
until opened. Closed peek is pixels or whole body lines, and the
cut fades into the card. `pattern::command_bar` is the dense action
row (ghost, meta type, no panel) with a light leading rail.
Gallery demos handle the messages their widgets emit.

## 0.2.0 — 2026-08-11

Widgets and chrome for iced 0.14 desktop applications.
`icedtea::run!` boots theme and starts the window. Constructors
return `Element`s and emit the application's messages.

- One `Action` for menus, toolbars, shortcuts, the command palette, and
  footer hints. `ctrl+s` is Command on macOS and Control on Linux and
  Windows. F1-F24 parse and press.
- Layout: dock, split, pad, form, overlay. Split sash drag uses
  window-space pointer events.
- Semantic tokens and `theme::mix`. Named colorways, high-contrast,
  light/dark families. Follow-OS can take the desktop accent.
- Application, dialog, and overlay windows. Overlay placement uses the
  display under the pointer.
- Every public widget takes `A11y`. Lists, tables, and logs virtualize.
- Image slots keep their box. Item-grid tiles share the row. An open
  accordion shows a body under its header.
- One constructor per catalog id. That function takes `A11y` and
  tokens. Rustdoc on the function is the intended call.
  `image_slot`, `key::listen`, `themed_scroll`, and
  `Breakpoint::from_width` are those jobs.
- Gallery pages host every `catalog::ENTRIES` id. Related controls
  share a page. `just gallery-gif` records the README tour. Guide:
  <https://indynull.github.io/icedtea/>. API docs:
  <https://docs.rs/icedtea>.

## 0.1.0

crates.io publish check. Tag and package path for `icedtea` 0.1.
