# Chrome

Generated from `catalog::ENTRIES`, constructor rustdoc, and
`m3::mapping`. Compose starts at `examples/hello.rs` (see
[llms.txt](llms.txt)).

## cheatsheet

A searchable shortcut list from the action table.

Empty query lists every enabled action. Disabled actions stay out.

Constructor: `pattern::cheatsheet`
Desktop: App bars (shortcuts)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.cheatsheet.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## pack

Measuring row or column. Children hug or share leftover; `Pack` places what stretch does not take. Empty slots yield an empty box.

Call this for a chrome strip (search that grows between two marks) or a caption above a filling editor. Disabled / empty: no slots is a zero-size box. Direction only mirrors a horizontal box.

Constructor: `layout::pack`
Desktop: Layout
[rustdoc](https://docs.rs/icedtea/latest/icedtea/layout/fn.pack.html) · [source](https://github.com/indynull/icedtea/blob/main/src/layout/flow.rs)

## wrap

Measuring wrap. Each child is measured; a new line starts when the next child does not fit. Unequal children are allowed. Window direction puts the first child on the start edge.

Pass slots, not a uniform child width or the parent width. Empty slots yield an empty box. Share slots on a line take leftover after hug siblings, so a tile wall reflows when the parent crosses a column count.

Constructor: `layout::wrap`
Desktop: Layout
[rustdoc](https://docs.rs/icedtea/latest/icedtea/layout/fn.wrap.html) · [source](https://github.com/indynull/icedtea/blob/main/src/layout/flow.rs)

## filter-chips

Multi-select filter chips (M3 filter chip set).

The application owns which indices are on. Press toggles one index.

Constructor: `widget::filter_chips`
Material: Chip (filter set)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.filter_chips.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## chip

A compact labeled pill.

Optional press and optional dismiss. META type, chip wash, shrink width. Disabled keeps the face and drops press.

Constructor: `widget::chip`
Material: Chip
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.chip.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## badge

A count or status mark.

Short text. Empty string is an empty mark. Both sizes use meta type; Large is not body reading type. Corners follow `Tokens::shape` (`crate::m3::shape::Component::Badge`).

Constructor: `widget::badge`
Material: Badge
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.badge.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## card

A titled panel around children.

Empty title is a border only. Same constructor paints a card. `trailing` sits on the header end (kind badge, close). `CardFace::Rail` adds an inset start rail and a label gutter.

Constructor: `widget::group_box`
Material: Card
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.group_box.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## rule

A horizontal divider.

`rule_v` is the vertical twin.

Constructor: `widget::rule_h`
Material: Divider
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.rule_h.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## banner

A page-level message with an optional action.

Use for “offline” or “update available”. Optional button message. `tone` paints a callout wash (`ToastKind`); `None` is the default banner face. Corners follow `Tokens::shape` (`crate::m3::shape::Component::Banner`).

Constructor: `widget::banner`
Material: Banner
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.banner.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## command-bar

The toolbar row, denser.

Same `Action` iterator as `toolbar`. Ghost, meta type, no panel. A light rail marks the group off the rest of the card. For a card footer or a tight chrome strip.

Constructor: `pattern::command_bar`
Desktop: App bars (desktop)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.command_bar.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## context-menu

Place a context menu at `origin` in the window. Left click-away dismisses.

Presses on the card are captured so they do not fall through. Right-click is `listen_cursor` (even when an editor captured the press). Row widgets also emit `crate::collection::ItemClick`. Empty `actions` still paints a card. `viewport` clamps the card to its real size. `progress` is 0 (gone) to 1 (rest). Labels start-align on the row.

Constructor: `pattern::context_menu`
Material: Menus
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.context_menu.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## sectioned-menu

Vertical menu list with optional section titles and hairline dividers.

Use for context menus and cascading flyouts. Actions still come from the application table.

Constructor: `pattern::sectioned_menu`
Material: Menus (sections)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.sectioned_menu.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## cascade-menu

Two-level cascade: primary list, optional open submenu panel.

The application owns which primary row is expanded (`open_sub`). `sub_progress` is 0 (gone) to 1 (rest) for that panel. Sub items share the same message type as top-level actions.

Constructor: `pattern::cascade_menu`
Material: Menus (cascade)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.cascade_menu.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## breadcrumb

A path of links.

Crumbs before the last send a message. The last crumb is the current page. Empty path is empty.

Constructor: `widget::breadcrumb`
Desktop: Navigation (breadcrumb)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.breadcrumb.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## menu

An in-window menu bar from one `ActionTable` (`crate::i18n::Catalog`, `on_open`).

Groups by the id prefix before `.`. Disabled actions stay out of the pick list.

Constructor: `pattern::menu_bar`
Material: Menus
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.menu_bar.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## toolbar

A row of action buttons from the same table as the menu.

Disabled actions paint muted.

Constructor: `pattern::toolbar`
Desktop: App bars (desktop)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.toolbar.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## status-bar

Footer text plus shortcut hints from the same table.

`tone` paints the left with `crate::widget::banner` when set, otherwise meta. `caption` is one right-rail string when set. `None` paints each enabled shortcut as two faces: the chord in `Tokens::text`, the title in `Tokens::muted` at meta size. An empty table (and no caption) shows status only.

Constructor: `pattern::status_bar`
Desktop: App bars (desktop)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/fn.status_bar.html) · [source](https://github.com/indynull/icedtea/blob/main/src/pattern.rs)

## busy

Dim plus spinner over `child` when `busy`.

When `busy` is false the child is unmodified. Advance spinner `phase` while true.

Constructor: `widget::busy_overlay`
Material: Progress indicator
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.busy_overlay.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## toast

A transient notice.

The application owns the queue and dismiss. Empty queue paints nothing. Corners follow `Tokens::shape` (`crate::m3::shape::Component::Toast`).

Constructor: `widget::toast_view`
Material: Snackbar
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.toast_view.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## scrollbar

A themed scroller with a usable handle.

`stick` pins to the end. `scroll_id` is for `scroll_to`. `on_scroll` receives the pixel offset from the start when the offset moves. The rail sits on the end side (`Tokens.direction`). Press and hover stay inside the pane. Move and Release continue after a press inside so a drag-select can finish outside. A nested `scroll` under the pointer takes the wheel first. Wheel lines are `crate::chrome::SCROLL_LINE` (60 px), same as iced. The pane is focusable for arrows, Page, Home, and End. It does not paint a focus ring: that ring is a control frame, not pane chrome.

Constructor: `widget::scroll`
Material: Lists (scroll)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.scroll.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## motion

Fade and slide a child for overlay enter/exit.

`progress` is 0 (gone) to 1 (at rest). The application owns `iced::Animation` and passes `interpolate(0.0, 1.0, now)`. Build the child with `Tokens::fade` so fills, ink, and icons fade with the slide. `Slide::None` skips the translate (fade only). Reduced-motion tokens snap to 0 or 1. Empty progress still occupies layout so a closing frame can run.

Constructor: `motion::overlay`
Material: Motion
[rustdoc](https://docs.rs/icedtea/latest/icedtea/motion/fn.overlay.html) · [source](https://github.com/indynull/icedtea/blob/main/src/motion.rs)

## switch-motion

Replace `outgoing` with `incoming` from a 0..=1 progress.

`SwitchFace::SharedAxis` is next/previous peers: incoming uses `progress` on `slide`, leaving uses `1 - progress` on the opposite slide, travel is `m3::motion::OVERLAY_SLIDE` (12 dp). Build each child with `Tokens::fade` from `SwitchFace::incoming_fade` / `SwitchFace::outgoing_fade`. `SwitchFace::FadeThrough` is tab bodies and other unrelated destinations. Reduced-motion tokens snap. Child overlays (pick lists) still open.

Constructor: `motion::switch`
Material: Motion (shared axis / fade through)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/motion/fn.switch.html) · [source](https://github.com/indynull/icedtea/blob/main/src/motion.rs)

## attention-motion

Shake or pulse a child from a 0..=1 progress.

`AttentionFace::Shake` is a decaying wiggle that starts and ends at rest (invalid field). `AttentionFace::Pulse` scales about the center (live mark). Reduced-motion tokens hold rest.

Constructor: `motion::attention`
Material: Motion (attention)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/motion/fn.attention.html) · [source](https://github.com/indynull/icedtea/blob/main/src/motion.rs)

## expand-motion

Clip a child between a peek size and its open size on one `Axis`.

`progress` 0 is `peek` pixels (0 hides). 1 is the child's laid-out size on that axis. `Axis::Block` is height; `Axis::Inline` is width (drawer, folder rail). Reduced-motion tokens snap.

Constructor: `motion::expand`
Material: Motion (expand)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/motion/fn.expand.html) · [source](https://github.com/indynull/icedtea/blob/main/src/motion.rs)
