//! Themed iced widget constructors for `view`. Every export returns an
//! [`iced::Element`] and is keyboard-complete via iced.

use iced::widget::canvas::Canvas;
use iced::widget::markdown;
use iced::widget::text_editor::Content;
use iced::widget::{
    button, checkbox, column, container, mouse_area, pick_list, progress_bar, radio, row, rule,
    scrollable, slider, svg, text, text_editor, text_input, toggler, tooltip, Space,
};
use iced::{Alignment, Element, Length, Padding};

use crate::host_canvas::ArcRing;

use crate::a11y::{self, A11y, Role};
use crate::collection::{page_range, Accordion, ListModel, Selection, TableModel, Tabs, TreeNode};
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
    let label = a11y.apply_name(title);
    let mut b = button(text(label).size(typo::BODY))
        .padding(pad())
        .style(style::button_style(tok, variant));
    if let Some(m) = a11y.apply_message(msg) {
        b = b.on_press(m);
    }
    a11y::attach(b.into(), &a11y)
}

/// Button that applies [`A11y`] name and disabled to the iced constructor.
pub fn themed_button_a11y<'a, M: Clone + 'a>(
    title: impl Into<String>,
    msg: Option<M>,
    tok: Tokens,
    variant: Variant,
    a11y: A11y,
) -> Element<'a, M> {
    themed_button(title, msg, tok, variant, a11y)
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
    themed_checkbox_a11y(label_s, checked, msg, tok, a11y)
}

/// Checkbox that applies [`A11y`] name, checked, and disabled.
pub fn themed_checkbox_a11y<'a, M: Clone + 'a>(
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

/// 1×1 PNG used when an app has no bitmap yet.
pub const PIXEL_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
    0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB0, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
    0x44, 0xAE, 0x42, 0x60, 0x82,
];

pub fn pixel_image<'a, M: 'a>(a11y: A11y) -> Element<'a, M> {
    a11y::attach(
        iced::widget::image(iced::widget::image::Handle::from_bytes(PIXEL_PNG))
            .width(48.0)
            .height(48.0)
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
    let shown = a11y.apply_name(format!("{value}"));
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
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let value = a11y.apply_name(value);
    let mut i = text_input(placeholder, &value)
        .style(style::search_style(tok))
        .padding(pad());
    if !a11y.disabled {
        i = i.on_input(on_input);
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
    let value = a11y.apply_name(value);
    let mut i = text_input(placeholder, &value)
        .secure(true)
        .style(style::search_style(tok))
        .padding(pad());
    if !a11y.disabled {
        i = i.on_input(on_input);
    }
    a11y::attach(i.into(), &a11y)
}

pub fn textarea<'a, M: Clone + 'a>(
    content: &'a Content,
    on_action: impl Fn(text_editor::Action) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let mut e = text_editor(content)
        .height(120)
        .padding(8)
        .style(editor_style(tok));
    if !a11y.disabled {
        e = e.on_action(on_action);
    }
    a11y::attach(e.into(), &a11y)
}

/// Syntax-highlighted code. `syntax` is an iced highlighter token (`rs`, `py`, …).
/// `theme_name` picks a highlighter face that fits the UI colorway.
pub fn highlighted_code<'a, M: Clone + 'a>(
    content: &'a Content,
    syntax: &str,
    on_action: impl Fn(text_editor::Action) -> M + 'a,
    tok: Tokens,
    theme_name: &str,
    a11y: A11y,
) -> Element<'a, M> {
    let theme = crate::theme::code_highlight(theme_name);
    let mut e = text_editor(content)
        .height(280)
        .padding(8)
        .style(editor_style(tok))
        .highlight(syntax, theme)
        .font(typo::MONO);
    if !a11y.disabled {
        e = e.on_action(on_action);
    }
    a11y::attach(e.into(), &a11y)
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

pub fn suggest_list<'a>(query: &str, corpus: &'a [String], limit: usize) -> Vec<&'a str> {
    let q = query.to_ascii_lowercase();
    corpus
        .iter()
        .filter(|s| s.to_ascii_lowercase().contains(&q) || q.is_empty())
        .take(limit.max(1))
        .map(String::as_str)
        .collect()
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

pub fn chip<'a, M: Clone + 'a>(
    title: impl Into<String>,
    msg: M,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let title = a11y.apply_name(title);
    a11y::attach(
        container(
            row![
                text(title.clone()).size(typo::META).color(tok.text),
                dismiss_button(msg, tok, A11y::button(format!("dismiss {title}"))),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .padding([4, 8])
        .style(move |_| style::fill(crate::theme::chip_fill(tok), tok.text))
        .into(),
        &a11y,
    )
}

pub fn badge<'a, M: 'a>(title: impl Into<String>, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    let title = a11y.apply_name(title);
    a11y::attach(
        container(text(title).size(typo::META).color(tok.muted))
            .padding([4, 8])
            .style(move |_| style::fill(crate::theme::chip_fill(tok), tok.muted))
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

pub fn themed_scroll<'a, M: 'a>(child: Element<'a, M>, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    a11y::attach(
        scrollable(child)
            .height(Length::Fill)
            .style(style::scroll_style(tok))
            .into(),
        &a11y,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn list_view<'a, M, L>(
    model: &L,
    selection: &Selection,
    on_select: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
    scroll: f32,
    row_h: f32,
    viewport: f32,
    on_scroll: impl Fn(f32) -> M + 'a,
    a11y: A11y,
) -> Element<'a, M>
where
    M: Clone + 'a,
    L: ListModel,
{
    let (top, vis, bot) = crate::collection::virtual_pads(model.len(), row_h, scroll, viewport);
    let mut col = column![].spacing(0);
    if model.is_empty() {
        col = col.push(meta("Empty", tok, A11y::new("Empty", Role::Status)));
    } else {
        col = col.push(Space::new().height(top));
        for i in vis {
            let selected = selection.contains(i);
            let name = model.label(i);
            col = col.push(a11y::attach(
                mouse_area(
                    container(label(
                        name.clone(),
                        tok,
                        A11y::new(name.clone(), Role::ListItem).with_checked(selected),
                    ))
                    .width(Length::Fill)
                    .height(row_h)
                    .padding(10)
                    .style(move |_| style::list_row(tok, selected)),
                )
                .on_press(on_select(i))
                .into(),
                &A11y::new(name, Role::ListItem).with_checked(selected),
            ));
        }
        col = col.push(Space::new().height(bot));
    }
    a11y::attach(
        scrollable(col)
            .height(viewport)
            .on_scroll(move |vp| on_scroll(vp.absolute_offset().y))
            .style(style::scroll_style(tok))
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
    model: &TableModel,
    selection: &Selection,
    scroll: f32,
    row_h: f32,
    viewport: f32,
    on_select: impl Fn(usize) -> M + Copy + 'a,
    on_sort: impl Fn(usize) -> M + Copy + 'a,
    on_scroll: impl Fn(f32) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let (top, vis, bot) =
        crate::collection::virtual_pads(model.rows.len(), row_h, scroll, viewport);
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
    body = body.push(Space::new().height(top));
    for i in vis {
        let selected = selection.contains(i);
        let line = model.rows.get(i).cloned().unwrap_or_default().join("  ");
        body = body.push(a11y::attach(
            mouse_area(
                container(label(
                    line.clone(),
                    tok,
                    A11y::new(line.clone(), Role::ListItem).with_checked(selected),
                ))
                .width(Length::Fill)
                .height(row_h)
                .padding(8)
                .style(move |_| style::list_row(tok, selected)),
            )
            .on_press(on_select(i))
            .into(),
            &A11y::new(line, Role::ListItem).with_checked(selected),
        ));
    }
    body = body.push(Space::new().height(bot));
    a11y::attach(
        column![
            header,
            scrollable(body)
                .height(viewport)
                .on_scroll(move |vp| on_scroll(vp.absolute_offset().y))
                .style(style::scroll_style(tok))
        ]
        .spacing(4)
        .into(),
        &a11y,
    )
}

pub fn tree_view<'a, M: Clone + 'a>(
    root: &TreeNode,
    on_toggle: impl Fn(u64) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let mut col = column![].spacing(2);
    for (depth, id, label_s, expanded, has_children) in root.flatten() {
        let prefix = if has_children {
            if expanded {
                "▾ "
            } else {
                "▸ "
            }
        } else {
            "  "
        };
        let indent = "  ".repeat(depth as usize);
        let title = format!("{indent}{prefix}{label_s}");
        col = col.push(themed_button(
            title.clone(),
            Some(on_toggle(id)),
            tok,
            Variant::Ghost,
            A11y::new(title, Role::Tree).with_checked(expanded),
        ));
    }
    a11y::attach(
        themed_scroll(col.into(), tok, A11y::new("tree-scroll", Role::Group)),
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

pub fn a11y_button(name: &str, disabled: bool) -> A11y {
    A11y::button(name).with_disabled(disabled)
}

pub fn a11y_checkbox(name: &str, checked: bool) -> A11y {
    A11y::new(name, Role::Checkbox).with_checked(checked)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::{Selection as Sel, VecList};
    use crate::theme::named;

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
        let corpus = vec!["Apple".into(), "Apricot".into(), "Banana".into()];
        assert_eq!(suggest_list("ap", &corpus, 10).len(), 2);
        assert_eq!(suggest_list("", &corpus, 1).len(), 1);
        let _ = a11y_button("x", true);
        let _ = a11y_checkbox("y", false);
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
        let _: Element<'_, ()> = pixel_image(role("px", Role::Image));
        let _: Element<'_, ()> = spinner(tok, 0.25, role("spin", Role::Progress));
        assert!(ring_angles(1.0).1 > ring_angles(0.2).1);
        assert!(
            (spinner_angles(0.0).1 - spinner_angles(0.0).0 - std::f32::consts::FRAC_PI_2).abs()
                < 0.01
        );
        assert!(ring_should_stroke(0.0, 1.0));
        assert!(!ring_should_stroke(0.0, 0.0));
        let a11y = A11y::button("Nope").with_disabled(true);
        let _: Element<'_, ()> = themed_button_a11y("Nope", Some(()), tok, Variant::Primary, a11y);
        let unnamed = A11y::button("");
        let _: Element<'_, ()> =
            themed_button_a11y("Shown", Some(()), tok, Variant::Primary, unnamed);
        let unnamed_c = A11y::new("", Role::Checkbox);
        let _: Element<'_, ()> = themed_checkbox_a11y("box", true, |_| (), tok, unnamed_c);
        let ca = A11y::new("off", Role::Checkbox)
            .with_checked(true)
            .with_disabled(true);
        let _: Element<'_, ()> = themed_checkbox_a11y("off", false, |_| (), tok, ca);
        let _: Element<'_, ()> = number_input(
            3.0,
            |_| (),
            tok,
            role("n", Role::SpinButton).with_value("3"),
        );
        let _: Element<'_, ()> = themed_text_input("p", "v", |_| (), tok, role("v", Role::TextBox));
        let _: Element<'_, ()> = password_input(
            "p",
            "v",
            |_| (),
            tok,
            role("pw", Role::TextBox).with_disabled(true),
        );
        let _: Element<'_, ()> = password_input("p", "v", |_| (), tok, role("pw2", Role::TextBox));
        let content = Content::new();
        let _: Element<'_, ()> = textarea(&content, |_| (), tok, role("ta", Role::TextBox));
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
        let full: Vec<_> = markdown::parse(crate::samples::MARKDOWN).collect();
        let _: Element<'_, ()> = markdown_view(&full, tok, |_| (), role("md", Role::Group));
        let rust = crate::samples::CodeLang::named("Rust").unwrap();
        let code = Content::with_text(rust.source);
        let _: Element<'_, ()> = highlighted_code(
            &code,
            rust.syntax,
            |_| (),
            tok,
            "dark",
            role("code", Role::Group),
        );
        let light = named("light").tokens;
        let _: Element<'_, ()> = highlighted_code(
            &code,
            "py",
            |_| (),
            light,
            "solarized-light",
            role("code", Role::Group),
        );
        let mocha = named("catppuccin-mocha").tokens;
        let _: Element<'_, ()> = highlighted_code(
            &code,
            rust.syntax,
            |_| (),
            mocha,
            "catppuccin-mocha",
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
        let _: Element<'_, ()> = chip("c", (), tok, btn("c"));
        let _: Element<'_, ()> = badge("b", tok, role("b", Role::Status));
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
            items: vec!["a".into()],
        };
        let empty = VecList::default();
        let _: Element<'_, ()> = list_view(
            &list,
            &Sel::Single(0),
            |_| (),
            tok,
            0.0,
            24.0,
            100.0,
            |_| (),
            role("list", Role::List),
        );
        let _: Element<'_, ()> = list_view(
            &empty,
            &Sel::None,
            |_| (),
            tok,
            0.0,
            24.0,
            100.0,
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
            headers: vec!["A".into()],
            rows: vec![vec!["1".into()], vec!["2".into()]],
            sort_col: None,
            sort_asc: true,
        };
        let _: Element<'_, ()> = data_table(
            &table,
            &Sel::Single(0),
            0.0,
            24.0,
            100.0,
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
            200.0,
            20.0,
            80.0,
            |_| (),
            |_| (),
            |_| (),
            tok,
            role("table", Role::Table),
        );
        let tree = TreeNode::branch(1, "r", vec![TreeNode::leaf(2, "c")]);
        let _: Element<'_, ()> = tree_view(&tree, |_| (), tok, role("tree", Role::Tree));
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
        let collapsed = TreeNode::branch(1, "r", vec![TreeNode::leaf(2, "c")]);
        let mut collapsed = collapsed;
        assert!(crate::collection::tree_toggle(&mut collapsed, 1));
        let _: Element<'_, ()> = tree_view(&collapsed, |_| (), tok, role("tree", Role::Tree));
        assert!(btn("Save")
            .with_value("idle")
            .with_disabled(true)
            .node_id()
            .contains("button|Save|idle|1|"));
    }
}
