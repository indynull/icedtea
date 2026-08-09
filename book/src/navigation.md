# Navigation

Keep page identity in application **state**. `NavStack` is push / pop /
replace with `can_back` when depth > 1. `view` calls
`pattern::navigation_view`, which places a sidebar beside content on
medium and expanded widths, and stacks with a back **message** on
compact.

List/detail, tab view, preferences, about, and status page are in
`icedtea::pattern` — they return `Element`s.
