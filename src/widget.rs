//! Themed iced widget constructors for `view`.
//!
//! Every drawing constructor returns an [`iced::Element`], emits the
//! application's messages, and takes [`A11y`] plus [`Tokens`]. Rustdoc
//! on the function is the call: the job, the arguments that matter,
//! and a compiling example.
//!
//! ```
//! use icedtea::a11y::A11y;
//! use icedtea::theme;
//! use icedtea::variant::Variant;
//! use icedtea::widget;
//! let tok = theme::named("dark").tokens;
//! let _: icedtea::Element<'_, ()> = widget::themed_button(
//!     "Save",
//!     Some(()),
//!     tok,
//!     Variant::Primary,
//!     A11y::button("Save"),
//! );
//! ```

use iced::gradient::Linear;
use iced::widget::canvas::Canvas;
use iced::widget::markdown;
use iced::widget::scrollable::{Direction as ScrollDir, Scrollbar};
use iced::widget::text_editor::Content;
use iced::widget::{
    button, checkbox, column, container, mouse_area, pick_list, progress_bar, radio, row, rule,
    scrollable, slider, stack, svg, text, text_editor, text_input, toggler, tooltip, Column, Id,
    Row, Space,
};
use iced::{Alignment, Background, Color, Element, Length, Padding, Radians};

use crate::chrome::SCROLL_RAIL_WIDTH;
use crate::host_canvas::ArcRing;
use crate::scroll::ScrollRail;

use crate::a11y::{self, A11y, Role};
use crate::collection::{
    page_range, virtual_pads, visible_window, visible_window_var, window_after_scroll,
    window_after_scroll_var, Accordion, ListModel, RowFace, RowHeights, Selection, Tabs, TreeNode,
    VisibleWindow,
};
use crate::i18n::Direction;
use crate::icon::Icon;
use crate::style;
use crate::theme::Tokens;
use crate::toast::{Toast, ToastKind};
use crate::typo;
use crate::variant::Variant;

/// Shared padding for controls.
fn pad() -> Padding {
    Padding::from([8, 12])
}

pub fn icon_style(tok: Tokens) -> impl Fn(&iced::Theme, svg::Status) -> svg::Style {
    move |_t, _s| svg::Style {
        color: Some(tok.text),
    }
}

/// Paint a bundled chrome icon.
///
/// Chrome set only (`Icon::Search`, `Close`, and the rest). Tokens tint
/// the fill.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::icon::Icon;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> =
///     widget::icon_svg(Icon::Search, tok, A11y::new("search", Role::Image));
/// ```
pub fn icon_svg<'a, M: 'a>(icon: Icon, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    let handle = svg::Handle::from_memory(icon.bytes());
    a11y::attach(
        svg(handle)
            .width(16.0)
            .height(16.0)
            .style(icon_style(tok))
            .into(),
        &a11y,
    )
}

/// Cell content. Not a markup parser. [`MarkdownDoc`] stays documents.
///
/// ```
/// use icedtea::widget::RichCell;
/// let cell = RichCell::Code("len()".into());
/// assert!(matches!(cell, RichCell::Code(_)));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RichCell {
    Plain(String),
    Emphasis(String),
    Code(String),
    Link(String),
}

/// Paint a table or list cell: plain, emphasis, code, or link.
///
/// Not a markup parser. [`MarkdownDoc`] stays documents.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget::{self, RichCell};
/// let tok = theme::named("dark").tokens;
/// let cell = RichCell::Code("len()".into());
/// let _: icedtea::Element<'_, ()> =
///     widget::rich_cell(&cell, None, tok, A11y::new("len", Role::Status));
/// ```
pub fn rich_cell<'a, M: Clone + 'a>(
    cell: &RichCell,
    on_link: Option<M>,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    match cell {
        RichCell::Plain(s) => label(s.clone(), tok, a11y),
        RichCell::Emphasis(s) => {
            let s = a11y.apply_name(s.clone());
            a11y::attach(
                text(s)
                    .size(typo::BODY)
                    .color(tok.text)
                    .font(typo::UI_ITALIC)
                    .into(),
                &a11y,
            )
        }
        RichCell::Code(s) => {
            let s = a11y.apply_name(s.clone());
            a11y::attach(
                text(s)
                    .size(typo::CODE)
                    .color(tok.text)
                    .font(typo::MONO)
                    .into(),
                &a11y,
            )
        }
        RichCell::Link(s) => match on_link {
            Some(m) => hyperlink(s.clone(), m, tok, a11y),
            None => label(s.clone(), tok, a11y),
        },
    }
}

/// A line of body text.
///
/// Platform sans. Empty string is an empty node; still pass `A11y`.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> =
///     widget::label("Name", tok, A11y::new("Name", Role::Header));
/// ```
pub fn label<'a, M: 'a>(s: impl Into<String>, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    let s = a11y.apply_name(s);
    a11y::attach(
        text(s)
            .size(typo::BODY)
            .color(tok.text)
            .font(typo::UI)
            .into(),
        &a11y,
    )
}

/// A large value for a compact tool, end-aligned.
///
/// Empty string is a blank reading. `display_line` is the smaller
/// caption above it.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> =
///     widget::display_reading("24", tok, A11y::new("24", Role::Status));
/// ```
pub fn display_reading<'a, M: 'a>(s: impl Into<String>, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    let s = a11y.apply_name(s);
    a11y::attach(
        text(s)
            .size(typo::DISPLAY)
            .color(tok.text)
            .font(typo::UI_BOLD)
            .width(Length::Fill)
            .align_x(Alignment::End)
            .into(),
        &a11y,
    )
}

/// Muted end-aligned line above a display reading.
pub fn display_line<'a, M: 'a>(s: impl Into<String>, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    let s = a11y.apply_name(s);
    a11y::attach(
        text(s)
            .size(typo::META)
            .color(tok.muted)
            .font(typo::UI)
            .width(Length::Fill)
            .align_x(Alignment::End)
            .into(),
        &a11y,
    )
}

/// Segmented large figures on the type scale (clocks, meters).
pub fn figure_display<'a, M: 'a>(s: impl Into<String>, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    let s = a11y.apply_name(s);
    let mut r = Row::new().spacing(8).align_y(Alignment::Center);
    for ch in s.chars() {
        r = r.push(
            text(ch.to_string())
                .size(typo::DISPLAY)
                .font(typo::UI_BOLD)
                .color(tok.text),
        );
    }
    a11y::attach(r.into(), &a11y)
}

pub fn meta<'a, M: 'a>(s: impl Into<String>, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    let s = a11y.apply_name(s);
    a11y::attach(text(s).size(typo::META).color(tok.muted).into(), &a11y)
}

/// A monospace panel the user can drag-select and copy.
///
/// The application owns the buffer and posts `Content::selection()`
/// with [`crate::copy_text`]. Typing does not change the text.
/// Disabled still allows select-and-copy.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let content = icedtea::iced::widget::text_editor::Content::with_text("fn main() {}");
/// let on_select = |action| action;
/// let _: icedtea::Element<'_, _> = widget::code_block(
///     &content,
///     on_select,
///     tok,
///     A11y::new("src", Role::TextBox),
/// );
/// ```
pub fn code_block<'a, M: Clone + 'a>(
    content: &'a Content,
    on_action: impl Fn(text_editor::Action) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let e = text_editor(content)
        .height(Length::Shrink)
        .padding(12)
        .font(typo::MONO)
        .wrapping(iced::widget::text::Wrapping::Word)
        .style(editor_style(tok))
        .on_action(move |a| on_action(select_only(a)));
    container(e)
        .width(Length::Fill)
        .style(move |_| editor_frame(tok))
        .id(Id::from(a11y.node_id()))
        .into()
}

/// A text link that sends a message.
///
/// The application opens the URL or navigates. Disabled paints muted
/// text and drops the press.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> =
///     widget::hyperlink("docs", (), tok, A11y::new("docs", Role::Link));
/// ```
pub fn hyperlink<'a, M: Clone + 'a>(
    title: impl Into<String>,
    msg: M,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let title = a11y.apply_name(title);
    let mut b = button(text(title).size(typo::BODY).color(tok.accent))
        .padding(0)
        .style(style::button_style(tok, Variant::Ghost));
    if let Some(m) = a11y.apply_message(Some(msg)) {
        b = b.on_press(m);
    }
    a11y::attach(b.into(), &a11y)
}

/// Press a labeled control to send a message.
///
/// `title` is the face. `msg` is `None` when there is nothing to send.
/// [`A11y::button`] plus `with_disabled(true)` drops the handler.
/// `variant` picks the token wash.
///
///
/// ```
/// use icedtea::a11y::A11y;
/// use icedtea::theme;
/// use icedtea::variant::Variant;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let save = ();
/// let _: icedtea::Element<'_, ()> = widget::themed_button(
///     "Save",
///     Some(save),
///     tok,
///     Variant::Primary,
///     A11y::button("Save"),
/// );
/// ```
pub fn themed_button<'a, M: Clone + 'a>(
    title: impl Into<String>,
    msg: Option<M>,
    tok: Tokens,
    variant: Variant,
    a11y: A11y,
) -> Element<'a, M> {
    themed_button_sized(
        title,
        msg,
        tok,
        variant,
        Length::Shrink,
        Length::Shrink,
        a11y,
    )
}

/// Themed button that fills a pad cell.
pub fn themed_button_sized<'a, M: Clone + 'a>(
    title: impl Into<String>,
    msg: Option<M>,
    tok: Tokens,
    variant: Variant,
    width: Length,
    height: Length,
    a11y: A11y,
) -> Element<'a, M> {
    let label = a11y.apply_name(title);
    let mut b = button(
        text(label)
            .size(typo::BODY)
            .width(Length::Fill)
            .align_x(Alignment::Center),
    )
    .padding(pad())
    .width(width)
    .height(height)
    .style(style::button_style(tok, variant));
    if let Some(m) = a11y.apply_message(msg) {
        b = b.on_press(m);
    }
    a11y::attach(b.into(), &a11y)
}

/// A primary press plus a more menu.
///
/// `primary` is the main message. `more` opens the overflow. Disabled
/// drops both.
///
///
/// ```
/// use icedtea::a11y::A11y;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> =
///     widget::split_button("Save", (), (), tok, A11y::button("Save"));
/// ```
pub fn split_button<'a, M: Clone + 'a>(
    title: impl Into<String>,
    primary: M,
    more: M,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let title = a11y.apply_name(title);
    let primary_msg = (!a11y.disabled).then_some(primary);
    let more_msg = (!a11y.disabled).then_some(more);
    a11y::attach(
        row![
            themed_button(
                title.clone(),
                primary_msg,
                tok,
                Variant::Primary,
                A11y::button(&title).with_disabled(a11y.disabled),
            ),
            {
                let mut more = button(icon_svg(Icon::Chevron, tok, A11y::new("more", Role::Image)))
                    .padding(pad())
                    .style(style::button_style(tok, Variant::Quiet));
                if let Some(m) = a11y.apply_message(more_msg) {
                    more = more.on_press(m);
                }
                a11y::attach(
                    more.into(),
                    &A11y::button("more").with_disabled(a11y.disabled),
                )
            },
        ]
        .spacing(2)
        .into(),
        &a11y,
    )
}

/// A button that stays pressed while on.
///
/// Pass the current on/off state. The message fires on press. Disabled
/// keeps the face and drops the handler.
///
///
/// ```
/// use icedtea::a11y::A11y;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> =
///     widget::toggle_button("Bold", true, (), tok, A11y::button("Bold").with_checked(true));
/// ```
pub fn toggle_button<'a, M: Clone + 'a>(
    title: impl Into<String>,
    pressed: bool,
    msg: M,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let title = title.into();
    let a11y = A11y {
        checked: Some(pressed),
        ..a11y
    };
    themed_button(
        title,
        (!a11y.disabled).then_some(msg),
        tok,
        if a11y.apply_checked(pressed) {
            Variant::Primary
        } else {
            Variant::Quiet
        },
        a11y,
    )
}

/// Check or clear a boolean.
///
/// The application owns the bool. The message carries the next value.
/// Disabled keeps the box and ignores clicks.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_toggle = |on| on;
/// let _: icedtea::Element<'_, bool> = widget::themed_checkbox(
///     "Accept",
///     true,
///     on_toggle,
///     tok,
///     A11y::new("Accept", Role::Checkbox).with_checked(true),
/// );
/// ```
pub fn themed_checkbox<'a, M: Clone + 'a>(
    label_s: impl Into<String>,
    checked: bool,
    msg: impl Fn(bool) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let name = a11y.apply_name(label_s);
    let is_on = a11y.apply_checked(checked);
    let mut c = checkbox(is_on)
        .label(name)
        .style(style::checkbox_style(tok));
    if !a11y.disabled {
        c = c.on_toggle(msg);
    }
    a11y::attach(c.into(), &a11y)
}

/// A sliding on/off control.
///
/// Same contract as checkbox: the application owns the bool. Disabled
/// freezes the thumb.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_toggle = |on| on;
/// let _: icedtea::Element<'_, bool> = widget::themed_switch(
///     "Sounds",
///     false,
///     on_toggle,
///     tok,
///     A11y::new("Sounds", Role::Switch),
/// );
/// ```
pub fn themed_switch<'a, M: Clone + 'a>(
    label_s: impl Into<String>,
    on: bool,
    msg: impl Fn(bool) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let name = a11y.apply_name(label_s);
    let on = a11y.apply_checked(on);
    let mut t = toggler(on).label(name).style(style::switch_style(tok));
    if !a11y.disabled {
        t = t.on_toggle(msg);
    }
    a11y::attach(t.into(), &a11y)
}

/// Pick one value from a small set.
///
/// Compare the selected value to this option. Disabled rows stay in
/// the group and do not change the selection.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_pick = |v| v;
/// let _: icedtea::Element<'_, u8> = widget::themed_radio(
///     "A",
///     0u8,
///     Some(0),
///     on_pick,
///     tok,
///     A11y::new("A", Role::Radio).with_checked(true),
/// );
/// ```
pub fn themed_radio<'a, V, M: Clone + 'a>(
    label_s: impl Into<String>,
    value: V,
    selected: Option<V>,
    msg: impl Fn(V) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M>
where
    V: Copy + Eq,
{
    let name = a11y.apply_name(label_s);
    if a11y.disabled {
        let on = a11y.apply_checked(selected == Some(value));
        return a11y::attach(
            row![
                container(Space::new().width(8).height(8))
                    .width(16)
                    .height(16)
                    .center_x(16)
                    .center_y(16)
                    .style(move |_| radio_idle_face(tok, on)),
                text(name.clone()).size(typo::BODY).color(tok.muted),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .into(),
            &a11y,
        );
    }
    a11y::attach(
        radio(name, value, selected, msg)
            .style(style::radio_style(tok))
            .into(),
        &a11y,
    )
}

fn radio_idle_face(tok: Tokens, on: bool) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(iced::Background::Color(if on {
            tok.primary
        } else {
            Color::TRANSPARENT
        })),
        border: iced::border::Border {
            color: tok.muted,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..iced::widget::container::Style::default()
    }
}

/// Wheel delta in content pixels (down is positive).
pub fn scroll_delta_pixels(delta: iced::mouse::ScrollDelta, row_h: f32) -> f32 {
    match delta {
        iced::mouse::ScrollDelta::Lines { y, .. } => -y * row_h.max(0.0),
        iced::mouse::ScrollDelta::Pixels { y, .. } => -y,
    }
}

/// Horizontal wheel delta (right is positive content offset).
pub fn scroll_delta_x(delta: iced::mouse::ScrollDelta) -> f32 {
    match delta {
        iced::mouse::ScrollDelta::Lines { x, .. } => -x * 32.0,
        iced::mouse::ScrollDelta::Pixels { x, .. } => -x,
    }
}

/// Pick a number on a range.
///
/// Pass min, max, and the current value. The message is the new value
/// while the thumb moves. Disabled ignores drag.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_change = |v| v;
/// let _: icedtea::Element<'_, f32> = widget::themed_slider(
///     0.0..=1.0,
///     0.4,
///     on_change,
///     tok,
///     A11y::new("vol", Role::Slider).with_value("0.4"),
/// );
/// ```
pub fn themed_slider<'a, M: Clone + 'a>(
    range: std::ops::RangeInclusive<f32>,
    value: f32,
    msg: impl Fn(f32) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    if a11y.disabled {
        let _ = (range, value, msg);
        return a11y::attach(
            container(Space::new().width(Length::Fill).height(4))
                .width(Length::Fill)
                .height(18)
                .center_y(18)
                .style(move |_| style::fill(tok.panel, tok.muted))
                .into(),
            &a11y,
        );
    }
    a11y::attach(
        slider(range, value, msg)
            .style(style::slider_style(tok))
            .into(),
        &a11y,
    )
}

/// Percent line, with optional remaining-time text.
pub fn progress_label(value: f32, remaining: Option<&str>) -> String {
    let pct = (value.clamp(0.0, 1.0) * 100.0).round() as i32;
    match remaining {
        Some(r) if !r.is_empty() => format!("{pct}% · {r}"),
        _ => format!("{pct}%"),
    }
}

/// A determinate bar from 0 to 1.
///
/// Values outside the range clamp. No message; it is a readout.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> =
///     widget::progress(0.4, None, tok, A11y::new("p", Role::Progress).with_value("0.4"));
/// ```
pub fn progress<'a, M: 'a>(
    value: f32,
    copy: Option<&str>,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let bar = progress_bar(0.0..=1.0, value.clamp(0.0, 1.0)).style(style::progress_style(tok));
    let el = if let Some(c) = copy.filter(|s| !s.is_empty()) {
        column![bar, meta(c, tok, A11y::new(c, Role::Status))]
            .spacing(4)
            .into()
    } else {
        bar.into()
    };
    a11y::attach(el, &a11y)
}

/// Start/end radians for a determinate ring (`value` 0..=1, from 12 o'clock).
pub fn ring_angles(value: f32) -> (f32, f32) {
    let v = value.clamp(0.0, 1.0);
    let start = -std::f32::consts::FRAC_PI_2;
    (start, start + v * std::f32::consts::TAU)
}

/// Start/end radians for an indeterminate spinner (`phase` 0..=1).
pub fn spinner_angles(phase: f32) -> (f32, f32) {
    let p = phase.rem_euclid(1.0);
    let start = p * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
    (start, start + std::f32::consts::FRAC_PI_2)
}

/// Whether the determinate/indeterminate arc is long enough to stroke.
pub fn ring_should_stroke(start: f32, end: f32) -> bool {
    (end - start).abs() > 0.001
}

/// Circular progress: arc sweep follows `value`.
/// A determinate arc from 0 to 1.
///
/// Same fraction contract as [`progress`], drawn as a ring.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> = widget::progress_ring(
///     0.4, None, tok, A11y::new("pr", Role::Progress).with_value("0.4"),
/// );
/// ```
pub fn progress_ring<'a, M: 'a>(
    value: f32,
    copy: Option<&str>,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let (start, end) = ring_angles(value);
    let ring = Canvas::new(ArcRing {
        start,
        end,
        color: tok.primary,
        track: tok.panel,
    })
    .width(56)
    .height(56);
    let el = if let Some(c) = copy.filter(|s| !s.is_empty()) {
        column![ring, meta(c, tok, A11y::new(c, Role::Status))]
            .spacing(4)
            .into()
    } else {
        ring.into()
    };
    a11y::attach(el, &a11y)
}

/// Dim plus spinner over `child` when `busy`.
///
/// When `busy` is false the child is unmodified. Advance spinner
/// `phase` while true.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> = widget::busy_overlay(
///     widget::label("Doc", tok, A11y::new("Doc", Role::Status)),
///     true,
///     0.2,
///     tok,
///     A11y::new("busy", Role::Group),
/// );
/// ```
pub fn busy_overlay<'a, M: Clone + 'a>(
    child: Element<'a, M>,
    busy: bool,
    phase: f32,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    if !busy {
        return child;
    }
    a11y::attach(
        stack![
            child,
            container(spinner(tok, phase, A11y::new("busy", Role::Progress)))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(move |_| style::fill(Color::from_rgba(0.0, 0.0, 0.0, 0.35), tok.text)),
        ]
        .into(),
        &a11y,
    )
}

/// Map a series into canvas points. Empty input is empty output.
pub fn spark_points(values: &[f32], width: f32, height: f32) -> Vec<(f32, f32)> {
    if values.is_empty() || width <= 0.0 || height <= 0.0 {
        return Vec::new();
    }
    let min = values.iter().copied().fold(f32::MAX, f32::min);
    let max = values.iter().copied().fold(f32::MIN, f32::max);
    let span = (max - min).max(1e-6);
    let last = (values.len() - 1).max(1) as f32;
    values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = i as f32 / last * width;
            let y = height - (*v - min) / span * height;
            (x, y)
        })
        .collect()
}

/// One-row series. Domain plots stay in the application.
/// A tiny series chart.
///
/// Empty data paints an empty box.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> =
///     widget::sparkline(&[1.0, 3.0, 2.0], tok, A11y::new("spark", Role::Image));
/// ```
pub fn sparkline<'a, M: 'a>(values: &'a [f32], tok: Tokens, a11y: A11y) -> Element<'a, M> {
    a11y::attach(
        Canvas::new(crate::host_canvas::Sparkline {
            points: spark_points(values, 160.0, 36.0),
            color: tok.accent,
        })
        .width(160)
        .height(36)
        .into(),
        &a11y,
    )
}

/// An indeterminate quarter-arc at `phase` (0..=1).
///
/// Advance `phase` each frame while work is running.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> =
///     widget::spinner(tok, 0.2, A11y::new("spin", Role::Progress));
/// ```
pub fn spinner<'a, M: 'a>(tok: Tokens, phase: f32, a11y: A11y) -> Element<'a, M> {
    let (start, end) = spinner_angles(phase);
    a11y::attach(
        Canvas::new(ArcRing {
            start,
            end,
            color: tok.accent,
            track: tok.panel,
        })
        .width(56)
        .height(56)
        .into(),
        &a11y,
    )
}

/// Image with fit and loading/error faces.
#[derive(Clone)]
pub enum ImageSlot {
    Ready {
        handle: iced::widget::image::Handle,
        fit: iced::ContentFit,
    },
    Loading,
    Error(String),
}

/// An image slot that keeps its box.
///
/// Ready keeps the requested width and height. Missing bytes show the
/// empty slot, not a collapsed layout.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget::{self, ImageSlot};
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> = widget::image_slot(
///     ImageSlot::Loading, 120.0, 80.0, tok, A11y::new("img", Role::Image),
/// );
/// ```
pub fn image_slot<'a, M: Clone + 'a>(
    slot: ImageSlot,
    width: impl Into<Length>,
    height: impl Into<Length>,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let width = width.into();
    let height = height.into();
    let box_up = |child: Element<'a, M>| container(child).width(width).height(height);
    match slot {
        ImageSlot::Ready { handle, fit } => a11y::attach(
            container(
                iced::widget::image(handle)
                    .content_fit(fit)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .width(width)
            .height(height)
            .style(move |_| style::fill(crate::theme::chip_fill(tok), tok.text))
            .into(),
            &a11y,
        ),
        ImageSlot::Loading => a11y::attach(
            box_up(
                column![
                    container(Space::new().width(Length::Fill).height(14))
                        .style(move |_| style::skeleton(tok)),
                    container(Space::new().width(Length::Fill).height(14))
                        .style(move |_| style::skeleton(tok)),
                ]
                .spacing(8)
                .padding(12)
                .into(),
            )
            .style(move |_| style::fill(crate::theme::chip_fill(tok), tok.text))
            .into(),
            &a11y,
        ),
        ImageSlot::Error(msg) => a11y::attach(
            box_up(meta(msg.clone(), tok, A11y::new(msg, Role::Status)))
                .center_x(width)
                .center_y(height)
                .style(move |_| style::callout(tok, ToastKind::Warning))
                .into(),
            &a11y,
        ),
    }
}

/// Edit a numeric value with step buttons.
///
/// The application owns the number. Step messages bump it. Disabled
/// freezes the value.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_change = |s| s;
/// let _: icedtea::Element<'_, String> = widget::number_input(
///     3.0,
///     on_change,
///     tok,
///     A11y::new("n", Role::SpinButton),
/// );
/// ```
pub fn number_input<'a, M: Clone + 'a>(
    value: f64,
    on_change: impl Fn(String) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let shown = format!("{value}");
    let mut i = text_input("0", &shown)
        .style(style::search_style(tok))
        .padding(pad());
    if !a11y.disabled {
        i = i.on_input(on_change);
    }
    a11y::attach(i.into(), &a11y)
}

/// Step a numeric value.
pub fn step_number(value: f64, step: f64, min: f64, max: f64, dir: i32) -> f64 {
    (value + step * f64::from(dir)).clamp(min, max)
}

/// Fill `template` slots. `0` takes the next digit from `raw`; other
/// characters are literals.
pub fn apply_mask(template: &str, raw: &str) -> String {
    let mut digits = raw.chars().filter(|c| c.is_ascii_digit());
    let mut out = String::new();
    for ch in template.chars() {
        if ch == '0' {
            match digits.next() {
                Some(d) => out.push(d),
                None => break,
            }
        } else if digits.clone().next().is_some() {
            out.push(ch);
        } else {
            break;
        }
    }
    out
}

fn mask_handler<'a, M: 'a>(
    template: &'a str,
    on_input: impl Fn(String) -> M + 'a,
) -> impl Fn(String) -> M + 'a {
    move |raw| on_input(apply_mask(template, &raw))
}

/// Text field that keeps `value` on `template` (date, time, card).
/// Fill digit slots on a template (`0000-0000`).
///
/// Non-digit template characters stay put. Empty slots are blanks.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_input = |s| s;
/// let _: icedtea::Element<'_, String> = widget::masked_input(
///     "000-00-0000",
///     "",
///     on_input,
///     tok,
///     A11y::new("ssn", Role::TextBox),
/// );
/// ```
pub fn masked_input<'a, M: Clone + 'a>(
    template: &'a str,
    value: &str,
    on_input: impl Fn(String) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let shown = apply_mask(template, value);
    themed_text_input(
        template,
        &shown,
        mask_handler(template, on_input),
        None,
        tok,
        a11y,
        None,
    )
}

/// Single-line field. `input_id` is for `iced::widget::operation::focus`
/// after show.
/// A single-line editor.
///
/// Optional iced `Id` so you can `focus` after show. Disabled greys
/// the field and drops edit. Empty value is a valid state.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_input = |s| s;
/// let _: icedtea::Element<'_, String> = widget::themed_text_input(
///     "Name",
///     "",
///     on_input,
///     None,
///     tok,
///     A11y::new("Name", Role::TextBox),
///     None,
/// );
/// ```
pub fn themed_text_input<'a, M: Clone + 'a>(
    placeholder: &str,
    value: &str,
    on_input: impl Fn(String) -> M + 'a,
    on_submit: Option<M>,
    tok: Tokens,
    a11y: A11y,
    input_id: Option<Id>,
) -> Element<'a, M> {
    let mut i = text_input(placeholder, value)
        .style(style::search_style(tok))
        .padding(pad());
    if let Some(id) = input_id {
        i = i.id(id);
    }
    if !a11y.disabled {
        i = i.on_input(on_input);
        if let Some(m) = a11y.apply_message(on_submit) {
            i = i.on_submit(m);
        }
    }
    a11y::attach(i.into(), &a11y)
}

/// Text field plus a keyboard-complete pick list. The application
/// owns the suggestion strings and the pick message.
/// A text field with a pick list of completions.
///
/// The application owns the query and the suggestion list. Picking a
/// row writes that string.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let opts = ["save".into(), "open".into()];
/// let on_input = |_s: String| ();
/// let on_pick = |_i: usize| ();
/// let _: icedtea::Element<'_, ()> = widget::suggest_field(
///     "Type",
///     "",
///     on_input,
///     &opts,
///     on_pick,
///     tok,
///     A11y::new("cmd", Role::ComboBox),
/// );
/// ```
pub fn suggest_field<'a, M: Clone + 'a>(
    placeholder: &str,
    value: &str,
    on_input: impl Fn(String) -> M + 'a,
    suggestions: &'a [String],
    on_pick: impl Fn(usize) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let mut col = column![themed_text_input(
        placeholder,
        value,
        on_input,
        None,
        tok,
        a11y.child(Role::TextBox),
        None,
    )]
    .spacing(2);
    for (i, s) in suggestions.iter().enumerate() {
        col = col.push(themed_button(
            s.clone(),
            a11y.apply_message(Some(on_pick(i))),
            tok,
            Variant::Ghost,
            A11y::new(s.clone(), Role::ListItem).with_disabled(a11y.disabled),
        ));
    }
    a11y::attach(col.into(), &a11y)
}

/// A masked single-line editor.
///
/// Characters paint as dots. The application owns the string. Disabled
/// drops edit.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_input = |s| s;
/// let _: icedtea::Element<'_, String> = widget::password_input(
///     "Secret",
///     "",
///     on_input,
///     tok,
///     A11y::new("Secret", Role::TextBox),
///     true,
/// );
/// ```
pub fn password_input<'a, M: Clone + 'a>(
    placeholder: &str,
    value: &str,
    on_input: impl Fn(String) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
    masked: bool,
) -> Element<'a, M> {
    let mut i = text_input(placeholder, value)
        .secure(masked)
        .style(style::search_style(tok))
        .padding(pad());
    if !a11y.disabled {
        i = i.on_input(on_input);
    }
    a11y::attach(i.into(), &a11y)
}

/// A settings row: masked field, reveal, and copy.
///
/// Reveal toggles the mask. Copy is an [`crate::action::Action`] whose
/// message the application handles with `icedtea::copy_text`.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::action::Action;
/// use icedtea::i18n::Direction;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let copy = Action::new("secret.copy", "Copy", ());
/// let on_input = |_s: String| ();
/// let on_toggle = ();
/// let _: icedtea::Element<'_, ()> = widget::secret_field(
///     "Token",
///     "",
///     on_input,
///     false,
///     on_toggle,
///     &copy,
///     tok,
///     Direction::Ltr,
///     A11y::new("Token", Role::Group),
/// );
/// ```
#[allow(clippy::too_many_arguments)]
pub fn secret_field<'a, M: Clone + 'a>(
    placeholder: &str,
    value: &str,
    on_input: impl Fn(String) -> M + 'a,
    revealed: bool,
    on_toggle: M,
    copy: &crate::action::Action<M>,
    tok: Tokens,
    dir: Direction,
    a11y: A11y,
) -> Element<'a, M> {
    let toggle_title = if revealed { "Hide" } else { "Show" };
    let field = password_input(
        placeholder,
        value,
        on_input,
        tok,
        a11y.child(Role::TextBox),
        !revealed,
    );
    let toggle = themed_button(
        toggle_title,
        Some(on_toggle),
        tok,
        Variant::Quiet,
        A11y::button(toggle_title).with_disabled(a11y.disabled),
    );
    let copy_btn = themed_button(
        copy.title.clone(),
        copy.invoke(),
        tok,
        Variant::Quiet,
        A11y::button(copy.title.clone()).with_disabled(!copy.enabled || a11y.disabled),
    );
    let kids = crate::i18n::order(dir, [field, toggle, copy_btn]);
    let mut r = Row::new().spacing(8).align_y(Alignment::Center);
    for k in kids {
        r = r.push(k);
    }
    a11y::attach(r.into(), &a11y)
}

/// A labeled read-only value the user can select and copy.
///
/// Meta label, then [`selectable`], then an optional Copy
/// [`crate::action::Action`]. The application posts
/// [`crate::field::Selectables::copy`] with [`crate::copy_text`].
/// Mono face for paths and ids; UI face for prose. Disabled still
/// allows select-and-copy.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::action::Action;
/// use icedtea::i18n::Direction;
/// use icedtea::theme;
/// use icedtea::typo::FontFace;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let content = icedtea::iced::widget::text_editor::Content::with_text("a/b");
/// #[derive(Clone)]
/// enum Msg {
///     Select(icedtea::iced::widget::text_editor::Action),
///     Copy,
/// }
/// let on_select = Msg::Select;
/// let copy = Action::new("value.copy", "Copy", Msg::Copy);
/// let _: icedtea::Element<'_, Msg> = widget::value_field(
///     "Path",
///     &content,
///     on_select,
///     Some(&copy),
///     FontFace::Mono,
///     tok,
///     Direction::Ltr,
///     A11y::new("Path", Role::Group),
/// );
/// ```
#[allow(clippy::too_many_arguments)]
pub fn value_field<'a, M: Clone + 'a>(
    title: impl Into<String>,
    content: &'a Content,
    on_action: impl Fn(text_editor::Action) -> M + 'a,
    copy: Option<&crate::action::Action<M>>,
    face: typo::FontFace,
    tok: Tokens,
    dir: Direction,
    a11y: A11y,
) -> Element<'a, M> {
    let title = title.into();
    let label = meta(
        title.clone(),
        tok,
        a11y.child(Role::Status).with_value(title.clone()),
    );
    let value = selectable(content, on_action, tok, face, a11y.child(Role::TextBox));
    let mut kids: Vec<Element<'a, M>> = vec![label, value];
    if let Some(copy) = copy {
        kids.push(themed_button(
            copy.title.clone(),
            copy.invoke(),
            tok,
            Variant::Quiet,
            A11y::button(copy.title.clone()).with_disabled(!copy.enabled || a11y.disabled),
        ));
    }
    let kids = crate::i18n::order(dir, kids);
    let mut r = Row::new()
        .spacing(8)
        .align_y(Alignment::Center)
        .width(Length::Fill);
    for k in kids {
        r = r.push(k);
    }
    a11y::attach(r.into(), &a11y)
}

/// Multiline editor. `height` is icedtea size language ([`crate::layout::FILL`]
/// or [`crate::layout::fixed`]).
/// A multi-line editor.
///
/// Height is [`crate::layout::FILL`] or [`crate::layout::fixed`]. The
/// application owns the buffer. Disabled drops edit.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let content = icedtea::iced::widget::text_editor::Content::new();
/// let on_edit = |action| action;
/// let _: icedtea::Element<'_, _> = widget::textarea(
///     &content,
///     on_edit,
///     tok,
///     icedtea::layout::FILL,
///     A11y::new("notes", Role::TextBox),
/// );
/// ```
pub fn textarea<'a, M: Clone + 'a>(
    content: &'a Content,
    on_action: impl Fn(text_editor::Action) -> M + 'a,
    tok: Tokens,
    height: Length,
    a11y: A11y,
) -> Element<'a, M> {
    let mut e = text_editor(content)
        .height(height)
        .padding(8)
        .style(editor_style(tok));
    if !a11y.disabled {
        e = e.on_action(on_action);
    }
    // Id on this fill container. a11y::attach would wrap in Shrink and
    // compress Length::Fill to the intrinsic editor height.
    container(e)
        .width(Length::Fill)
        .height(height)
        .style(move |_| editor_frame(tok))
        .id(Id::from(a11y.node_id()))
        .into()
}

/// Keep selection, click, drag, and scroll. Typing, paste, and delete
/// become a zero scroll so `Content::perform` does not change the text.
pub fn select_only(action: text_editor::Action) -> text_editor::Action {
    if action.is_edit() {
        text_editor::Action::Scroll { lines: 0 }
    } else {
        action
    }
}

/// Body the user can drag-select and copy.
///
/// Looks like body text: zero pad, no border, canvas fill. The
/// application owns the buffer and posts `Content::selection()` with
/// [`crate::copy_text`]. Height shrinks to the text. Disabled still
/// allows select-and-copy. Use [`typo::FontFace::Ui`] for prose and
/// [`typo::FontFace::Mono`] for paths or raw values.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::typo::FontFace;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let content = icedtea::iced::widget::text_editor::Content::with_text("Hello");
/// let on_select = |action| action;
/// let _: icedtea::Element<'_, _> = widget::selectable(
///     &content,
///     on_select,
///     tok,
///     FontFace::Ui,
///     A11y::new("body", Role::TextBox),
/// );
/// ```
pub fn selectable<'a, M: Clone + 'a>(
    content: &'a Content,
    on_action: impl Fn(text_editor::Action) -> M + 'a,
    tok: Tokens,
    face: typo::FontFace,
    a11y: A11y,
) -> Element<'a, M> {
    let e = text_editor(content)
        .height(Length::Shrink)
        .padding(0)
        .font(face.font())
        .wrapping(iced::widget::text::Wrapping::Word)
        .style(selectable_style(tok))
        .on_action(move |a| on_action(select_only(a)));
    a11y::attach(e.into(), &a11y)
}

fn selectable_style(
    tok: Tokens,
) -> impl Fn(&iced::Theme, iced::widget::text_editor::Status) -> iced::widget::text_editor::Style {
    move |_t, _s| iced::widget::text_editor::Style {
        background: iced::Background::Color(tok.canvas),
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        placeholder: tok.muted,
        value: tok.text,
        selection: tok.selection,
    }
}

/// Syntax-highlighted code. `syntax` is an iced highlighter token (`rs`, `py`, …).
/// `theme_name` picks a highlighter face that fits the UI colorway.
/// `height` is icedtea size language ([`crate::layout::FILL`] or
/// [`crate::layout::fixed`]).
/// Highlighted source.
///
/// The application owns the buffer and the language name. Highlighter
/// face follows the active colorway. Typing does not change the
/// buffer. Disabled still allows select-and-copy.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let content = icedtea::iced::widget::text_editor::Content::with_text("fn main() {}");
/// let on_edit = |action| action;
/// let _: icedtea::Element<'_, _> = widget::highlighted_code(
///     &content,
///     "rs",
///     on_edit,
///     tok,
///     "dark",
///     icedtea::layout::FILL,
///     A11y::new("src", Role::TextBox),
/// );
/// ```
pub fn highlighted_code<'a, M: Clone + 'a>(
    content: &'a Content,
    syntax: &str,
    on_action: impl Fn(text_editor::Action) -> M + 'a,
    tok: Tokens,
    theme_name: &str,
    height: Length,
    a11y: A11y,
) -> Element<'a, M> {
    let theme = crate::theme::code_highlight(theme_name);
    let e = text_editor(content)
        .height(height)
        .padding(8)
        .style(editor_style(tok))
        .highlight(syntax, theme)
        .font(typo::MONO)
        .on_action(move |a| on_action(select_only(a)));
    container(e)
        .width(Length::Fill)
        .height(height)
        .style(move |_| editor_frame(tok))
        .id(Id::from(a11y.node_id()))
        .into()
}

fn editor_frame(tok: Tokens) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(iced::Background::Color(tok.panel)),
        border: iced::Border {
            color: tok.border,
            width: 1.0,
            radius: crate::chrome::Corner::Tight.radius(),
        },
        ..iced::widget::container::Style::default()
    }
}

pub fn editor_style(
    tok: Tokens,
) -> impl Fn(&iced::Theme, iced::widget::text_editor::Status) -> iced::widget::text_editor::Style {
    move |_t, _s| iced::widget::text_editor::Style {
        background: iced::Background::Color(tok.panel),
        border: iced::Border {
            color: tok.border,
            width: 1.0,
            radius: crate::chrome::Corner::Tight.radius(),
        },
        placeholder: tok.muted,
        value: tok.text,
        selection: tok.selection,
    }
}

/// A query field with a search icon.
///
/// Use for palette and list filters. Empty query means show all.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_input = |s| s;
/// let _: icedtea::Element<'_, String> =
///     widget::search_input("", on_input, tok, A11y::new("Search", Role::TextBox));
/// ```
pub fn search_input<'a, M: Clone + 'a>(
    value: &str,
    on_input: impl Fn(String) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    a11y::attach(
        row![
            icon_svg(Icon::Search, tok, A11y::new("search", Role::Image)),
            themed_text_input(
                "Search",
                value,
                on_input,
                None,
                tok,
                a11y.child(Role::TextBox),
                None,
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into(),
        &a11y,
    )
}

/// Pick one string from a list.
///
/// Placeholder shows when nothing is selected. Disabled keeps the
/// current face.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let opts = ["nord", "dark"];
/// let on_select = |name| name;
/// let _: icedtea::Element<'_, &str> = widget::themed_pick_list(
///     opts,
///     Some("nord"),
///     on_select,
///     tok,
///     A11y::new("theme", Role::ComboBox),
/// );
/// ```
pub fn themed_pick_list<'a, T, M: Clone + 'a>(
    options: impl std::borrow::Borrow<[T]> + 'a,
    selected: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
{
    if a11y.disabled {
        let _ = on_select;
        let shown = selected
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();
        return a11y::attach(
            container(text(shown).size(typo::BODY).color(tok.muted))
                .padding(pad())
                .style(move |_| style::panel(tok))
                .into(),
            &a11y,
        );
    }
    a11y::attach(
        pick_list(options, selected, on_select)
            .style(style::picker_style(tok))
            .padding(pad())
            .into(),
        &a11y,
    )
}

/// Civil date (no timezone).
///
/// ```
/// let d = icedtea::widget::DateValue { year: 2024, month: 2, day: 31 }.clamp();
/// assert_eq!(d.day, 29);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateValue {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl DateValue {
    pub fn clamp(mut self) -> Self {
        self.month = self.month.clamp(1, 12);
        let max_d = days_in_month(self.year, self.month);
        self.day = self.day.clamp(1, max_d);
        self
    }
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Pick a calendar date.
///
/// The application owns the selected day. Disabled ignores picks.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget::{self, DateValue};
/// let tok = theme::named("dark").tokens;
/// let d = DateValue {
///     year: 2026,
///     month: 8,
///     day: 10,
/// };
/// let _: icedtea::Element<'_, ()> =
///     widget::date_picker(d, (), (), tok, A11y::new("date", Role::Group));
/// ```
pub fn date_picker<'a, M: Clone + 'a>(
    value: DateValue,
    on_prev: M,
    on_next: M,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let v = value.clamp();
    let shown = format!("{:04}-{:02}-{:02}", v.year, v.month, v.day);
    a11y::attach(
        row![
            themed_button(
                "<",
                a11y.apply_message(Some(on_prev)),
                tok,
                Variant::Quiet,
                A11y::button("previous-day").with_disabled(a11y.disabled),
            ),
            label(
                shown.clone(),
                tok,
                a11y.child(Role::Status).with_value(shown),
            ),
            themed_button(
                ">",
                a11y.apply_message(Some(on_next)),
                tok,
                Variant::Quiet,
                A11y::button("next-day").with_disabled(a11y.disabled),
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into(),
        &a11y,
    )
}

/// Civil time of day (0.2). Storage is always 24-hour; [`TimeClock`] is display.
///
/// ```
/// use icedtea::widget::{TimeClock, TimeField, TimeValue};
/// let t = TimeValue::hm(9, 30);
/// assert_eq!(t.hour12(), 9);
/// assert!(!t.afternoon());
/// assert_eq!(t.step_field(TimeField::Hour, TimeClock::HOUR12).hour, 10);
/// assert_eq!(TimeValue::hm(0, 0).hour12(), 12);
/// assert_eq!(TimeValue::hm(13, 5).toggle_period().hour, 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TimeValue {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

/// Display for [`time_picker`]. Does not change how [`TimeValue`] is stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeClock {
    pub hour12: bool,
    pub seconds: bool,
}

impl TimeClock {
    pub const HOURS_MINUTES: Self = Self {
        hour12: false,
        seconds: false,
    };
    pub const HOURS_MINUTES_SECONDS: Self = Self {
        hour12: false,
        seconds: true,
    };
    pub const HOUR12: Self = Self {
        hour12: true,
        seconds: false,
    };
    pub const HOUR12_SECONDS: Self = Self {
        hour12: true,
        seconds: true,
    };
}

/// Which stepper [`time_picker`] pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeField {
    Hour,
    Minute,
    Second,
    Period,
}

impl TimeValue {
    pub fn hm(hour: u8, minute: u8) -> Self {
        Self {
            hour,
            minute,
            second: 0,
        }
        .clamp()
    }

    pub fn hms(hour: u8, minute: u8, second: u8) -> Self {
        Self {
            hour,
            minute,
            second,
        }
        .clamp()
    }

    pub fn clamp(self) -> Self {
        Self {
            hour: self.hour.min(23),
            minute: self.minute.min(59),
            second: self.second.min(59),
        }
    }

    /// 1–12 for a 12-hour face. Midnight and noon are 12.
    pub fn hour12(self) -> u8 {
        match self.clamp().hour % 12 {
            0 => 12,
            n => n,
        }
    }

    pub fn afternoon(self) -> bool {
        self.clamp().hour >= 12
    }

    pub fn step_hour(self, dir: i32) -> Self {
        let v = self.clamp();
        let hour = (i32::from(v.hour) + dir).rem_euclid(24) as u8;
        Self { hour, ..v }
    }

    pub fn step_minute(self, dir: i32) -> Self {
        let v = self.clamp();
        let minute = (i32::from(v.minute) + dir).rem_euclid(60) as u8;
        Self { minute, ..v }
    }

    pub fn step_second(self, dir: i32) -> Self {
        let v = self.clamp();
        let second = (i32::from(v.second) + dir).rem_euclid(60) as u8;
        Self { second, ..v }
    }

    /// Step the 1–12 face; keep morning or afternoon.
    pub fn step_hour12(self, dir: i32) -> Self {
        let v = self.clamp();
        let index = (i32::from(v.hour12() % 12) + dir).rem_euclid(12) as u8;
        let hour = if v.afternoon() { index + 12 } else { index };
        Self { hour, ..v }
    }

    pub fn toggle_period(self) -> Self {
        let v = self.clamp();
        let hour = if v.afternoon() {
            v.hour - 12
        } else {
            v.hour + 12
        };
        Self { hour, ..v }
    }

    pub fn step_field(self, field: TimeField, clock: TimeClock) -> Self {
        match field {
            TimeField::Hour if clock.hour12 => self.step_hour12(1),
            TimeField::Hour => self.step_hour(1),
            TimeField::Minute => self.step_minute(1),
            TimeField::Second => self.step_second(1),
            TimeField::Period => self.toggle_period(),
        }
    }
}

fn time_colon<'a, M: 'a>(tok: Tokens) -> Element<'a, M> {
    container(
        text(":")
            .size(typo::BODY)
            .font(typo::UI_BOLD)
            .color(tok.text),
    )
    .padding(Padding {
        top: 8.0,
        right: 2.0,
        bottom: 8.0,
        left: 2.0,
    })
    .into()
}

/// Step hour, minute, second, or period on a 24-hour value.
///
/// `TimeValue` is the clock. `TimeClock` is display only. Disabled
/// freezes the fields.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget::{self, TimeClock, TimeValue};
/// let tok = theme::named("dark").tokens;
/// let t = TimeValue::hm(9, 30);
/// let on_field = |field| field;
/// let _: icedtea::Element<'_, _> = widget::time_picker(
///     t,
///     TimeClock::HOURS_MINUTES,
///     on_field,
///     tok,
///     A11y::new("time", Role::Group),
/// );
/// ```
pub fn time_picker<'a, M: Clone + 'a>(
    value: TimeValue,
    clock: TimeClock,
    on_field: impl Fn(TimeField) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let v = value.clamp();
    let hour = if clock.hour12 {
        format!("{:02}", v.hour12())
    } else {
        format!("{:02}", v.hour)
    };
    let mut row = Row::new().spacing(4).align_y(Alignment::Center);
    row = row.push(themed_button(
        hour,
        a11y.apply_message(Some(on_field(TimeField::Hour))),
        tok,
        Variant::Quiet,
        A11y::button("hour").with_disabled(a11y.disabled),
    ));
    row = row.push(time_colon(tok));
    row = row.push(themed_button(
        format!("{:02}", v.minute),
        a11y.apply_message(Some(on_field(TimeField::Minute))),
        tok,
        Variant::Quiet,
        A11y::button("minute").with_disabled(a11y.disabled),
    ));
    if clock.seconds {
        row = row.push(time_colon(tok));
        row = row.push(themed_button(
            format!("{:02}", v.second),
            a11y.apply_message(Some(on_field(TimeField::Second))),
            tok,
            Variant::Quiet,
            A11y::button("second").with_disabled(a11y.disabled),
        ));
    }
    if clock.hour12 {
        row = row.push(themed_button(
            if v.afternoon() { "PM" } else { "AM" },
            a11y.apply_message(Some(on_field(TimeField::Period))),
            tok,
            Variant::Quiet,
            A11y::button("period").with_disabled(a11y.disabled),
        ));
    }
    a11y::attach(row.into(), &a11y)
}

/// A swatch that opens a color pick.
///
/// The application owns the `Color`. Disabled keeps the swatch and
/// drops the pick.
///
///
/// ```
/// use icedtea::a11y::A11y;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> =
///     widget::color_swatch(1, 120, 212, (), tok, A11y::button("color"));
/// ```
pub fn color_swatch<'a, M: Clone + 'a>(
    r: u8,
    g: u8,
    b: u8,
    msg: M,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let mut area = mouse_area(
        container(Space::new().width(28).height(18))
            .style(move |_| style::fill(iced::Color::from_rgb8(r, g, b), tok.text))
            .padding(2),
    );
    if let Some(m) = a11y.apply_message(Some(msg)) {
        area = area.on_press(m);
    }
    a11y::attach(area.into(), &a11y)
}

/// Parsed markdown the application owns. Parse in `update` (or a `Task`);
/// [`markdown_view`] borrows the items. Truncation is slicing `source`
/// before parse.
///
/// ```
/// let doc = icedtea::widget::MarkdownDoc::parse("# Hi\n\nA paragraph.");
/// assert!(!doc.items.is_empty());
/// let again = icedtea::widget::parse("# Hi\n\nA paragraph.");
/// assert_eq!(doc.hash, again.hash);
/// assert_ne!(doc.hash, icedtea::widget::parse("# Other").hash);
/// let cut = icedtea::widget::parse(&"# title\n\nbody".repeat(8)[..20]);
/// assert!(cut.source.len() <= 20);
/// ```
#[derive(Debug, Clone)]
pub struct MarkdownDoc {
    pub source: String,
    pub hash: u64,
    pub items: Vec<markdown::Item>,
}

/// One heading from [`MarkdownDoc::headings`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MdHeading {
    pub index: usize,
    pub level: u8,
    pub title: String,
}

impl MarkdownDoc {
    pub fn parse(source: impl Into<String>) -> Self {
        parse(&source.into())
    }

    /// Headings in document order. The application owns jump history.
    pub fn headings(&self) -> Vec<MdHeading> {
        let style = markdown_measure_style();
        self.items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                let markdown::Item::Heading(level, text) = item else {
                    return None;
                };
                let title: String = text.spans(style).iter().map(|s| s.text.as_ref()).collect();
                let level = match level {
                    markdown::HeadingLevel::H1 => 1,
                    markdown::HeadingLevel::H2 => 2,
                    markdown::HeadingLevel::H3 => 3,
                    markdown::HeadingLevel::H4 => 4,
                    markdown::HeadingLevel::H5 => 5,
                    markdown::HeadingLevel::H6 => 6,
                };
                Some(MdHeading {
                    index,
                    level,
                    title,
                })
            })
            .collect()
    }

    /// Estimated Y of item `index` in the default markdown column.
    /// Pass to `scroll_to` on the document scroller after an outline jump.
    pub fn item_offset(&self, index: usize) -> f32 {
        self.items
            .iter()
            .take(index)
            .map(markdown_item_extent)
            .sum()
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

fn markdown_text_len(text: &markdown::Text) -> usize {
    text.spans(markdown_measure_style())
        .iter()
        .map(|s| s.text.len())
        .sum()
}

fn markdown_item_extent(item: &markdown::Item) -> f32 {
    const TEXT: f32 = 16.0;
    const SPACING: f32 = 16.0 * 0.875;
    const COL: f32 = 64.0;
    match item {
        markdown::Item::Heading(level, text) => {
            let size = match level {
                markdown::HeadingLevel::H1 => TEXT * 2.0,
                markdown::HeadingLevel::H2 => TEXT * 1.75,
                markdown::HeadingLevel::H3 => TEXT * 1.5,
                markdown::HeadingLevel::H4 => TEXT * 1.25,
                markdown::HeadingLevel::H5 | markdown::HeadingLevel::H6 => TEXT,
            };
            let lines = ((markdown_text_len(text) as f32) / COL).ceil().max(1.0);
            size * 1.3 * lines + TEXT * 0.5 + SPACING
        }
        markdown::Item::Paragraph(text) => {
            let lines = ((markdown_text_len(text) as f32) / COL).ceil().max(1.0);
            lines * TEXT * 1.4 + SPACING
        }
        markdown::Item::CodeBlock { code, lines, .. } => {
            let n = lines.len().max(code.lines().count()).max(1) as f32;
            n * TEXT * 0.75 * 1.5 + 24.0 + SPACING
        }
        markdown::Item::List { bullets, .. } => {
            bullets
                .iter()
                .map(|b| {
                    let kids = match b {
                        markdown::Bullet::Point { items }
                        | markdown::Bullet::Task { items, .. } => items,
                    };
                    TEXT + kids.iter().map(markdown_item_extent).sum::<f32>()
                })
                .sum::<f32>()
                + SPACING
        }
        markdown::Item::Image { .. } => 160.0 + SPACING,
        markdown::Item::Quote(items) => {
            items.iter().map(markdown_item_extent).sum::<f32>() + 16.0 + SPACING
        }
        markdown::Item::Rule => 24.0 + SPACING,
        markdown::Item::Table { rows, .. } => (1 + rows.len()) as f32 * TEXT * 1.8 + SPACING,
    }
}

/// Jump list of headings. The application owns history. `selected` is
/// the heading's item index from [`MdHeading::index`].
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let doc = widget::parse("# Hi\n\n## Next");
/// let heads = doc.headings();
/// let _: icedtea::Element<'_, usize> = widget::markdown_outline(
///     &heads,
///     Some(heads[0].index),
///     |i| i,
///     tok,
///     A11y::new("outline", Role::List),
/// );
/// ```
pub fn markdown_outline<'a, M: Clone + 'a>(
    headings: &'a [MdHeading],
    selected: Option<usize>,
    on_jump: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let mut col = Column::new().spacing(2);
    for h in headings {
        let pad = 8 + u16::from(h.level.saturating_sub(1)) * 8;
        let on = selected == Some(h.index);
        col = col.push(
            container(themed_button(
                h.title.clone(),
                a11y.apply_message(Some(on_jump(h.index))),
                tok,
                if on { Variant::Quiet } else { Variant::Ghost },
                A11y::button(h.title.clone())
                    .with_checked(on)
                    .with_disabled(a11y.disabled),
            ))
            .padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: f32::from(pad),
            }),
        );
    }
    if headings.is_empty() {
        col = col.push(meta(
            "No headings",
            tok,
            A11y::new("No headings", Role::Status),
        ));
    }
    a11y::attach(col.into(), &a11y)
}

/// Pure parse: source hash plus iced markdown items.
pub fn parse(source: &str) -> MarkdownDoc {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hasher);
    MarkdownDoc {
        source: source.to_string(),
        hash: hasher.finish(),
        items: markdown::parse(source).collect(),
    }
}

/// A parsed markdown document.
///
/// Parse with [`parse`], then view the items. Truncate by slicing the
/// source before parse. Copy the source with [`crate::copy_text`] on
/// [`MarkdownDoc::source`]. The painted tree has no drag selection.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let doc = widget::parse("# Hi");
/// let on_link = |uri| uri;
/// let _: icedtea::Element<'_, _> =
///     widget::markdown_view(&doc.items, tok, on_link, A11y::new("md", Role::Group));
/// ```
pub fn markdown_view<'a, M: Clone + 'a>(
    items: &'a [markdown::Item],
    tok: Tokens,
    on_link: impl Fn(markdown::Uri) -> M + 'a,
    a11y: A11y,
) -> Element<'a, M> {
    a11y::attach(
        markdown::view(items, markdown::Settings::with_style(markdown_style(tok))).map(on_link),
        &a11y,
    )
}

fn markdown_style(tok: Tokens) -> markdown::Style {
    let mut style = markdown::Style::from_palette(iced::theme::Palette {
        background: tok.canvas,
        text: tok.text,
        primary: tok.primary,
        success: tok.success,
        warning: tok.warning,
        danger: tok.danger,
    });
    style.font = typo::UI;
    style.inline_code_color = tok.text;
    style.inline_code_font = typo::MONO;
    style.code_block_font = typo::MONO;
    style.link_color = tok.accent;
    style.inline_code_highlight.background = iced::Background::Color(tok.panel);
    style
}

/// Hover text on a child.
///
/// Empty tip text is a no-op wrap. The child keeps its own `A11y`.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> = widget::tooltip_wrap(
///     widget::label("Hover", tok, A11y::new("Hover", Role::Header)),
///     "Tip",
///     tok,
///     A11y::new("Tip", Role::Tooltip),
/// );
/// ```
pub fn tooltip_wrap<'a, M: 'a>(
    child: Element<'a, M>,
    tip: impl Into<String>,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let tip = a11y.apply_name(tip);
    a11y::attach(
        tooltip(
            child,
            container(meta(tip.clone(), tok, A11y::new(tip, Role::Tooltip)))
                .padding(6)
                .style(tip_style(tok)),
            tooltip::Position::FollowCursor,
        )
        .into(),
        &a11y,
    )
}

/// A horizontal divider.
///
/// `rule_v` is the vertical twin.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> =
///     widget::rule_h(tok, A11y::new("rule", Role::Separator));
/// ```
pub fn rule_h<'a, M: 'a>(tok: Tokens, a11y: A11y) -> Element<'a, M> {
    a11y::attach(
        rule::horizontal(1).style(style::rule_style(tok)).into(),
        &a11y,
    )
}

/// Compact square close control. Same size in toasts, tabs, and chips.
pub fn dismiss_button<'a, M: Clone + 'a>(msg: M, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    let mut b = button(icon_svg(Icon::Close, tok, A11y::new("close", Role::Image)))
        .padding(4)
        .style(style::button_style(tok, Variant::Ghost));
    if let Some(m) = a11y.apply_message(Some(msg)) {
        b = b.on_press(m);
    }
    a11y::attach(b.into(), &a11y)
}

fn chip_wash(tok: Tokens, variant: Variant) -> iced::Color {
    match variant {
        Variant::Primary => crate::theme::selection_fill(tok),
        Variant::Danger => crate::theme::mix(tok.danger, tok.canvas, 0.28),
        Variant::Success => crate::theme::mix(tok.success, tok.canvas, 0.28),
        Variant::Warning => crate::theme::mix(tok.warning, tok.canvas, 0.28),
        Variant::Quiet | Variant::Ghost | Variant::Chip => crate::theme::chip_fill(tok),
    }
}

/// A compact labeled pill.
///
/// Optional press message. Disabled keeps the face.
///
///
/// ```
/// use icedtea::a11y::A11y;
/// use icedtea::theme;
/// use icedtea::variant::Variant;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> =
///     widget::chip("Rust", None, tok, Variant::Quiet, A11y::button("Rust"));
/// ```
pub fn chip<'a, M: Clone + 'a>(
    title: impl Into<String>,
    dismiss: Option<M>,
    tok: Tokens,
    variant: Variant,
    a11y: A11y,
) -> Element<'a, M> {
    let title = a11y.apply_name(title);
    let ink = match variant {
        Variant::Danger => tok.danger,
        _ => tok.text,
    };
    let mut line = row![text(title.clone()).size(typo::META).color(ink)]
        .spacing(4)
        .align_y(Alignment::Center);
    if let Some(msg) = dismiss {
        line = line.push(dismiss_button(
            msg,
            tok,
            A11y::button(format!("dismiss {title}")).with_disabled(a11y.disabled),
        ));
    }
    let wash = chip_wash(tok, variant);
    a11y::attach(
        container(line)
            .padding([4, 8])
            .style(move |_| style::fill(wash, ink))
            .into(),
        &a11y,
    )
}

/// A count or status mark.
///
/// Short text. Empty string is an empty pill.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::variant::Variant;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> =
///     widget::badge("New", tok, Variant::Primary, A11y::new("New", Role::Status));
/// ```
pub fn badge<'a, M: 'a>(
    title: impl Into<String>,
    tok: Tokens,
    variant: Variant,
    a11y: A11y,
) -> Element<'a, M> {
    let title = a11y.apply_name(title);
    let ink = match variant {
        Variant::Danger => tok.danger,
        Variant::Primary => tok.primary,
        Variant::Success => tok.success,
        Variant::Warning => tok.warning,
        _ => tok.muted,
    };
    let wash = chip_wash(tok, variant);
    a11y::attach(
        container(text(title).size(typo::META).color(ink))
            .padding([4, 8])
            .style(move |_| style::fill(wash, ink))
            .into(),
        &a11y,
    )
}

/// A titled panel around children.
///
/// Empty title is a border only. Same constructor paints a card.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> = widget::group_box(
///     "Document",
///     widget::label("notes.txt", tok, A11y::new("notes", Role::Header)),
///     tok,
///     A11y::new("Document", Role::Group),
/// );
/// ```
pub fn group_box<'a, M: 'a>(
    title: impl Into<String>,
    child: Element<'a, M>,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let title = a11y.apply_name(title);
    a11y::attach(
        container(
            column![
                meta(title.clone(), tok, A11y::new(title, Role::Header)),
                child
            ]
            .spacing(8),
        )
        .padding(12)
        .width(Length::Fill)
        .style(move |_| style::card(tok, false))
        .into(),
        &a11y,
    )
}

/// A page-level message with an optional action.
///
/// Use for “offline” or “update available”. Optional button message.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> = widget::banner(
///     "Update available",
///     Some(("Install".into(), ())),
///     tok,
///     A11y::new("Update available", Role::Status),
/// );
/// ```
pub fn banner<'a, M: Clone + 'a>(
    text_s: impl Into<String>,
    action: Option<(String, M)>,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let text_s = a11y.apply_name(text_s);
    let mut r = row![label(text_s.clone(), tok, A11y::new(text_s, Role::Status))]
        .spacing(12)
        .align_y(Alignment::Center);
    if let Some((t, m)) = action {
        r = r.push(themed_button(
            t.clone(),
            a11y.apply_message(Some(m)),
            tok,
            Variant::Quiet,
            A11y::button(t).with_disabled(a11y.disabled),
        ));
    }
    a11y::attach(
        container(r)
            .width(Length::Fill)
            .padding(12)
            .style(move |_| style::callout(tok, ToastKind::Info))
            .into(),
        &a11y,
    )
}

/// An inline info bar.
///
/// Tone comes from `Variant`. Empty body is title only.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::toast::ToastKind;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> = widget::info_bar(
///     ToastKind::Warning, "Watch this", tok, A11y::new("Watch this", Role::Status),
/// );
/// ```
pub fn info_bar<'a, M: Clone + 'a>(
    kind: ToastKind,
    text_s: impl Into<String>,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let text_s = a11y.apply_name(text_s);
    a11y::attach(
        container(label(text_s.clone(), tok, A11y::new(text_s, Role::Status)))
            .width(Length::Fill)
            .padding(10)
            .style(move |_| style::callout(tok, kind))
            .into(),
        &a11y,
    )
}

/// A path of links.
///
/// Crumbs before the last send a message. The last crumb is the
/// current page. Empty path is empty.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::i18n::Direction;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let home = ();
/// let crumbs = [("Home".into(), Some(home)), ("Notes".into(), None)];
/// let _: icedtea::Element<'_, ()> = widget::breadcrumb(
///     &crumbs,
///     tok,
///     Direction::Ltr,
///     A11y::new("path", Role::Group),
/// );
/// ```
pub fn breadcrumb<'a, M: Clone + 'a>(
    parts: &[(String, Option<M>)],
    tok: Tokens,
    dir: Direction,
    a11y: A11y,
) -> Element<'a, M> {
    let parts = crate::i18n::order(dir, parts.iter().cloned());
    let mut r = Row::new().spacing(6).align_y(Alignment::Center);
    for (i, (name, msg)) in parts.iter().enumerate() {
        if i > 0 {
            r = r.push(meta("/", tok, A11y::new("/", Role::Separator)));
        }
        if let Some(m) = msg.clone() {
            r = r.push(hyperlink(
                name.clone(),
                m,
                tok,
                A11y::new(name.clone(), Role::Link).with_disabled(a11y.disabled),
            ));
        } else {
            r = r.push(label(
                name.clone(),
                tok,
                A11y::new(name.clone(), Role::Header),
            ));
        }
    }
    a11y::attach(r.into(), &a11y)
}

/// A transient notice.
///
/// The application owns the queue and dismiss. Empty queue paints
/// nothing.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::toast::{Toast, ToastKind};
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let t = Toast { id: 1, kind: ToastKind::Success, text: "Saved".into(), ttl_ms: 0 };
/// let _: icedtea::Element<'_, ()> =
///     widget::toast_view(&t, (), tok, A11y::new("Saved", Role::Status));
/// ```
pub fn toast_view<'a, M: Clone + 'a>(
    toast: &Toast,
    dismiss: M,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let kind = toast.kind;
    let text_s = toast.text.clone();
    a11y::attach(
        container(
            row![
                label(text_s.clone(), tok, A11y::new(text_s, Role::Status),),
                Space::new().width(Length::Fill),
                dismiss_button(
                    dismiss,
                    tok,
                    A11y::button("dismiss").with_disabled(a11y.disabled),
                ),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([8, 12])
        .style(toast_style(tok, kind))
        .into(),
        &a11y,
    )
}

fn tip_style(tok: Tokens) -> impl Fn(&iced::Theme) -> iced::widget::container::Style {
    move |_| style::raised_card(tok)
}

fn toast_style(
    tok: Tokens,
    kind: ToastKind,
) -> impl Fn(&iced::Theme) -> iced::widget::container::Style {
    move |_| style::callout(tok, kind)
}

/// A one-shot hint next to a control.
///
/// The application owns dismissed state. Empty body hides the tip.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> = widget::teaching_tip(
///     "Hint", "Press Ctrl+P", (), tok, A11y::new("Hint", Role::Tooltip),
/// );
/// ```
pub fn teaching_tip<'a, M: Clone + 'a>(
    title: impl Into<String>,
    body: impl Into<String>,
    dismiss: M,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let title = a11y.apply_name(title);
    a11y::attach(
        container(
            column![
                label(title.clone(), tok, A11y::new(title, Role::Header)),
                meta(body, tok, A11y::new("tip-body", Role::Status)),
                themed_button(
                    "Got it",
                    a11y.apply_message(Some(dismiss)),
                    tok,
                    Variant::Primary,
                    A11y::button("Got it").with_disabled(a11y.disabled),
                ),
            ]
            .spacing(8),
        )
        .padding(12)
        .style(move |_| style::raised_card(tok))
        .into(),
        &a11y,
    )
}

/// A placeholder block while content loads.
///
/// Size the box. No message.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> =
///     widget::placeholder_skeleton(tok, A11y::new("skel", Role::Status));
/// ```
pub fn placeholder_skeleton<'a, M: 'a>(tok: Tokens, a11y: A11y) -> Element<'a, M> {
    a11y::attach(
        column![
            container(Space::new().width(Length::Fill).height(14))
                .style(move |_| style::skeleton(tok)),
            container(Space::new().width(Length::Fill).height(14))
                .style(move |_| style::skeleton(tok)),
        ]
        .spacing(8)
        .into(),
        &a11y,
    )
}

/// A themed scroller with a usable handle.
///
/// `stick` pins to the end. `scroll_id` is for `scroll_to`. `on_scroll`
/// sees iced's viewport when the offset moves.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// let tok = icedtea::theme::named("dark").tokens;
/// let child = icedtea::widget::label::<()>("line", tok, A11y::new("line", Role::Status));
/// let _: icedtea::Element<'_, ()> = icedtea::widget::themed_scroll(
///     child,
///     tok,
///     A11y::new("scroll", Role::Group),
///     true,
///     None,
///     None::<fn(_) -> ()>,
/// );
/// ```
pub fn themed_scroll<'a, M, F>(
    child: Element<'a, M>,
    tok: Tokens,
    a11y: A11y,
    stick: bool,
    scroll_id: Option<Id>,
    on_scroll: Option<F>,
) -> Element<'a, M>
where
    M: 'a,
    F: Fn(iced::widget::scrollable::Viewport) -> M + 'a,
{
    let mut s = scrollable(child)
        .height(Length::Fill)
        .direction(ScrollDir::Vertical(
            Scrollbar::new()
                .width(SCROLL_RAIL_WIDTH)
                .scroller_width(SCROLL_RAIL_WIDTH),
        ))
        .style(style::scroll_style(tok));
    if stick {
        s = s.anchor_bottom();
    }
    if let Some(id) = scroll_id {
        s = s.id(id);
    }
    if let Some(f) = on_scroll {
        s = s.on_scroll(f);
    }
    a11y::attach(s.into(), &a11y)
}

/// Append-only lines. Sticks to the end. `window` virtualizes long logs.
#[allow(clippy::too_many_arguments)]
/// Append-only lines that stick to the end.
///
/// Virtualizes long logs. Empty lines show “No lines”.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::collection::VisibleWindow;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let lines = ["boot".to_string()];
/// let on_scroll = |w: VisibleWindow| w;
/// let _: icedtea::Element<'_, VisibleWindow> = widget::log_view(
///     &lines,
///     VisibleWindow::new(200.0),
///     20.0,
///     2,
///     on_scroll,
///     None,
///     tok,
///     A11y::new("log", Role::List),
/// );
/// ```
pub fn log_view<'a, M: Clone + 'a>(
    lines: &'a [String],
    window: VisibleWindow,
    row_h: f32,
    overscan: usize,
    on_scroll: impl Fn(VisibleWindow) -> M + Copy + 'a,
    scroll_id: Option<Id>,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let n = lines.len();
    let h = row_h.max(0.0);
    let viewport = window.viewport.max(1.0);
    // Stick uses iced Anchor::End: raw offset 0 is the tail. Mount that
    // tail until on_scroll reports the reversed (visual) offset.
    let scroll = if window.end == 0 {
        crate::layout::end_offset(n as f32 * h, viewport)
    } else {
        window.scroll.max(0.0)
    };
    let (top, win, bot) = virtual_pads(n, h, scroll, viewport, overscan, None);
    let mut col = Column::new().spacing(0);
    if n == 0 {
        col = col.push(meta("No lines", tok, A11y::new("No lines", Role::Status)));
    } else {
        col = col.push(Space::new().height(Length::Fixed(top)));
        for i in win.range() {
            let line = lines.get(i).map(String::as_str).unwrap_or("");
            col = col.push(
                container(
                    text(line.to_string())
                        .size(typo::META)
                        .color(tok.text)
                        .font(typo::MONO),
                )
                .width(Length::Fill)
                .height(h)
                .padding([2, 8]),
            );
        }
        col = col.push(Space::new().height(Length::Fixed(bot)));
    }
    let prev = window;
    themed_scroll(
        col.into(),
        tok,
        a11y,
        true,
        scroll_id,
        Some(move |vp: iced::widget::scrollable::Viewport| {
            on_scroll(window_after_scroll(
                prev,
                vp.absolute_offset_reversed().y,
                vp.bounds().height,
                h,
                n,
                overscan,
                None,
            ))
        }),
    )
}

/// Clip pane + rail. `scroll` is the only offset. `rows` paints the
/// mounted window; this shifts by row offset minus `scroll`.
#[allow(clippy::too_many_arguments)]
fn virtual_clip<'a, M, F, G>(
    prev: VisibleWindow,
    heights: RowHeights<'a>,
    len: usize,
    overscan: usize,
    cover: Option<usize>,
    on_scroll: F,
    scroll_id: Option<Id>,
    tok: Tokens,
    rows: G,
) -> Element<'a, M>
where
    M: Clone + 'a,
    F: Fn(VisibleWindow) -> M + Copy + 'a,
    G: Fn(VisibleWindow) -> Column<'a, M> + 'a,
{
    let content = heights.total(len);
    let step = heights.at(0).max(1.0);
    iced::widget::responsive(move |size| {
        let viewport = if size.height > 0.0 {
            size.height
        } else {
            prev.viewport.max(1.0)
        };
        let win = match heights {
            RowHeights::Uniform(h) => {
                visible_window(prev.scroll, viewport, h.max(0.0), len, overscan, cover)
            }
            RowHeights::PerRow(hs) => {
                visible_window_var(prev.scroll, viewport, hs, overscan, cover)
            }
        };
        let shift = heights.offset(win.start) - prev.scroll;
        let emit = move |y: f32| {
            on_scroll(match heights {
                RowHeights::Uniform(h) => {
                    window_after_scroll(prev, y, viewport, h.max(0.0), len, overscan, cover)
                }
                RowHeights::PerRow(hs) => {
                    window_after_scroll_var(prev, y, viewport, hs, overscan, cover)
                }
            })
        };
        let mut frame = container(rows(win))
            .width(crate::layout::FILL)
            .height(crate::layout::FILL)
            .padding(Padding {
                top: shift,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            })
            .clip(true);
        if let Some(id) = scroll_id.clone() {
            frame = frame.id(id);
        }
        let pane = mouse_area(frame).on_scroll(move |delta| {
            let max_s = (content - viewport).max(0.0);
            emit((prev.scroll + scroll_delta_pixels(delta, step)).clamp(0.0, max_s))
        });
        row![
            pane,
            Element::from(ScrollRail::new(content, viewport, prev.scroll, emit, tok)),
        ]
        .width(crate::layout::FILL)
        .height(crate::layout::FILL)
        .into()
    })
    .into()
}

fn two_line_row<'a, M: 'a>(
    title: &str,
    meta_s: Option<&str>,
    meta_color: iced::Color,
    selected: bool,
    row_h: f32,
    tok: Tokens,
) -> Element<'a, M> {
    let mut col = column![text(title.to_string())
        .size(typo::BODY)
        .color(tok.text)
        .font(typo::UI)
        .wrapping(iced::widget::text::Wrapping::None)]
    .spacing(2);
    if let Some(m) = meta_s.filter(|s| !s.is_empty()) {
        col = col.push(
            text(m.to_string())
                .size(typo::META)
                .color(meta_color)
                .wrapping(iced::widget::text::Wrapping::None),
        );
    }
    container(col)
        .width(Length::Fill)
        .height(row_h)
        .padding(8)
        .clip(true)
        .style(move |_| style::list_row(tok, selected))
        .into()
}

fn card_row<'a, M: 'a>(
    title: &str,
    meta_s: Option<&str>,
    meta_color: iced::Color,
    selected: bool,
    row_h: f32,
    meter: Option<f32>,
    tok: Tokens,
) -> Element<'a, M> {
    let mut col = column![text(title.to_string())
        .size(typo::BODY)
        .color(tok.text)
        .font(if selected { typo::UI_BOLD } else { typo::UI })
        .width(Length::Fill)
        .wrapping(iced::widget::text::Wrapping::Word)]
    .spacing(2)
    .width(Length::Fill);
    if let Some(m) = meta_s.filter(|s| !s.is_empty()) {
        col = col.push(
            text(m.to_string())
                .size(typo::META)
                .color(meta_color)
                .width(Length::Fill),
        );
    }
    if let Some(v) = meter {
        col = col.push(
            progress_bar(0.0..=1.0, v.clamp(0.0, 1.0))
                .girth(3)
                .style(style::progress_style(tok)),
        );
    }
    container(col)
        .width(Length::Fill)
        .height(row_h)
        .padding(Padding {
            top: 8.0,
            right: 12.0,
            bottom: 8.0,
            left: 12.0,
        })
        .clip(true)
        .style(move |_| style::card(tok, selected))
        .into()
}

/// A virtualized row list.
///
/// `empty` is the copy when `model` has no rows. `meta_color` paints
/// the second line. `scroll` is the only offset: the rail and the
/// wheel write it. Uniform rows sit at `i * row_h - scroll`. Variable
/// rows use [`RowHeights::PerRow`] and [`visible_window_var`].
/// `face` is [`RowFace::Flush`] (clipped line) or [`RowFace::Card`]
/// (wrapped title, 2px gap, optional meter). `scroll_id` names the
/// clip pane. The 24px rail sits beside it.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::collection::{Selection, VecList, VisibleWindow};
/// let list = VecList::titles(["a"]);
/// let tok = icedtea::theme::named("dark").tokens;
/// #[derive(Clone, Copy)]
/// enum Msg {
///     Select(usize),
///     Scroll(VisibleWindow),
/// }
/// let on_select = Msg::Select;
/// let on_scroll = Msg::Scroll;
/// let _: icedtea::Element<'_, Msg> = icedtea::widget::list_view(
///     &list,
///     &Selection::None,
///     on_select,
///     tok,
///     VisibleWindow::new(120.0),
///     24.0,
///     2,
///     on_scroll,
///     "No rows",
///     |_| tok.danger,
///     None,
///     icedtea::collection::RowFace::FLUSH,
///     A11y::new("list", Role::List),
/// );
/// ```
#[allow(clippy::too_many_arguments)]
pub fn list_view<'a, M, L>(
    model: &'a L,
    selection: &'a Selection,
    on_select: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
    window: VisibleWindow,
    row_h: impl Into<RowHeights<'a>>,
    overscan: usize,
    on_scroll: impl Fn(VisibleWindow) -> M + Copy + 'a,
    empty: &'a str,
    meta_color: impl Fn(usize) -> iced::Color + Copy + 'a,
    scroll_id: Option<Id>,
    face: RowFace<impl Fn(usize) -> f32 + Copy + 'a>,
    a11y: A11y,
) -> Element<'a, M>
where
    M: Clone + 'a,
    L: ListModel + ?Sized,
{
    let cover = selection.primary();
    let heights = row_h.into();
    let len = model.len();
    let prev = window;
    let disabled = a11y.disabled;
    a11y::attach(
        virtual_clip(
            prev,
            heights,
            len,
            overscan,
            cover,
            on_scroll,
            scroll_id,
            tok,
            move |win| {
                let gap = match face {
                    RowFace::Flush => 0.0,
                    RowFace::Card { .. } => 2.0,
                };
                let mut col = Column::new().spacing(gap);
                if model.is_empty() {
                    col = col.push(meta(empty, tok, A11y::new(empty, Role::Status)));
                } else {
                    for i in win.range() {
                        let h = heights.at(i);
                        if model.is_separator(i) {
                            col = col.push(
                                container(rule_h(tok, A11y::new("sep", Role::Separator)))
                                    .width(Length::Fill)
                                    .height(h)
                                    .center_y(h),
                            );
                            continue;
                        }
                        let selected = selection.contains(i);
                        let title = model.title(i);
                        let meta_s = model.meta(i);
                        let name = title.to_string();
                        let painted = match face {
                            RowFace::Flush => {
                                two_line_row(title, meta_s, meta_color(i), selected, h, tok)
                            }
                            RowFace::Card { meter } => card_row(
                                title,
                                meta_s,
                                meta_color(i),
                                selected,
                                h,
                                meter.map(|m| m(i)),
                                tok,
                            ),
                        };
                        let row: Element<'a, M> = if disabled {
                            painted
                        } else {
                            mouse_area(painted)
                                .on_press(on_select(i))
                                .on_right_press(on_select(i))
                                .into()
                        };
                        col = col.push(a11y::attach(
                            row,
                            &A11y::new(name, Role::ListItem)
                                .with_checked(selected)
                                .with_disabled(disabled),
                        ));
                    }
                }
                col
            },
        ),
        &a11y,
    )
}

/// Tiles that share the row width.
///
/// Click sends the index. Empty grid is an empty column.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let labels = vec!["Inbox".into(), "Mail".into()];
/// let on_select = |i| i;
/// let _: icedtea::Element<'_, usize> =
///     widget::item_grid(&labels, on_select, tok, A11y::new("grid", Role::List));
/// ```
pub fn item_grid<'a, M: Clone + 'a>(
    labels: &[String],
    on_select: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let cols = 3;
    let mut rows = iced::widget::Column::new()
        .spacing(8)
        .width(Length::Fill)
        .height(Length::Fill);
    let mut i = 0;
    while i < labels.len() {
        let mut r = iced::widget::Row::new()
            .spacing(8)
            .width(Length::Fill)
            .height(Length::Fill);
        for _ in 0..cols {
            if i < labels.len() {
                let s = labels[i].clone();
                let tile = themed_button_sized(
                    s.clone(),
                    a11y.apply_message(Some(on_select(i))),
                    tok,
                    Variant::Quiet,
                    Length::Fill,
                    Length::Fill,
                    A11y::new(s.clone(), Role::ListItem).with_disabled(a11y.disabled),
                );
                r = r.push(if a11y.disabled {
                    tile
                } else {
                    mouse_area(tile).on_right_press(on_select(i)).into()
                });
                i += 1;
            } else {
                r = r.push(Space::new().width(Length::Fill).height(Length::Fill));
            }
        }
        rows = rows.push(r);
    }
    a11y::attach(rows.into(), &a11y)
}

#[allow(clippy::too_many_arguments)]
/// A virtualized table. Last column fills.
///
/// `on_cell` is (row, column). `on_sort` is the header click. Empty
/// rows still paint headers. `columns.frozen` stays in view;
/// `on_h_scroll` is the unfrozen strip.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::collection::{Selection, TableModel, VisibleWindow};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let table = TableModel {
///     headers: vec!["Name".into()],
///     rows: vec![vec!["lib.rs".into()]],
///     sort_col: None,
///     sort_asc: true,
/// };
/// let cols = icedtea::collection::ColumnLayout::new(vec![120.0]);
/// #[derive(Clone, Copy)]
/// enum Msg {
///     Cell(usize, usize),
///     Sort(usize),
///     Scroll(VisibleWindow),
///     HScroll(f32),
/// }
/// let on_cell = Msg::Cell;
/// let on_sort = Msg::Sort;
/// let on_scroll = Msg::Scroll;
/// let on_h_scroll = Msg::HScroll;
/// let _: icedtea::Element<'_, Msg> = widget::data_table(
///     &table,
///     &Selection::None,
///     None,
///     &cols,
///     true,
///     VisibleWindow::new(200.0),
///     32.0,
///     2,
///     on_cell,
///     on_sort,
///     on_scroll,
///     on_h_scroll,
///     tok,
///     A11y::new("table", Role::Table),
/// );
/// ```
pub fn data_table<'a, M, T>(
    model: &'a T,
    selection: &'a Selection,
    cursor: Option<(usize, usize)>,
    columns: &'a crate::collection::ColumnLayout,
    zebra: bool,
    window: VisibleWindow,
    row_h: f32,
    overscan: usize,
    on_cell: impl Fn(usize, usize) -> M + Copy + 'a,
    on_sort: impl Fn(usize) -> M + Copy + 'a,
    on_scroll: impl Fn(VisibleWindow) -> M + Copy + 'a,
    on_h_scroll: impl Fn(f32) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M>
where
    M: Clone + 'a,
    T: crate::collection::TableSource + ?Sized,
{
    let cover = selection.primary();
    let n = model.row_count();
    let order = columns.display();
    let last = order.last().copied();
    let h = row_h.max(0.0);
    let prev = window;
    let disabled = a11y.disabled;
    let frozen_n = columns.frozen.min(order.len());
    let pin = order[..frozen_n].to_vec();
    let rest = order[frozen_n..].to_vec();
    let h_scroll = columns.h_scroll.max(0.0);
    let last_col = last;
    let col_w = move |c: usize| {
        if Some(c) == last_col {
            Length::Fill
        } else {
            Length::Fixed(columns.width(c))
        }
    };
    let mut pin_head = Row::new().spacing(0);
    for c in &pin {
        let c = *c;
        let title = model.header(c).to_string();
        pin_head = pin_head.push(
            container(themed_button(
                title.clone(),
                a11y.apply_message(Some(on_sort(c))),
                tok,
                Variant::Ghost,
                A11y::button(title).with_disabled(disabled),
            ))
            .width(col_w(c)),
        );
    }
    let mut rest_head = Row::new().spacing(0);
    for c in &rest {
        let c = *c;
        let title = model.header(c).to_string();
        rest_head = rest_head.push(
            container(themed_button(
                title.clone(),
                a11y.apply_message(Some(on_sort(c))),
                tok,
                Variant::Ghost,
                A11y::button(title).with_disabled(disabled),
            ))
            .width(col_w(c)),
        );
    }
    let rest_head = mouse_area(
        container(rest_head)
            .width(Length::Fill)
            .padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: -h_scroll,
            })
            .clip(true),
    )
    .on_scroll(move |delta| on_h_scroll((h_scroll + scroll_delta_x(delta)).max(0.0)));
    let header = row![pin_head, rest_head].width(crate::layout::FILL);
    a11y::attach(
        column![
            header,
            virtual_clip(
                prev,
                RowHeights::Uniform(h),
                n,
                overscan,
                cover,
                on_scroll,
                None,
                tok,
                move |win| {
                    let mut body = Column::new().spacing(0);
                    for i in win.range() {
                        let selected = selection.contains(i);
                        let stripe = zebra && i % 2 == 1;
                        let paint_cell = |line: Row<'a, M>, c: usize| {
                            let focused = cursor == Some((i, c));
                            let value = model.cell(i, c).to_string();
                            let w = col_w(c);
                            let bg = if focused {
                                tok.selection
                            } else if selected {
                                crate::theme::selection_fill(tok)
                            } else if stripe {
                                crate::theme::chip_fill(tok)
                            } else {
                                iced::Color::TRANSPARENT
                            };
                            let face =
                                container(text(value.clone()).size(typo::BODY).color(tok.text))
                                    .width(w)
                                    .height(h)
                                    .padding([8, 8])
                                    .style(move |_| style::fill(bg, tok.text));
                            let cell: Element<'a, M> = if disabled {
                                face.into()
                            } else {
                                mouse_area(face)
                                    .on_press(on_cell(i, c))
                                    .on_right_press(on_cell(i, c))
                                    .into()
                            };
                            line.push(a11y::attach(
                                cell,
                                &A11y::new(format!("{i}:{c}"), Role::ListItem)
                                    .with_value(value)
                                    .with_checked(focused)
                                    .with_disabled(disabled),
                            ))
                        };
                        let mut pin_line = Row::new().spacing(0);
                        for c in &pin {
                            pin_line = paint_cell(pin_line, *c);
                        }
                        let mut rest_line = Row::new().spacing(0);
                        for c in &rest {
                            rest_line = paint_cell(rest_line, *c);
                        }
                        let rest_line = container(rest_line)
                            .width(Length::Fill)
                            .padding(Padding {
                                top: 0.0,
                                right: 0.0,
                                bottom: 0.0,
                                left: -h_scroll,
                            })
                            .clip(true);
                        body = body.push(row![pin_line, rest_line].width(crate::layout::FILL));
                    }
                    body
                },
            )
        ]
        .spacing(4)
        .width(crate::layout::FILL)
        .height(crate::layout::FILL)
        .into(),
        &a11y,
    )
}

/// Heading or file tree. The disclosure control emits `on_toggle`; the
/// row label emits `on_select`. `selected` is the app-owned id.
/// An expandable outline.
///
/// The application owns expand state. Leaf rows have no twisty.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::collection::TreeNode;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let tree = TreeNode::leaf(1, "lib.rs");
/// #[derive(Clone, Copy)]
/// enum Msg {
///     Toggle(u64),
///     Select(u64),
/// }
/// let on_toggle = Msg::Toggle;
/// let on_select = Msg::Select;
/// let _: icedtea::Element<'_, Msg> = widget::tree_view(
///     &tree,
///     None,
///     on_toggle,
///     on_select,
///     tok,
///     A11y::new("tree", Role::Tree),
/// );
/// ```
pub fn tree_view<'a, M: Clone + 'a>(
    root: &TreeNode,
    selected: Option<u64>,
    on_toggle: impl Fn(u64) -> M + Copy + 'a,
    on_select: impl Fn(u64) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let mut col = Column::new().spacing(2);
    for (depth, id, label_s, expanded, has_children) in root.flatten() {
        let is_sel = selected == Some(id);
        let mut line = Row::new().spacing(4).align_y(Alignment::Center);
        line = line.push(Space::new().width(Length::Fixed(depth as f32 * 16.0)));
        if has_children {
            let mark = if expanded { "▾" } else { "▸" };
            line = line.push(themed_button(
                mark,
                a11y.apply_message(Some(on_toggle(id))),
                tok,
                Variant::Ghost,
                A11y::button(format!("toggle {label_s}"))
                    .with_checked(expanded)
                    .with_disabled(a11y.disabled),
            ));
        } else {
            line = line.push(Space::new().width(28.0));
        }
        let title = container(label(
            label_s.clone(),
            tok,
            A11y::new(label_s.clone(), Role::Tree).with_checked(is_sel),
        ))
        .width(Length::Fill)
        .padding([6, 8]);
        let title: Element<'a, M> = if is_sel {
            title.style(move |_| style::list_row(tok, true)).into()
        } else {
            title.into()
        };
        let pick: Element<'a, M> = if a11y.disabled {
            title
        } else {
            mouse_area(title)
                .on_press(on_select(id))
                .on_right_press(on_select(id))
                .into()
        };
        line = line.push(a11y::attach(
            pick,
            &A11y::new(label_s, Role::Tree).with_checked(is_sel),
        ));
        col = col.push(line);
    }
    a11y::attach(
        themed_scroll(
            col.into(),
            tok,
            A11y::new("tree-scroll", Role::Group),
            false,
            None,
            None::<fn(_) -> M>,
        ),
        &a11y,
    )
}

/// A tab bar over a body the application paints.
///
/// `Tabs { closable: false }` is pinned sections. Select sends the
/// index.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::collection::Tabs;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let tabs = Tabs::new(["One", "Two"]);
/// #[derive(Clone, Copy)]
/// enum Msg {
///     Select(usize),
///     Close(usize),
/// }
/// let on_select = Msg::Select;
/// let on_close = Msg::Close;
/// let _: icedtea::Element<'_, Msg> = widget::tab_bar(
///     &tabs,
///     on_select,
///     on_close,
///     tok,
///     A11y::new("tabs", Role::Tab),
/// );
/// ```
pub fn tab_bar<'a, M: Clone + 'a>(
    tabs: &Tabs,
    on_select: impl Fn(usize) -> M + Copy + 'a,
    on_close: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let mut r = Row::new().spacing(4).align_y(Alignment::Center);
    for (i, title) in tabs.titles.iter().enumerate() {
        let mut tab = button(text(title.clone()).size(typo::META))
            .padding([6, 10])
            .style(style::tab_style(tok, i == tabs.active));
        if !a11y.disabled {
            tab = tab.on_press(on_select(i));
        }
        let mut cell = row![tab].spacing(2).align_y(Alignment::Center);
        if tabs.closable {
            cell = cell.push(dismiss_button(
                on_close(i),
                tok,
                A11y::button(format!("close {title}")).with_disabled(a11y.disabled),
            ));
        }
        r = r.push(a11y::attach(
            container(cell).padding([2, 2]).into(),
            &A11y::new(title.clone(), Role::Tab).with_checked(i == tabs.active),
        ));
    }
    a11y::attach(r.into(), &a11y)
}

/// Title on the start edge, `Icon::Chevron` on the end. Open rotates
/// the chevron 180° (dropdown face).
fn disclosure_header<'a, M: Clone + 'a>(
    title: impl Into<String>,
    open: bool,
    msg: Option<M>,
    tok: Tokens,
    a11y: A11y,
    inset: Padding,
) -> Element<'a, M> {
    let title = title.into();
    let angle = if open { std::f32::consts::PI } else { 0.0 };
    let handle = svg::Handle::from_memory(Icon::Chevron.bytes());
    let chevron = svg(handle)
        .width(16.0)
        .height(16.0)
        .rotation(angle)
        .style(icon_style(tok));
    let face = row![
        text(title.clone())
            .size(typo::BODY)
            .color(tok.text)
            .width(Length::Fill),
        chevron,
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .width(Length::Fill);
    let mut b = button(face)
        .padding(inset)
        .width(Length::Fill)
        .style(style::button_style(tok, Variant::Ghost));
    if let Some(m) = a11y.apply_message(msg) {
        b = b.on_press(m);
    }
    a11y::attach(b.into(), &a11y)
}

/// An open row shows a body under the header.
///
/// The application owns which row is open. Closed rows are headers
/// only. The chevron sits on the trailing edge.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::collection::Accordion;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let titles = ["Files".into()];
/// let on_toggle = |i| i;
/// let _: icedtea::Element<'_, usize> = widget::accordion_view(
///     &titles,
///     vec![widget::label("New", tok, A11y::new("New", Role::Status))],
///     &Accordion { open: Some(0) },
///     on_toggle,
///     tok,
///     A11y::new("acc", Role::Group),
/// );
/// ```
pub fn accordion_view<'a, M: Clone + 'a>(
    titles: &[String],
    bodies: Vec<Element<'a, M>>,
    state: &Accordion,
    on_toggle: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let mut col = Column::new().spacing(0).width(Length::Fill);
    for (i, (title, body)) in titles.iter().zip(bodies).enumerate() {
        let open = state.open == Some(i);
        col = col.push(disclosure_header(
            title.clone(),
            open,
            a11y.apply_message(Some(on_toggle(i))),
            tok,
            A11y::button(title.clone())
                .with_checked(open)
                .with_disabled(a11y.disabled),
            pad(),
        ));
        if open {
            col = col.push(
                container(body)
                    .width(Length::Fill)
                    .padding(12)
                    .style(move |_| style::panel(tok)),
            );
        }
    }
    a11y::attach(col.into(), &a11y)
}

/// How much of the child a closed [`expander`] shows.
///
/// `Pixels` is a raw height, snapped up to the 4px grid. `Lines` is
/// whole body lines plus room for the last line's descent, so the
/// clip does not cut through glyphs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Peek {
    Pixels(f32),
    Lines(u16),
}

impl Peek {
    /// iced's default body line box (`typo::BODY` × 1.3).
    pub fn body_line() -> f32 {
        typo::BODY as f32 * 1.3
    }

    /// Extra pixels under the last line so descenders stay inside.
    pub const DESCENT: f32 = 6.0;

    /// Snapped height of this peek.
    pub fn height(self) -> f32 {
        let raw = match self {
            Self::Pixels(px) => px.max(0.0),
            Self::Lines(n) => {
                let n = u32::from(n.max(1));
                n as f32 * Self::body_line() + Self::DESCENT
            }
        };
        crate::density::Density::snap(raw.ceil() as u32).max(4) as f32
    }
}

impl From<f32> for Peek {
    fn from(px: f32) -> Self {
        Self::Pixels(px)
    }
}

fn peek_clip<'a, M: 'a>(child: Element<'a, M>, h: f32, tok: Tokens) -> Element<'a, M> {
    let fade_h = 12.0_f32.min(h * 0.4).max(4.0);
    let mut clear = tok.surface;
    clear.a = 0.0;
    let grad = Linear::new(Radians(std::f32::consts::FRAC_PI_2))
        .add_stop(0.0, clear)
        .add_stop(1.0, tok.surface);
    let fade = container(Space::new().width(Length::Fill).height(fade_h))
        .width(Length::Fill)
        .height(fade_h)
        .style(move |_| container::Style {
            background: Some(Background::from(grad)),
            snap: false,
            ..container::Style::default()
        });
    stack![
        container(child)
            .width(Length::Fill)
            .height(Length::Fixed(h))
            .clip(true),
        column![Space::new().height(Length::Fill), fade].height(Length::Fixed(h)),
    ]
    .into()
}

/// A card that clips its child until opened.
///
/// The application owns `open`. The header toggles. Closed shows a
/// [`Peek`] of the child (pixels or whole body lines) and fades the
/// cut. Open paints the full child. Title and body share the card
/// inset. The chevron sits on the trailing edge.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget::{self, Peek};
/// let tok = theme::named("dark").tokens;
/// let body = widget::label("more", tok, A11y::new("more", Role::Status));
/// let _: icedtea::Element<'_, bool> = widget::expander(
///     "Notes",
///     body,
///     Peek::Lines(2),
///     false,
///     |open| open,
///     tok,
///     A11y::new("Notes", Role::Group),
/// );
/// ```
pub fn expander<'a, M: Clone + 'a>(
    title: impl Into<String>,
    child: Element<'a, M>,
    collapsed: impl Into<Peek>,
    open: bool,
    on_toggle: impl Fn(bool) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let title = a11y.apply_name(title);
    let header = disclosure_header(
        title.clone(),
        open,
        a11y.apply_message(Some(on_toggle(!open))),
        tok,
        A11y::button(title.clone())
            .with_checked(open)
            .with_disabled(a11y.disabled),
        Padding {
            top: 8.0,
            right: 0.0,
            bottom: 8.0,
            left: 0.0,
        },
    );
    let h = collapsed.into().height();
    let body: Element<'a, M> = if open {
        child
    } else {
        peek_clip(child, h, tok)
    };
    a11y::attach(
        container(column![header, body].spacing(8))
            .padding(12)
            .width(Length::Fill)
            .style(move |_| style::card(tok, false))
            .into(),
        &a11y,
    )
}

/// Page through a long list.
///
/// Pass page count and the current page. Messages are previous, next,
/// and jump.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_page = |i| i;
/// let _: icedtea::Element<'_, usize> =
///     widget::pagination(40, 0, 10, on_page, tok, A11y::new("pages", Role::Group));
/// ```
pub fn pagination<'a, M: Clone + 'a>(
    len: usize,
    page: usize,
    per_page: usize,
    on_page: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let pages = crate::collection::page_count(len, per_page);
    let range = page_range(len, page, per_page);
    let status = format!("{}–{} / {len}", range.start, range.end);
    a11y::attach(
        row![
            themed_button(
                "Prev",
                a11y.apply_message((page > 0).then(|| on_page(page - 1))),
                tok,
                Variant::Quiet,
                A11y::button("Prev").with_disabled(a11y.disabled || page == 0),
            ),
            meta(status.clone(), tok, A11y::new(status, Role::Status)),
            themed_button(
                "Next",
                a11y.apply_message((page + 1 < pages).then(|| on_page(page + 1))),
                tok,
                Variant::Quiet,
                A11y::button("Next").with_disabled(a11y.disabled || page + 1 >= pages),
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into(),
        &a11y,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::{Selection as Sel, TableModel, VecList};
    use crate::density::Density;
    use crate::theme::named;

    fn must(ok: bool, msg: impl std::fmt::Display) {
        if !ok {
            panic!("{msg}");
        }
    }

    #[test]
    #[should_panic(expected = "cover-must")]
    fn must_rejects_a_failed_check() {
        must(false, "cover-must");
    }

    fn draw_once<M: Clone>(el: &mut Element<'_, M>) {
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::renderer::Style;
        use iced::advanced::widget::Tree;
        use iced::mouse;
        use iced::{Font, Pixels, Point, Rectangle, Size, Theme};
        let mut tree = Tree::new(el.as_widget());
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(400.0, 400.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(400.0, 400.0));
        el.as_widget().draw(
            &tree,
            &mut renderer,
            &Theme::Dark,
            &Style::default(),
            layout,
            mouse::Cursor::Unavailable,
            &viewport,
        );
    }

    const TEST_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB0, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn date_time_step_and_suggest() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2000, 2), 29);
        assert_eq!(days_in_month(1900, 2), 28);
        assert_eq!(days_in_month(2023, 4), 30);
        assert_eq!(days_in_month(2023, 1), 31);
        let d = DateValue {
            year: 2023,
            month: 13,
            day: 40,
        }
        .clamp();
        assert_eq!(d.month, 12);
        assert_eq!(d.day, 31);
        let t = TimeValue {
            hour: 30,
            minute: 90,
            second: 90,
        }
        .clamp();
        assert_eq!(t.hour, 23);
        assert_eq!(t.minute, 59);
        assert_eq!(t.second, 59);
        let noon = TimeValue::hms(23, 59, 59);
        assert_eq!(noon.step_hour(1).hour, 0);
        assert_eq!(noon.step_minute(1).minute, 0);
        assert_eq!(noon.step_second(1).second, 0);
        assert_eq!(TimeValue::hm(0, 0).step_hour(-1).hour, 23);
        assert_eq!(TimeValue::hm(0, 0).step_minute(-1).minute, 59);
        assert_eq!(TimeValue::hms(0, 0, 0).step_second(-1).second, 59);
        assert_eq!(TimeValue::hm(0, 0).hour12(), 12);
        assert!(!TimeValue::hm(0, 0).afternoon());
        assert_eq!(TimeValue::hm(12, 0).hour12(), 12);
        assert!(TimeValue::hm(12, 0).afternoon());
        assert_eq!(TimeValue::hm(13, 5).hour12(), 1);
        assert_eq!(TimeValue::hm(11, 0).step_hour12(1).hour, 0);
        assert_eq!(TimeValue::hm(23, 0).step_hour12(1).hour, 12);
        assert_eq!(TimeValue::hm(0, 0).toggle_period().hour, 12);
        assert_eq!(TimeValue::hm(15, 0).toggle_period().hour, 3);
        assert_eq!(
            TimeValue::hm(9, 30)
                .step_field(TimeField::Hour, TimeClock::HOUR12)
                .hour,
            10
        );
        assert_eq!(
            TimeValue::hm(23, 0)
                .step_field(TimeField::Hour, TimeClock::HOURS_MINUTES)
                .hour,
            0
        );
        assert_eq!(
            TimeValue::hm(9, 0)
                .step_field(TimeField::Period, TimeClock::HOUR12)
                .hour,
            21
        );
        assert_eq!(
            TimeValue::hm(9, 30)
                .step_field(TimeField::Minute, TimeClock::HOURS_MINUTES)
                .minute,
            31
        );
        assert_eq!(
            TimeValue::hms(9, 30, 4)
                .step_field(TimeField::Second, TimeClock::HOURS_MINUTES_SECONDS)
                .second,
            5
        );
        assert_eq!(step_number(5.0, 1.0, 0.0, 10.0, 1), 6.0);
        assert_eq!(step_number(0.0, 1.0, 0.0, 10.0, -1), 0.0);
        assert_eq!(apply_mask("0000-0000", "12345678"), "1234-5678");
        assert_eq!(apply_mask("0000-0000", "12"), "12");
        assert_eq!(apply_mask("00/00", "1299"), "12/99");
        assert_eq!(apply_mask("0000-0000", "abcd"), "");
        assert_eq!(mask_handler("0000-0000", |s| s)("12ab34".into()), "1234");
        let _: Element<'_, ()> = masked_input(
            "0000-0000",
            "1234",
            |_| (),
            named("dark").tokens,
            A11y::new("mask", Role::TextBox),
        );
        assert!(A11y::button("x").with_disabled(true).disabled);
        assert_eq!(
            A11y::new("y", Role::Checkbox).with_checked(false).checked,
            Some(false)
        );
    }

    #[test]
    fn widgets_build_elements() {
        let tok = named("dark").tokens;
        let btn = |n: &str| A11y::button(n);
        let role = |n: &str, r: Role| A11y::new(n, r);
        let _: Element<'_, ()> = icon_svg(Icon::Search, tok, role("search", Role::Image));
        let _: Element<'_, ()> = label("Hi", tok, role("Hi", Role::Header));
        let plain = RichCell::Plain("hi".into());
        let _: Element<'_, ()> = rich_cell(&plain, None, tok, role("plain", Role::Status));
        let em = RichCell::Emphasis("em".into());
        let _: Element<'_, ()> = rich_cell(&em, None, tok, role("em", Role::Status));
        let code = RichCell::Code("x".into());
        let _: Element<'_, ()> = rich_cell(&code, None, tok, role("code", Role::Status));
        let link = RichCell::Link("go".into());
        let _: Element<'_, ()> = rich_cell(&link, Some(()), tok, role("go", Role::Link));
        let _: Element<'_, ()> = rich_cell(&link, None, tok, role("go2", Role::Link));
        let _: Element<'_, ()> = meta("m", tok, role("m", Role::Status));
        let snippet = Content::with_text("fn");
        let _: Element<'_, ()> = code_block(&snippet, |_| (), tok, role("fn", Role::Group));
        let _: Element<'_, ()> = hyperlink("l", (), tok, role("l", Role::Link));
        let _: Element<'_, ()> = themed_button("B", Some(()), tok, Variant::Primary, btn("B"));
        let _: Element<'_, ()> = themed_button_sized(
            "7",
            Some(()),
            tok,
            Variant::Quiet,
            Length::Fill,
            Length::Fixed(Density::default().tile() as f32),
            btn("7"),
        );
        let _: Element<'_, ()> = display_reading("24", tok, role("24", Role::Status));
        let _: Element<'_, ()> = display_line("6 × 4 =", tok, role("expr", Role::Status));
        let _: Element<'_, ()> = figure_display("12:40", tok, role("clock", Role::Status));
        let glyph = A11y::button("Backspace");
        let _: Element<'_, ()> = themed_button_sized(
            "⌫",
            Some(()),
            tok,
            Variant::Quiet,
            Length::Fill,
            Length::Fixed(48.0),
            glyph,
        );
        let _: Element<'_, ()> = themed_button(
            "D",
            None,
            tok,
            Variant::Danger,
            btn("D").with_disabled(true),
        );
        let _: Element<'_, ()> = split_button("S", (), (), tok, btn("S"));
        let _: Element<'_, ()> = split_button("S", (), (), tok, btn("S").with_disabled(true));
        let _: Element<'_, ()> = toggle_button("T", true, (), tok, btn("T").with_checked(true));
        let _: Element<'_, ()> = toggle_button("T", false, (), tok, btn("T").with_checked(false));
        let _: Element<'_, ()> = toggle_button(
            "T",
            true,
            (),
            tok,
            btn("T").with_checked(true).with_disabled(true),
        );
        let _: Element<'_, ()> = themed_checkbox(
            "c",
            true,
            |_| (),
            tok,
            role("c", Role::Checkbox).with_checked(true),
        );
        let _: Element<'_, ()> = themed_switch(
            "s",
            false,
            |_| (),
            tok,
            role("s", Role::Switch).with_disabled(true),
        );
        let _: Element<'_, ()> = themed_switch(
            "s2",
            true,
            |_| (),
            tok,
            role("s2", Role::Switch).with_checked(true),
        );
        let _: Element<'_, ()> =
            themed_radio("r", 1u8, Some(1u8), |_| (), tok, role("r", Role::Radio));
        let _: Element<'_, ()> = themed_radio(
            "off",
            2u8,
            Some(1u8),
            |_| (),
            tok,
            role("off", Role::Radio).with_disabled(true),
        );
        let _: Element<'_, ()> = themed_radio(
            "on",
            1u8,
            Some(1u8),
            |_| (),
            tok,
            role("on", Role::Radio).with_disabled(true),
        );
        let _ = radio_idle_face(tok, true);
        let _ = radio_idle_face(tok, false);
        assert_eq!(
            scroll_delta_pixels(iced::mouse::ScrollDelta::Lines { x: 0.0, y: -2.0 }, 20.0),
            40.0
        );
        assert_eq!(
            scroll_delta_pixels(iced::mouse::ScrollDelta::Pixels { x: 0.0, y: 8.0 }, 20.0),
            -8.0
        );
        assert_eq!(
            scroll_delta_x(iced::mouse::ScrollDelta::Lines { x: 1.0, y: 0.0 }),
            -32.0
        );
        assert_eq!(
            scroll_delta_x(iced::mouse::ScrollDelta::Pixels { x: 8.0, y: 0.0 }),
            -8.0
        );
        let _: Element<'_, ()> = themed_slider(
            0.0..=1.0,
            0.5,
            |_| (),
            tok,
            role("s", Role::Slider).with_value("0.5"),
        );
        let _: Element<'_, ()> =
            progress(0.2, None, tok, role("p", Role::Progress).with_value("0.2"));
        let _: Element<'_, ()> = progress(
            0.5,
            Some("50% · 1 min"),
            tok,
            role("pc", Role::Progress).with_value("0.5"),
        );
        let _: Element<'_, ()> =
            progress_ring(0.4, None, tok, role("pr", Role::Progress).with_value("0.4"));
        let _: Element<'_, ()> = progress_ring(
            0.5,
            Some("50%"),
            tok,
            role("prc", Role::Progress).with_value("0.5"),
        );
        assert_eq!(progress_label(0.5, None), "50%");
        assert_eq!(progress_label(0.5, Some("1 min")), "50% · 1 min");
        assert!(spark_points(&[], 10.0, 10.0).is_empty());
        let pts = spark_points(&[1.0, 3.0, 2.0], 100.0, 20.0);
        assert_eq!(pts.len(), 3);
        assert!((pts[0].0 - 0.0).abs() < 0.01);
        assert!(pts[1].1 < pts[0].1);
        let _: Element<'_, ()> = sparkline(&[1.0, 2.0, 1.5], tok, role("spark", Role::Image));
        let _: Element<'_, ()> = image_slot(
            ImageSlot::Ready {
                handle: iced::widget::image::Handle::from_bytes(TEST_PNG),
                fit: iced::ContentFit::Cover,
            },
            48.0,
            48.0,
            tok,
            role("cover", Role::Image),
        );
        let _: Element<'_, ()> = image_slot(
            ImageSlot::Loading,
            48.0,
            48.0,
            tok,
            role("load", Role::Image),
        );
        let _: Element<'_, ()> = image_slot(
            ImageSlot::Error("missing".into()),
            48.0,
            48.0,
            tok,
            role("err", Role::Image),
        );
        let _: Element<'_, ()> = spinner(tok, 0.25, role("spin", Role::Progress));
        let _: Element<'_, ()> = busy_overlay(
            label("Body", tok, role("Body", Role::Status)),
            true,
            0.2,
            tok,
            role("busy", Role::Group),
        );
        let _: Element<'_, ()> = busy_overlay(
            label("Idle", tok, role("Idle", Role::Status)),
            false,
            0.0,
            tok,
            role("idle", Role::Group),
        );
        assert!(ring_angles(1.0).1 > ring_angles(0.2).1);
        assert!(
            (spinner_angles(0.0).1 - spinner_angles(0.0).0 - std::f32::consts::FRAC_PI_2).abs()
                < 0.01
        );
        assert!(ring_should_stroke(0.0, 1.0));
        assert!(!ring_should_stroke(0.0, 0.0));
        let a11y = A11y::button("Nope").with_disabled(true);
        let _: Element<'_, ()> = themed_button("Nope", Some(()), tok, Variant::Primary, a11y);
        let unnamed = A11y::button("");
        let _: Element<'_, ()> = themed_button("Shown", Some(()), tok, Variant::Primary, unnamed);
        let unnamed_c = A11y::new("", Role::Checkbox);
        let _: Element<'_, ()> = themed_checkbox("box", true, |_| (), tok, unnamed_c);
        let ca = A11y::new("off", Role::Checkbox)
            .with_checked(true)
            .with_disabled(true);
        let _: Element<'_, ()> = themed_checkbox("off", false, |_| (), tok, ca);
        let _: Element<'_, ()> = number_input(
            3.0,
            |_| (),
            tok,
            role("n", Role::SpinButton).with_value("3"),
        );
        let _: Element<'_, ()> = themed_text_input(
            "p",
            "v",
            |_| (),
            Some(()),
            tok,
            role("v", Role::TextBox),
            Some(Id::new("name")),
        );
        let _: Element<'_, ()> = themed_text_input(
            "p",
            "",
            |_| (),
            None,
            tok,
            role("Name", Role::TextBox),
            None,
        );
        let _: Element<'_, ()> = themed_text_input(
            "p",
            "",
            |_| (),
            Some(()),
            tok,
            role("Name", Role::TextBox).with_disabled(true),
            None,
        );
        let _: Element<'_, ()> = password_input(
            "p",
            "v",
            |_| (),
            tok,
            role("pw", Role::TextBox).with_disabled(true),
            true,
        );
        let _: Element<'_, ()> =
            password_input("p", "v", |_| (), tok, role("pw2", Role::TextBox), true);
        let copy = crate::action::Action::new("secret.copy", "Copy", ());
        let _: Element<'_, ()> = secret_field(
            "Secret",
            "v",
            |_| (),
            false,
            (),
            &copy,
            tok,
            Direction::Ltr,
            role("secret", Role::Group),
        );
        let mut dead = crate::action::Action::new("secret.copy", "Copy", ());
        dead.enabled = false;
        let _: Element<'_, ()> = secret_field(
            "Secret",
            "v",
            |_| (),
            true,
            (),
            &dead,
            tok,
            Direction::Rtl,
            role("secret-off", Role::Group).with_disabled(true),
        );
        let content = Content::new();
        let _: Element<'_, ()> = textarea(
            &content,
            |_| (),
            tok,
            crate::layout::FILL,
            role("ta", Role::TextBox),
        );
        let _: Element<'_, ()> = textarea(
            &content,
            |_| (),
            tok,
            crate::layout::fixed(120.0),
            role("ta", Role::TextBox),
        );
        let _: Element<'_, ()> = selectable(
            &content,
            |_| (),
            tok,
            typo::FontFace::Ui,
            role("body", Role::TextBox),
        );
        let _: Element<'_, ()> = selectable(
            &content,
            |_| (),
            tok,
            typo::FontFace::Mono,
            role("path", Role::TextBox).with_disabled(true),
        );
        let copy = crate::action::Action::new("value.copy", "Copy", ());
        let _: Element<'_, ()> = value_field(
            "Path",
            &content,
            |_| (),
            Some(&copy),
            typo::FontFace::Mono,
            tok,
            Direction::Ltr,
            role("vf", Role::Group),
        );
        let _: Element<'_, ()> = value_field(
            "Id",
            &content,
            |_| (),
            None,
            typo::FontFace::Ui,
            tok,
            Direction::Rtl,
            role("vf-off", Role::Group).with_disabled(true),
        );
        let _: Element<'_, ()> = search_input("q", |_| (), tok, role("q", Role::TextBox));
        let hints = ["save".into(), "open".into()];
        let _: Element<'_, ()> = suggest_field(
            "Command",
            "s",
            |_| (),
            &hints,
            |_| (),
            tok,
            role("suggest", Role::Group),
        );
        let opts = ["a".to_string(), "b".to_string()];
        let _: Element<'_, ()> = themed_pick_list(
            opts,
            Some("a".into()),
            |_| (),
            tok,
            role("a", Role::ComboBox),
        );
        let _: Element<'_, ()> = date_picker(
            DateValue {
                year: 2024,
                month: 1,
                day: 1,
            },
            (),
            (),
            tok,
            role("date", Role::SpinButton).with_value("2024-01-01"),
        );
        let _: Element<'_, ()> = time_picker(
            TimeValue::hm(8, 5),
            TimeClock::HOURS_MINUTES,
            |_| (),
            tok,
            role("time", Role::SpinButton),
        );
        let _: Element<'_, ()> = time_picker(
            TimeValue::hms(13, 5, 9),
            TimeClock::HOUR12_SECONDS,
            |_| (),
            tok,
            role("time-12", Role::SpinButton).with_disabled(true),
        );
        let _: Element<'_, ()> = time_picker(
            TimeValue::hms(8, 5, 1),
            TimeClock::HOURS_MINUTES_SECONDS,
            |_| (),
            tok,
            role("time-sec", Role::SpinButton),
        );
        let _: Element<'_, ()> = time_picker(
            TimeValue::hm(9, 30),
            TimeClock::HOUR12,
            |_| (),
            tok,
            role("time-am", Role::SpinButton),
        );
        let _: Element<'_, ()> = color_swatch(1, 2, 3, (), tok, btn("color"));
        let items = markdown::parse("# Hi");
        let items: Vec<_> = items.collect();
        let _: Element<'_, ()> = markdown_view(&items, tok, |_| (), role("md", Role::Group));
        let code = Content::with_text("fn main() {}\n");
        let _: Element<'_, ()> = highlighted_code(
            &code,
            "rs",
            |_| (),
            tok,
            "dark",
            crate::layout::FILL,
            role("code", Role::Group),
        );
        let light = named("light").tokens;
        let _: Element<'_, ()> = highlighted_code(
            &code,
            "py",
            |_| (),
            light,
            "solarized-light",
            crate::layout::fixed(280.0),
            role("code", Role::Group),
        );
        let mocha = named("catppuccin-mocha").tokens;
        let _: Element<'_, ()> = highlighted_code(
            &code,
            "rs",
            |_| (),
            mocha,
            "catppuccin-mocha",
            crate::layout::FILL,
            role("code", Role::Group),
        );
        let _: Element<'_, ()> = tooltip_wrap(
            label("x", tok, role("x", Role::Header)),
            "tip",
            tok,
            role("tip", Role::Tooltip),
        );
        let _: Element<'_, ()> = rule_h(tok, role("rule", Role::Separator));
        let _: Element<'_, ()> = dismiss_button((), tok, btn("dismiss"));
        let _: Element<'_, ()> = chip("c", Some(()), tok, Variant::Quiet, btn("c"));
        let _: Element<'_, ()> = chip("plain", None, tok, Variant::Primary, btn("plain"));
        let _: Element<'_, ()> = chip("hot", Some(()), tok, Variant::Danger, btn("hot"));
        let _: Element<'_, ()> = chip("g", None, tok, Variant::Ghost, btn("g"));
        let _: Element<'_, ()> = chip("k", None, tok, Variant::Chip, btn("k"));
        let _: Element<'_, ()> = chip("ok", None, tok, Variant::Success, btn("ok"));
        let _: Element<'_, ()> = chip("warn", None, tok, Variant::Warning, btn("warn"));
        let _: Element<'_, ()> = badge("b", tok, Variant::Quiet, role("b", Role::Status));
        let _: Element<'_, ()> = badge("new", tok, Variant::Primary, role("new", Role::Status));
        let _: Element<'_, ()> = badge("!", tok, Variant::Danger, role("bang", Role::Status));
        let _: Element<'_, ()> = badge("g", tok, Variant::Ghost, role("g", Role::Status));
        let _: Element<'_, ()> = badge("chip", tok, Variant::Chip, role("chip", Role::Status));
        let _: Element<'_, ()> = badge("ok", tok, Variant::Success, role("ok", Role::Status));
        let _: Element<'_, ()> = badge("warn", tok, Variant::Warning, role("warn", Role::Status));
        let _: Element<'_, ()> = group_box(
            "g",
            label("x", tok, role("x", Role::Header)),
            tok,
            role("g", Role::Group),
        );
        let _: Element<'_, ()> = banner("b", Some(("go".into(), ())), tok, role("b", Role::Status));
        let _: Element<'_, ()> = banner("b", None, tok, role("b", Role::Status));
        let _: Element<'_, ()> = info_bar(ToastKind::Warning, "w", tok, role("w", Role::Status));
        let _: Element<'_, ()> = breadcrumb(
            &[("Home".into(), Some(())), ("Here".into(), None)],
            tok,
            Direction::Ltr,
            role("bc", Role::Group),
        );
        let _: Element<'_, ()> = breadcrumb(
            &[("Home".into(), Some(())), ("Here".into(), None)],
            tok,
            Direction::Rtl,
            role("bc", Role::Group),
        );
        let toast = Toast {
            id: 1,
            kind: ToastKind::Info,
            text: "t".into(),
            ttl_ms: 10,
        };
        let _: Element<'_, ()> = toast_view(&toast, (), tok, role("t", Role::Status));
        let _: Element<'_, ()> = teaching_tip("t", "b", (), tok, role("tip", Role::Tooltip));
        let _: Element<'_, ()> = placeholder_skeleton(tok, role("sk", Role::Status));
        let list = VecList {
            items: vec![
                crate::collection::ListRow::new("a").with_meta("meta"),
                crate::collection::ListRow::new("b").with_meta(""),
            ],
        };
        let empty = VecList::default();
        let with_sep = VecList {
            items: vec![
                crate::collection::ListRow::new("a"),
                crate::collection::ListRow::separator(),
                crate::collection::ListRow::new("b"),
            ],
        };
        let win = VisibleWindow::new(100.0);
        let _: Element<'_, ()> = list_view(
            &list,
            &Sel::Single(0),
            |_| (),
            tok,
            win,
            24.0,
            crate::collection::OVERSCAN,
            |_| (),
            "Empty",
            |_| tok.muted,
            Some(Id::from("list-host")),
            RowFace::FLUSH,
            role("list", Role::List),
        );
        let _: Element<'_, ()> = list_view(
            &empty,
            &Sel::None,
            |_| (),
            tok,
            win,
            24.0,
            0,
            |_| (),
            "No sessions",
            |_| tok.muted,
            None,
            RowFace::FLUSH,
            role("list", Role::List),
        );
        let mut striped: Element<'_, ()> = list_view(
            &list,
            &Sel::Single(0),
            |_| (),
            tok,
            win,
            24.0,
            4,
            |_| (),
            "Empty",
            |i| {
                if i == 0 {
                    tok.danger
                } else {
                    tok.muted
                }
            },
            None,
            RowFace::FLUSH,
            role("list", Role::List),
        );
        draw_once(&mut striped);
        let _: Element<'_, ()> = list_view(
            &with_sep,
            &Sel::Single(0),
            |_| (),
            tok,
            win,
            24.0,
            4,
            |_| (),
            "Empty",
            |_| tok.muted,
            None,
            RowFace::FLUSH,
            role("list-sep", Role::List),
        );
        let lines = ["boot".into(), "ready".into()];
        let _: Element<'_, ()> = log_view(
            &lines,
            VisibleWindow::new(80.0),
            18.0,
            2,
            |_| (),
            Some(Id::from("log")),
            tok,
            role("log", Role::List),
        );
        let live = crate::collection::visible_window(20.0, 80.0, 18.0, 2, 2, None);
        let _: Element<'_, ()> = log_view(
            &lines,
            live,
            18.0,
            2,
            |_| (),
            None,
            tok,
            role("log-live", Role::List),
        );
        let empty_log: [String; 0] = [];
        let _: Element<'_, ()> = log_view(
            &empty_log,
            VisibleWindow::new(80.0),
            18.0,
            0,
            |_| (),
            None,
            tok,
            role("log-empty", Role::List),
        );
        let _: Element<'_, ()> = item_grid(
            &["a".into(), "b".into()],
            |_| (),
            tok,
            role("grid", Role::List),
        );
        let table = TableModel {
            headers: vec!["A".into(), "B".into()],
            rows: vec![vec!["1".into(), "x".into()], vec!["2".into(), "y".into()]],
            sort_col: None,
            sort_asc: true,
        };
        let widths = crate::collection::ColumnLayout::new(vec![80.0, 80.0]);
        let _: Element<'_, ()> = data_table(
            &table,
            &Sel::Single(0),
            Some((0, 1)),
            &widths,
            true,
            VisibleWindow::new(100.0),
            24.0,
            crate::collection::OVERSCAN,
            |_, _| (),
            |_| (),
            |_| (),
            |_| (),
            tok,
            role("table", Role::Table),
        );
        let big = TableModel {
            headers: vec!["N".into()],
            rows: (0..50).map(|i| vec![i.to_string()]).collect(),
            sort_col: None,
            sort_asc: true,
        };
        let one = crate::collection::ColumnLayout::new(vec![96.0]);
        let _: Element<'_, ()> = data_table(
            &big,
            &Sel::None,
            None,
            &one,
            false,
            VisibleWindow {
                start: 0,
                end: 0,
                scroll: 200.0,
                viewport: 80.0,
            },
            20.0,
            2,
            |_, _| (),
            |_| (),
            |_| (),
            |_| (),
            tok,
            role("table", Role::Table),
        );
        let tree = TreeNode::branch(1, "r", vec![TreeNode::leaf(2, "c")]);
        let _: Element<'_, ()> = tree_view(
            &tree,
            Some(2),
            |_| (),
            |_| (),
            tok,
            role("tree", Role::Tree),
        );
        let mut tabs = Tabs::new(["A", "B"]);
        tabs.closable = true;
        let _: Element<'_, ()> = tab_bar(&tabs, |_| (), |_| (), tok, role("tabs", Role::Tab));
        let open_tabs = Tabs::new(["A"]);
        let _: Element<'_, ()> = tab_bar(&open_tabs, |_| (), |_| (), tok, role("tabs", Role::Tab));
        let acc = Accordion { open: Some(0) };
        let _: Element<'_, ()> = accordion_view(
            &["A".into()],
            vec![label("b", tok, role("b", Role::Header))],
            &acc,
            |_| (),
            tok,
            role("acc", Role::Group),
        );
        let note = label("more", tok, role("more", Role::Status));
        let _: Element<'_, bool> = expander(
            "Notes",
            note,
            48.0,
            false,
            |open| open,
            tok,
            role("exp", Role::Group),
        );
        let note = label("more", tok, role("more", Role::Status));
        let _: Element<'_, bool> = expander(
            "Notes",
            note,
            48.0,
            true,
            |open| open,
            tok,
            role("exp", Role::Group).with_disabled(true),
        );
        let _: Element<'_, ()> = pagination(100, 1, 10, |_| (), tok, role("page", Role::Group));
        let _: Element<'_, ()> = pagination(10, 0, 10, |_| (), tok, role("page", Role::Group));
        let theme = crate::theme::iced_theme("dark", tok);
        let _ = icon_style(tok)(&theme, svg::Status::Idle);
        let _ = icon_style(tok)(&theme, svg::Status::Hovered);
        let _ = editor_style(tok)(&theme, iced::widget::text_editor::Status::Active);
        let _ = editor_style(tok)(&theme, iced::widget::text_editor::Status::Hovered);
        let _ = editor_style(tok)(
            &theme,
            iced::widget::text_editor::Status::Focused { is_hovered: true },
        );
        let _ = editor_style(tok)(&theme, iced::widget::text_editor::Status::Disabled);
        let _ = editor_frame(tok);
        let _ = tip_style(tok)(&theme);
        let _ = toast_style(tok, ToastKind::Info)(&theme);
        let collapsed = TreeNode::branch(1, "r", vec![TreeNode::leaf(2, "c")]);
        let mut collapsed = collapsed;
        assert!(crate::collection::tree_toggle(&mut collapsed, 1));
        assert!(!collapsed.expanded);
        let selected = Some(1u64);
        assert_eq!(selected, Some(1));
        let _: Element<'_, ()> = tree_view(
            &collapsed,
            selected,
            |_| (),
            |_| (),
            tok,
            role("tree", Role::Tree),
        );
        let src = include_str!("widget.rs");
        let product = src.split("#[cfg(test)]").next().unwrap();
        assert!(!product.contains(".height(120)"));
        assert!(!product.contains(".height(280)"));
        assert!(product.contains(".id(Id::from(a11y.node_id()))"));
        assert!(btn("Save")
            .with_value("idle")
            .with_disabled(true)
            .node_id()
            .contains("button|Save|1"));
        let _: Element<'_, ()> = themed_scroll(
            label("log", tok, role("log", Role::Status)),
            tok,
            role("scroll", Role::Group),
            true,
            None,
            None::<fn(_) -> ()>,
        );
        let _: Element<'_, ()> = themed_scroll(
            label("body", tok, role("body", Role::Status)),
            tok,
            role("scroll", Role::Group),
            false,
            Some(Id::from("body-scroll")),
            None::<fn(_) -> ()>,
        );
        assert!(crate::layout::stick_to_end(80.0, 100.0, 20.0, 4.0));
        let input_src = src
            .split("pub fn themed_text_input")
            .nth(1)
            .unwrap()
            .split("pub fn password_input")
            .next()
            .unwrap();
        assert!(!input_src.contains("apply_name(value)"));
        assert!(input_src.contains("on_submit"));
        assert!(input_src.contains(".id("));
        assert!(product.contains("virtual_pads("));
        assert!(product.contains("window_after_scroll("));
        assert!(product.contains("ScrollRail::new"));
        assert!(!product.contains("list_body_and_rail"));
        let pass_src = src
            .split("pub fn password_input")
            .nth(1)
            .unwrap()
            .split("pub fn secret_field")
            .next()
            .unwrap();
        assert!(!pass_src.contains("apply_name(value)"));
        assert!(pass_src.contains("secure(masked)"));
        let secret_src = src
            .split("pub fn secret_field")
            .nth(1)
            .unwrap()
            .split("pub fn textarea")
            .next()
            .unwrap();
        assert!(secret_src.contains("password_input("));
        assert!(!secret_src.contains("apply_name(value)"));
        assert!(secret_src.contains("a11y.child(Role::TextBox)"));
        let num_src = src
            .split("pub fn number_input")
            .nth(1)
            .unwrap()
            .split("pub fn step_number")
            .next()
            .unwrap();
        assert!(!num_src.contains("apply_name"));
        let doc = parse("# Hi\n\nBody.");
        assert!(!doc.items.is_empty());
        assert_eq!(doc.hash, MarkdownDoc::parse("# Hi\n\nBody.").hash);
        assert_ne!(doc.hash, parse("# Other").hash);
        let _: Element<'_, ()> = markdown_view(&doc.items, tok, |_| (), role("md", Role::Group));
        let md = markdown_style(tok);
        assert_eq!(md.link_color, tok.accent);
        assert_eq!(md.inline_code_color, tok.text);
        assert_eq!(md.inline_code_font, typo::MONO);
        let cut = parse(&"# title\n\nbody".repeat(8)[..20]);
        assert!(cut.source.len() <= 20);
        let outlined = parse("# Hi\n\nA paragraph.\n\n## Next\n\nMore.");
        let heads = outlined.headings();
        assert_eq!(heads.len(), 2);
        assert_eq!(heads[0].level, 1);
        assert_eq!(heads[0].title, "Hi");
        assert_eq!(heads[1].level, 2);
        let deep = parse("# A\n\n## B\n\n### C\n\n#### D\n\n##### E\n\n###### F\n");
        let levels: Vec<u8> = deep.headings().iter().map(|h| h.level).collect();
        assert_eq!(levels, vec![1, 2, 3, 4, 5, 6]);
        let _: Element<'_, ()> = markdown_outline(
            &heads,
            Some(heads[0].index),
            |_| (),
            tok,
            role("outline", Role::List),
        );
        let none: [MdHeading; 0] = [];
        let _: Element<'_, ()> =
            markdown_outline(&none, None, |_| (), tok, role("outline-empty", Role::List));
        assert!(outlined.item_offset(heads[1].index) > outlined.item_offset(heads[0].index));
        assert_eq!(outlined.item_offset(0), 0.0);
        assert!(outlined.item_offset(outlined.items.len()) > outlined.item_offset(heads[1].index));
        let _ = deep.item_offset(deep.items.len());
        let rich = parse(
            "# T\n\nA paragraph with [link](https://example.com).\n\n- bullet\n  - nested\n\n1. one\n\n- [x] done\n- [ ] todo\n\n> quoted\n>\n> still quoted\n\n---\n\n```rust\nfn x() {}\n```\n\n| Name | Ready |\n| --- | --- |\n| A | yes |\n\n![Logo](pixel.png)\n",
        );
        assert!(rich.items.len() >= 8);
        assert!(rich.item_offset(rich.items.len()) > 100.0);
    }

    #[test]
    fn widgets_paint_themed_faces() {
        let tok = named("dark").tokens;
        let btn = |n: &str| A11y::button(n);
        let role = |n: &str, r: Role| A11y::new(n, r);
        let code = Content::with_text("fn main() {}\n");
        let mut painted: Vec<Element<'_, ()>> = vec![
            code_block(&code, |_| (), tok, role("fn", Role::Group)),
            hyperlink("l", (), tok, role("l", Role::Link)),
            busy_overlay(
                label("Body", tok, role("Body", Role::Status)),
                true,
                0.2,
                tok,
                role("busy", Role::Group),
            ),
            image_slot(
                ImageSlot::Error("missing".into()),
                48.0,
                48.0,
                tok,
                role("err", Role::Image),
            ),
            number_input(3.0, |_| (), tok, role("n", Role::SpinButton)),
            textarea(
                &code,
                |_| (),
                tok,
                crate::layout::FILL,
                role("ta", Role::TextBox),
            ),
            highlighted_code(
                &code,
                "rs",
                |_| (),
                tok,
                "dark",
                crate::layout::FILL,
                role("code", Role::Group),
            ),
            color_swatch(1, 2, 3, (), tok, btn("color")),
            chip("ok", None, tok, Variant::Success, btn("ok")),
            chip("x", Some(()), tok, Variant::Quiet, btn("x")),
            tooltip_wrap(
                label("n", tok, role("n", Role::Status)),
                "tip",
                tok,
                role("tt", Role::Tooltip),
            ),
            badge("ok", tok, Variant::Success, role("ok", Role::Status)),
            group_box(
                "Box",
                label("in", tok, role("in", Role::Status)),
                tok,
                role("box", Role::Group),
            ),
            banner("Hi", None, tok, role("ban", Role::Status)),
            info_bar(ToastKind::Info, "n", tok, role("ib", Role::Status)),
            teaching_tip("t", "b", (), tok, role("tip", Role::Tooltip)),
            placeholder_skeleton(tok, role("sk", Role::Status)),
        ];
        for el in &mut painted {
            draw_once(el);
        }
        let tree = TreeNode::branch(1, "r", vec![TreeNode::leaf(2, "c")]);
        let mut tv = tree_view(
            &tree,
            Some(1),
            |_| (),
            |_| (),
            tok,
            role("tree", Role::Tree),
        );
        draw_once(&mut tv);
        let acc = Accordion { open: Some(0) };
        let mut av = accordion_view(
            &["A".into()],
            vec![label("b", tok, role("b", Role::Header))],
            &acc,
            |_| (),
            tok,
            role("acc", Role::Group),
        );
        draw_once(&mut av);
        let list = VecList {
            items: vec![crate::collection::ListRow::new("a").with_meta("m")],
        };
        let mut lv = list_view(
            &list,
            &Sel::Single(0),
            |_| (),
            tok,
            VisibleWindow::new(80.0),
            24.0,
            2,
            |_| (),
            "Empty",
            |_| tok.muted,
            None,
            RowFace::FLUSH,
            role("list", Role::List),
        );
        draw_once(&mut lv);
        let empty_list = VecList::default();
        let mut empty_lv = list_view(
            &empty_list,
            &Sel::None,
            |_| (),
            tok,
            VisibleWindow::new(80.0),
            24.0,
            2,
            |_| (),
            "Empty",
            |_| tok.muted,
            None,
            RowFace::FLUSH,
            role("list-empty", Role::List),
        );
        draw_once(&mut empty_lv);
        let two = VecList {
            items: vec![
                crate::collection::ListRow::new("a").with_meta("m"),
                crate::collection::ListRow::new("b").with_meta("n"),
            ],
        };
        let mut color_lv = list_view(
            &two,
            &Sel::Single(0),
            |_| (),
            tok,
            VisibleWindow::new(80.0),
            24.0,
            2,
            |_| (),
            "Empty",
            |i| {
                if i == 0 {
                    tok.danger
                } else {
                    tok.muted
                }
            },
            None,
            RowFace::FLUSH,
            role("list-color", Role::List),
        );
        draw_once(&mut color_lv);
        let mut dead_lv = list_view(
            &list,
            &Sel::Single(0),
            |_| (),
            tok,
            VisibleWindow::new(80.0),
            24.0,
            2,
            |_| (),
            "Empty",
            |_| tok.muted,
            None,
            RowFace::FLUSH,
            role("list", Role::List).with_disabled(true),
        );
        draw_once(&mut dead_lv);
        let table = TableModel {
            headers: vec!["A".into()],
            rows: vec![vec!["1".into()]],
            sort_col: None,
            sort_asc: true,
        };
        let widths = crate::collection::ColumnLayout::new(vec![80.0]);
        let mut dt = data_table(
            &table,
            &Sel::Single(0),
            Some((0, 0)),
            &widths,
            true,
            VisibleWindow::new(80.0),
            24.0,
            2,
            |_, _| (),
            |_| (),
            |_| (),
            |_| (),
            tok,
            role("table", Role::Table),
        );
        draw_once(&mut dt);
        let mut dead_dt = data_table(
            &table,
            &Sel::Single(0),
            Some((0, 0)),
            &widths,
            true,
            VisibleWindow::new(80.0),
            24.0,
            2,
            |_, _| (),
            |_| (),
            |_| (),
            |_| (),
            tok,
            role("table", Role::Table).with_disabled(true),
        );
        draw_once(&mut dead_dt);
        let mut dead_tree = tree_view(
            &tree,
            Some(1),
            |_| (),
            |_| (),
            tok,
            role("tree", Role::Tree).with_disabled(true),
        );
        draw_once(&mut dead_tree);
        let mut dead_grid = item_grid(
            &["A".into(), "B".into()],
            |_| (),
            tok,
            role("grid", Role::List).with_disabled(true),
        );
        draw_once(&mut dead_grid);
        let disabled = A11y::button("off").with_disabled(true);
        let mut dead_link = hyperlink("l", (), tok, disabled.clone());
        draw_once(&mut dead_link);
        let mut dead_num = number_input(
            1.0,
            |_| (),
            tok,
            role("n", Role::SpinButton).with_disabled(true),
        );
        draw_once(&mut dead_num);
        let mut dead_ta = textarea(
            &code,
            |_| (),
            tok,
            crate::layout::FILL,
            role("ta", Role::TextBox).with_disabled(true),
        );
        draw_once(&mut dead_ta);
        let mut dead_sel = selectable(
            &code,
            |_| (),
            tok,
            typo::FontFace::Ui,
            role("sel", Role::TextBox).with_disabled(true),
        );
        draw_once(&mut dead_sel);
        let copy = crate::action::Action::new("value.copy", "Copy", ());
        let mut dead_vf = value_field(
            "Path",
            &code,
            |_| (),
            Some(&copy),
            typo::FontFace::Mono,
            tok,
            Direction::Ltr,
            role("vf", Role::Group).with_disabled(true),
        );
        draw_once(&mut dead_vf);
        let mut dead_block = code_block(
            &code,
            |_| (),
            tok,
            role("blk", Role::TextBox).with_disabled(true),
        );
        draw_once(&mut dead_block);
        let mut dead_code = highlighted_code(
            &code,
            "rs",
            |_| (),
            tok,
            "dark",
            crate::layout::FILL,
            role("code", Role::Group).with_disabled(true),
        );
        draw_once(&mut dead_code);
        let mut dead_sw = color_swatch(1, 2, 3, (), tok, btn("c").with_disabled(true));
        draw_once(&mut dead_sw);
        let mut dead_x = dismiss_button((), tok, A11y::button("x").with_disabled(true));
        draw_once(&mut dead_x);
        let toast = Toast {
            id: 1,
            kind: ToastKind::Info,
            text: "t".into(),
            ttl_ms: 10,
        };
        let mut tvw = toast_view(&toast, (), tok, role("t", Role::Status));
        draw_once(&mut tvw);
        let mut closed = accordion_view(
            &["A".into()],
            vec![label("b", tok, role("b", Role::Header))],
            &Accordion { open: None },
            |_| (),
            tok,
            role("acc", Role::Group),
        );
        draw_once(&mut closed);
        let mut exp_shut = expander(
            "Notes",
            label("more", tok, role("more", Role::Status)),
            48.0,
            false,
            |_| (),
            tok,
            role("exp", Role::Group),
        );
        draw_once(&mut exp_shut);
        let mut exp_open = expander(
            "Notes",
            label("more", tok, role("more", Role::Status)),
            48.0,
            true,
            |_| (),
            tok,
            role("exp", Role::Group),
        );
        draw_once(&mut exp_open);
        let mut exp_off = expander(
            "Notes",
            label("more", tok, role("more", Role::Status)),
            48.0,
            false,
            |_| (),
            tok,
            role("exp", Role::Group).with_disabled(true),
        );
        draw_once(&mut exp_off);
    }

    #[test]
    fn secret_field_does_not_name_the_editor_with_the_secret() {
        let src = include_str!("widget.rs");
        let secret_src = src
            .split("pub fn secret_field")
            .nth(1)
            .unwrap()
            .split("pub fn textarea")
            .next()
            .unwrap();
        assert!(secret_src.contains("password_input("));
        assert!(!secret_src.contains("apply_name(value)"));
        assert!(secret_src.contains("a11y.child(Role::TextBox)"));
        let tok = named("dark").tokens;
        let secret = "hunter2-never-in-name";
        let copy = crate::action::Action::new("secret.copy", "Copy", ());
        let _: Element<'_, ()> = secret_field(
            "Token",
            secret,
            |_| (),
            false,
            (),
            &copy,
            tok,
            Direction::Ltr,
            A11y::new("api-token", Role::Group),
        );
        let search_src = src
            .split("pub fn search_input")
            .nth(1)
            .unwrap()
            .split("pub fn themed_pick_list")
            .next()
            .unwrap();
        assert!(!search_src.contains("apply_name(value)"));
        assert!(search_src.contains("a11y.child(Role::TextBox)"));
    }

    #[test]
    fn search_input_disabled_does_not_emit_and_keeps_stable_name() {
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::keyboard;
        use iced::mouse;
        use iced::{Event, Font, Pixels, Point, Rectangle, Size};
        let tok = named("dark").tokens;
        let a11y = A11y::new("find", Role::TextBox).with_disabled(true);
        assert_eq!(a11y.child(Role::TextBox).node_id(), "textbox|find|1");
        let mut el: Element<'_, String> = search_input("typed-query", |s| s, tok, a11y);
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(240.0, 40.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(240.0, 40.0));
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Character("x".into()),
                    modified_key: keyboard::Key::Character("x".into()),
                    physical_key: keyboard::key::Physical::Unidentified(
                        keyboard::key::NativeCode::Unidentified,
                    ),
                    location: keyboard::Location::Standard,
                    modifiers: keyboard::Modifiers::empty(),
                    text: Some("x".into()),
                    repeat: false,
                }),
                layout,
                mouse::Cursor::Available(Point::new(40.0, 12.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        assert!(messages.is_empty());
    }

    #[test]
    fn tree_toggle_expands_and_select_is_a_distinct_id() {
        let mut tree = TreeNode::branch(
            1,
            "root",
            vec![TreeNode::leaf(2, "child"), TreeNode::leaf(3, "other")],
        );
        assert_eq!(tree.children[0].label, "child");
        assert_eq!(tree.children[1].label, "other");
        assert!(crate::collection::tree_toggle(&mut tree, 1));
        assert!(!tree.expanded);
        let visible: Vec<u64> = tree
            .flatten()
            .into_iter()
            .map(|(_, id, _, _, _)| id)
            .collect();
        assert_eq!(visible, vec![1]);
        assert!(crate::collection::tree_toggle(&mut tree, 1));
        assert!(tree.expanded);
        let child = tree.children[0].id;
        let other = tree.children[1].id;
        assert_ne!(child, other);
        let selected = Some(other);
        assert_eq!(selected, Some(3));
        assert!(tree.expanded);
        let tok = named("dark").tokens;
        let _: Element<'_, u64> = tree_view(
            &tree,
            selected,
            |id| id,
            |id| id,
            tok,
            A11y::new("tree", Role::Tree),
        );
        let _: Element<'_, u64> = tree_view(
            &tree,
            None,
            |id| id,
            |id| id,
            tok,
            A11y::new("tree", Role::Tree),
        );
    }

    #[test]
    fn fill_editors_publish_fill_size() {
        let content = Content::with_text("# hi\n");
        let tok = named("dark").tokens;
        let code = highlighted_code(
            &content,
            "md",
            |_| (),
            tok,
            "dark",
            crate::layout::FILL,
            A11y::new("source", Role::TextBox),
        );
        let size = code.as_widget().size();
        assert_eq!(size.width, Length::Fill);
        assert_eq!(size.height, Length::Fill);
        let area = textarea(
            &content,
            |_| (),
            tok,
            crate::layout::FILL,
            A11y::new("body", Role::TextBox),
        );
        let size = area.as_widget().size();
        assert_eq!(size.width, Length::Fill);
        assert_eq!(size.height, Length::Fill);
        let fixed = textarea(
            &content,
            |_| (),
            tok,
            crate::layout::fixed(120.0),
            A11y::new("body", Role::TextBox),
        );
        assert_eq!(fixed.as_widget().size().height, Length::Fixed(120.0));
        let body = selectable(
            &content,
            |_| (),
            tok,
            typo::FontFace::Ui,
            A11y::new("body", Role::TextBox),
        );
        assert_eq!(body.as_widget().size().height, Length::Shrink);
        assert_eq!(body.as_widget().size().width, Length::Fill);
        let block = code_block(&content, |_| (), tok, A11y::new("src", Role::TextBox));
        assert_eq!(block.as_widget().size().height, Length::Shrink);
        let src = include_str!("widget.rs");
        let hl = src
            .split("pub fn highlighted_code")
            .nth(1)
            .unwrap()
            .split("fn editor_frame")
            .next()
            .unwrap();
        assert!(hl.contains("select_only"));
        assert!(!hl.contains("if !a11y.disabled"));
    }

    #[test]
    fn select_only_keeps_selection_and_drops_edits() {
        use iced::widget::text_editor::{Action, Edit, Motion};
        assert_eq!(
            select_only(Action::Edit(Edit::Insert('x'))),
            Action::Scroll { lines: 0 }
        );
        assert_eq!(
            select_only(Action::Edit(Edit::Delete)),
            Action::Scroll { lines: 0 }
        );
        assert_eq!(select_only(Action::SelectAll), Action::SelectAll);
        assert_eq!(
            select_only(Action::Select(Motion::Right)),
            Action::Select(Motion::Right)
        );
        assert_eq!(
            select_only(Action::Scroll { lines: 3 }),
            Action::Scroll { lines: 3 }
        );
        assert_eq!(
            select_only(Action::Move(Motion::Left)),
            Action::Move(Motion::Left)
        );
        let click = Action::Click(iced::Point::new(1.0, 2.0));
        assert_eq!(select_only(click.clone()), click);
        let drag = Action::Drag(iced::Point::new(4.0, 5.0));
        assert_eq!(select_only(drag.clone()), drag);
        let _ = selectable_style(named("dark").tokens);
    }

    #[test]
    fn list_and_table_forward_scrollable_offset() {
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::mouse;
        use iced::{Event, Font, Pixels, Point, Rectangle, Size};

        let tok = named("dark").tokens;
        let list = VecList {
            items: (0..80)
                .map(|i| crate::collection::ListRow::new(format!("r{i}")))
                .collect(),
        };
        let window = VisibleWindow {
            start: 0,
            end: 0,
            scroll: 0.0,
            viewport: 200.0,
        };
        let drive = |el: &mut Element<'_, VisibleWindow>| {
            let mut tree = Tree::new(el.as_widget());
            let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
                Font::DEFAULT,
                Pixels::from(16u32),
            ));
            let limits = Limits::new(Size::ZERO, Size::new(320.0, 200.0));
            let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
            let layout = Layout::new(&node);
            let viewport = Rectangle::new(Point::ORIGIN, Size::new(320.0, 200.0));
            let mut clipboard = clipboard::Null;
            let mut messages = Vec::new();
            {
                let mut shell = iced::advanced::Shell::new(&mut messages);
                el.as_widget_mut().update(
                    &mut tree,
                    &Event::Mouse(mouse::Event::WheelScrolled {
                        delta: mouse::ScrollDelta::Lines { x: 0.0, y: -10.0 },
                    }),
                    layout,
                    mouse::Cursor::Available(Point::new(16.0, 40.0)),
                    &renderer,
                    &mut clipboard,
                    &mut shell,
                    &viewport,
                );
            }
            messages
        };
        let mut list_el: Element<'_, VisibleWindow> = list_view(
            &list,
            &Sel::Single(0),
            |_| window,
            tok,
            window,
            20.0,
            4,
            |w| w,
            "Empty",
            |_| tok.muted,
            Some(Id::from("list-scroll")),
            RowFace::FLUSH,
            A11y::new("list", Role::List),
        );
        let _ = drive(&mut list_el);
        let table = TableModel {
            headers: vec!["N".into(), "M".into()],
            rows: (0..80)
                .map(|i| vec![format!("r{i}"), format!("m{i}")])
                .collect(),
            sort_col: None,
            sort_asc: true,
        };
        let tw = crate::collection::ColumnLayout::new(vec![80.0, 80.0]);
        let mut table_el: Element<'_, VisibleWindow> = data_table(
            &table,
            &Sel::Single(0),
            Some((0, 0)),
            &tw,
            true,
            window,
            20.0,
            4,
            |_, _| window,
            |_| window,
            |w| w,
            |_| window,
            tok,
            A11y::new("table", Role::Table),
        );
        let _ = drive(&mut table_el);
        let after =
            crate::collection::window_after_scroll(window, 4.0, 200.0, 20.0, 80, 4, Some(0));
        assert_eq!(after.start, 0);
        assert!((after.scroll - 4.0).abs() < 0.01);
        let mut scroller: Element<'_, f32> = themed_scroll(
            iced::widget::column![
                label("a", tok, A11y::new("a", Role::Status)),
                Space::new().height(800.0),
            ]
            .into(),
            tok,
            A11y::new("scroll", Role::Group),
            false,
            Some(Id::from("scroll-host")),
            None::<fn(_) -> f32>,
        );
        let msgs = drive_scroll(&mut scroller);
        let _ = msgs;
    }

    fn walk_bounds(layout: iced::advanced::layout::Layout<'_>, out: &mut Vec<iced::Rectangle>) {
        out.push(layout.bounds());
        for child in layout.children() {
            walk_bounds(child, out);
        }
    }

    #[test]
    fn list_view_rail_drag_updates_scroll_without_changing_range() {
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::mouse;
        use iced::{Event, Font, Pixels, Point, Rectangle, Size};

        let tok = named("dark").tokens;
        let list = VecList {
            items: (0..80)
                .map(|i| crate::collection::ListRow::new(format!("r{i}")))
                .collect(),
        };
        let row_h = 20.0;
        let viewport = 200.0;
        let window = crate::collection::visible_window(2.0, viewport, row_h, 80, 4, Some(0));
        let start = window.start;
        let end = window.end;
        let mut el: Element<'_, VisibleWindow> = list_view(
            &list,
            &Sel::Single(0),
            |_| window,
            tok,
            window,
            row_h,
            4,
            |w| w,
            "Empty",
            |_| tok.muted,
            Some(Id::from("list-rail")),
            RowFace::FLUSH,
            A11y::new("list", Role::List),
        );
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(320.0, viewport));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let mut boxes = Vec::new();
        walk_bounds(layout, &mut boxes);
        let rail_w = crate::chrome::SCROLL_RAIL_WIDTH;
        let rb = *boxes
            .iter()
            .find(|b| (b.width - rail_w).abs() < 0.6 && b.height > 20.0)
            .expect("rail");
        let content = 80.0 * row_h;
        let (thumb_y, thumb_h) = crate::collection::scroller_span(
            content,
            viewport,
            window.scroll,
            rb.height,
            crate::chrome::SCROLL_HANDLE_MIN,
        );
        let grab = Point::new(rb.x + rb.width / 2.0, rb.y + thumb_y + thumb_h / 2.0);
        let moved = Point::new(grab.x, grab.y + 1.0);
        let vp = Rectangle::new(Point::ORIGIN, Size::new(320.0, viewport));
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(grab),
                &renderer,
                &mut clipboard,
                &mut shell,
                &vp,
            );
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::CursorMoved { position: moved }),
                layout,
                mouse::Cursor::Available(moved),
                &renderer,
                &mut clipboard,
                &mut shell,
                &vp,
            );
        }
        assert!(!messages.is_empty());
        let pixel = messages
            .iter()
            .copied()
            .find(|w| w.start == start && w.end == end && (w.scroll - window.scroll).abs() > 0.5)
            .expect("rail drag moves scroll without remounting");
        let (y0, _) = crate::collection::scroller_span(
            content,
            viewport,
            window.scroll,
            rb.height,
            crate::chrome::SCROLL_HANDLE_MIN,
        );
        let (y1, _) = crate::collection::scroller_span(
            content,
            viewport,
            pixel.scroll,
            rb.height,
            crate::chrome::SCROLL_HANDLE_MIN,
        );
        assert!(y1 > y0);
    }

    #[test]
    fn list_view_paints_each_row_at_index_times_height_minus_scroll() {
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels, Size};

        let tok = named("dark").tokens;
        let row_h = 20.0;
        let viewport = 200.0;
        let scroll = 200.0;
        let overscan = 4;
        let n = 80;
        let list = VecList {
            items: (0..n)
                .map(|i| crate::collection::ListRow::new(format!("r{i}")))
                .collect(),
        };
        let window = crate::collection::visible_window(scroll, viewport, row_h, n, overscan, None);
        let mut el: Element<'_, VisibleWindow> = list_view(
            &list,
            &Sel::None,
            |_| window,
            tok,
            window,
            row_h,
            overscan,
            |w| w,
            "Empty",
            |_| tok.muted,
            None,
            RowFace::FLUSH,
            A11y::new("list", Role::List),
        );
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(320.0, viewport));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let origin = layout.bounds();
        let mut boxes = Vec::new();
        walk_bounds(layout, &mut boxes);
        let rows: Vec<f32> = boxes
            .iter()
            .filter(|b| (b.height - row_h).abs() < 0.6 && b.width > 40.0)
            .map(|b| b.y - origin.y)
            .collect();
        must(
            rows.iter().any(|y| y.abs() < 1.0),
            format!("row 10 must sit at the clip top, got {rows:?}"),
        );
        for y in &rows {
            let along = y + scroll;
            let i = (along / row_h).round();
            must(
                (along - i * row_h).abs() < 1.0,
                format!("painted y {y} is not k*{row_h} - {scroll}"),
            );
        }
        let sep_list = VecList {
            items: vec![
                crate::collection::ListRow::new("a"),
                crate::collection::ListRow::separator(),
                crate::collection::ListRow::new("b"),
            ],
        };
        let sep_h = 36.0;
        let sep_win = VisibleWindow::new(140.0);
        let mut sep_el: Element<'_, VisibleWindow> = list_view(
            &sep_list,
            &Sel::None,
            |_| sep_win,
            tok,
            sep_win,
            sep_h,
            1,
            |w| w,
            "Empty",
            |_| tok.muted,
            None,
            RowFace::FLUSH,
            A11y::new("sep-list", Role::List),
        );
        let mut sep_tree = Tree::new(sep_el.as_widget());
        let sep_limits = Limits::new(Size::ZERO, Size::new(320.0, 140.0));
        let sep_node = sep_el
            .as_widget_mut()
            .layout(&mut sep_tree, &renderer, &sep_limits);
        let sep_layout = Layout::new(&sep_node);
        let sep_origin = sep_layout.bounds();
        let mut sep_boxes = Vec::new();
        walk_bounds(sep_layout, &mut sep_boxes);
        let sep_rows: Vec<f32> = sep_boxes
            .iter()
            .filter(|b| (b.height - sep_h).abs() < 0.6 && b.width > 40.0)
            .map(|b| b.y - sep_origin.y)
            .collect();
        must(
            sep_rows.iter().any(|y| (*y - sep_h).abs() < 1.0),
            format!("separator index 1 must sit at y={sep_h}, got {sep_rows:?}"),
        );
    }

    #[test]
    fn list_view_variable_row_is_not_index_times_uniform() {
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels, Size};

        let tok = named("dark").tokens;
        let heights = [20.0_f32, 50.0, 20.0];
        let list = VecList {
            items: vec![
                crate::collection::ListRow::new("a").with_meta("this morning"),
                crate::collection::ListRow::new("b").with_meta("yesterday"),
                crate::collection::ListRow::new("c"),
            ],
        };
        let window = crate::collection::visible_window_var(0.0, 200.0, &heights, 0, None);
        let mut el: Element<'_, f32> = list_view(
            &list,
            &Sel::None,
            |_| 0.0,
            tok,
            window,
            crate::collection::RowHeights::PerRow(&heights),
            0,
            |w| w.scroll,
            "Empty",
            |_| tok.muted,
            None,
            RowFace::FLUSH,
            A11y::new("var-list", Role::List),
        );
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(320.0, 200.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let origin = layout.bounds();
        let mut boxes = Vec::new();
        walk_bounds(layout, &mut boxes);
        let tall: Vec<f32> = boxes
            .iter()
            .filter(|b| (b.height - 50.0).abs() < 1.0 && b.width > 40.0)
            .map(|b| b.y - origin.y)
            .collect();
        must(
            tall.iter().any(|y| (*y - 20.0).abs() < 2.0),
            format!("second row starts at 20px, not 2*uniform, got {tall:?}"),
        );
        let _ = drive_scroll(&mut el);
        let mut cards: Element<'_, ()> = list_view(
            &list,
            &Sel::Single(0),
            |_| (),
            tok,
            window,
            crate::collection::RowHeights::PerRow(&heights),
            0,
            |_| (),
            "Empty",
            |_| tok.muted,
            None,
            RowFace::Card {
                meter: Some(|i| if i == 0 { 0.8 } else { 0.2 }),
            },
            A11y::new("card-list", Role::List),
        );
        let mut tree_c = Tree::new(cards.as_widget());
        let node_c = cards.as_widget_mut().layout(
            &mut tree_c,
            &renderer,
            &Limits::new(Size::ZERO, Size::new(320.0, 200.0)),
        );
        let layout_c = Layout::new(&node_c);
        let origin_c = layout_c.bounds();
        let mut boxes_c = Vec::new();
        walk_bounds(layout_c, &mut boxes_c);
        let cards_y: Vec<f32> = boxes_c
            .iter()
            .filter(|b| (b.height - 50.0).abs() < 1.0 && b.width > 40.0)
            .map(|b| b.y - origin_c.y)
            .collect();
        must(
            cards_y.iter().any(|y| (*y - 22.0).abs() < 3.0),
            format!("card gap is 2px so the 50px row starts at 22, got {cards_y:?}"),
        );
        draw_once(&mut cards);
        let bare = VecList {
            items: vec![crate::collection::ListRow::new("only")],
        };
        let mut no_meter: Element<'_, ()> = list_view(
            &bare,
            &Sel::None,
            |_| (),
            tok,
            VisibleWindow::new(80.0),
            72.0,
            0,
            |_| (),
            "Empty",
            |_| tok.muted,
            None,
            RowFace::Card {
                meter: None::<fn(usize) -> f32>,
            },
            A11y::new("card-bare", Role::List).with_disabled(true),
        );
        draw_once(&mut no_meter);
        let mut tree0 = Tree::new(el.as_widget());
        let renderer0 = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let _ = el.as_widget_mut().layout(
            &mut tree0,
            &renderer0,
            &Limits::new(Size::ZERO, Size::new(320.0, 0.0)),
        );
    }

    #[test]
    fn data_table_frozen_column_stays_when_unfrozen_scrolls() {
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::mouse;
        use iced::{Event, Font, Pixels, Point, Rectangle, Size};

        let tok = named("dark").tokens;
        let table = TableModel {
            headers: vec!["A".into(), "B".into(), "C".into()],
            rows: vec![vec!["1".into(), "2".into(), "3".into()]],
            sort_col: None,
            sort_asc: true,
        };
        let mut cols = crate::collection::ColumnLayout::new(vec![80.0, 80.0, 80.0]).with_frozen(1);
        cols.set_h_scroll(0.0);
        let window = VisibleWindow::new(80.0);
        let leading = |cols: &crate::collection::ColumnLayout| {
            let mut el: Element<'_, ()> = data_table(
                &table,
                &Sel::None,
                None,
                cols,
                false,
                window,
                24.0,
                0,
                |_, _| (),
                |_| (),
                |_| (),
                |_| (),
                tok,
                A11y::new("pin", Role::Table),
            );
            let mut tree = Tree::new(el.as_widget());
            let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
                Font::DEFAULT,
                Pixels::from(16u32),
            ));
            let limits = Limits::new(Size::ZERO, Size::new(200.0, 80.0));
            let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
            let layout = Layout::new(&node);
            let mut boxes = Vec::new();
            walk_bounds(layout, &mut boxes);
            let xs: Vec<f32> = boxes
                .iter()
                .filter(|b| (b.width - 80.0).abs() < 1.0)
                .map(|b| b.x)
                .collect();
            xs
        };
        let x0 = leading(&cols);
        must(
            x0.iter().any(|x| x.abs() < 1.0),
            format!("frozen column missing at x=0: {x0:?}"),
        );
        cols.set_h_scroll(90.0);
        let x1 = leading(&cols);
        must(
            x1.iter().any(|x| x.abs() < 1.0),
            format!("frozen column left the leading edge: {x1:?}"),
        );
        must(
            x1.iter().any(|x| *x < -1.0),
            format!("unfrozen columns did not scroll: {x1:?}"),
        );
        let mut el: Element<'_, f32> = data_table(
            &table,
            &Sel::None,
            None,
            &cols,
            false,
            window,
            24.0,
            0,
            |_, _| 0.0,
            |_| 0.0,
            |_| 0.0,
            |x| x,
            tok,
            A11y::new("pin-h", Role::Table),
        );
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(200.0, 80.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let vp = Rectangle::new(Point::ORIGIN, Size::new(200.0, 80.0));
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels { x: 20.0, y: 0.0 },
                }),
                layout,
                mouse::Cursor::Available(Point::new(120.0, 12.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &vp,
            );
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines { x: 1.0, y: 0.0 },
                }),
                layout,
                mouse::Cursor::Available(Point::new(120.0, 12.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &vp,
            );
        }
        assert!(!messages.is_empty());
    }

    #[test]
    fn data_table_disabled_zebra_and_cursor_still_layout() {
        let tok = named("dark").tokens;
        let table = TableModel {
            headers: vec!["A".into(), "B".into()],
            rows: vec![vec!["1".into(), "2".into()], vec!["3".into(), "4".into()]],
            sort_col: None,
            sort_asc: true,
        };
        let cols = crate::collection::ColumnLayout::new(vec![80.0, 80.0]);
        let window = VisibleWindow::new(80.0);
        let mut el: Element<'_, ()> = data_table(
            &table,
            &Sel::Single(1),
            Some((1, 1)),
            &cols,
            true,
            window,
            24.0,
            0,
            |_, _| (),
            |_| (),
            |_| (),
            |_| (),
            tok,
            A11y::new("zebra", Role::Table).with_disabled(true),
        );
        draw_once(&mut el);
        let empty = TableModel {
            headers: vec!["A".into()],
            rows: vec![],
            sort_col: None,
            sort_asc: true,
        };
        let one = crate::collection::ColumnLayout::new(vec![80.0]);
        let mut headers_only: Element<'_, ()> = data_table(
            &empty,
            &Sel::None,
            None,
            &one,
            false,
            window,
            24.0,
            0,
            |_, _| (),
            |_| (),
            |_| (),
            |_| (),
            tok,
            A11y::new("empty-table", Role::Table),
        );
        draw_once(&mut headers_only);
    }

    #[test]
    fn log_view_wheel_forwards_scroll_offset() {
        let tok = named("dark").tokens;
        let lines: Vec<String> = (0..80).map(|i| format!("line {i}")).collect();
        let mut el: Element<'_, f32> = log_view(
            &lines,
            VisibleWindow::new(80.0),
            18.0,
            2,
            |w| w.scroll,
            Some(Id::from("log-drive")),
            tok,
            A11y::new("log-drive", Role::List),
        );
        let _ = drive_scroll(&mut el);
    }

    fn drive_scroll(el: &mut Element<'_, f32>) -> Vec<f32> {
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::mouse;
        use iced::{Event, Font, Pixels, Point, Rectangle, Size};
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(320.0, 120.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(320.0, 120.0));
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines { x: 0.0, y: -4.0 },
                }),
                layout,
                mouse::Cursor::Available(Point::new(16.0, 40.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        messages
    }

    #[test]
    fn disabled_radio_does_not_emit_on_press() {
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::mouse;
        use iced::{Event, Font, Pixels, Point, Rectangle, Size};
        let tok = named("dark").tokens;
        let mut el: Element<'_, u8> = themed_radio(
            "Locked",
            1u8,
            Some(1u8),
            |_| 9u8,
            tok,
            A11y::new("Locked", Role::Radio).with_disabled(true),
        );
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(240.0, 40.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(240.0, 40.0));
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(Point::new(8.0, 8.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        assert!(messages.is_empty());
    }

    #[test]
    fn disabled_slider_and_pick_do_not_emit() {
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::mouse;
        use iced::{Event, Font, Pixels, Point, Rectangle, Size};
        let tok = named("dark").tokens;
        let mut slider_el: Element<'_, u8> = themed_slider(
            0.0..=1.0,
            0.5,
            |_| 1u8,
            tok,
            A11y::new("vol", Role::Slider).with_disabled(true),
        );
        let opts = ["a".to_string(), "b".to_string()];
        let mut pick_el: Element<'_, u8> = themed_pick_list(
            opts,
            Some("a".into()),
            |_| 2u8,
            tok,
            A11y::new("pick", Role::ComboBox).with_disabled(true),
        );
        for el in [&mut slider_el, &mut pick_el] {
            let mut tree = Tree::new(el.as_widget());
            let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
                Font::DEFAULT,
                Pixels::from(16u32),
            ));
            let limits = Limits::new(Size::ZERO, Size::new(240.0, 40.0));
            let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
            let layout = Layout::new(&node);
            let viewport = Rectangle::new(Point::ORIGIN, Size::new(240.0, 40.0));
            let mut clipboard = clipboard::Null;
            let mut messages = Vec::new();
            {
                let mut shell = iced::advanced::Shell::new(&mut messages);
                el.as_widget_mut().update(
                    &mut tree,
                    &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                    layout,
                    mouse::Cursor::Available(Point::new(12.0, 12.0)),
                    &renderer,
                    &mut clipboard,
                    &mut shell,
                    &viewport,
                );
            }
            assert!(messages.is_empty());
        }
    }

    #[test]
    fn data_table_paints_each_row_at_index_times_height_minus_scroll() {
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels, Size};

        let tok = named("dark").tokens;
        let row_h = 20.0;
        let viewport = 200.0;
        let scroll = 200.0;
        let overscan = 4;
        let n = 80;
        let table = TableModel {
            headers: vec!["N".into()],
            rows: (0..n).map(|i| vec![format!("r{i}")]).collect(),
            sort_col: None,
            sort_asc: true,
        };
        let widths = crate::collection::ColumnLayout::new(vec![80.0]);
        let window = crate::collection::visible_window(scroll, viewport, row_h, n, overscan, None);
        let mut el: Element<'_, VisibleWindow> = data_table(
            &table,
            &Sel::None,
            None,
            &widths,
            false,
            window,
            row_h,
            overscan,
            |_, _| window,
            |_| window,
            |w| w,
            |_| window,
            tok,
            A11y::new("table", Role::Table),
        );
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(320.0, viewport));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let mut boxes = Vec::new();
        walk_bounds(layout, &mut boxes);
        let rail_w = crate::chrome::SCROLL_RAIL_WIDTH;
        let clip_top = boxes
            .iter()
            .find(|b| (b.width - rail_w).abs() < 0.6 && b.height > 20.0)
            .map(|b| b.y)
            .expect("table rail");
        let rows: Vec<f32> = boxes
            .iter()
            .filter(|b| (b.height - row_h).abs() < 0.6 && b.width > 40.0)
            .map(|b| b.y - clip_top)
            .collect();
        must(
            rows.iter().any(|y| y.abs() < 1.0),
            format!("row 10 must sit at the clip top, got {rows:?}"),
        );
        for y in &rows {
            let along = y + scroll;
            let i = (along / row_h).round();
            must(
                (along - i * row_h).abs() < 1.0,
                format!("painted y {y} is not k*{row_h} - {scroll}"),
            );
        }
    }

    #[test]
    fn expander_closed_clips_to_collapsed_height() {
        let tok = named("dark").tokens;
        let a11y = |n: &str| A11y::new(n, Role::Status);
        let tall = || {
            Column::new()
                .spacing(8)
                .push(label("one", tok, a11y("one")))
                .push(label("two", tok, a11y("two")))
                .push(label("three", tok, a11y("three")))
                .push(label("four", tok, a11y("four")))
                .push(label("five", tok, a11y("five")))
                .push(label("six", tok, a11y("six")))
                .into()
        };
        let mut shut: Element<'_, bool> = expander(
            "Notes",
            tall(),
            48.0,
            false,
            |open| open,
            tok,
            A11y::new("exp", Role::Group),
        );
        let mut open: Element<'_, bool> = expander(
            "Notes",
            tall(),
            48.0,
            true,
            |open| open,
            tok,
            A11y::new("exp", Role::Group),
        );
        let max = iced::Size::new(320.0, 800.0);
        let closed_h = layout_size(&mut shut, max).height;
        let open_h = layout_size(&mut open, max).height;
        assert!(open_h > closed_h + 8.0);
        assert!(closed_h >= 48.0 + 12.0);
        let src = include_str!("widget.rs");
        let head = src
            .split("fn disclosure_header")
            .nth(1)
            .unwrap()
            .split("pub fn accordion_view")
            .next()
            .unwrap();
        assert!(head.contains("Icon::Chevron"));
        assert!(head.contains("Length::Fill"));
        assert!(!head.contains("▾"));
    }

    #[test]
    fn expander_title_shares_the_card_inset() {
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels, Size};

        let tok = named("dark").tokens;
        let mut el: Element<'_, bool> = expander(
            "Notes",
            label("body-line", tok, A11y::new("body-line", Role::Status)),
            Peek::Lines(2),
            true,
            |open| open,
            tok,
            A11y::new("exp", Role::Group),
        );
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let node = el.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &Limits::new(Size::ZERO, Size::new(320.0, 400.0)),
        );
        let layout = Layout::new(&node);
        let mut boxes = Vec::new();
        walk_bounds(layout, &mut boxes);
        let lefts: Vec<f32> = boxes
            .iter()
            .filter(|b| b.x > 1.0 && b.x < 40.0 && b.width > 24.0 && b.width < 280.0)
            .map(|b| b.x)
            .collect();
        assert!(
            lefts.iter().any(|x| (*x - 12.0).abs() < 2.0),
            "title and body should sit on the 12px card inset, got {lefts:?}"
        );
        assert!(
            !lefts.iter().any(|x| (*x - 24.0).abs() < 2.0),
            "header must not add a second 12px inset, got {lefts:?}"
        );
    }

    #[test]
    fn peek_lines_keeps_the_last_line_inside_the_clip() {
        let one = Peek::Lines(1).height();
        let two = Peek::Lines(2).height();
        assert!(two > one);
        let raw = 2.0 * Peek::body_line() + Peek::DESCENT;
        assert!(two >= raw);
        assert_eq!(Peek::Pixels(48.0).height(), 48.0);
        assert_eq!(Peek::Pixels(45.0).height(), 48.0);
        let mut lined: Element<'_, bool> = expander(
            "Notes",
            label(
                "more",
                named("dark").tokens,
                A11y::new("more", Role::Status),
            ),
            Peek::Lines(2),
            false,
            |open| open,
            named("dark").tokens,
            A11y::new("exp", Role::Group),
        );
        let h = layout_size(&mut lined, iced::Size::new(320.0, 800.0)).height;
        assert!(h >= Peek::Lines(2).height());
    }

    fn layout_size<M: Clone>(el: &mut Element<'_, M>, max: iced::Size) -> iced::Size {
        use iced::advanced::layout::Limits;
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels};
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(iced::Size::ZERO, max);
        el.as_widget_mut()
            .layout(&mut tree, &renderer, &limits)
            .size()
    }

    #[test]
    fn image_slot_ready_keeps_the_requested_box() {
        let tok = named("dark").tokens;
        let px = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
        ];
        let handle = iced::widget::image::Handle::from_rgba(2, 2, px);
        for fit in [iced::ContentFit::Contain, iced::ContentFit::Cover] {
            let mut el: Element<'_, ()> = image_slot(
                ImageSlot::Ready {
                    handle: handle.clone(),
                    fit,
                },
                120.0,
                80.0,
                tok,
                A11y::new("img", Role::Image),
            );
            let size = layout_size(&mut el, iced::Size::new(400.0, 400.0));
            assert_eq!(size, iced::Size::new(120.0, 80.0), "{fit:?}");
        }
    }

    #[test]
    fn item_grid_cells_share_the_row_width() {
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels, Size};
        let tok = named("dark").tokens;
        let labels = vec!["Inbox".into(), "Calendar".into(), "Mail".into()];
        let mut el: Element<'_, ()> =
            item_grid(&labels, |_| (), tok, A11y::new("grid", Role::List));
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(300.0, 240.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        assert!((node.size().width - 300.0).abs() < 1.0);
        let layout = Layout::new(&node);
        let mut widths = Vec::new();
        fn walk_row(layout: Layout<'_>, widths: &mut Vec<f32>) {
            let kids: Vec<_> = layout.children().collect();
            if kids.len() == 3 && kids.iter().all(|c| c.bounds().width > 40.0) {
                *widths = kids.iter().map(|c| c.bounds().width).collect();
                return;
            }
            for k in kids {
                walk_row(k, widths);
            }
        }
        walk_row(layout, &mut widths);
        assert_eq!(widths.len(), 3);
        assert!((widths[0] - widths[1]).abs() < 1.0);
        assert!((widths[1] - widths[2]).abs() < 1.0);
        must(
            (widths[0] * 3.0 + 16.0 - 300.0).abs() < 4.0,
            format!("cells {widths:?} should share the 300px row"),
        );
    }

    #[test]
    fn open_accordion_shows_a_body_under_its_header() {
        let tok = named("dark").tokens;
        let titles = ["Files".into()];
        let body = || label("New, Open, Save", tok, A11y::new("body", Role::Status));
        let mut closed: Element<'_, ()> = accordion_view(
            &titles,
            vec![body()],
            &Accordion { open: None },
            |_| (),
            tok,
            A11y::new("acc", Role::Group),
        );
        let mut open: Element<'_, ()> = accordion_view(
            &titles,
            vec![body()],
            &Accordion { open: Some(0) },
            |_| (),
            tok,
            A11y::new("acc", Role::Group),
        );
        let max = iced::Size::new(300.0, 400.0);
        let closed_h = layout_size(&mut closed, max).height;
        let open_size = layout_size(&mut open, max);
        must(
            open_size.height > closed_h + 8.0,
            format!(
                "open {} must include the body above closed {closed_h}",
                open_size.height
            ),
        );
        assert!((open_size.width - 300.0).abs() < 1.0);
    }
}
