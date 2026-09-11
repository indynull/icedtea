# Content

Generated from `catalog::ENTRIES`, constructor rustdoc, and
`m3::mapping`. Compose starts at `examples/hello.rs` (see
[llms.txt](llms.txt)).

## label

A line of UI text (`LabelFace`: Body, Meta, Display, Figure).

`LabelFace::Body` is platform sans. Empty string is an empty node; still pass `A11y`. `LabelFace::Meta` shrinks. `LabelFace::Display` is an end-aligned mute line. `LabelFace::Figure` splits glyphs.

Constructor: `widget::label`
Material: Typography
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.label.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## icon

Paint a chrome glyph.

Pass a shipped `Icon` or `Glyph::Bytes` (filled black SVG). Tokens recolor non-transparent pixels (Linux, macOS Metal, and Windows). `Icon` is the desktop chrome set.

Constructor: `widget::icon_svg`
Material: Icons
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.icon_svg.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## tooltip

Hover text on a child.

Empty tip text is a no-op wrap. The child keeps its own `A11y`. Corners follow `Tokens::shape` (`crate::m3::shape::Component::Tooltip`).

Constructor: `widget::tooltip_wrap`
Material: Tooltip
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.tooltip_wrap.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## rich-tooltip

Hover title plus supporting copy on a child (M3 rich tooltip).

Empty title and body is a no-op wrap. The child keeps its own `A11y`.

Constructor: `widget::tooltip_rich`
Material: Tooltip (rich)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.tooltip_rich.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## link

A text link that sends a message.

The application opens the URL or navigates. Disabled paints muted text and drops the press.

Constructor: `widget::hyperlink`
Material: Button (text)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.hyperlink.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## markdown

A parsed markdown document (`parse`, `MarkdownDoc::headings`, `item_offset`, `MarkdownOpts`).

Parse with `parse`, then view with `markdown_view`. Truncate by slicing the source before parse.

# Select and copy

Painted with real markdown layout (headings, lists, code frames, quotes). Body is `Tokens::body`; H1 is `Tokens::page` (window title), not iced's 2× blog heading. Drag a range with `crate::select::markdown_select` so it can start in one block and end in another; pass the live `crate::select::MarkdownSpan` here. The view is not flattened into one mixed-size `Rich`. The document tree stays one `view_with` whether a range is empty or not. Pointer events reach `on_pointer` first so paint and Copy share that span. Consecutive clicks expand the range: word, sentence, then the block. The primary+A chord selects every block. A drag that leaves the document still posts Move (clamped) and Release. Ctrl+C / Cmd+C on a span is `crate::select::MarkdownSpan::text` plus `crate::select::MarkdownSpan::html` via `crate::copy_rich` (plain cells and a `<table>`, like a browser copy of rendered markdown). Full document source is `MarkdownDoc::source`.

Constructor: `widget::markdown_view`
Desktop: Typography (rich)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.markdown_view.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## code

Source panel. `syntax` is an iced highlighter token (`rs`, `py`, …); `None` is plain monospace (no highlighter). `theme_name` picks a highlighter face that fits the UI colorway. `height` is icedtea size language (`crate::layout::FILL` or `crate::layout::fixed`).

The application owns the buffer and the language name. Highlighter face follows the active colorway. Typing does not change the buffer. Disabled still allows select-and-copy. `wrap` is word wrap; `false` keeps each source line on one row (diff hunks, search hits).

Constructor: `widget::highlighted_code`
Desktop: Typography (mono)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.highlighted_code.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## image

An image slot that keeps its box.

Ready keeps the requested width and height. Missing bytes show the empty slot, not a collapsed layout.

Constructor: `widget::image_slot`
Material: Image
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.image_slot.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)

## selectable

Body the user can drag-select and copy.

Looks like body text: zero pad, no border, canvas fill. The application owns the buffer and posts `Content::selection()` with `crate::copy_text`. Height shrinks to the text. Disabled still allows select-and-copy. Use `typo::FontFace::Ui` for prose and `typo::FontFace::Mono` for paths or raw values.

For painted markdown (not an editor buffer), use `markdown_view`: selection and Ctrl+C are paint-side.

Constructor: `widget::selectable`
Desktop: Typography (select)
[rustdoc](https://docs.rs/icedtea/latest/icedtea/widget/fn.selectable.html) · [source](https://github.com/indynull/icedtea/blob/main/src/widget.rs)
