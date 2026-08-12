# Widgets

Every public constructor returns an iced `Element` and emits the
application's messages. It takes `Tokens` and `A11y` (name, role,
disabled, checked). The application owns state.

| Group | Page |
| --- | --- |
| Controls | [Controls](reference/controls.md) |
| Fields | [Fields](reference/fields.md) |
| Readout | [Readout](reference/readout.md) |
| Content | [Content](reference/content.md) |
| Collections | [Collections](reference/collections.md) |
| Chrome | [Chrome](reference/chrome.md) |
| Patterns | [Patterns](reference/patterns.md) |

Each entry names the job, the shipped constructor, and links to
[rustdoc](https://docs.rs/icedtea),
[source](https://github.com/indynull/icedtea), and
[crates.io](https://crates.io/crates/icedtea).

Body, code, markdown, and value fields share one
[select-and-copy](reference/content.md#select-and-copy) contract
([`select`](https://docs.rs/icedtea/latest/icedtea/select/index.html)).
