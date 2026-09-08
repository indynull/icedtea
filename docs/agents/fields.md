# Fields

Generated from `catalog::ENTRIES`, constructor rustdoc, and
`m3::mapping`. Compose starts at `examples/hello.rs` (see
[llms.txt](llms.txt)).

## search

A query field: one Search-radius bar with the glass inside.

Use for palette and list filters. Empty query means show all. Placeholder is the a11y name. `on_submit` is Enter. `input_id` focuses the field (palette, find-in-page). `highlight` is a syntax highlighter: byte ranges the application computed. Empty is one ink. Corners follow `Tokens::shape` (`crate::m3::shape::Component::Search`). Leading glass, value, and optional clear sit inside that face at one control height.

Constructor: `widget::search_input`
Material: Search
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.search_input.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## search-view

Docked search results under a search field (M3 search view, desktop).

Hits are application-filtered. Empty `hits` shows `empty`. Disabled drops pick and clear. `selected` is the highlighted hit; arrows from the query field move it (Spotlight).

Constructor: `widget::search_view`
Material: Search (view)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.search_view.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## text-input

Single-line field. `input_id` is for `iced::widget::operation::focus` after show. A single-line editor.

Optional iced `Id` so you can `focus` after show. Disabled greys the field and drops edit. Empty value is a valid state. `FieldOpts::highlight` is a syntax highlighter: application `FieldRun`s on the typed value. Caret, selection, and placeholder stay iced's.

Constructor: `widget::text_input`
Material: Text field
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.text_input.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## field-support

Field stack with optional supporting or error text under the control.

`support` is quiet helper copy. `error` paints error role ink and wins when both are set. Pass an already-built field as `child`.

Constructor: `widget::field_support`
Material: Text field (supporting / error)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.field_support.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## password

A masked single-line editor.

Characters paint as dots. The application owns the string. Disabled drops edit.

Constructor: `widget::password_input`
Material: Text field
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.password_input.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## secret

A settings row: masked field, reveal, and copy.

Reveal toggles the mask. Copy is an `crate::action::Action` whose message the application handles with `icedtea::copy_text`.

Constructor: `widget::secret_field`
Material: Text field
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.secret_field.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## value-field

A labeled read-only value the user can select and copy.

Meta label in a fixed gutter, then `selectable` (fill), then an optional Copy `crate::action::Action`. Pass `crate::layout::FORM_LABEL` so multi-row stacks share one column (same gutter as `crate::layout::form`). The application posts `crate::field::Selectables::copy` with `crate::copy_text`. Mono face for paths and ids; UI face for prose. Disabled still allows select-and-copy.

Constructor: `widget::value_field`
Material: Text field (labeled)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.value_field.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## textarea

Multiline editor. `height` is icedtea size language (`crate::layout::FILL` or `crate::layout::fixed`). A multi-line editor.

Height is `crate::layout::FILL` or `crate::layout::fixed`. The application owns the buffer. Disabled drops edit.

Constructor: `widget::textarea`
Material: Text field (multi-line)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.textarea.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## suggest

Text field plus a keyboard-complete pick list; the application owns the suggestion slice for the whole `view` (store it on the app; a local `Vec` cannot outlive the `Element`). A text field with a pick list of completions.

The application owns the query and the suggestion list. Picking a row writes that string.

Constructor: `widget::suggest_field`
Material: Menus (suggest)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.suggest_field.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## select

Pick one string from a list.

`size` is `ControlSize`. Compact uses tight pad and meta type so a toolbar or HUD can nest a dropdown. Default keeps the field body look. The trailing mark is Material `arrow_drop_down` (24 dp, 20 dp Compact), inset `Density::inset` from the **end**. A press on the mark opens the menu. Focused Enter and Space open it too. The list uses Menu shape (extra-small under) Soft and Pill) so the drawer is a box, not a stack of stadiums. Placeholder shows when nothing is selected. Wheel over the control moves the selection. Disabled keeps the current face.

Constructor: `widget::pick_list`
Material: Menus
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.pick_list.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## form

Form group that owns Tab and Shift+Tab among mixed fields.

`layout::form` only stacks label/field rows. This constructor walks those rows: Tab and Shift+Tab wrap, and the first text field takes iced focus on mount. Space activates the focused non-text row (checkbox, radio, pick, chips, segmented). An empty row title leaves the label column blank so a checkbox or chip can carry its own caption. Pick lists, chips, checkboxes, radios, and segmented buttons sit in the same order. The application owns values, messages, and `active`.

Constructor: `widget::form_group`
Material: Text fields (form)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.form_group.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## number

Edit a numeric value with step buttons.

The application owns the number. Wheel steps by 1. Disabled freezes the value.

Constructor: `widget::number_input`
Material: Text field
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.number_input.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## date

Pick a calendar date.

The application owns the selected day. Disabled ignores picks.

Constructor: `widget::date_stepper`
Desktop: Date pickers (desktop)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.date_stepper.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## time

Step hour, minute, second, or period on a 24-hour value.

`TimeValue` is the clock. `TimeClock` is display only. Disabled freezes the fields.

Constructor: `widget::time_picker`
Desktop: Time pickers (desktop)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.time_picker.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)
