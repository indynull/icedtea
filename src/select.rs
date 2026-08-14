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
//! | Markdown | [`crate::widget::markdown_view`] | Structured paint (per block) | [`MarkdownSpan`] across blocks → [`crate::copy_text`] | [`crate::copy_text`] on [`crate::widget::MarkdownDoc::source`] |
//!
//! Chrome (menus, buttons, status meta) is not drag-selectable.
//!
//! Editor surfaces use [`select_only`] so the buffer cannot be mutated
//! by typing. Markdown keeps real block layout. Drag uses
//! [`markdown_select`] so a range can start in one block and end in
//! another. Flattening every block into one rich surface breaks layout
//! and multi-line selection paint, so it is not the shipped path.

use iced::advanced::text::Span;
use iced::widget::markdown::{self, Bullet, HeadingLevel, Item, Settings, Text};
use iced::Font;

use crate::typo;

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

/// Paint settings for [`crate::widget::markdown_view`].
///
/// Body, code, and headings use [`typo`] steps (H1 is [`typo::PAGE`]).
pub(crate) fn markdown_paint_settings(style: markdown::Style) -> Settings {
    let body = typo::BODY as f32;
    Settings {
        text_size: body.into(),
        h1_size: (typo::PAGE as f32).into(),
        h2_size: (typo::TITLE as f32).into(),
        h3_size: body.into(),
        h4_size: body.into(),
        h5_size: (typo::META as f32).into(),
        h6_size: (typo::META as f32).into(),
        code_size: (typo::CODE as f32).into(),
        spacing: (body * 0.875).into(),
        style,
    }
}

/// Plain text of a painted markdown document in document order.
///
/// Top-level blocks are separated by blank lines. Useful for tests and
/// for building a linear copy string; the live view selects per block.
pub fn markdown_plain(items: &[Item]) -> String {
    let settings = markdown_paint_settings(markdown_measure_style());
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

/// Byte caret inside one top-level markdown item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MarkdownPos {
    pub item: usize,
    pub offset: usize,
}

/// Ordered range across markdown blocks (start ≤ end in document order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MarkdownSpan {
    pub start: MarkdownPos,
    pub end: MarkdownPos,
}

impl MarkdownSpan {
    /// Span from a press caret to the current caret.
    pub fn from_drag(press: MarkdownPos, now: MarkdownPos) -> Self {
        if (now.item, now.offset) < (press.item, press.offset) {
            Self {
                start: now,
                end: press,
            }
        } else {
            Self {
                start: press,
                end: now,
            }
        }
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Plain text of this range in document order (blocks joined by blank lines).
    pub fn text(self, items: &[Item]) -> String {
        if items.is_empty() {
            return String::new();
        }
        let last = items.len().saturating_sub(1);
        let start_i = self.start.item.min(last);
        let end_i = self.end.item.min(last);
        let mut out = String::new();
        for (i, item) in items.iter().enumerate().take(end_i + 1).skip(start_i) {
            let plain = markdown_item_plain(item);
            let n = plain.len();
            let a = if i == start_i {
                floor_char_boundary(&plain, self.start.offset.min(n))
            } else {
                0
            };
            let b = if i == end_i {
                ceil_char_boundary(&plain, self.end.offset.min(n).max(a))
            } else {
                n
            };
            if i > start_i {
                out.push_str("\n\n");
            }
            out.push_str(&plain[a..b]);
        }
        out
    }

    pub fn covers(self, item: usize) -> bool {
        item >= self.start.item && item <= self.end.item && !self.is_empty()
    }
}

/// Pointer event for [`markdown_select`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarkdownPointer {
    Press,
    Move(f32),
    Release,
}

/// Live drag state for [`crate::widget::markdown_view`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MarkdownSelect {
    pub span: MarkdownSpan,
    pub dragging: bool,
    pub anchor: MarkdownPos,
    pub hover_y: f32,
}

/// Apply a pointer event to a document span. Layout stays structured.
pub fn markdown_select(
    items: &[Item],
    mut state: MarkdownSelect,
    ev: MarkdownPointer,
) -> MarkdownSelect {
    match ev {
        MarkdownPointer::Press => {
            let pos = markdown_pos_at(items, state.hover_y);
            state.dragging = true;
            state.anchor = pos;
            state.span = MarkdownSpan::from_drag(pos, pos);
        }
        MarkdownPointer::Move(y) => {
            state.hover_y = y;
            if state.dragging {
                state.span = MarkdownSpan::from_drag(state.anchor, markdown_pos_at(items, y));
            }
        }
        MarkdownPointer::Release => {
            state.dragging = false;
        }
    }
    state
}

/// Map a Y offset in the markdown column to a caret.
pub fn markdown_pos_at(items: &[Item], y: f32) -> MarkdownPos {
    if items.is_empty() {
        return MarkdownPos::default();
    }
    let y = y.max(0.0);
    let mut acc = 0.0;
    let mut pos = MarkdownPos::default();
    for (i, item) in items.iter().enumerate() {
        let h = markdown_item_extent(item).max(1.0);
        let frac = ((y - acc) / h).clamp(0.0, 1.0);
        let plain = markdown_item_plain(item);
        let raw = (plain.len() as f32 * frac).round() as usize;
        pos = MarkdownPos {
            item: i,
            offset: floor_char_boundary(&plain, raw.min(plain.len())),
        };
        if y < acc + h {
            break;
        }
        acc += h;
    }
    pos
}

pub(crate) fn markdown_item_plain(item: &Item) -> String {
    let settings = markdown_paint_settings(markdown_measure_style());
    let mut spans = Vec::new();
    flatten_spans(std::slice::from_ref(item), &settings, &mut spans, 0);
    spans.iter().map(|s| s.text.as_ref()).collect()
}

pub(crate) fn markdown_text_len(text: &Text) -> usize {
    text.spans(markdown_measure_style())
        .iter()
        .map(|s| s.text.len())
        .sum()
}

/// Estimated block height (same map [`crate::widget::MarkdownDoc::item_offset`] uses).
pub fn markdown_item_extent(item: &Item) -> f32 {
    let settings = markdown_paint_settings(markdown_measure_style());
    let text = f32::from(settings.text_size);
    let spacing = f32::from(settings.spacing);
    const COL: f32 = 64.0;
    match item {
        Item::Heading(level, heading) => {
            let size = f32::from(heading_size(&settings, *level));
            let lines = ((markdown_text_len(heading) as f32) / COL).ceil().max(1.0);
            size * 1.3 * lines + text * 0.5 + spacing
        }
        Item::Paragraph(paragraph) => {
            let lines = ((markdown_text_len(paragraph) as f32) / COL)
                .ceil()
                .max(1.0);
            lines * text * 1.4 + spacing
        }
        Item::CodeBlock { code, lines, .. } => {
            let n = lines.len().max(code.lines().count()).max(1) as f32;
            n * f32::from(settings.code_size) * 1.5 + 24.0 + spacing
        }
        Item::List { bullets, .. } => {
            bullets
                .iter()
                .map(|b| {
                    let kids = match b {
                        Bullet::Point { items } | Bullet::Task { items, .. } => items,
                    };
                    text + kids.iter().map(markdown_item_extent).sum::<f32>()
                })
                .sum::<f32>()
                + spacing
        }
        Item::Image { .. } => 160.0 + spacing,
        Item::Quote(items) => items.iter().map(markdown_item_extent).sum::<f32>() + 16.0 + spacing,
        Item::Rule => 24.0 + spacing,
        Item::Table { rows, .. } => (1 + rows.len()) as f32 * text * 1.8 + spacing,
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
    fn markdown_paint_settings_follow_ui_type_scale() {
        let s = markdown_paint_settings(markdown_measure_style());
        assert_eq!(f32::from(s.text_size), crate::typo::BODY as f32);
        assert_eq!(f32::from(s.h1_size), crate::typo::PAGE as f32);
        assert_eq!(f32::from(s.h2_size), crate::typo::TITLE as f32);
        assert_eq!(f32::from(s.h3_size), crate::typo::BODY as f32);
        assert_eq!(f32::from(s.h4_size), crate::typo::BODY as f32);
        assert_eq!(f32::from(s.h5_size), crate::typo::META as f32);
        assert_eq!(f32::from(s.h6_size), crate::typo::META as f32);
        assert_eq!(f32::from(s.code_size), crate::typo::CODE as f32);
        assert!(f32::from(s.h1_size) > f32::from(s.text_size));
        let items: Vec<_> = markdown::parse("# Title\n\nBody.").collect();
        let h1 = items
            .iter()
            .find(|i| matches!(i, Item::Heading(HeadingLevel::H1, _)))
            .expect("h1");
        let page = crate::typo::PAGE as f32;
        let body = crate::typo::BODY as f32;
        assert!(markdown_item_extent(h1) <= page * 1.3 + body * 0.5 + body * 0.875 + 0.5);
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
        let settings = markdown_paint_settings(markdown_measure_style());
        let spans = markdown_document_spans(&items, &settings);
        let joined: String = spans.iter().map(|s| s.text.as_ref()).collect();
        assert!(joined.contains('A'));
        assert!(joined.contains("B line"));
        assert!(joined.contains("C line"));
        assert!(joined.contains("\n\n"));
        let start = joined.find('A').unwrap();
        let end = joined.find("C line").unwrap() + "C line".len();
        let multi = copy_range(&joined, start, end);
        assert!(multi.contains('A') && multi.contains('B') && multi.contains('C'));
        assert!(multi.matches('\n').count() >= 2);
        // Fence with no line tokens still contributes the raw code body.
        let bare: Vec<_> = markdown::parse("```\nplain code\n```").collect();
        let bare_spans = markdown_document_spans(&bare, &settings);
        let bare_joined: String = bare_spans.iter().map(|s| s.text.as_ref()).collect();
        assert!(bare_joined.contains("plain code"));
        // Empty `lines` uses the raw `code` field (host items sometimes omit lines).
        let empty_lines = [Item::CodeBlock {
            language: None,
            code: "solo".into(),
            lines: vec![],
        }];
        let empty_spans = markdown_document_spans(&empty_lines, &settings);
        let empty_joined: String = empty_spans.iter().map(|s| s.text.as_ref()).collect();
        assert_eq!(empty_joined, "solo");
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
        assert!(plain.contains("[table]"));
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

    #[test]
    fn markdown_select_spans_heading_paragraph_and_list() {
        let items: Vec<_> = markdown::parse("# Title\n\nA paragraph.\n\n- alpha\n- beta").collect();
        assert!(items.len() >= 3);
        let mut st = MarkdownSelect::default();
        st = markdown_select(&items, st, MarkdownPointer::Move(0.0));
        st = markdown_select(&items, st, MarkdownPointer::Press);
        let end_y = items.iter().map(markdown_item_extent).sum::<f32>();
        st = markdown_select(&items, st, MarkdownPointer::Move(end_y));
        st = markdown_select(&items, st, MarkdownPointer::Release);
        assert!(!st.dragging);
        assert!(st.span.start.item < st.span.end.item);
        let copied = st.span.text(&items);
        assert!(copied.contains("Title"), "{copied}");
        assert!(copied.contains("paragraph"), "{copied}");
        assert!(copied.contains("alpha") || copied.contains('•'), "{copied}");
        let mut back = MarkdownSelect::default();
        back = markdown_select(&items, back, MarkdownPointer::Move(end_y));
        back = markdown_select(&items, back, MarkdownPointer::Press);
        back = markdown_select(&items, back, MarkdownPointer::Move(0.0));
        let rev = back.span.text(&items);
        assert!(rev.contains("Title") && rev.contains("paragraph"));
        assert_eq!(
            markdown_select(&[], MarkdownSelect::default(), MarkdownPointer::Press)
                .span
                .text(&[]),
            ""
        );
        assert!(MarkdownSpan::default().is_empty());
        assert!(!st.span.covers(99));
        assert!(st.span.covers(st.span.start.item));
    }
}
