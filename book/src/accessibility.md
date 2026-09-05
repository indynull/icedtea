# Accessibility

Drawing constructors in
[`widget`](https://docs.rs/icedtea/latest/icedtea/widget/index.html)
take [`A11y`](https://docs.rs/icedtea/latest/icedtea/a11y/struct.A11y.html):
name, role, value, hint, disabled, checked / selected / toggled,
expanded, live, required, and error. The constructor calls `attach`
and fills unset fields from its arguments (slider value, field error,
expander open, toast copy).

Chrome rows (`toolbar`, `menu_bar`, `status_bar`, `command_bar`) take
the action table; the buttons they paint already carry `A11y`. Window
recipes (`dialog_sheet`, `list_detail`, `main_window`) take children
and tokens. Layout recipes such as `layout::pack` and `layout::wrap`
do not take `A11y`.

Empty caption uses the accessible name. A visible caption is left
alone; decorative chrome may pass an empty name. `disabled` drops
the activate handler and paints the disabled face.

iced 0.14 has no AccessKit slot. `attach` sets the iced widget id
from role, name, and disabled. A screen reader does not receive
role, value, or hint today.

Keyboard order is the working desktop path. An open modal consumes
keys first. A focused field owns unmodified typing. Otherwise the
focused constructor owns arrows, Enter, and Space. Tab walks
[`focus::cycle`](https://docs.rs/icedtea/latest/icedtea/focus/fn.cycle.html).

A [`focus::target`](https://docs.rs/icedtea/latest/icedtea/focus/fn.target.html)
ring is 2 dp, inset one density grid step, on **one control face**.
A list, virtual column, tree, grid, or table is
[`focus::group`](https://docs.rs/icedtea/latest/icedtea/focus/fn.group.html):
the selected row or tile owns chrome. The pane does not paint a
second ring around several item faces (WCAG 2.4.7; APG roving
tabindex; Material list focus is on the item).
Otherwise
[`key::handle`](https://docs.rs/icedtea/latest/icedtea/key/fn.handle.html)
matches the action table. See [Architecture](architecture.md#keys).
