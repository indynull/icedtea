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
keys first. A focused field owns unmodified typing. Otherwise
[`key::handle`](https://docs.rs/icedtea/latest/icedtea/key/fn.handle.html)
matches the action table. See [Architecture](architecture.md).
