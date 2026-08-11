# Chrome

Theme, marks, rows, and feedback.
[rustdoc](https://docs.rs/icedtea) ·
[source](https://github.com/indynull/icedtea) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

### Theme

**`theme`** — Look up a built-in colorway by name.

Constructor: [`theme::named`](https://docs.rs/icedtea/latest/icedtea/theme/fn.named.html)

[source](https://github.com/indynull/icedtea/blob/master/src/theme.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Unknown names resolve to `dark`. Register more on `ThemeCatalog`.
See [Theming](../theming.md).

### Colors

**`colors`** — Blend two colors for washes.

Constructor: [`theme::mix`](https://docs.rs/icedtea/latest/icedtea/theme/fn.mix.html)

[source](https://github.com/indynull/icedtea/blob/master/src/theme.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`amount` 0 is the background, 1 is the foreground. Hover, pressed,
and selection washes use this.

### Keys

**`keys`** — Resolve a key event against the action table.

Constructor: [`key::handle`](https://docs.rs/icedtea/latest/icedtea/key/fn.handle.html)

[source](https://github.com/indynull/icedtea/blob/master/src/key.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Subscribe with [`key::listen`](https://docs.rs/icedtea/latest/icedtea/key/fn.listen.html).
An open modal consumes. Focused text owns unmodified typing.
Otherwise chords match the table. See [Actions](../actions.md).

### Cheatsheet

**`cheatsheet`** — A searchable shortcut list from the action table.

Constructor: [`pattern::cheatsheet`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.cheatsheet.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Empty query lists every enabled action. Disabled actions stay out.

### Card

**`card`** — A titled panel around children.

Constructor: [`widget::group_box`](https://docs.rs/icedtea/latest/icedtea/widget/fn.group_box.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Same constructor as group-box. Empty title is a border only.

### Rule

**`rule`** — A horizontal divider.

Constructor: [`widget::rule_h`](https://docs.rs/icedtea/latest/icedtea/widget/fn.rule_h.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`rule_v` is the vertical twin.

### Chip

**`chip`** — A compact labeled pill.

Constructor: [`widget::chip`](https://docs.rs/icedtea/latest/icedtea/widget/fn.chip.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Optional press message. Disabled keeps the face.

### Badge

**`badge`** — A count or status mark.

Constructor: [`widget::badge`](https://docs.rs/icedtea/latest/icedtea/widget/fn.badge.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Short text. Empty string is an empty pill.

### Wrap

**`wrap`** — Flow children to the next line.

Constructor: [`layout::wrap`](https://docs.rs/icedtea/latest/icedtea/layout/fn.wrap.html)

[source](https://github.com/indynull/icedtea/blob/master/src/layout/recipes.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Pass child width, gap, and available width. Empty children yield an
empty column.

### Pad

**`pad`** — Equal-fill tiles.

Constructor: [`layout::pad`](https://docs.rs/icedtea/latest/icedtea/layout/fn.pad.html)

[source](https://github.com/indynull/icedtea/blob/master/src/layout/recipes.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Pair with `themed_button_sized` for a key pad. `columns` is the row
length.

### Callout

**`callout`** — An inline info bar.

Constructor: [`widget::info_bar`](https://docs.rs/icedtea/latest/icedtea/widget/fn.info_bar.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Tone comes from `Variant`. Empty body is title only.

### Banner

**`banner`** — A page-level message with an optional action.

Constructor: [`widget::banner`](https://docs.rs/icedtea/latest/icedtea/widget/fn.banner.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Use for “offline” or “update available”. Optional button message.

### Group box

**`group-box`** — A titled panel around children.

Constructor: [`widget::group_box`](https://docs.rs/icedtea/latest/icedtea/widget/fn.group_box.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Same constructor as card. Prefer this name for form sections.

### Skeleton

**`skeleton`** — A placeholder block while content loads.

Constructor: [`widget::placeholder_skeleton`](https://docs.rs/icedtea/latest/icedtea/widget/fn.placeholder_skeleton.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Size the box. No message.

### Teaching tip

**`teaching-tip`** — A one-shot hint next to a control.

Constructor: [`widget::teaching_tip`](https://docs.rs/icedtea/latest/icedtea/widget/fn.teaching_tip.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

The application owns dismissed state. Empty body hides the tip.

### Command bar

**`command-bar`** — The toolbar row, denser.

Constructor: [`pattern::command_bar`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.command_bar.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Same `Action` iterator as `toolbar`. Disabled actions paint muted.

### Context menu

**`context-menu`** — Action list under the pointer.

Constructor: [`pattern::context_menu`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.context_menu.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Stack on the window with the click point. Click-away dismisses.
Empty table still paints a card.

### Breadcrumb

**`breadcrumb`** — A path of links.

Constructor: [`widget::breadcrumb`](https://docs.rs/icedtea/latest/icedtea/widget/fn.breadcrumb.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Crumbs before the last send a message. The last crumb is the current
page. Empty path is empty.

### Menu

**`menu`** — An in-window menu bar from one action table.

Constructor: [`pattern::menu_bar`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.menu_bar.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Groups by the id prefix before `.` (`file.save` → File). Disabled
actions stay out of the pick list.

### Toolbar

**`toolbar`** — A row of action buttons.

Constructor: [`pattern::toolbar`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.toolbar.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

[First window](../first-window.md) uses this with `count.inc`.
Direction comes from `Boot`.

### Status bar

**`status-bar`** — Footer text plus shortcut hints from the table.

Constructor: [`pattern::status_bar`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.status_bar.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Left is status copy. Right is `table.footer_hints()`.

### Jobs

**`jobs`** — Progress rows for background work.

Constructor: [`pattern::job_strip`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.job_strip.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

The application owns job titles and fractions. Empty strip hides.

### Scrollbar

**`scrollbar`** — A themed scroller with a usable handle.

Constructor: [`widget::themed_scroll`](https://docs.rs/icedtea/latest/icedtea/widget/fn.themed_scroll.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Lists and tables use a 24px rail. This constructor is the themed
iced scroller for other panes.

### Toast

**`toast`** — A transient notice.

Constructor: [`widget::toast_view`](https://docs.rs/icedtea/latest/icedtea/widget/fn.toast_view.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

The application owns the `Toast` queue and dismiss. Empty queue
paints nothing.

### Busy overlay

**`busy`** — Dim plus spinner over a child.

Constructor: [`widget::busy_overlay`](https://docs.rs/icedtea/latest/icedtea/widget/fn.busy_overlay.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

When `busy` is false the child is unmodified. Advance spinner
`phase` while true.
