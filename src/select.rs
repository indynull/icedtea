//! Unified select-and-copy for content text.
//!
//! # App contract
//!
//! Readable content the user is meant to copy behaves like a web page
//! body: drag a contiguous range of **visible** text, then copy
//! (Ctrl/Cmd+C or the host clipboard path the surface documents).
//! Typing does not apply on read-only surfaces.
//!
//! | Surface | Constructor | Who owns text | Range copy | Full document |
//! | --- | --- | --- | --- | --- |
//! | Body / path | [`crate::widget::selectable`], [`crate::widget::value_field`] | App `text_editor::Content` / [`crate::field::Selectables`] | `Content::selection()` → [`crate::copy_text`] | whole buffer via `text()` / `Selectables::copy` |
//! | Code | [`crate::widget::highlighted_code`], [`crate::widget::code_block`] | App `Content` | same | same |
//! | Markdown | [`crate::widget::markdown_view`] | Structured paint (per block) | Ctrl/Cmd+C on a selected block | [`crate::copy_text`] on [`crate::widget::MarkdownDoc::source`] |
//!
//! Chrome (menus, buttons, status meta) is not drag-selectable.
//!
//! Editor surfaces use [`select_only`] so the buffer cannot be mutated
//! by typing. Markdown keeps real block layout; selection is paint-side
//! within each block. Flattening every block into one rich surface
//! breaks layout and multi-line selection paint, so it is not the
//! shipped path.

use iced::advanced::text::Span;
use iced::widget::markdown::{self, Bullet, HeadingLevel, Item, Settings, Text};
use iced::Font;

/// Keep selection, click, drag, and scroll. Typing, paste, and delete
/// become a zero scroll so `Content::perform` does not change the text.
pub fn select_only(action: iced::widget::text_editor::Action) -> iced::widget::text_editor::Action {
    if action.is_edit() {
        iced::widget::text_editor::Action::Scroll { lines: 0 }
    } else {
        action
    }
}

/// Slice `text` from byte index `start` to `end` (clamped, start ≤ end).
pub fn copy_range(text: &str, start: usize, end: usize) -> String {
    let n = text.len();
    let start = floor_char_boundary(text, start.min(n));
    let end = ceil_char_boundary(text, end.min(n).max(start));
    text[start..end].to_string()
}

fn floor_char_boundary(s: &str, i: usize) -> usize {
    if i >= s.len() {
        return s.len();
    }
    let mut i = i;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn ceil_char_boundary(s: &str, i: usize) -> usize {
    if i >= s.len() {
        return s.len();
    }
    let mut i = i;
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

/// Plain text of a painted markdown document in document order.
///
/// Top-level blocks are separated by blank lines. Useful for tests and
/// for building a linear copy string; the live view selects per block.
pub fn markdown_plain(items: &[Item]) -> String {
    let settings = Settings::with_style(markdown_measure_style());
    let spans = markdown_document_spans(items, &settings);
    spans.iter().map(|s| s.text.as_ref()).collect()
}

/// Continuous document spans for one selectable rich surface.
pub(crate) fn markdown_document_spans(
    items: &[Item],
    settings: &Settings,
) -> Vec<Span<'static, markdown::Uri, Font>> {
    let mut spans = Vec::new();
    flatten_spans(items, settings, &mut spans, 0);
    spans
}

fn flatten_spans(
    items: &[Item],
    settings: &Settings,
    out: &mut Vec<Span<'static, markdown::Uri, Font>>,
    depth: usize,
) {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(Span::new(if depth == 0 { "\n\n" } else { "\n" }));
        }
        match item {
            Item::Heading(level, text) => {
                let size = heading_size(settings, *level);
                push_styled(text, settings.style, out, Some(size), false);
            }
            Item::Paragraph(text) => {
                push_styled(text, settings.style, out, Some(settings.text_size), false);
            }
            Item::CodeBlock { lines, code, .. } => {
                if lines.is_empty() {
                    let mut s = Span::new(code.clone());
                    s.font = Some(Font::MONOSPACE);
                    s.size = Some(settings.code_size);
                    out.push(s);
                } else {
                    for (li, line) in lines.iter().enumerate() {
                        if li > 0 {
                            out.push(Span::new("\n"));
                        }
                        push_styled(line, settings.style, out, Some(settings.code_size), true);
                    }
                }
            }
            Item::List { start, bullets } => {
                for (bi, bullet) in bullets.iter().enumerate() {
                    if bi > 0 {
                        out.push(Span::new("\n"));
                    }
                    let prefix = match bullet {
                        Bullet::Task { done: true, .. } => "[x] ".to_string(),
                        Bullet::Task { done: false, .. } => "[ ] ".to_string(),
                        Bullet::Point { .. } => {
                            if let Some(n) = *start {
                                format!("{}. ", n + bi as u64)
                            } else {
                                "• ".to_string()
                            }
                        }
                    };
                    out.push(Span::new(prefix).size(settings.text_size));
                    let kids = match bullet {
                        Bullet::Point { items } | Bullet::Task { items, .. } => items.as_slice(),
                    };
                    flatten_spans(kids, settings, out, depth + 1);
                }
            }
            Item::Quote(inner) => {
                flatten_spans(inner, settings, out, depth + 1);
            }
            Item::Rule => {
                out.push(Span::new("———").size(settings.text_size));
            }
            Item::Image { alt, title, .. } => {
                push_styled(alt, settings.style, out, Some(settings.text_size), false);
                if !title.is_empty() {
                    out.push(Span::new(format!(" ({title})")).size(settings.text_size));
                }
            }
            Item::Table { .. } => {
                // Row cell storage is private in iced 0.14; keep a marker so
                // the document stays one continuous surface.
                out.push(Span::new("[table]").size(settings.text_size));
            }
        }
    }
}

fn heading_size(settings: &Settings, level: HeadingLevel) -> iced::Pixels {
    match level {
        HeadingLevel::H1 => settings.h1_size,
        HeadingLevel::H2 => settings.h2_size,
        HeadingLevel::H3 => settings.h3_size,
        HeadingLevel::H4 => settings.h4_size,
        HeadingLevel::H5 => settings.h5_size,
        HeadingLevel::H6 => settings.h6_size,
    }
}

fn push_styled(
    text: &Text,
    style: markdown::Style,
    out: &mut Vec<Span<'static, markdown::Uri, Font>>,
    size: Option<iced::Pixels>,
    mono: bool,
) {
    for mut s in text.spans(style).iter().cloned() {
        if let Some(sz) = size {
            s.size = Some(sz);
        }
        if mono {
            s.font = Some(Font::MONOSPACE);
        }
        out.push(s);
    }
}

fn markdown_measure_style() -> markdown::Style {
    markdown::Style::from_palette(iced::theme::Palette {
        background: iced::Color::BLACK,
        text: iced::Color::WHITE,
        primary: iced::Color::WHITE,
        success: iced::Color::WHITE,
        warning: iced::Color::WHITE,
        danger: iced::Color::WHITE,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::markdown;
    use iced::widget::text_editor::{Action, Edit, Motion};

    #[test]
    fn copy_range_clamps_and_joins() {
        let s = "hello\nworld";
        assert_eq!(copy_range(s, 0, 5), "hello");
        assert_eq!(copy_range(s, 0, 11), "hello\nworld");
        assert_eq!(copy_range(s, 3, 8), "lo\nwo");
        assert_eq!(copy_range(s, 100, 200), "");
        // Mid-codepoint offsets snap to char boundaries (é is 2 bytes).
        let uni = "aéb";
        assert_eq!(copy_range(uni, 1, 3), "é");
        // start=end mid-char expands to that full char after floor/ceil.
        assert_eq!(copy_range(uni, 2, 2), "é");
        assert_eq!(copy_range(uni, 0, 0), "");
        assert_eq!(copy_range("", 0, 1), "");
    }

    #[test]
    fn select_only_drops_edits() {
        assert!(matches!(
            select_only(Action::Edit(Edit::Insert('x'))),
            Action::Scroll { lines: 0 }
        ));
        assert!(matches!(
            select_only(Action::Edit(Edit::Delete)),
            Action::Scroll { lines: 0 }
        ));
        assert_eq!(
            select_only(Action::Select(Motion::Right)),
            Action::Select(Motion::Right)
        );
        assert_eq!(select_only(Action::SelectAll), Action::SelectAll);
        assert_eq!(
            select_only(Action::Scroll { lines: 2 }),
            Action::Scroll { lines: 2 }
        );
    }

    #[test]
    fn markdown_plain_preserves_document_order() {
        let items: Vec<_> =
            markdown::parse("# Title\n\nFirst paragraph.\n\nSecond block.").collect();
        let plain = markdown_plain(&items);
        assert!(plain.contains("Title"));
        assert!(plain.contains("First paragraph."));
        assert!(plain.contains("Second block."));
        let a = plain.find("Title").unwrap();
        let b = plain.find("Second block.").unwrap();
        assert!(a < b);
        let span = copy_range(&plain, a, b + "Second".len());
        assert!(span.contains("Title"));
        assert!(span.contains("First"));
        assert!(span.contains('\n'));
        assert!(span.contains("Second"));
    }

    #[test]
    fn markdown_document_spans_join_blocks_for_plain_order() {
        let items: Vec<_> = markdown::parse("# A\n\nB line\n\nC line").collect();
        let settings = Settings::with_style(markdown_measure_style());
        let spans = markdown_document_spans(&items, &settings);
        let joined: String = spans.iter().map(|s| s.text.as_ref()).collect();
        assert!(joined.contains('A'));
        assert!(joined.contains("B line"));
        assert!(joined.contains("C line"));
        assert!(
            joined.contains("\n\n"),
            "plain extract separates top-level blocks: {joined:?}"
        );
        let start = joined.find('A').unwrap();
        let end = joined.find("C line").unwrap() + "C line".len();
        let multi = copy_range(&joined, start, end);
        assert!(multi.contains('A') && multi.contains('B') && multi.contains('C'));
        assert!(multi.matches('\n').count() >= 2);
        // Fence with no line tokens still contributes the raw code body.
        let bare: Vec<_> = markdown::parse("```\nplain code\n```").collect();
        let bare_spans = markdown_document_spans(&bare, &settings);
        let bare_joined: String = bare_spans.iter().map(|s| s.text.as_ref()).collect();
        assert!(
            bare_joined.contains("plain code"),
            "empty line list falls back to code body: {bare_joined:?}"
        );
    }

    #[test]
    fn markdown_plain_covers_lists_code_quote_rule_image_table() {
        let source = r#"# H1
## H2
### H3
#### H4
##### H5
###### H6

Paragraph with **bold**.

```rs
fn a() {}
fn b() {}
```

- unordered
- second

1. ordered
2. next

- [x] done
- [ ] todo

> quote line

---

![alt text](https://example.com/x.png "title here")

| a | b |
| - | - |
| 1 | 2 |
"#;
        let items: Vec<_> = markdown::parse(source).collect();
        let plain = markdown_plain(&items);
        assert!(plain.contains("H1") && plain.contains("H6"));
        assert!(plain.contains("Paragraph with"));
        assert!(plain.contains("fn a()") && plain.contains("fn b()"));
        assert!(plain.contains('•') || plain.contains("unordered"));
        assert!(plain.contains("1.") || plain.contains("ordered"));
        assert!(plain.contains("[x]") || plain.contains("done"));
        assert!(plain.contains("[ ]") || plain.contains("todo"));
        assert!(plain.contains("quote"));
        assert!(plain.contains("———"));
        assert!(plain.contains("alt text"));
        assert!(plain.contains("title here") || plain.contains('('));
        assert!(
            plain.contains("[table]"),
            "table cells private; marker keeps one surface: {plain:?}"
        );
        // Nested depth uses single newline between list kids when present.
        let nested: Vec<_> = markdown::parse("- outer\n  - inner\n").collect();
        let nested_plain = markdown_plain(&nested);
        assert!(nested_plain.contains("outer"));
        assert!(nested_plain.contains("inner"));
    }

    #[test]
    fn markdown_empty_and_single_block_still_flatten() {
        let empty: Vec<_> = markdown::parse("").collect();
        assert_eq!(markdown_plain(&empty), "");
        let one: Vec<_> = markdown::parse("only").collect();
        assert!(markdown_plain(&one).contains("only"));
        // Fenced code with body hits multi-line mono path.
        let code: Vec<_> = markdown::parse("```\nline1\nline2\n```").collect();
        let plain = markdown_plain(&code);
        assert!(plain.contains("line1") && plain.contains("line2"));
        assert!(plain.contains('\n'));
    }
}
