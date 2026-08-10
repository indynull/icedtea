//! Themed iced widget constructors for `view`. Every export returns an
//! [`iced::Element`] and is keyboard-complete via iced.

use iced::widget::canvas::Canvas;
use iced::widget::markdown;
use iced::widget::scrollable::{Direction as ScrollDir, Scrollbar};
use iced::widget::text_editor::Content;
use iced::widget::{
    button, checkbox, column, container, mouse_area, pick_list, progress_bar, radio, row, rule,
    scrollable, slider, svg, text, text_editor, text_input, toggler, tooltip, Id, Space,
};
use iced::{Alignment, Element, Length, Padding};

use crate::chrome::SCROLL_RAIL_WIDTH;
use crate::host_canvas::ArcRing;

use crate::a11y::{self, A11y, Role};
use crate::collection::{
    page_range, virtual_pads, window_after_scroll, Accordion, ListModel, Selection, TableModel,
    Tabs, TreeNode, VisibleWindow,
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

/// Large reading on the type scale, end-aligned (a tool's current value).
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

pub fn meta<'a, M: 'a>(s: impl Into<String>, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    let s = a11y.apply_name(s);
    a11y::attach(text(s).size(typo::META).color(tok.muted).into(), &a11y)
}

pub fn code_block<'a, M: 'a>(s: impl Into<String>, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    let s = a11y.apply_name(s);
    a11y::attach(
        container(text(s).size(typo::CODE).font(typo::MONO).color(tok.text))
            .padding(12)
            .style(move |_| style::panel(tok))
            .into(),
        &a11y,
    )
}

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
            themed_button(
                "▾",
                more_msg,
                tok,
                Variant::Quiet,
                A11y::button("more").with_disabled(a11y.disabled),
            ),
        ]
        .spacing(2)
        .into(),
        &a11y,
    )
}

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
    a11y::attach(
        radio(name, value, selected, msg)
            .style(style::radio_style(tok))
            .into(),
        &a11y,
    )
}

pub fn themed_slider<'a, M: Clone + 'a>(
    range: std::ops::RangeInclusive<f32>,
    value: f32,
    msg: impl Fn(f32) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    a11y::attach(
        slider(range, value, msg)
            .style(style::slider_style(tok))
            .into(),
        &a11y,
    )
}

pub fn progress<'a, M: 'a>(value: f32, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    a11y::attach(
        progress_bar(0.0..=1.0, value.clamp(0.0, 1.0))
            .style(style::progress_style(tok))
            .into(),
        &a11y,
    )
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
pub fn progress_ring<'a, M: 'a>(value: f32, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    let (start, end) = ring_angles(value);
    a11y::attach(
        Canvas::new(ArcRing {
            start,
            end,
            color: tok.primary,
            track: tok.panel,
        })
        .width(56)
        .height(56)
        .into(),
        &a11y,
    )
}

/// Indeterminate spinner: a quarter-arc at `phase` (0..=1).
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

/// Bitmap the application owns (`Handle::from_bytes` / path).
pub fn image<'a, M: 'a>(
    handle: iced::widget::image::Handle,
    width: impl Into<Length>,
    height: impl Into<Length>,
    a11y: A11y,
) -> Element<'a, M> {
    a11y::attach(
        iced::widget::image(handle)
            .width(width)
            .height(height)
            .into(),
        &a11y,
    )
}

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

pub fn themed_text_input<'a, M: Clone + 'a>(
    placeholder: &str,
    value: &str,
    on_input: impl Fn(String) -> M + 'a,
    on_submit: Option<M>,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let mut i = text_input(placeholder, value)
        .style(style::search_style(tok))
        .padding(pad());
    if !a11y.disabled {
        i = i.on_input(on_input);
        if let Some(m) = a11y.apply_message(on_submit) {
            i = i.on_submit(m);
        }
    }
    a11y::attach(i.into(), &a11y)
}

pub fn password_input<'a, M: Clone + 'a>(
    placeholder: &str,
    value: &str,
    on_input: impl Fn(String) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let mut i = text_input(placeholder, value)
        .secure(true)
        .style(style::search_style(tok))
        .padding(pad());
    if !a11y.disabled {
        i = i.on_input(on_input);
    }
    a11y::attach(i.into(), &a11y)
}

/// Multiline editor. `height` is icedtea size language ([`crate::layout::FILL`]
/// or [`crate::layout::fixed`]).
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

/// Syntax-highlighted code. `syntax` is an iced highlighter token (`rs`, `py`, …).
/// `theme_name` picks a highlighter face that fits the UI colorway.
/// `height` is icedtea size language ([`crate::layout::FILL`] or
/// [`crate::layout::fixed`]).
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
    let mut e = text_editor(content)
        .height(height)
        .padding(8)
        .style(editor_style(tok))
        .highlight(syntax, theme)
        .font(typo::MONO);
    if !a11y.disabled {
        e = e.on_action(on_action);
    }
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
                A11y::new(a11y.apply_name(value), Role::TextBox),
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into(),
        &a11y,
    )
}

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
                A11y::button("previous-day"),
            ),
            label(
                shown.clone(),
                tok,
                A11y::new(a11y.apply_name(shown), Role::SpinButton),
            ),
            themed_button(
                ">",
                a11y.apply_message(Some(on_next)),
                tok,
                Variant::Quiet,
                A11y::button("next-day"),
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into(),
        &a11y,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeValue {
    pub hour: u8,
    pub minute: u8,
}

impl TimeValue {
    pub fn clamp(self) -> Self {
        Self {
            hour: self.hour.min(23),
            minute: self.minute.min(59),
        }
    }
}

pub fn time_picker<'a, M: Clone + 'a>(
    value: TimeValue,
    on_hour: M,
    on_minute: M,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let v = value.clamp();
    a11y::attach(
        row![
            themed_button(
                format!("{:02}", v.hour),
                a11y.apply_message(Some(on_hour)),
                tok,
                Variant::Quiet,
                A11y::button("hour"),
            ),
            label(":", tok, A11y::new(":", Role::Separator)),
            themed_button(
                format!("{:02}", v.minute),
                a11y.apply_message(Some(on_minute)),
                tok,
                Variant::Quiet,
                A11y::button("minute"),
            ),
        ]
        .spacing(4)
        .into(),
        &a11y,
    )
}

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
/// [`markdown_view`] borrows the items.
///
/// ```
/// let doc = icedtea::widget::MarkdownDoc::parse("# Hi\n\nA paragraph.");
/// assert!(!doc.items.is_empty());
/// let again = icedtea::widget::parse("# Hi\n\nA paragraph.");
/// assert_eq!(doc.hash, again.hash);
/// assert_ne!(doc.hash, icedtea::widget::parse("# Other").hash);
/// ```
#[derive(Debug, Clone)]
pub struct MarkdownDoc {
    pub source: String,
    pub hash: u64,
    pub items: Vec<markdown::Item>,
}

impl MarkdownDoc {
    pub fn parse(source: impl Into<String>) -> Self {
        parse(&source.into())
    }
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

pub fn markdown_view<'a, M: Clone + 'a>(
    items: &'a [markdown::Item],
    _tok: Tokens,
    on_link: impl Fn(markdown::Uri) -> M + 'a,
    a11y: A11y,
) -> Element<'a, M> {
    a11y::attach(
        markdown::view(
            items,
            markdown::Settings::with_style(markdown::Style::from_palette(iced::theme::Palette {
                background: _tok.canvas,
                text: _tok.text,
                primary: _tok.primary,
                success: _tok.success,
                warning: _tok.warning,
                danger: _tok.danger,
            })),
        )
        .map(on_link),
        &a11y,
    )
}

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
                .style(move |_| style::raised_card(tok)),
            tooltip::Position::FollowCursor,
        )
        .into(),
        &a11y,
    )
}

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
        Variant::Quiet | Variant::Ghost | Variant::Chip => crate::theme::chip_fill(tok),
    }
}

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
            A11y::button(format!("dismiss {title}")),
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
        .style(move |_| style::card(tok, false))
        .into(),
        &a11y,
    )
}

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
            Some(m),
            tok,
            Variant::Quiet,
            A11y::button(t),
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

pub fn breadcrumb<'a, M: Clone + 'a>(
    parts: &[(String, Option<M>)],
    tok: Tokens,
    dir: Direction,
    a11y: A11y,
) -> Element<'a, M> {
    let parts = crate::i18n::order(dir, parts.iter().cloned());
    let mut r = row![].spacing(6).align_y(Alignment::Center);
    for (i, (name, msg)) in parts.iter().enumerate() {
        if i > 0 {
            r = r.push(meta("/", tok, A11y::new("/", Role::Separator)));
        }
        if let Some(m) = msg.clone() {
            r = r.push(hyperlink(
                name.clone(),
                m,
                tok,
                A11y::new(name.clone(), Role::Link),
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
                dismiss_button(dismiss, tok, A11y::button("dismiss")),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([8, 12])
        .style(move |_| style::callout(tok, kind))
        .into(),
        &a11y,
    )
}

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
                    Some(dismiss),
                    tok,
                    Variant::Primary,
                    A11y::button("Got it"),
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

pub fn themed_scroll<'a, M: 'a>(
    child: Element<'a, M>,
    tok: Tokens,
    a11y: A11y,
    stick: bool,
) -> Element<'a, M> {
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
    a11y::attach(s.into(), &a11y)
}

fn two_line_row<'a, M: 'a>(
    title: &str,
    meta_s: Option<&str>,
    selected: bool,
    row_h: f32,
    tok: Tokens,
) -> Element<'a, M> {
    let mut col = column![text(title.to_string())
        .size(typo::BODY)
        .color(tok.text)
        .font(typo::UI)]
    .spacing(2);
    if let Some(m) = meta_s.filter(|s| !s.is_empty()) {
        col = col.push(text(m.to_string()).size(typo::META).color(tok.muted));
    }
    container(col)
        .width(Length::Fill)
        .height(row_h)
        .padding(8)
        .style(move |_| style::list_row(tok, selected))
        .into()
}

#[allow(clippy::too_many_arguments)]
pub fn list_view<'a, M, L>(
    model: &'a L,
    selection: &Selection,
    on_select: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
    window: VisibleWindow,
    row_h: f32,
    overscan: usize,
    on_scroll: impl Fn(VisibleWindow) -> M + Copy + 'a,
    a11y: A11y,
) -> Element<'a, M>
where
    M: Clone + 'a,
    L: ListModel + ?Sized,
{
    let cover = selection.primary();
    let (top, win, bot) = virtual_pads(
        model.len(),
        row_h,
        window.scroll,
        window.viewport,
        overscan,
        cover,
    );
    let mut col = column![].spacing(0);
    if model.is_empty() {
        col = col.push(meta("Empty", tok, A11y::new("Empty", Role::Status)));
    } else {
        col = col.push(Space::new().height(Length::Fixed(top)));
        for i in win.range() {
            let selected = selection.contains(i);
            let title = model.title(i);
            let meta_s = model.meta(i);
            let name = title.to_string();
            col = col.push(a11y::attach(
                mouse_area(two_line_row(title, meta_s, selected, row_h, tok))
                    .on_press(on_select(i))
                    .into(),
                &A11y::new(name, Role::ListItem).with_checked(selected),
            ));
        }
        col = col.push(Space::new().height(Length::Fixed(bot)));
    }
    let len = model.len();
    let prev = window;
    a11y::attach(
        scrollable(col)
            .width(crate::layout::FILL)
            .height(crate::layout::FILL)
            .direction(ScrollDir::Vertical(
                Scrollbar::new()
                    .width(SCROLL_RAIL_WIDTH)
                    .scroller_width(SCROLL_RAIL_WIDTH),
            ))
            .style(style::scroll_style(tok))
            .on_scroll(move |vp| {
                on_scroll(window_after_scroll(
                    prev,
                    vp.absolute_offset().y,
                    vp.bounds().height,
                    row_h,
                    len,
                    overscan,
                    cover,
                ))
            })
            .into(),
        &a11y,
    )
}

pub fn item_grid<'a, M: Clone + 'a>(
    labels: &[String],
    on_select: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let cells: Vec<Element<'a, M>> = labels
        .iter()
        .enumerate()
        .map(|(i, s)| {
            themed_button(
                s.clone(),
                Some(on_select(i)),
                tok,
                Variant::Quiet,
                A11y::new(s.clone(), Role::ListItem),
            )
        })
        .collect();
    a11y::attach(crate::layout::grid(cells, 3, 8), &a11y)
}

#[allow(clippy::too_many_arguments)]
pub fn data_table<'a, M: Clone + 'a>(
    model: &'a TableModel,
    selection: &Selection,
    window: VisibleWindow,
    row_h: f32,
    overscan: usize,
    on_select: impl Fn(usize) -> M + Copy + 'a,
    on_sort: impl Fn(usize) -> M + Copy + 'a,
    on_scroll: impl Fn(VisibleWindow) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let cover = selection.primary();
    let n = model.rows.len();
    let (top, win, bot) = virtual_pads(n, row_h, window.scroll, window.viewport, overscan, cover);
    let mut header = row![].spacing(8);
    for (i, h) in model.headers.iter().enumerate() {
        header = header.push(themed_button(
            h.clone(),
            Some(on_sort(i)),
            tok,
            Variant::Ghost,
            A11y::button(h.clone()),
        ));
    }
    let mut body = column![].spacing(0);
    body = body.push(Space::new().height(Length::Fixed(top)));
    for i in win.range() {
        let selected = selection.contains(i);
        let title = model.cell(i, 0);
        let meta_s = if model.headers.len() > 1 {
            Some(model.cell(i, 1))
        } else {
            None
        };
        let name = title.to_string();
        body = body.push(a11y::attach(
            mouse_area(two_line_row(title, meta_s, selected, row_h, tok))
                .on_press(on_select(i))
                .into(),
            &A11y::new(name, Role::ListItem).with_checked(selected),
        ));
    }
    body = body.push(Space::new().height(Length::Fixed(bot)));
    let prev = window;
    a11y::attach(
        column![
            header,
            scrollable(body)
                .width(crate::layout::FILL)
                .height(crate::layout::FILL)
                .direction(ScrollDir::Vertical(
                    Scrollbar::new()
                        .width(SCROLL_RAIL_WIDTH)
                        .scroller_width(SCROLL_RAIL_WIDTH),
                ))
                .style(style::scroll_style(tok))
                .on_scroll(move |vp| {
                    on_scroll(window_after_scroll(
                        prev,
                        vp.absolute_offset().y,
                        vp.bounds().height,
                        row_h,
                        n,
                        overscan,
                        cover,
                    ))
                }),
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
pub fn tree_view<'a, M: Clone + 'a>(
    root: &TreeNode,
    selected: Option<u64>,
    on_toggle: impl Fn(u64) -> M + Copy + 'a,
    on_select: impl Fn(u64) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let mut col = column![].spacing(2);
    for (depth, id, label_s, expanded, has_children) in root.flatten() {
        let is_sel = selected == Some(id);
        let mut line = row![].spacing(4).align_y(Alignment::Center);
        line = line.push(Space::new().width(Length::Fixed(depth as f32 * 16.0)));
        if has_children {
            let mark = if expanded { "▾" } else { "▸" };
            line = line.push(themed_button(
                mark,
                Some(on_toggle(id)),
                tok,
                Variant::Ghost,
                A11y::button(format!("toggle {label_s}")).with_checked(expanded),
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
        line = line.push(a11y::attach(
            mouse_area(title).on_press(on_select(id)).into(),
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
        ),
        &a11y,
    )
}

pub fn tab_bar<'a, M: Clone + 'a>(
    tabs: &Tabs,
    on_select: impl Fn(usize) -> M + Copy + 'a,
    on_close: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let mut r = row![].spacing(4).align_y(Alignment::Center);
    for (i, title) in tabs.titles.iter().enumerate() {
        let mut cell = row![button(text(title.clone()).size(typo::META))
            .on_press(on_select(i))
            .padding([6, 10])
            .style(style::tab_style(tok, i == tabs.active))]
        .spacing(2)
        .align_y(Alignment::Center);
        if tabs.closable {
            cell = cell.push(dismiss_button(
                on_close(i),
                tok,
                A11y::button(format!("close {title}")),
            ));
        }
        r = r.push(a11y::attach(
            container(cell).padding([2, 2]).into(),
            &A11y::new(title.clone(), Role::Tab).with_checked(i == tabs.active),
        ));
    }
    a11y::attach(r.into(), &a11y)
}

pub fn accordion_view<'a, M: Clone + 'a>(
    titles: &[String],
    bodies: Vec<Element<'a, M>>,
    state: &Accordion,
    on_toggle: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let mut col = column![].spacing(4);
    for (i, (title, body)) in titles.iter().zip(bodies).enumerate() {
        let open = state.open == Some(i);
        col = col.push(themed_button(
            title.clone(),
            Some(on_toggle(i)),
            tok,
            Variant::Quiet,
            A11y::button(title.clone()).with_checked(open),
        ));
        if open {
            col = col.push(body);
        }
    }
    a11y::attach(col.into(), &a11y)
}

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
                (page > 0).then(|| on_page(page - 1)),
                tok,
                Variant::Quiet,
                A11y::button("Prev").with_disabled(page == 0),
            ),
            meta(status.clone(), tok, A11y::new(status, Role::Status)),
            themed_button(
                "Next",
                (page + 1 < pages).then(|| on_page(page + 1)),
                tok,
                Variant::Quiet,
                A11y::button("Next").with_disabled(page + 1 >= pages),
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
    use crate::collection::{Selection as Sel, VecList};
    use crate::density::Density;
    use crate::theme::named;

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
        }
        .clamp();
        assert_eq!(t.hour, 23);
        assert_eq!(t.minute, 59);
        assert_eq!(step_number(5.0, 1.0, 0.0, 10.0, 1), 6.0);
        assert_eq!(step_number(0.0, 1.0, 0.0, 10.0, -1), 0.0);
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
        let _: Element<'_, ()> = meta("m", tok, role("m", Role::Status));
        let _: Element<'_, ()> = code_block("fn", tok, role("fn", Role::Group));
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
        let _: Element<'_, ()> = themed_slider(
            0.0..=1.0,
            0.5,
            |_| (),
            tok,
            role("s", Role::Slider).with_value("0.5"),
        );
        let _: Element<'_, ()> = progress(0.2, tok, role("p", Role::Progress).with_value("0.2"));
        let _: Element<'_, ()> =
            progress_ring(0.4, tok, role("pr", Role::Progress).with_value("0.4"));
        let _: Element<'_, ()> = image(
            iced::widget::image::Handle::from_bytes(TEST_PNG),
            48.0,
            48.0,
            role("px", Role::Image),
        );
        let _: Element<'_, ()> = spinner(tok, 0.25, role("spin", Role::Progress));
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
        let _: Element<'_, ()> =
            themed_text_input("p", "v", |_| (), Some(()), tok, role("v", Role::TextBox));
        let _: Element<'_, ()> =
            themed_text_input("p", "", |_| (), None, tok, role("Name", Role::TextBox));
        let _: Element<'_, ()> = themed_text_input(
            "p",
            "",
            |_| (),
            Some(()),
            tok,
            role("Name", Role::TextBox).with_disabled(true),
        );
        let _: Element<'_, ()> = password_input(
            "p",
            "v",
            |_| (),
            tok,
            role("pw", Role::TextBox).with_disabled(true),
        );
        let _: Element<'_, ()> = password_input("p", "v", |_| (), tok, role("pw2", Role::TextBox));
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
        let _: Element<'_, ()> = search_input("q", |_| (), tok, role("q", Role::TextBox));
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
            TimeValue { hour: 8, minute: 5 },
            (),
            (),
            tok,
            role("time", Role::SpinButton),
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
        let _: Element<'_, ()> = badge("b", tok, Variant::Quiet, role("b", Role::Status));
        let _: Element<'_, ()> = badge("new", tok, Variant::Primary, role("new", Role::Status));
        let _: Element<'_, ()> = badge("!", tok, Variant::Danger, role("bang", Role::Status));
        let _: Element<'_, ()> = badge("g", tok, Variant::Ghost, role("g", Role::Status));
        let _: Element<'_, ()> = badge("chip", tok, Variant::Chip, role("chip", Role::Status));
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
            role("list", Role::List),
        );
        let scrolled = VisibleWindow {
            start: 0,
            end: 0,
            scroll: 24.0,
            viewport: 100.0,
        };
        let _: Element<'_, ()> = list_view(
            &list,
            &Sel::Single(0),
            |_| (),
            tok,
            scrolled,
            24.0,
            4,
            |_| (),
            role("list", Role::List),
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
        let _: Element<'_, ()> = data_table(
            &table,
            &Sel::Single(0),
            VisibleWindow::new(100.0),
            24.0,
            crate::collection::OVERSCAN,
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
        let _: Element<'_, ()> = data_table(
            &big,
            &Sel::None,
            VisibleWindow {
                start: 0,
                end: 0,
                scroll: 200.0,
                viewport: 80.0,
            },
            20.0,
            2,
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
            .contains("button|Save|idle|1|"));
        let _: Element<'_, ()> = themed_scroll(
            label("log", tok, role("log", Role::Status)),
            tok,
            role("scroll", Role::Group),
            true,
        );
        let _: Element<'_, ()> = themed_scroll(
            label("body", tok, role("body", Role::Status)),
            tok,
            role("scroll", Role::Group),
            false,
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
        assert!(product.contains("virtual_pads("));
        assert!(product.contains("window_after_scroll("));
        assert!(!product.contains("list_body_and_rail"));
        let pass_src = src
            .split("pub fn password_input")
            .nth(1)
            .unwrap()
            .split("pub fn textarea")
            .next()
            .unwrap();
        assert!(!pass_src.contains("apply_name(value)"));
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
        let mut table_el: Element<'_, VisibleWindow> = data_table(
            &table,
            &Sel::Single(0),
            window,
            20.0,
            4,
            |_| window,
            |_| window,
            |w| w,
            tok,
            A11y::new("table", Role::Table),
        );
        let _ = drive(&mut table_el);
        assert_eq!(
            crate::collection::window_after_scroll(window, 4.0, 200.0, 20.0, 80, 4, Some(0)).start,
            0
        );
    }
}
