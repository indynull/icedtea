# Changelog

## Unreleased

Material Design 3 foundations live under `m3`: color `Scheme` roles
(light and dark baselines), type scale, shape, elevation, density on a
4dp grid, and `ControlState`. `Tokens` seed from those schemes;
`Tokens::scheme()` exposes the role map. Light and dark colorways use
the M3 baselines; hover, pressed, and selection layers use M3 state
mixes. The guide has an M3 foundations chapter.

Catalog surfaces without an M3 counterpart are gone from the public
catalog and gallery: mask, color, sparkline, display, rich-cell,
document-tabs, pad, callout, group-box, skeleton, teaching-tip, and
jobs. Apps that used those constructors need replacements or own
widgets.

## 0.4.0 — 2026-08-12

Workspace panes, virtual collections, and multi-window chrome.

`pattern::workspace` calls `pane` with each leaf id. `list_view` takes
`RowHeights` (`visible_range_var` heights) and `RowFace::Card` for a
wrapped title on a surface. `widget::virtual_column` virtualizes
app-built rows with known heights; `collection::expand_card_heights`
builds the closed/open slice (same rail and overscan as list). Open
faces clamp to their row height. `data_table` keeps
`ColumnLayout.frozen` leading columns in view; `on_h_scroll` moves the
rest. Lists and tables paint through `ClipLayer` so virtual panes do
not bleed. `collection::page_range` / `page_count` page
application-owned sets. `list_detail` pads the list rail and detail
inset. `command_palette_view` paints a parameter field when
`CommandPalette::ask` is set. `window::place_pinned` clamps an overlay
onto a chosen display. `daemon!` starts `iced::daemon` with the same
`Prepared` settings; `Prepared::open` and `Prepared::open_desktop` map
windows. `expander` title and body share the 12px card inset.
`pattern::modal_card` takes tokens and paints a black dim wash over
the scene.

Select-and-copy for body content.

`selectable` is body text the user can drag-select and copy on a
transparent field. `field::Selectables` binds buffers by id (`get` is
`Option`; unbound `perform` is a no-op; `ensure` / `retain` / `unbind`
for lazy open cards). `value_field` is the labeled row with a fixed
label gutter (`label_width`; pass `layout::FORM_LABEL` for the same
140px column as `layout::form`). `highlighted_code` and `code_block`
share the contract. `markdown_view` keeps structured layout with
paint-side select per block; full document copy posts
`MarkdownDoc.source`. Contract: `select` module.
`key::WhileInput::Chrome` / `KeyContext::chrome_over_input` match
Escape, Enter, and F1–F24 while a field is focused (modifier chords
already did).

Controls, theme, and host faces.

`themed_slider` steps continuous ranges (~100 positions) so a `0..=1`
drag is not stuck on the endpoints. `progress` fills its host at 8px
girth; `progress_ring` track mixes text into surface. `split_button`
takes overflow rows from a chevron menu. Chrome icons use solid black
fills so iced's svg recolor tints them on macOS Metal as well as
Linux and Windows. `light` and `dark` are a neutral desktop pair.
Persist defaults `follow_os` on. When follow-OS is on, `OsChrome` /
`os_chrome` / `listen_os_chrome` / `apply_os_chrome` layer optional
desktop colors onto the active colorway: accent as `primary` on every
host; `canvas`, `surface`, `panel`, `text`, `muted`, and `border` on
macOS and Windows; Linux (settings portal) supplies accent only.
Unset fields and follow-OS off leave the colorway unchanged. Success,
warning, and danger stay on the colorway. `run!` and `daemon!` call
`typo::install_platform_faces` so SansSerif and Monospace bind to
installed faces (UI needs normal and bold weight 700). Apps that
start iced without those macros call it before the first frame. The
spinner is eight dots around a circle. `chip` takes optional press
and dismiss. `status_bar` takes an optional tone and caption. Gallery
list demos search, Unread/Flagged buckets, and pagination over a
large seed with selection isolated from list-detail.

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
