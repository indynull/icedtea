# TODO

## Do

### Blocked on iced

- RTL `textarea`: iced 0.14 `text_editor` has no writing direction
  (caret, hit-test, and line origin stay LTR). Do **not** shrink-wrap
  or pin a measured width — that growing right-hand slab puts the
  caret ahead of Urdu/Arabic and splits a new line to the opposite
  edge. Keep a stable Fill field. Replace the editor when iced 0.15
  lands [iced#3294](https://github.com/iced-rs/iced/pull/3294)
  (fixes [iced#2102](https://github.com/iced-rs/iced/issues/2102),
  [iced#1877](https://github.com/iced-rs/iced/issues/1877),
  [iced#1454](https://github.com/iced-rs/iced/issues/1454)).
- Field highlighter bidi: `HighlightField` packs one left-to-right
  paragraph (caret, scroll, span bounds). Arabic and Urdu queries
  do not follow writing direction. Stay on this path until iced
  styles the input paragraph. Do not treat English SQL highlighting
  or the empty-search clear mark as this item.

### 0.16.0 — Focus, keys, and pointer

Theme: a window that uses icedtea constructors is usable from the
keyboard and the pointer without an application `update` arm for
Tab, arrows, page, Home, End, Space, Enter, Escape, click-to-focus,
or footer activate. One `ActionTable`. Constructors own local keys.
`key::handle` matches the table after the focused target consumes
its keys. No per-widget BINDINGS class.

Contract (write into `key` rustdoc, `actions.md`, and AGENTS when
this ships):

- A **focus target** owns unmodified arrows, Page, Home, End, Space,
  and Enter. Click focuses that target. Tab / Shift+Tab walk targets.
  The first frame focuses the first target. `Boot` names an `Id` when
  the product wants a specific field (palette query).
- Tab always cycles focus targets. `form_group` and an open overlay
  cycle their own set. An application that used Tab to switch panes
  binds a different chord on the table.
- Open menus, picks, context menus, palettes, dialogs, and sheets
  close on Escape themselves. `should_hide` stays the overlay-window
  policy. Applications keep a layered dismiss *above* that (help,
  detail, compose, parent).
- iced 0.14 `text_input` captures Escape: constructors that put a
  field in an overlay forward it. `WhileInput::Chrome` includes Tab.
- Footer hints invoke the same `Action`. `WindowKind::Application`
  seeds `app.quit` (`ctrl` = host accelerator). Overlay windows do
  not; Escape hides.
- `item_press` activates on click (press and release on the same
  hit). Secondary click still reports immediately for context.
- Keyboard-complete means the gallery page proves the keys, not
  that a helper exists (`Press::step_index`, `focus::rove`).

Ship in this order. Each step is one story; recapture stills when
paint or chrome in a published still changes.

1. **Focus target.** Grow `focus` from `rove` / `spatial_next` /
   `trap_escape` into a named target the constructor or application
   can set. Visible ring uses the same 2 dp primary frame
   `form_group` already paints (a face or helper, not a new catalog
   id). Click-to-focus on `list_view`, `virtual_column`, `data_table`,
   `item_grid`, `tree_view`, `tab_bar`, accordion headers, nav rail,
   and a focused `scroll`. Empty or disabled targets do not take Tab.

2. **Tab cycle and first frame.** Window-wide Tab / Shift+Tab.
   `form_group` stays the mixed-field specialist (Space on a non-text
   row). First frame focuses the first target; `Boot` can name an
   `Id`. `hello.rs` focuses the editor. Palette / search-view name
   the query field. `run!` subscribes `key::listen` so hello does
   not.

3. **Collections and scroll.** Focused list / table / grid / tree /
   log: arrows and Page move the primary, Home / End the ends,
   `scroll_to_show` keeps the row in view, Enter activates, Space
   toggles a check. Tree Left / Right collapse and expand. Focused
   `scroll` (markdown, code, dialog body, log with no selection)
   moves the offset only. `Press::step_index` / `step_cell` stay the
   math. Search-view / suggest / palette: arrows from the query
   field move the hit list (the Spotlight case).

4. **Overlays dismiss.** `context_menu`, `dialog_sheet`, side sheet,
   `command_palette_view`, and any constructor that already takes
   `on_dismiss` / cancel close on Escape. Menubar / `drop_menu` /
   `split_more` already do. Click-out stays. Forward captured Escape
   from a field inside the overlay.

5. **Menus complete.** Open menu, pick overlay, context, sectioned,
   and cascade: arrows move the hover, Enter picks, first-letter
   jumps. Closed `pick_list` Space / Enter opens (same as
   `form_group` Space).

6. **Controls pass.** Prove or own keyboard on slider / range
   (`slider_nudge`), checkbox / radio / switch / toggle, segmented,
   tabs, breadcrumb, number / date / time steppers, accordion /
   expander, split sash (arrow nudge while the grip is the target).
   Missing iced behavior is owned in the constructor, not documented
   as "the application should."

7. **Chrome.** `status_bar` hints invoke. Seed `app.quit` on
   application windows. Toolbar / command-bar items join the Tab
   ring. Workspace / drawer / tab-view: focus can enter the rail
   and arrow between items; drawer Escape closes.

8. **Pointer.** `item_press` click. `ListOpts` takes a context
   callback; secondary click opens `context_menu`. Double-click
   activate is the list-detail recipe, not a second catalog widget.

9. **First path and proof.** `run!` listens. `hello.rs` and First
   window: handle, focused editor, Save, clickable footer, quit.
   Keys gallery page is the live proof (Tab, list arrows, footer
   click, Escape). Book `actions.md` / `architecture.md` /
   `overlay-windows.md` teach the contract. Recapture handbook
   stills. Unreleased changelog: one public thing per bullet.

## Bugs

## Consider

- Typeahead jump in long lists and trees (beyond first-letter menus).
- `spatial_next` when Tab leaves a pane (today it is a helper).
- Horizontal keyboard scroll (`ctrl+Page`) on tables with an
  unfrozen strip.
- Hover-open menubar titles (desktop convention). Click-open stays
  the default.

## Discard

Not library API. Applications own these, or they contradict icedtea
Non-goals:

- A per-widget BINDINGS list that bubbles beside `ActionTable`.
  Grow the one table and the constructors that consume it.
- Library-owned layered dismiss (help → detail → compose → parent
  → hide). `should_hide` is the overlay-window policy only.
- Forcing first-widget focus when `Boot` names an `Id` or none
  (summon search, empty chrome).
- Language-service hooks, editors' language servers.
- Timeline, audio, or video engines; CAD kernels; live telemetry daemons.
  Applications own decode and paint (`iced_video_player`, shader blit).
  `image_slot` is stills.
- In-process web view (WebKitGTK, Servo, CEF, WebView2). Applications
  host the engine as a custom iced element.
- Document undo/redo.
- Multiple-document-interface window mosaics.
- A second collection widget for variable-height cards.
- System-wide hotkeys or host focus steal.
- A stylesheet, markup language, or second renderer.
- Threading views as a mail-specific widget (compose from list + detail).
- Offline indicators as a domain widget (status/toasts cover the chrome).
- FAB, extended FAB, FAB menu, bottom navigation, pull-to-refresh.
- Carousel and marketing hero sheets.
- Snackbar as a second path next to `toast`.
- Bottom sheets (mobile).
- Layer-shell via a Smithay client loop in the widget crate. Window
  roles stay iced `window::Settings`. An exclusive-zone panel binds
  `zwlr_layer_shell_v1` in the application until iced can open that
  surface. A later `host*` opener is only if iced still cannot and a
  second app needs the role.
- A foreign Wayland client surface hosted inside a pane. The compositor
  maps that surface; icedtea paints the chrome around it.
- Domain columns, permission cards, mail list/read/compose, or
  host-file chips as catalog widgets. Applications compose those from
  `chip`, field, table, dialog, list, and textarea.
- Loading indicator as M3 Expressive shape-morph.
- Floating toolbar as a pill / expressive float.
- Compact `tab_bar` as a second size. In-pane exclusive tabs are
  `segmented_button`.
