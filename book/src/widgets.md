# Widgets

Drawing constructors in `widget` return an iced `Element` and emit
the application's messages. They take `Tokens` and `A11y` (name,
role, value, hint, disabled, checked / selected / toggled, expanded,
live, required, error). Chrome rows take an `ActionTable`. Layout
recipes do not take `A11y`. iced 0.14 publishes the widget id only.
Keyboard order is the working path: see
[Accessibility](accessibility.md). The application owns state.

| Group | Page |
| --- | --- |
| Controls | [Controls](reference/controls.md) |
| Fields | [Fields](reference/fields.md) |
| Readout | [Readout](reference/readout.md) |
| Content | [Content](reference/content.md) |
| Collections | [Collections](reference/collections.md) |
| Chrome | [Chrome](reference/chrome.md) |
| Patterns | [Patterns](reference/patterns.md) |

Each group page shows a still of the shipped constructors, then names
the job, the constructor, and links to
[rustdoc](https://docs.rs/icedtea),
[source](https://github.com/indynull/icedtea), and
[crates.io](https://crates.io/crates/icedtea).

Body, code, markdown, and value fields share one
[select-and-copy](reference/content.md#select-and-copy) contract
([`select`](https://docs.rs/icedtea/latest/icedtea/select/index.html)).
