# Screens

Full-window views apps open from menus (Help → About, settings,
empty state). Not multi-pane layout chrome.
[rustdoc](https://docs.rs/icedtea/latest/icedtea/pattern/index.html) ·
[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea) ·
[iced](https://crates.io/crates/iced)

### Preferences

**`preferences`** — Searchable preference groups.

Constructor: [`pattern::preferences_page`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.preferences_page.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

`PrefGroup` is a title plus key/value rows. Empty query shows every
group.

### About

**`about`** — Name, version, license, and credits.

Constructor: [`pattern::about_page`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.about_page.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Strings are the application's. Catalog supplies chrome labels.

### Status page

**`status-page`** — Centered empty or error state.

Constructor: [`pattern::status_page`](https://docs.rs/icedtea/latest/icedtea/pattern/fn.status_page.html)

[source](https://github.com/indynull/icedtea/blob/master/src/pattern.rs) ·
[icedtea](https://crates.io/crates/icedtea)

Title, body, and an optional action. Use when a list has no rows.
