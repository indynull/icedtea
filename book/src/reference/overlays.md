# Overlays

Chrome that floats over the current window content.
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/index.html) ·
[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

See also [Overlay windows](../overlay-windows.md) for a second
host window (palette, picker).

### Dialogs

**`dialogs`** — A confirm / message / save sheet.

Constructor: [`pattern::dialog_sheet`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.dialog_sheet.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Primary and optional cancel messages. Native file dialogs go through
`icedtea::native_dialog`. In-window modals sit on `pattern::modal_card`.

### Command palette

**`palette`** — Fuzzy find over the action table.

Constructor: [`pattern::command_palette_view`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.command_palette_view.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

`CommandPalette` owns the query and hits. Empty query can show
recent and favorites. See [Overlay windows](../overlay-windows.md).
