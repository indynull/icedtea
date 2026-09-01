# Changelog

## Unreleased

### Feature

- `focus::target` is a click-to-focus wrap with a 2 dp primary ring.
  Empty and disabled collections are not focusable.
- `focus::cycle` walks Tab / Shift+Tab across targets and focuses the
  first target (or a named iced id) on the first frame.
- `run!` and `daemon!` subscribe `key::listen`. The application
  message implements `From<keyboard::Event>`.
- A focused `list_view` moves the primary row with arrows, Page, Home,
  and End.
- A focused `scroll` pane moves the offset with arrows, Page, Home,
  and End.
- `context_menu` and `dialog_sheet` (when Cancel is set) close on
  Escape.
- `status_bar` hint chords invoke the same `Action`.
- Focused slider, checkbox, switch, radio, toggle, segmented, tabs,
  breadcrumb, number, date, time, accordion, expander, pick list,
  button group, pagination, and split sash handle arrows, Enter, and
  Space on the constructor.
- `item_press` sends a primary click on release while the pointer is
  still over the row. `ListOpts.on_context` is the secondary click.
- A focused `tree_view` expands and collapses with arrows. Arrows in
  `search_view` move the hit list.
- Closed `pick_list` Enter and Space open the overlay.
- `nav_rail` arrows move the selected dest. An open `drawer` closes
  on Escape. `ActionTable::seed_quit` inserts `app.quit`.

## 0.15.0 — 2026-08-31

### Feature

- `status_bar` paints each enabled shortcut as two faces: the chord in
  `Tokens.text`, the title in `Tokens.muted` at meta size.
- `layout::pack` measures each child and allocates leftover on a row
  or column (`Slot::hug` / `Slot::share`, `Pack` start / end / center
  / between, `Cross` stretch / start / center / end). `BoxOpts` is the
  extras on that call.
- `layout::wrap` measures unequal children and starts a new line when
  the next does not fit. Share slots with a min width reflow a tile
  wall. Window direction puts the first child on the start edge.
  Callers pass slots, not a child width or the parent width.
- `layout::allocate` shares leftover across `SizePolicy` items.
- `layout::wrap_per_row` and `layout::wrap_rows` are gone.
- `layout::row_box` and `layout::column_box` are gone.
- `layout::form_columns` is gone.
- `layout::distribute` is gone.

### Bug fix

- `pick_list` paints its overlay list with `overlay_menu_style`. Menu
  stays extra-small under Soft and Pill so selected rows keep a 4 dp
  box.
- `layout::form` packs a `FORM_LABEL` gutter and gives leftover to the
  field. The label stays the first child on the start edge.
- `value_field` packs the label gutter, a sharing value, and an
  optional hug Copy.
- `secret_field` packs a sharing field and hug Reveal / Copy.

### Chore

- Workspace `debug` is `line-tables-only`. `cargo build --profile
  debugging` is full debug. Release is thin LTO, stripped, eight
  codegen units.

## 0.14.0 — 2026-08-23

### Feature

- Constructors use the job name (`button`, `checkbox`, `switch`,
  `radio`, `slider`, `text_input`, `pick_list`, `scroll`). Search
  clear is `on_clear` on `search_input`. Date is `date_stepper`.
  Width and height sit on `ButtonOpts`. List extras sit on
  `ListOpts`.
- `FieldOpts.highlight` and `search_input` run a syntax highlighter
  on the typed value (`FieldRun` plus `FieldInk` roles).
  `FieldInk::Error` keeps body ink and underlines in the danger
  role.
- Gallery look strip offers Hebrew. Clocks stay on Western digits.
- `Boot::transparent` sets an ARGB window buffer.
- `Boot::decorations` turns the client title bar off on an application
  window.
- `group_box` takes a header trailing child (`None` when the header
  is title only).
- `workspace` takes a header callback for leaf trailing chrome
  (`|_| None` when the leaf has none).
- `CardFace::Rail` paints an inset start rail and a label gutter.
- `daemon!` theme function receives the window id so an overlay
  canvas can differ from a desktop window.

### Bug fix

- `dialog_sheet` sits extra, cancel, then confirm toward the end, so
  Save is on the left of a right-to-left card.
- Marks document and empty cards follow window direction.
- `search_input` paints the clear mark only when `on_clear` is set
  and the value is non-empty.
- Elevation Level 1 is a visible drop on dark Desktop.
- Elevated buttons raise one elevation level on hover.
- Elevation resting heights match the Material table (0 / 1 / 3 / 6 /
  8 / 12 dp).
- The Material shape scale includes Large increased (20 dp), Extra
  large increased (32 dp), and Extra extra large (48 dp).
- Gallery type sample uses the locale Code page name (Vietnamese
  `Mã`), not leftover English `Code`.
- `slider` fills from the start edge. Min sits on start, max on end.
- `ClockDigits::map_str` maps `%` to the Arabic percent sign so
  `40%` paints as `٤٠٪`.
- `progress` and `progress_ring` fill from the start edge and sit
  percent copy there. `progress_label` maps digits with `ClockDigits`.
- `range_slider` maps its low/high label with `ClockDigits`.
- Gallery page titles, button rows, and the Controls pack follow
  window direction. Range slider sits with the slider on the first
  screen. Image slots, type labels, the workspace dock, markdown
  outline, tab body, and the side-sheet action follow start.
- The Chinese markdown sample document title is 标记.
- Gallery control, field, and keys columns start-align their
  captions, so a Fill sibling cannot leave the hint on physical left.
- `layout::form` sits the label gutter on the start edge.
- `text_input` and `search_input` sit placeholder, value, and
  leading icons on the start edge. The character count uses
  `ClockDigits`.
- `number_input` paints its value with `ClockDigits` and sits it on
  the start edge.
- RTL `textarea` stays a stable Fill field. iced 0.14 `text_editor`
  has no writing direction; shrink-wrapping the buffer put the caret
  ahead of Urdu (see iced#3294).
- `layout::wrap` takes window direction so a tile row starts at the
  start edge. Inspector properties and the gallery scroll demo
  start-align.
- `icon_svg` flips Chevron and Back on a right-to-left window.
- `ClockDigits::western_str` maps Eastern digits back to ASCII.
- `group_box` and `about_page` start-align their title and body.
  Credits wrap on the card so a long locale blurb stays inside.
- `status_page` centers a start-aligned empty block.
- `tab_bar` sits the first tab on the start edge and a closable
  tab's close mark on the end.
- `dialog_sheet` fills its caller width so long locale actions keep
  their labels.
- Clock faces pick Eastern or Western digits from the language, so
  Hebrew keeps 123 on a right-to-left window.
- `menu_bar` titles stay one line for Arabic and Hebrew headings.
- The gallery look strip sizes picks to the option so RTL labels
  keep their gap.
- Gallery list-detail, inspector, navigation, and workspace jobs
  name the pane, not a physical side.
- `side_sheet` docks on the trailing edge and slides from there, so
  an RTL end sheet sits on the left.
- Gallery grid inject note uses the locale Tile word plus mapped
  digits.

### Chore

- `just gallery-gif` writes `tmp/gallery.gif`. Persist into `assets/`
  and the handbook with `just gallery-gif persist` when tagging a
  version. The recording is a live pointer demo; Primary invokes an
  Action.
- `just material-snapshot` fetches Material spec pages for gallery QA.

## 0.13.0 — 2026-08-21

### Feature

- `data_table` takes `scroll_id` so `scroll_to` jumps the body clip.

### Bug fix

- `virtual_clip` keeps the pixel offset in widget state. Constructors
  use `VisibleWindow` `start..end` for the `view` row build. Jumps
  use `scroll_to`. A new list identity uses a new `scroll_id`.
- `virtual_clip` `on_scroll` fires when the mounted range changes or
  the offset arrives at 0 or max.
- A pixel-only wheel or rail move on `virtual_clip` redraws. Layout
  runs when the mounted range changes.
- A new cover or selection that sits outside the viewport moves the
  clip so that row is in view.

## 0.12.1 — 2026-08-20

### Bug fix

- `virtual_clip` keeps the pixel offset in widget state. Wheel and
  rail moves that do not change the mounted range redraw only.
  `on_scroll` fires when the range changes or the scroll hits an edge.
- An incoming `VisibleWindow` with a different mounted range (filter,
  page, session) still moves the clip. A stale pixel copy of the same
  range does not.

## 0.12.0 — 2026-08-19

### Feature

- `form_group` walks Tab and Shift+Tab among mixed fields and wraps.
  The first text field can take focus on mount. Space activates the
  focused checkbox, radio, pick, chips, or segmented row.
- `command_palette_view` takes `PaletteOpts` for group headings, row
  face, empty omit, panel size, highlight, and leading or trailing
  slots. The query field stays visible with a nested page or `ask`.
  0.11 callers pass `PaletteOpts::new()` as the new argument.
- `Action` carries `section`, `keywords`, and `children`. A 0.11
  `Action { ... }` literal must set those fields (or use `Action::new`).
- `Icon` adds the Material Sharp chrome set (window, edit, file, mail,
  media, view). Exhaustive `match` on `Icon` from 0.11 must cover the
  new variants. A product mark is still `Glyph::Bytes`.
- `CommandPalette` opens a child page and accepts a rank override.
- Command palette rows paint the action's icon, title, tooltip, and
  shortcut when those fields are set.
- Chrome icons and tree folder/file marks use Material Symbols Sharp
  (Apache 2.0). See `NOTICE`.
- `icon::adapt_material_svg` prepares a Material Symbols SVG for
  `icon_svg`. `icon::material_symbol_sharp_url` is the Sharp FILL 1
  URL the application downloads.

### Bug fix

- `themed_pick_list` paints Material `arrow_drop_down` on the end
  (24 dp, 20 dp Compact, `Density::inset`). A press on the mark opens
  the menu. Right-to-left puts the mark on the physical left.
- `form_group` leaves the label column blank when the row title is
  empty, so the generated a11y node id is not painted.
- Tab labels, banners, app bars, and exclusive segments stay
  rectangular under Tight, Soft, and Pill. Segment cells are not
  independent stadiums. Disclosure, menu, search, and suggest rows
  use the Menu family; list checks use Checkbox.
- Command palette Enter on the query invokes the highlighted row.
  Empty query lists favorites, then recent.
- `themed_scroll` uses iced's 60 px wheel line and keeps a short wheel
  gesture on the pane under the pointer.
- `item_grid` tiles hug control height instead of filling the pane.
- A tree trailing `RowSlot::Text` badge stays in its end slot; the
  title clips instead of painting over it.
- Dialog dim is a 64 percent scrim so the sheet reads over dark
  surface-container.

### Chore

- rustdoc fails the build on warnings.
- `cargo deny` checks advisories, yanked crates, and licenses.

## 0.11.0 — 2026-08-17

### Feature

- `list_view` takes per-row indent on `ListModel` and `RowSlot::Text`
  (small badge) on lead or trail.
- `expander` takes an optional trailing child on the title row.
  Right-to-left order is title, trail, then the mark.
- `Tokens::from_aliases` builds a scheme from the short color fields.
- `split_view` paints the 6 px sash as a hairline and a short centered
  handle. The constructor takes `Tokens`.
- `tree_view` paints an optional trailing `RowSlot` on `TreeNode`.
  `Text` is a badge.

### Bug fix

- `badge` uses `Component::Badge` so Tight, Soft, Pill, and Material
  corners apply.
- `toast` and tooltip use `Component::Toast` and `Component::Tooltip`
  (Material extra-small).
- `banner` uses `Component::Banner` (flush under Material and Pill).
- `search_input` uses `Component::Search` (Material extra-large, Pill
  full).
- Switch, slider, and progress tracks use `Component::Track` (Full
  under Material and Pill).
- `themed_scroll` ignores Press and a pointer cursor outside the pane.
  Keyboard still reaches the child. Move and Release continue after a
  press inside.
- `markdown_view` publishes Move (clamped to the document) and Release
  while a drag is active outside the pane.
- `themed_checkbox` with an empty label is the box only (shrink
  width).
- `accordion_view` start-aligns the open body so right-to-left
  copy sits on the start edge.
- `tree_view` paints the selected wash across the full row.

## 0.10.0 — 2026-08-16

### Feature

- `search_input` and `search_input_clear` take submit and an input id.
  Placeholder is the a11y name.
- `Tabs::with_disabled` freezes one tab. `tab_bar` skips its press.
- `segmented_button` takes `ControlSize`. Compact is the in-pane exclusive
  strip.
- `highlighted_code` takes wrap. `false` keeps each source line on one row.

### Bug fix

- `themed_text_input`, `search_input`, and `themed_pick_list` use
  `sized_control_height` so Default fields match Default picks under
  compact density.
- `chip_face` Success and Warning ink is the role color, lifted to 4.5:1
  on the wash.

## 0.9.0 — 2026-08-16

### Feature

- `tree_view` takes `TreeFace`: Outline is a tight heading tree, Files
  is an explorer (inset wash, folder and file marks).
- Constructors take pad and gap from `Tokens.density`. `ControlSize`
  Default follows that density. `context_card_size` takes `Density`.
- `Catalog::for_locale` fills English, Vietnamese, Japanese, Chinese,
  Arabic, and Urdu. `direction_for` sets Arabic and Urdu right-to-left.
- `Tokens` carries type scale, corner policy, and elevation
  (`with_font_scale`, `with_shape`, `with_elevation`). `Boot` and
  `UiState::look` apply the same fields.
- `A11y` keeps hint, selected, toggled, expanded, live, required, and
  error next to name, role, value, disabled, and checked.
- List, table, grid, and tree row presses are `ItemClick`.
  `Selection::apply_item_click` is the desktop rule. `listen_cursor`
  Context fires on right-click when an editor captured the press.
- Markdown drag-select uses X and Y. `markdown_select_all` selects
  every block. Copy is the span; Copy all is the source.
- `command_palette_view` takes the query placeholder. `tool_panel`
  takes the Dock label.
- `themed_pick_list` takes `ControlSize`. Compact uses tight pad and
  meta type.
- Chrome that takes `Icon` also accepts `Glyph::Bytes` (filled black
  SVG, token recolor).

### Bug fix

- The crate package is source, examples, icons, and the theme catalog
  (not the gallery tour GIF).
- Right-to-left chrome uses start/end: rails, splits, tree twisties,
  picks, button groups, inspector, drawer, and command bar.
- `themed_button`, `context_menu`, catalog nav, list cards, and table
  cells keep right-to-left titles (shrink text inside a fill pad).
- `time_picker` uses Eastern Arabic digits when direction is
  right-to-left. `secret_field` takes Show and Hide labels.
- `themed_scroll` scissors through `ClipLayer` so rows cannot paint
  through sticky Search.
- `markdown_view` paints the live `MarkdownSpan` on one document tree.
- `dim_backdrop` rest opacity is 50%. Disabled button ink uses a 68%
  mute.
- `motion::overlay` hit-tests the slid child. `context_menu` captures
  pointer moves on the dismiss layer.

### Chore

- README and Install use `cargo add icedtea`. The crates.io badge is
  the crate version.
- README shows a Codecov coverage badge.
- The guide cookbook walks a personal task list that writes a SQLite
  file (`examples/tasks.rs`).

## 0.8.0 — 2026-08-14

### Theme

- `m3::DurationStep` and `m3::Ease` are the Material motion tokens.
- `Tokens::with_reduced_motion` collapses every duration to 0 ms.
- `Tokens::fade` scales every scheme role's alpha.

### Chrome

- `motion::overlay` slides a child from a 0–1 progress. Build the child
  with `Tokens::fade` so fills and ink fade with the slide.
- `motion::expand` clips a child between a peek height and its open
  height.
- `motion::bounce_out` is an ease-out bounce curve.
- `motion::pulse` is a 0–1–0 attention curve.
- `motion::shake` is a decaying wiggle that starts and ends at rest.
- Toast enter and the last slice of TTL fade through `motion::overlay`.
- Side sheet enter starts toward the docked edge.
- A reduced-motion toast uses `DurationStep::duration` (0 ms) and
  appears at rest immediately.

### Patterns

- `modal_card` takes overlay `progress`.
- `side_sheet` takes overlay `progress`.
- `command_palette_view` takes overlay `progress`.
- `drawer` takes pane `progress` (width 0–220 dp).
- `context_menu` takes overlay `progress`.
- `cascade_menu` takes submenu `progress`.

### Collections

- `expander` takes height `progress`.
- `accordion_view` takes height `progress`.
- `tree_view` takes `animating` (`Option<(id, progress)>`) so a branch
  can grow and shrink.

### Readout

- `motion::value_animation` eases a determinate 0–1 value for
  `progress` and `progress_ring`.
- `motion::progress_run` is the linear busy-bar phase (grows, travels,
  shrinks). Reduced motion holds a static chunk.
- `progress` omits empty fill portions so 0% and 100% are a single
  track or a full bar.

### Controls

- Wheel over `themed_slider`, `range_slider`, `number_input`, and
  `themed_pick_list` steps the value.

### Guide

- Motion is the 0–1 progress path: overlay, expand, and sampled
  curves (`Ease`, `bounce_out`, `pulse`, `shake`).

## 0.7.0 — 2026-08-14

### Content

- `markdown_view` sizes body and headings from `typo` (H1 is the page
  title step).
- `highlighted_code` and `code_block` use `typo::CODE`.
- `selectable` uses `typo::CODE` for mono and `typo::BODY` for UI.
- Text fields and `textarea` set `typo::BODY` on the iced input.

### Patterns

- Dialog sheet title uses `typo::TITLE`.

## 0.6.3 — 2026-08-14

### Controls

- Selected and assist chips paint `primary` / `on_primary` so their ink
  differs from the idle outline.

## 0.6.2 — 2026-08-14

### Controls

- Selected and assist chips paint `primary` / `on_primary` so their ink
  differs from the idle outline.

## 0.6.1 — 2026-08-14

### Controls

- Selected and assist chips paint `primary` / `on_primary` so their ink
  differs from the idle outline.

## 0.6.0 — 2026-08-14

### Theme

- `light` and `dark` are the desktop pair. Follow-OS layers host chrome
  on that pair; a named colorway is a choice.

### Guide

- Architecture walks a multi-action window: Boot, tokens, ActionTable,
  constructors, patterns, and `layout` recipes.
- The handbook sidebar nests Start, Compose, Cookbook, and Reference
  as sections with indented pages.
- First-window and each visual reference group show a captured still
  of the shipped constructors.

### Controls

- `widget::search_view` docks application-filtered hits under a search field.
- `widget::button_group` is a strip of related actions (not exclusive).
- `Variant::Outlined` and `Variant::Elevated` complete the five Material button faces (filled / tonal / text already map to Primary / Quiet / Ghost).
- `Tokens::with_density` threads compact / default / comfortable pad and three distinct control heights.
- Labeled buttons, split, and toggle take `Icons` for leading and trailing chrome.
- `Cell` carries an optional icon on segmented button and button group.
- `icon_button` takes `ControlSize`. `icon_button_toggle` is the pressed icon face.

### Fields

- `FieldOpts` adds filled vs outlined, prefix/suffix icons, a floating label, and a character count.

### Readout

- `progress` takes `indeterminate` for a linear busy bar.
- `SliderMarks` adds `vertical` and a thumb value label. The vertical rail uses the same themed handle as the horizontal slider.

### Content

- `widget::tooltip_rich` is a hover title plus supporting copy.
- `chip` takes `ChipKind` (assist / filter / input / suggestion) and `Icons`.
- `badge` takes `BadgeSize` and an optional host to overlap.
- `group_box` takes `CardFace` (elevated / filled / outlined).
- Tooltips take `TooltipAnchor`. Rich tips take an optional action.

### Navigation

- `pattern::nav_rail` takes `expanded` so the compact letter face and the labeled 220px face share one constructor.
- `dialog_sheet` takes extra actions and an optional header icon.
- `tab_bar` takes a secondary underbar; `Tabs::with_icon` paints icon-plus-text.
- `RailDest` gives the navigation rail real icons and badges.

## 0.5.0 — 2026-08-13

### Foundations

- `m3` holds color `Scheme` roles, type scale, shape, elevation, density, and `ControlState`.
- Light and dark keep the M3 baselines.
- Catalog constructors paint through `Tokens::scheme()`.
- Desktop chrome is shape None (0 dp).
- Community colorways recompute solid-fill `on_*` roles (including `on_primary`) from text/canvas contrast.
- Inventory is `m3::mapping`.
- Surfaces without an M3 counterpart are gone: `sparkline`, `display_reading`, `rich_cell`, `masked_input`, `color_swatch`, `teaching_tip`, `placeholder_skeleton`, `document_tabs`, `job_strip`, and related types.

### Controls

- Segmented button, icon button, and range slider.
- Indeterminate checkbox (`CheckState`).
- Field supporting and error text.
- Filter chip set: outline idle, filled selected.
- Sectioned and cascade menus.
- Side sheet.
- Search optional clear (`search_input_clear`).
- Segmented and split buttons share the labeled button height (body line box plus pad).
- Tabs use a 3dp primary underbar.

### Collections

- `markdown_view` drag-selects across blocks via `select::markdown_select` and `MarkdownSpan` (layout stays structured; covered blocks paint the span wash).
- Navigation rail is `pattern::nav_rail`.
- Lists take leading and trailing `RowSlot`s.
- Sliders take `SliderMarks`.
- Tabs take badges and overflow width.
- Progress takes a buffer fill and labeled remaining.
- Tables paint a checkbox column from `TableSource::row_checked`; sort permutes checks with the rows.
- Table selection is secondary container, separate from zebra stripes.

## 0.4.0 — 2026-08-12

### Workspace and collections

- `pattern::workspace` calls `pane` with each leaf id.
- `list_view` takes `RowHeights` (`visible_range_var` heights) and `RowFace::Card` for a wrapped title on a surface.
- `widget::virtual_column` virtualizes app-built rows with known heights.
- `collection::expand_card_heights` builds the closed/open slice (same rail and overscan as list).
- Open faces clamp to their row height.
- `data_table` keeps `ColumnLayout.frozen` leading columns in view; `on_h_scroll` moves the rest.
- Lists and tables paint through `ClipLayer` so virtual panes do not bleed.
- `collection::page_range` / `page_count` page application-owned sets.
- `list_detail` pads the list rail and detail inset.
- `command_palette_view` paints a parameter field when `CommandPalette::ask` is set.
- `window::place_pinned` clamps an overlay onto a chosen display.
- `daemon!` starts `iced::daemon` with the same `Prepared` settings.
- `Prepared::open` and `Prepared::open_desktop` map windows.
- `expander` title and body share the 12px card inset.
- `pattern::modal_card` takes tokens and paints a black dim wash over the scene.

### Select and copy

- `selectable` is body text the user can drag-select and copy on a transparent field.
- `field::Selectables` binds buffers by id (`get` is `Option`; unbound `perform` is a no-op; `ensure` / `retain` / `unbind` for lazy open cards).
- `value_field` is the labeled row with a fixed label gutter (`label_width`; pass `layout::FORM_LABEL` for the same 140px column as `layout::form`).
- `highlighted_code` and `code_block` share the contract.
- `markdown_view` keeps structured layout with paint-side select per block; full document copy posts `MarkdownDoc.source`. Contract: `select` module.
- `key::WhileInput::Chrome` / `KeyContext::chrome_over_input` match Escape, Enter, and F1–F24 while a field is focused (modifier chords already did).

### Controls and theme

- `themed_slider` steps continuous ranges (~100 positions) so a `0..=1` drag is not stuck on the endpoints.
- `progress` fills its host at 8px girth; `progress_ring` uses the scheme track under a primary arc.
- `split_button` takes overflow rows from a chevron menu.
- Chrome icons use solid black fills so iced's svg recolor tints them on macOS Metal as well as Linux and Windows.
- `light` and `dark` are a neutral desktop pair.
- Persist defaults `follow_os` on.
- When follow-OS is on, `OsChrome` / `os_chrome` / `listen_os_chrome` / `apply_os_chrome` layer optional desktop colors onto the active colorway: accent as `primary` on every host; `canvas`, `surface`, `panel`, `text`, `muted`, and `border` on macOS and Windows; Linux (settings portal) supplies accent only.
- Unset fields and follow-OS off leave the colorway unchanged. Success, warning, and danger stay on the colorway.
- `run!` and `daemon!` call `typo::install_platform_faces` so SansSerif and Monospace bind to installed faces (UI needs normal and bold weight 700). Apps that start iced without those macros call it before the first frame.
- The spinner is eight dots around a circle.
- `chip` takes optional press and dismiss.
- `status_bar` takes an optional tone and caption.
- Gallery list demos search, Unread/Flagged buckets, and pagination over a large seed with selection isolated from list-detail.

## 0.3.0 — 2026-08-11

### First path and docs

- The first window is a Save tool (`examples/hello.rs`).
- Crate-root rustdoc walks compose, boot, keys, tokens, a widget, a pattern, and scope.
- The guide has a four-job cookbook.
- Constructor rustdoc names the message.
- The reference lists every public constructor by group.

### Workspace and widgets

- `pattern::workspace` paints a `DockNode` (splits, sash, tabs).
- `center` fills the first leaf.
- `move_panel` takes a leaf out of a split by collapsing onto the sibling.
- `data_table` follows `ColumnLayout` order.
- Empty-query palette lists favorites, then recent.
- `context_menu` clamps to the card size.
- `DropAccept::Text` is text only.
- `widget::expander` is a card that clips its child until opened. Closed peek is pixels or whole body lines, and the cut fades into the card.
- `pattern::command_bar` is the dense action row (ghost, meta type, no panel) with a light leading rail.
- Gallery demos handle the messages their widgets emit.

## 0.2.0 — 2026-08-11

### Boot and actions

- Widgets and chrome for iced 0.14 desktop applications.
- `icedtea::run!` boots theme and starts the window. Constructors return `Element`s and emit the application's messages.
- One `Action` for menus, toolbars, shortcuts, the command palette, and footer hints. `ctrl+s` is Command on macOS and Control on Linux and Windows. F1-F24 parse and press.

### Layout and theme

- Layout: dock, split, pad, form, overlay. Split sash drag uses window-space pointer events.
- Semantic tokens and `theme::mix`. Named colorways, high-contrast, light/dark families. Follow-OS can take the desktop accent.
- Application, dialog, and overlay windows. Overlay placement uses the display under the pointer.

### Widgets

- Every public widget takes `A11y`. Lists, tables, and logs virtualize.
- Image slots keep their box. Item-grid tiles share the row. An open accordion shows a body under its header.

### Catalog and docs

- One constructor per catalog id. That function takes `A11y` and tokens. Rustdoc on the function is the intended call. `image_slot`, `key::listen`, `themed_scroll`, and `Breakpoint::from_width` are those jobs.
- Gallery pages host every `catalog::ENTRIES` id. Related controls share a page. `just gallery-gif` records the README tour. Guide: <https://indynull.github.io/icedtea/>. API docs: <https://docs.rs/icedtea>.

## 0.1.0

- crates.io publish check. Tag and package path for `icedtea` 0.1.
