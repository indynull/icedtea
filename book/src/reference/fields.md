# Fields

Text, numbers, dates, and picks.
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/index.html) ·
[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

### Text input

**`text-input`** — A single-line editor.

Constructor: [`widget::themed_text_input`](https://docs.rs/icedtea/latest/icedtea/widget/fn.themed_text_input.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Optional iced `Id` so you can `focus` after show. Disabled greys the
field and drops edit. Empty value is a valid state.

### Password

**`password`** — A masked single-line editor.

Constructor: [`widget::password_input`](https://docs.rs/icedtea/latest/icedtea/widget/fn.password_input.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Characters paint as dots. The application owns the string. Disabled
drops edit.

### Secret field

**`secret`** — A settings row: masked field, reveal, and copy.

Constructor: [`widget::secret_field`](https://docs.rs/icedtea/latest/icedtea/widget/fn.secret_field.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Reveal toggles the mask. Copy is an `Action` whose message the
application handles with `icedtea::copy_text`.

### Value field

**`value-field`** — A labeled read-only value the user can select and copy.

Constructor: [`widget::value_field`](https://docs.rs/icedtea/latest/icedtea/widget/fn.value_field.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Meta label in a fixed gutter, selectable value (fill), optional Copy
`Action`. Pass `layout::FORM_LABEL` (140px, same as `layout::form`) so
stacked rows align; pass another width when the stack needs a wider
gutter. Bind the text with `field::Selectables` (`get` is `Option`,
unbound `perform` is a no-op). Mono face for paths and ids. Same
select-and-copy contract as body and code: app-owned buffer,
`select_only`, range via `Content::selection()` and
`icedtea::copy_text`. See
[Content: Select and copy](content.md#select-and-copy) and
[`select`](https://docs.rs/icedtea/latest/icedtea/select/index.html).

### Text area

**`textarea`** — A multi-line editor.

Constructor: [`widget::textarea`](https://docs.rs/icedtea/latest/icedtea/widget/fn.textarea.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Height is `layout::FILL` or `layout::fixed`. The application owns the
`text_editor::Content`. Disabled drops edit.

### Search

**`search`** — A query field with a search icon.

Constructor: [`widget::search_input`](https://docs.rs/icedtea/latest/icedtea/widget/fn.search_input.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Use for palette and list filters. Empty query means “show all”.

### Search view

**`search-view`** — Docked results under a search field.

Constructor: [`widget::search_view`](https://docs.rs/icedtea/latest/icedtea/widget/fn.search_view.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea)

The application owns the query and the hit list. Empty hits show
`empty`. Disabled drops pick and clear.

### Suggest

**`suggest`** — A text field with a pick list of completions.

Constructor: [`widget::suggest_field`](https://docs.rs/icedtea/latest/icedtea/widget/fn.suggest_field.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea)

The application owns the query and the suggestion list. Picking a
row writes that string.

### Select

**`select`** — Pick one string from a list.

Constructor: [`widget::themed_pick_list`](https://docs.rs/icedtea/latest/icedtea/widget/fn.themed_pick_list.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

Placeholder shows when nothing is selected. Disabled keeps the
current face.

### Number

**`number`** — Edit a numeric value with step buttons.

Constructor: [`widget::number_input`](https://docs.rs/icedtea/latest/icedtea/widget/fn.number_input.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea)

The application owns the number. Step messages bump it. Disabled
freezes the value.


### Date

**`date`** — Pick a calendar date.

Constructor: [`widget::date_picker`](https://docs.rs/icedtea/latest/icedtea/widget/fn.date_picker.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea)

The application owns the selected day. Disabled ignores picks.

### Time

**`time`** — Step hour, minute, second, or period on a 24-hour value.

Constructor: [`widget::time_picker`](https://docs.rs/icedtea/latest/icedtea/widget/fn.time_picker.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea)

`TimeValue` is hour, minute, second. `TimeClock` is display only
(12-hour or 24-hour, optional seconds). `TimeField` is the unit that
steps. Disabled freezes the fields.

### Field support

**`field-support`** — Supporting or error text under a field.

Constructor: [`widget::field_support`](https://docs.rs/icedtea/latest/icedtea/widget/fn.field_support.html)

[source](https://github.com/indynull/icedtea/blob/master/src/widget.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

`support` is quiet helper copy. `error` uses the error role and wins when both are set.
