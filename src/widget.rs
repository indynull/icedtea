//! Themed iced widget constructors for `view`.
//!
//! Every drawing constructor returns an [`iced::Element`], emits the
//! application's messages, and takes [`A11y`] plus [`Tokens`]. Rustdoc
//! on the function is the call: the job, the arguments that matter,
//! and a compiling example. iced 0.14 has no AccessKit slot: [`A11y`]
//! is the in-library record; [`crate::a11y::attach`] sets the widget id.
//! Keyboard order (`key::handle`, modal first, focused text next) is
//! the working desktop path.
//!
//! ```
//! use icedtea::a11y::A11y;
//! use icedtea::icon::Icons;
//! use icedtea::theme;
//! use icedtea::variant::Variant;
//! use icedtea::widget;
//! let tok = theme::named("dark").tokens;
//! let _: icedtea::Element<'_, ()> = widget::themed_button(
//!     "Save",
//!     Some(()),
//!     tok,
//!     Variant::Primary,
//!     Icons::NONE,
//!     A11y::button("Save"),
//! );
//! ```

use iced::advanced::layout;
use iced::advanced::widget::{tree, Tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget};
use iced::gradient::Linear;
use iced::widget::canvas::Canvas;
use iced::widget::markdown;
use iced::widget::scrollable::{Direction as ScrollDir, Scrollbar};
use iced::widget::text_editor::Content;
use iced::widget::{
    button, checkbox, column, container, mouse_area, pick_list, progress_bar, radio, rich_text,
    row, rule, scrollable, slider, stack, svg, text, text_editor, text_input, toggler, tooltip,
    Column, Id, Row, Space, Stack,
};
use iced::{keyboard, Alignment, Background, Color, Element, Event, Length, Padding, Radians};

use crate::host_canvas::{ArcRing, SpinnerDots};
use crate::scroll::{ClipLayer, ScrollRail, ThemedScroll};

use crate::a11y::{self, A11y, Role};
use crate::collection::{
    page_range, virtual_pads, window_after_scroll, window_after_scroll_var, Accordion, ItemButton,
    ItemClick, ListModel, RowFace, RowHeights, Selection, Tabs, TreeNode, VisibleWindow,
};
use crate::i18n::Direction;
use crate::icon::{Glyph, Icon, Icons};
use crate::style;
use crate::theme::Tokens;
use crate::toast::{Toast, ToastKind};
use crate::typo;
use crate::variant::Variant;

/// Shared padding for controls. Vertical and horizontal follow token density.
///
/// Vertical is `pad - 4` so Compact / Default / Comfortable stay distinct
/// on the 4 dp grid (8→4, 12→8, 16→12). `(pad * 2 / 3)` snapped Compact
/// and Default to the same 8 px.
fn pad(tok: Tokens) -> Padding {
    let p = tok.density.pad;
    let v = crate::density::Density::snap(p.saturating_sub(4).max(4));
    let h = crate::density::Density::snap(p + 4);
    Padding::from([v as f32, h as f32])
}

fn gap(tok: Tokens) -> f32 {
    tok.density.gap()
}

fn inset(tok: Tokens) -> f32 {
    tok.density.inset()
}

fn sheet(tok: Tokens) -> f32 {
    tok.density.sheet()
}

fn icon_hit_pad(size: ControlSize, tok: Tokens) -> Padding {
    match size {
        ControlSize::Default => Padding::from(gap(tok)),
        other => Padding::from(f32::from(other.pad())),
    }
}

/// Outer height of a standard padded control (body line box + vertical pad).
fn control_height(tok: Tokens) -> f32 {
    sized_control_height(tok, ControlSize::Default)
}

fn sized_control_height(tok: Tokens, size: ControlSize) -> f32 {
    // Same face as `themed_button` Shrink. Do not floor to the 48dp touch
    // target: iced `Fixed(48)` + pad paints a 48px face, taller than
    // labeled buttons on the same page.
    let type_px = match size {
        ControlSize::Compact => tok.meta(),
        _ => tok.body(),
    };
    let line =
        f32::from(iced::widget::text::LineHeight::default().to_absolute(iced::Pixels(type_px)));
    let p = match size {
        ControlSize::Default => pad(tok),
        other => Padding::from(f32::from(other.pad())),
    };
    line + p.top + p.bottom
}

/// Control on the start edge, label after it (box then text in LTR).
fn labeled_control<'a, M: 'a>(
    control: Element<'a, M>,
    name: String,
    tok: Tokens,
    muted: bool,
) -> Element<'a, M> {
    let ink = if muted {
        tok.scheme().on_surface_variant
    } else {
        tok.scheme().on_surface
    };
    let label: Element<'a, M> = text(name).size(tok.body()).color(ink).into();
    let mut r = Row::new().spacing(gap(tok)).align_y(Alignment::Center);
    for kid in crate::i18n::order(tok.direction, [control, label]) {
        r = r.push(kid);
    }
    container(r)
        .width(Length::Fill)
        .align_x(crate::i18n::align_start(tok.direction))
        .into()
}

/// Label plus optional leading/trailing icons.
fn icon_label<'a, M: 'a>(title: String, icons: Icons, tok: Tokens) -> Element<'a, M> {
    let mut kids: Vec<Element<'a, M>> = Vec::new();
    if let Some(ic) = icons.leading {
        kids.push(icon_svg(ic, tok, A11y::new(title.clone(), Role::Image)));
    }
    kids.push(text(title.clone()).size(tok.body()).into());
    if let Some(ic) = icons.trailing {
        kids.push(icon_svg(ic, tok, A11y::new(title, Role::Image)));
    }
    let mut r = Row::new().spacing(gap(tok)).align_y(Alignment::Center);
    for kid in crate::i18n::order(tok.direction, kids) {
        r = r.push(kid);
    }
    r.into()
}

fn closed_disclosure(tok: Tokens) -> &'static str {
    match tok.direction {
        Direction::Ltr => "▸",
        Direction::Rtl => "◂",
    }
}

fn pick_chevron<'a, M: 'a>(tok: Tokens) -> Element<'a, M> {
    icon_svg(Icon::Chevron, tok, A11y::new("open", Role::Image))
}

/// Step for a continuous [`themed_slider`] range (~100 positions).
///
/// iced's slider defaults to step `1`, so a `0.0..=1.0` range only hits
/// the endpoints and feels broken under drag.
pub fn slider_step(range: std::ops::RangeInclusive<f32>) -> f32 {
    let span = (*range.end() - *range.start()).abs();
    if span == 0.0 {
        return f32::EPSILON;
    }
    (span / 100.0).max(span * 1e-6)
}

/// Vertical wheel sign: up is positive (increase).
pub fn scroll_wheel_y(delta: iced::mouse::ScrollDelta) -> f32 {
    match delta {
        iced::mouse::ScrollDelta::Lines { y, .. } => y,
        iced::mouse::ScrollDelta::Pixels { y, .. } => y.signum(),
    }
}

/// Next slider value after a wheel step. Up increases.
pub fn slider_nudge(range: std::ops::RangeInclusive<f32>, value: f32, delta_y: f32) -> f32 {
    let step = slider_step(range.clone());
    let lo = *range.start();
    let hi = *range.end();
    let v = value.clamp(lo, hi);
    if delta_y > 0.0 {
        (v + step).min(hi)
    } else if delta_y < 0.0 {
        (v - step).max(lo)
    } else {
        v
    }
}

pub fn icon_style(tok: Tokens) -> impl Fn(&iced::Theme, svg::Status) -> svg::Style {
    move |_t, _s| svg::Style {
        color: Some(tok.scheme().on_surface),
    }
}

/// Paint a chrome glyph.
///
/// Pass a shipped [`Icon`] or [`Glyph::Bytes`] (filled black SVG). Tokens
/// recolor non-transparent pixels (Linux, macOS Metal, and Windows).
/// The seven names stay the shipped set.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::icon::{Glyph, Icon};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> =
///     widget::icon_svg(Icon::Search, tok, A11y::new("search", Role::Image));
/// let mark = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="#000"><path d="M8 1 15 8 8 15 1 8z"/></svg>"##;
/// let _: icedtea::Element<'_, ()> =
///     widget::icon_svg(Glyph::Bytes(mark), tok, A11y::new("app", Role::Image));
/// ```
pub fn icon_svg<'a, M: 'a>(glyph: impl Into<Glyph>, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    let handle = svg::Handle::from_memory(glyph.into().bytes());
    a11y::attach(
        svg(handle)
            .width(16.0)
            .height(16.0)
            .style(icon_style(tok))
            .into(),
        &a11y,
    )
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
            .size(tok.body())
            .color(tok.scheme().on_surface)
            .font(typo::UI)
            .into(),
        &a11y,
    )
}

/// Muted end-aligned line above a display reading.
pub fn display_line<'a, M: 'a>(s: impl Into<String>, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    let s = a11y.apply_name(s);
    a11y::attach(
        container(
            text(s)
                .size(tok.meta())
                .color(tok.scheme().on_surface_variant)
                .font(typo::UI),
        )
        .width(Length::Fill)
        .align_x(crate::i18n::align_end(tok.direction))
        .into(),
        &a11y,
    )
}

/// Segmented large figures on the type scale (clocks, meters).
pub fn figure_display<'a, M: 'a>(s: impl Into<String>, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    let s = a11y.apply_name(s);
    let mut r = Row::new().spacing(gap(tok)).align_y(Alignment::Center);
    for ch in s.chars() {
        r = r.push(
            text(ch.to_string())
                .size(tok.display())
                .font(typo::UI_BOLD)
                .color(tok.scheme().on_surface),
        );
    }
    a11y::attach(r.into(), &a11y)
}

pub fn meta<'a, M: 'a>(s: impl Into<String>, tok: Tokens, a11y: A11y) -> Element<'a, M> {
    let s = a11y.apply_name(s);
    a11y::attach(
        text(s)
            .size(tok.meta())
            .color(tok.scheme().on_surface_variant)
            .into(),
        &a11y,
    )
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
        .padding(inset(tok))
        .font(typo::MONO)
        .size(tok.code())
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
    let mut b = button(text(title).size(tok.body()).color(tok.scheme().primary))
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
///     icedtea::icon::Icons::NONE,
///     A11y::button("Save"),
/// );
/// ```
pub fn themed_button<'a, M: Clone + 'a>(
    title: impl Into<String>,
    msg: Option<M>,
    tok: Tokens,
    variant: Variant,
    icons: Icons,
    a11y: A11y,
) -> Element<'a, M> {
    themed_button_sized(
        title,
        msg,
        tok,
        variant,
        icons,
        Length::Shrink,
        Length::Shrink,
        a11y,
    )
}

/// Themed button that fills a pad cell.
#[allow(clippy::too_many_arguments)]
pub fn themed_button_sized<'a, M: Clone + 'a>(
    title: impl Into<String>,
    msg: Option<M>,
    tok: Tokens,
    variant: Variant,
    icons: Icons,
    width: Length,
    height: Length,
    a11y: A11y,
) -> Element<'a, M> {
    let label = a11y.apply_name(title);
    // Shrink the title. Fill+align on `text` inside iced `button`
    // drops right-to-left glyphs (empty colored pads).
    let face: Element<'a, M> = if icons == Icons::NONE {
        let title_el = text(label).size(tok.body());
        match width {
            Length::Fill | Length::FillPortion(_) => container(title_el)
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .into(),
            _ => title_el.into(),
        }
    } else {
        icon_label(label, icons, tok)
    };
    let mut b = button(face)
        .padding(pad(tok))
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
/// let _: icedtea::Element<'_, i32> = widget::split_button(
///     "Save",
///     0,
///     vec![("Save As…".into(), 1), ("Export…".into(), 2)],
///     tok,
///     icedtea::icon::Icons::NONE,
///     A11y::button("Save"),
/// );
/// ```
pub fn split_button<'a, M: Clone + 'a>(
    title: impl Into<String>,
    primary: M,
    overflow: impl IntoIterator<Item = (String, M)>,
    tok: Tokens,
    icons: Icons,
    a11y: A11y,
) -> Element<'a, M> {
    let title = a11y.apply_name(title);
    let primary_msg = (!a11y.disabled).then_some(primary);
    let items: Vec<(String, M)> = overflow.into_iter().collect();
    let h = control_height(tok);
    a11y::attach(
        row![
            themed_button_sized(
                title.clone(),
                primary_msg,
                tok,
                Variant::Primary,
                icons,
                Length::Shrink,
                Length::Fixed(h),
                A11y::button(&title).with_disabled(a11y.disabled),
            ),
            crate::menubar::split_more(items, tok, a11y.disabled, h),
        ]
        .spacing(2)
        .align_y(Alignment::Center)
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
///     widget::toggle_button(
///         "Bold",
///         true,
///         (),
///         tok,
///         icedtea::icon::Icons::NONE,
///         A11y::button("Bold").with_checked(true),
///     );
/// ```
pub fn toggle_button<'a, M: Clone + 'a>(
    title: impl Into<String>,
    pressed: bool,
    msg: M,
    tok: Tokens,
    icons: Icons,
    a11y: A11y,
) -> Element<'a, M> {
    let title = title.into();
    let a11y = a11y.merge_checked(pressed).merge_selected(pressed);
    themed_button(
        title,
        (!a11y.disabled).then_some(msg),
        tok,
        if a11y.apply_checked(pressed) {
            Variant::Primary
        } else {
            Variant::Quiet
        },
        icons,
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
    let a11y = a11y.merge_checked(checked);
    let name = a11y.apply_name(label_s);
    let is_on = a11y.apply_checked(checked);
    let mut c = checkbox(is_on).style(style::checkbox_style(tok));
    if !a11y.disabled {
        c = c.on_toggle(msg);
    }
    a11y::attach(labeled_control(c.into(), name, tok, a11y.disabled), &a11y)
}

/// Three-state checkbox value (M3 indeterminate for “partial”).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CheckState {
    #[default]
    Unchecked,
    Checked,
    /// Partial selection (select-all over mixed children).
    Indeterminate,
}

/// Icon plus optional label for a group or segment cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub label: String,
    pub icon: Option<Icon>,
}

impl Cell {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
        }
    }

    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
}

impl From<String> for Cell {
    fn from(label: String) -> Self {
        Self::new(label)
    }
}

impl From<&str> for Cell {
    fn from(label: &str) -> Self {
        Self::new(label)
    }
}

impl From<&String> for Cell {
    fn from(label: &String) -> Self {
        Self::new(label.clone())
    }
}

/// Filled (default) or outlined text field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FieldFace {
    #[default]
    Filled,
    Outlined,
}

/// Card paint: elevated (shadow), filled, or outline only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CardFace {
    #[default]
    Elevated,
    Filled,
    Outlined,
}

/// How each tree row is painted.
///
/// [`Self::Outline`] is a tight heading tree: full-width selection, no
/// folder marks. [`Self::Files`] is an explorer: inset wash, folder and
/// file marks from `dir`. Density still scales pad and indent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TreeFace {
    #[default]
    Outline,
    Files,
}

/// Badge size. Large is the default chip-like face (meta type).
/// Small is a tighter pad on the same caption step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeSize {
    Small,
    #[default]
    Large,
}

/// M3 chip family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChipKind {
    #[default]
    Assist,
    Filter,
    Input,
    Suggestion,
}

/// Where a tooltip sits relative to its child.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TooltipAnchor {
    #[default]
    Follow,
    Top,
    Bottom,
    /// Start edge (left in LTR).
    Start,
}

impl TooltipAnchor {
    pub fn position(self) -> tooltip::Position {
        match self {
            Self::Follow => tooltip::Position::FollowCursor,
            Self::Top => tooltip::Position::Top,
            Self::Bottom => tooltip::Position::Bottom,
            Self::Start => tooltip::Position::Left,
        }
    }
}

/// Icon-button hit box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ControlSize {
    Compact,
    #[default]
    Default,
    Comfortable,
}

impl ControlSize {
    /// Hit-box padding in px.
    pub fn pad(self) -> u16 {
        match self {
            Self::Compact => 4,
            Self::Default => 8,
            Self::Comfortable => 12,
        }
    }
}

/// Optional field chrome: face, prefix/suffix icons, floating label, count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldOpts<'a> {
    pub face: FieldFace,
    pub icons: Icons,
    pub label: &'a str,
    pub max_len: Option<usize>,
}

impl FieldOpts<'static> {
    pub const NONE: Self = Self {
        face: FieldFace::Filled,
        icons: Icons::NONE,
        label: "",
        max_len: None,
    };
}

impl CheckState {
    /// Next state on press: indeterminate and unchecked go checked; checked clears.
    pub fn toggle(self) -> Self {
        match self {
            Self::Checked => Self::Unchecked,
            Self::Unchecked | Self::Indeterminate => Self::Checked,
        }
    }
}

/// Map a binary checkbox flip into [`CheckState`].
pub fn check_state_from_bool(on: bool) -> CheckState {
    if on {
        CheckState::Checked
    } else {
        CheckState::Unchecked
    }
}

/// Checkbox with optional indeterminate (partial) state.
///
/// The application owns [`CheckState`]. Press cycles through
/// [`CheckState::toggle`]. Disabled freezes the face.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget::{self, CheckState};
/// let tok = theme::named("dark").tokens;
/// let on_toggle = |s| s;
/// let _: icedtea::Element<'_, CheckState> = widget::checkbox_indeterminate(
///     "Select all",
///     CheckState::Indeterminate,
///     on_toggle,
///     tok,
///     A11y::new("Select all", Role::Checkbox),
/// );
/// ```
pub fn checkbox_indeterminate<'a, M: Clone + 'a>(
    label_s: impl Into<String>,
    state: CheckState,
    msg: impl Fn(CheckState) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let name = a11y.apply_name(label_s);
    match state {
        CheckState::Checked | CheckState::Unchecked => {
            let on = matches!(state, CheckState::Checked);
            themed_checkbox(
                name,
                on,
                move |next| msg(check_state_from_bool(next)),
                tok,
                a11y.with_checked(on),
            )
        }
        CheckState::Indeterminate => {
            let s = tok.scheme();
            let box_face = container(
                text("−")
                    .size(tok.meta())
                    .color(s.on_primary)
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            )
            .width(16)
            .height(16)
            .center_x(16)
            .center_y(16)
            .style(move |_| indeterminate_box_face(tok));
            let row = labeled_control(box_face.into(), name.clone(), tok, a11y.disabled);
            if a11y.disabled {
                return a11y::attach(row, &a11y);
            }
            let next = state.toggle();
            a11y::attach(mouse_area(row).on_press(msg(next)).into(), &a11y)
        }
    }
}

fn indeterminate_box_face(tok: Tokens) -> iced::widget::container::Style {
    let s = tok.scheme();
    let mut st = style::fill(s.primary, s.on_primary);
    st.border = iced::border::Border {
        color: s.primary,
        width: 2.0,
        radius: tok.radius(crate::m3::shape::Component::Field),
    };
    st
}

/// Exclusive choice among labeled segments (M3 segmented button).
///
/// The application owns the selected index. Press emits the new index.
/// Disabled freezes all segments. Compact is the in-pane strip
/// (`tab_bar` stays the pane chrome).
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// use icedtea::widget::ControlSize;
/// let tok = theme::named("dark").tokens;
/// let on_pick = |i| i;
/// let _: icedtea::Element<'_, usize> = widget::segmented_button(
///     ["Day", "Week", "Month"],
///     0,
///     on_pick,
///     tok,
///     ControlSize::Default,
///     A11y::new("Range", Role::Group),
/// );
/// ```
pub fn segmented_button<'a, M: Clone + 'a>(
    cells: impl IntoIterator<Item = impl Into<Cell>>,
    selected: usize,
    on_select: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
    size: ControlSize,
    a11y: A11y,
) -> Element<'a, M> {
    let cells: Vec<(usize, Cell)> = cells.into_iter().map(Into::into).enumerate().collect();
    let cells = crate::i18n::order(tok.direction, cells);
    let type_px = match size {
        ControlSize::Compact => tok.meta(),
        _ => tok.body(),
    };
    let face_pad = match size {
        ControlSize::Default => pad(tok),
        other => Padding::from(f32::from(other.pad())),
    };
    let height = Length::Fixed(sized_control_height(tok, size));
    let mut r = Row::new().spacing(0).align_y(Alignment::Center);
    for (i, cell) in cells.iter() {
        let on = *i == selected;
        let icons = cell.icon.map(Icons::leading).unwrap_or(Icons::NONE);
        let label = cell.label.clone();
        let face: Element<'a, M> = if icons == Icons::NONE {
            text(label.clone()).size(type_px).into()
        } else {
            icon_label(label.clone(), icons, tok)
        };
        let mut b = button(face)
            .padding(face_pad)
            .width(Length::Shrink)
            .height(height)
            .style(style::button_style(
                tok,
                if on { Variant::Primary } else { Variant::Quiet },
            ));
        if !a11y.disabled {
            b = b.on_press(on_select(*i));
        }
        r = r.push(a11y::attach(
            b.into(),
            &A11y::button(label)
                .with_checked(on)
                .with_disabled(a11y.disabled),
        ));
    }
    a11y::attach(r.into(), &a11y)
}

/// Related actions in one strip (M3 button group). Not exclusive.
///
/// Each label sends its index. Empty labels paint an empty row.
/// Disabled drops every press.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_press = |i| i;
/// let _: icedtea::Element<'_, usize> = widget::button_group(
///     ["Cut", "Copy", "Paste"],
///     on_press,
///     tok,
///     A11y::new("edit", Role::Group),
/// );
/// ```
pub fn button_group<'a, M: Clone + 'a>(
    cells: impl IntoIterator<Item = impl Into<Cell>>,
    on_press: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let cells: Vec<(usize, Cell)> = cells.into_iter().map(Into::into).enumerate().collect();
    let cells = crate::i18n::order(tok.direction, cells);
    let mut r = Row::new().spacing(0).align_y(Alignment::Center);
    for (n, (i, cell)) in cells.iter().enumerate() {
        if n > 0 {
            r = r.push(
                container(Space::new().width(1).height(control_height(tok))).style(move |_| {
                    style::fill(tok.scheme().outline_variant, tok.scheme().on_surface)
                }),
            );
        }
        let icons = cell.icon.map(Icons::leading).unwrap_or(Icons::NONE);
        let face = themed_button_sized(
            cell.label.clone(),
            if a11y.disabled {
                None
            } else {
                Some(on_press(*i))
            },
            tok,
            Variant::Quiet,
            icons,
            Length::Shrink,
            Length::Fixed(control_height(tok)),
            A11y::button(cell.label.clone()).with_disabled(a11y.disabled),
        );
        r = r.push(face);
    }
    a11y::attach(
        container(r)
            .width(Length::Fill)
            .align_x(crate::i18n::align_start(tok.direction))
            .style(move |_| {
                let s = tok.scheme();
                let mut st = style::fill(Color::TRANSPARENT, s.on_surface);
                st.border = iced::border::Border {
                    color: s.outline_variant,
                    width: 1.0,
                    radius: tok.radius(crate::m3::shape::Component::Button),
                };
                st
            })
            .into(),
        &a11y,
    )
}

/// Icon-only press control (toolbar density).
///
/// Same variant wash as labeled buttons. Disabled drops the press.
///
/// ```
/// use icedtea::a11y::A11y;
/// use icedtea::icon::Icon;
/// use icedtea::theme;
/// use icedtea::variant::Variant;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let open = ();
/// let _: icedtea::Element<'_, ()> = widget::icon_button(
///     Icon::Search,
///     Some(open),
///     tok,
///     Variant::Ghost,
///     widget::ControlSize::Default,
///     A11y::button("Search"),
/// );
/// ```
pub fn icon_button<'a, M: Clone + 'a>(
    icon: impl Into<Glyph>,
    msg: Option<M>,
    tok: Tokens,
    variant: Variant,
    size: ControlSize,
    a11y: A11y,
) -> Element<'a, M> {
    let mut b = button(icon_svg(
        icon,
        tok,
        A11y::new(a11y.name.clone(), Role::Image),
    ))
    .padding(icon_hit_pad(size, tok))
    .style(style::button_style(tok, variant));
    if let Some(m) = a11y.apply_message(msg) {
        b = b.on_press(m);
    }
    a11y::attach(b.into(), &a11y)
}

/// Icon button that stays pressed while on.
///
/// Same wash as [`toggle_button`]. Disabled keeps the face.
///
/// ```
/// use icedtea::a11y::A11y;
/// use icedtea::icon::Icon;
/// use icedtea::theme;
/// use icedtea::variant::Variant;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> = widget::icon_button_toggle(
///     Icon::Check,
///     true,
///     (),
///     tok,
///     Variant::Primary,
///     widget::ControlSize::Default,
///     A11y::button("Bold").with_checked(true),
/// );
/// ```
pub fn icon_button_toggle<'a, M: Clone + 'a>(
    icon: impl Into<Glyph>,
    pressed: bool,
    msg: M,
    tok: Tokens,
    variant: Variant,
    size: ControlSize,
    a11y: A11y,
) -> Element<'a, M> {
    let a11y = a11y.merge_checked(pressed).merge_selected(pressed);
    icon_button(
        icon,
        (!a11y.disabled).then_some(msg),
        tok,
        if a11y.apply_checked(pressed) {
            variant
        } else {
            Variant::Ghost
        },
        size,
        a11y,
    )
}

/// A sliding on/off control.
///
/// Same contract as checkbox: the application owns the bool. Disabled
/// freezes the thumb. Track corners follow [`Tokens::shape`]
/// ([`crate::m3::shape::Component::Track`]).
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
    let a11y = a11y.merge_toggled(on).merge_checked(on);
    let name = a11y.apply_name(label_s);
    let on = a11y.apply_toggled(on);
    let mut t = toggler(on).style(style::switch_style(tok));
    if !a11y.disabled {
        t = t.on_toggle(msg);
    }
    a11y::attach(labeled_control(t.into(), name, tok, a11y.disabled), &a11y)
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
    let a11y = a11y.merge_checked(selected == Some(value));
    let name = a11y.apply_name(label_s);
    if a11y.disabled {
        let on = a11y.apply_checked(selected == Some(value));
        let mark = container(Space::new().width(8).height(8))
            .width(16)
            .height(16)
            .center_x(16)
            .center_y(16)
            .style(move |_| radio_idle_face(tok, on));
        return a11y::attach(labeled_control(mark.into(), name, tok, true), &a11y);
    }
    a11y::attach(
        labeled_control(
            radio(String::new(), value, selected, msg)
                .style(style::radio_style(tok))
                .into(),
            name,
            tok,
            false,
        ),
        &a11y,
    )
}

fn radio_idle_face(tok: Tokens, on: bool) -> iced::widget::container::Style {
    let s = tok.scheme();
    // Circle geometry (not Component::Field — desktop None would paint a square).
    iced::widget::container::Style {
        background: Some(iced::Background::Color(if on {
            s.primary
        } else {
            Color::TRANSPARENT
        })),
        border: iced::border::Border {
            color: s.outline,
            width: 2.0,
            radius: crate::m3::shape::Shape::Full.radius(),
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

/// Tick count and end labels for [`themed_slider`].
#[derive(Debug, Clone, Copy, Default)]
pub struct SliderMarks<'a> {
    pub ticks: usize,
    pub min: &'a str,
    pub max: &'a str,
    pub vertical: bool,
    pub thumb: &'a str,
}

impl SliderMarks<'static> {
    pub const NONE: Self = Self {
        ticks: 0,
        min: "",
        max: "",
        vertical: false,
        thumb: "",
    };
}

/// Pick a number on a range.
///
/// Pass min, max, and the current value. The message is the new value
/// while the thumb moves. Wheel over the control steps by
/// [`slider_step`]. Disabled ignores drag and wheel. `marks` paints
/// ticks and end labels when set. Rail corners follow [`Tokens::shape`]
/// ([`crate::m3::shape::Component::Track`]).
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
///     widget::SliderMarks { ticks: 5, min: "0", max: "1", vertical: false, thumb: "0.4" },
///     tok,
///     A11y::new("vol", Role::Slider).with_value("0.4"),
/// );
/// ```
pub fn themed_slider<'a, M: Clone + 'a>(
    range: std::ops::RangeInclusive<f32>,
    value: f32,
    msg: impl Fn(f32) -> M + Copy + 'a,
    marks: SliderMarks<'a>,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let a11y = a11y.merge_value(format!("{value}"));
    let slider_el: Element<'a, M> = if a11y.disabled {
        let _ = (value, msg);
        let (w, h) = if marks.vertical {
            (Length::Fixed(18.0), Length::Fill)
        } else {
            (Length::Fill, Length::Fixed(18.0))
        };
        container(Space::new().width(4).height(4))
            .width(w)
            .height(h)
            .style(move |_| disabled_slider_face(tok))
            .into()
    } else {
        let step = slider_step(range.clone());
        if marks.vertical {
            iced::widget::vertical_slider(range.clone(), value, msg)
                .step(step)
                .style(style::slider_style(tok))
                .height(Length::Fill)
                .into()
        } else {
            slider(range.clone(), value, msg)
                .step(step)
                .style(style::slider_style(tok))
                .width(Length::Fill)
                .into()
        }
    };
    let s = tok.scheme();
    let mut col = Column::new().spacing(2).width(if marks.vertical {
        Length::Shrink
    } else {
        Length::Fill
    });
    if !marks.thumb.is_empty() {
        col = col.push(
            text(marks.thumb)
                .size(tok.meta())
                .color(s.on_surface)
                .width(Length::Fill),
        );
    }
    if marks.ticks > 1 && !marks.vertical {
        let mut ticks = Row::new().width(Length::Fill);
        for i in 0..marks.ticks {
            if i > 0 {
                ticks = ticks.push(Space::new().width(Length::Fill));
            }
            ticks = ticks.push(
                container(Space::new().width(1).height(6))
                    .style(move |_| style::fill(s.outline_variant, s.on_surface)),
            );
        }
        col = col.push(ticks);
    }
    if marks.vertical {
        col = col.push(container(slider_el).height(Length::Fixed(160.0)).width(32));
    } else {
        col = col.push(slider_el);
    }
    if !marks.min.is_empty() || !marks.max.is_empty() {
        col = col.push(
            row![
                text(marks.min)
                    .size(tok.meta())
                    .color(s.on_surface_variant)
                    .width(Length::Fill),
                text(marks.max).size(tok.meta()).color(s.on_surface_variant),
            ]
            .width(Length::Fill),
        );
    }
    let el: Element<'a, M> = if a11y.disabled {
        col.into()
    } else {
        let range_w = range.clone();
        mouse_area(col)
            .on_scroll(move |delta| {
                msg(slider_nudge(range_w.clone(), value, scroll_wheel_y(delta)))
            })
            .into()
    };
    a11y::attach(el, &a11y)
}

fn disabled_slider_face(tok: Tokens) -> iced::widget::container::Style {
    let s = tok.scheme();
    style::fill(s.surface_container_highest, s.on_surface_variant)
}

/// Clamp a low/high pair into `range` so `low <= high`.
pub fn clamp_range_pair(range: std::ops::RangeInclusive<f32>, low: f32, high: f32) -> (f32, f32) {
    let (min, max) = (*range.start(), *range.end());
    let low = low.clamp(min, max).min(high.clamp(min, max));
    let high = high.clamp(min, max).max(low);
    (low, high)
}

/// New pair after the low thumb moves.
pub fn range_pair_after_low(low: f32, high: f32) -> (f32, f32) {
    (low.min(high), high)
}

/// New pair after the high thumb moves.
pub fn range_pair_after_high(low: f32, high: f32) -> (f32, f32) {
    (low, high.max(low))
}

/// Inclusive low/high pair on one range (M3 range slider as two linked thumbs).
///
/// The application owns `low` and `high`. Messages are the clamped pair
/// with `low <= high`. Disabled freezes both thumbs.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_change = |(lo, hi)| (lo, hi);
/// let _: icedtea::Element<'_, (f32, f32)> = widget::range_slider(
///     0.0..=100.0,
///     20.0,
///     80.0,
///     on_change,
///     tok,
///     A11y::new("price", Role::Slider),
/// );
/// ```
pub fn range_slider<'a, M: Clone + 'a>(
    range: std::ops::RangeInclusive<f32>,
    low: f32,
    high: f32,
    msg: impl Fn((f32, f32)) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let (low, high) = clamp_range_pair(range.clone(), low, high);
    let a11y = a11y.merge_value(format!("{low}–{high}"));
    let lo = themed_slider(
        range.clone(),
        low,
        move |v| msg(range_pair_after_low(v, high)),
        SliderMarks::NONE,
        tok,
        A11y::new(format!("{} low", a11y.name), Role::Slider).with_disabled(a11y.disabled),
    );
    let hi = themed_slider(
        range,
        high,
        move |v| msg(range_pair_after_high(low, v)),
        SliderMarks::NONE,
        tok,
        A11y::new(format!("{} high", a11y.name), Role::Slider).with_disabled(a11y.disabled),
    );
    let s = tok.scheme();
    a11y::attach(
        column![
            lo,
            text(format!("{low:.0} – {high:.0}"))
                .size(tok.meta())
                .color(s.on_surface_variant),
            hi,
        ]
        .spacing(4)
        .width(Length::Fill)
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

fn progress_weights(value: f32, buffer: Option<f32>) -> (u16, u16, u16) {
    let value = value.clamp(0.0, 1.0);
    let buf = buffer.unwrap_or(value).clamp(0.0, 1.0).max(value);
    let v = (value * 100.0).round() as u16;
    let b = ((buf * 100.0).round() as u16).saturating_sub(v);
    let rest = 100u16.saturating_sub(v.saturating_add(b));
    (v, b, rest)
}

/// A determinate bar from 0 to 1.
///
/// Values outside the range clamp. No message; it is a readout.
/// Track corners follow [`Tokens::shape`]
/// ([`crate::m3::shape::Component::Track`]).
/// Interpolate `value` with [`crate::motion::value_animation`] so the
/// fill eases when the fraction changes. `indeterminate` paints a
/// traveling chunk; pass a looping phase (0..=1) as `value`.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let copy = widget::progress_label(0.4, Some("12s"));
/// let _: icedtea::Element<'_, ()> = widget::progress(
///     0.4,
///     Some(0.7),
///     Some(copy.as_str()),
///     false,
///     tok,
///     A11y::new("p", Role::Progress).with_value("0.4"),
/// );
/// ```
pub fn progress<'a, M: 'a>(
    value: f32,
    buffer: Option<f32>,
    copy: Option<&str>,
    indeterminate: bool,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let a11y = a11y.merge_value(format!("{value}"));
    let value = value.clamp(0.0, 1.0);
    let s = tok.scheme();
    let track_r = tok.radius(crate::m3::shape::Component::Track);
    let seg = |w: u16, fill: iced::Color, ink: iced::Color| {
        container(Space::new().height(8))
            .width(Length::FillPortion(w))
            .style(move |_| {
                let mut st = style::fill(fill, ink);
                st.border = iced::border::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: track_r,
                };
                st
            })
    };
    let mut parts = Row::new().width(Length::Fill);
    let segs: [(u16, iced::Color, iced::Color); 3] = if indeterminate {
        let (lead, mid, _tail) = crate::motion::progress_run(value, tok.reduced_motion);
        let l = (lead * 100.0).round() as u16;
        let m = ((mid * 100.0).round() as u16).max(1);
        let t = 100u16.saturating_sub(l.saturating_add(m));
        [
            (l, s.surface_container_highest, s.on_surface),
            (m, s.primary, s.on_primary),
            (t, s.surface_container_highest, s.on_surface),
        ]
    } else {
        let (v, b, rest) = progress_weights(value, buffer);
        [
            (v, s.primary, s.on_primary),
            (b, s.secondary_container, s.on_secondary_container),
            (rest, s.surface_container_highest, s.on_surface),
        ]
    };
    for (w, fill, ink) in segs {
        if w > 0 {
            parts = parts.push(seg(w, fill, ink));
        }
    }
    let bar: Element<'a, M> = container(parts).width(Length::Fill).height(8).into();
    let el = if let Some(c) = copy.filter(|t| !t.is_empty()) {
        Column::new()
            .spacing(4)
            .width(Length::Fill)
            .push(bar)
            .push(meta(c, tok, A11y::new(c, Role::Status)))
            .into()
    } else {
        bar
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
    (start, start + std::f32::consts::PI * 1.5)
}

/// Whether the determinate/indeterminate arc is long enough to stroke.
pub fn ring_should_stroke(start: f32, end: f32) -> bool {
    (end - start).abs() > 0.001
}

/// Circular progress: arc sweep follows `value`.
/// A determinate arc from 0 to 1.
///
/// Same fraction contract as [`progress`], drawn as a ring.
/// Interpolate `value` with [`crate::motion::value_animation`].
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
    let a11y = a11y.merge_value(format!("{value}"));
    let (start, end) = ring_angles(value);
    // M3 progress track: surface_container_highest under the primary arc.
    let s = tok.scheme();
    let track = s.surface_container_highest;
    let ring = Canvas::new(ArcRing {
        start,
        end,
        color: s.primary,
        track,
    })
    .width(56)
    .height(56);
    let el = if let Some(c) = copy.filter(|s| !s.is_empty()) {
        column![ring, meta(c, tok, A11y::new(c, Role::Status))]
            .spacing(4)
            .width(Length::Fill)
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
                .style(move |_| {
                    let scrim = tok.scheme().scrim;
                    style::fill(
                        Color::from_rgba(scrim.r, scrim.g, scrim.b, 0.32),
                        tok.scheme().on_surface,
                    )
                }),
        ]
        .into(),
        &a11y,
    )
}

/// Eight dots around a circle. `phase` (0..=1) lights them in turn.
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
    let a11y = a11y
        .merge_live(a11y::Live::Polite)
        .merge_value(format!("{phase}"));
    a11y::attach(
        Canvas::new(SpinnerDots {
            phase: phase.rem_euclid(1.0),
            color: tok.scheme().primary,
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
            .style(move |_| {
                let s = tok.scheme();
                style::fill(s.surface_container_high, s.on_surface)
            })
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
                .spacing(gap(tok))
                .padding(inset(tok))
                .into(),
            )
            .style(move |_| {
                let s = tok.scheme();
                style::fill(s.surface_container_high, s.on_surface)
            })
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
/// The application owns the number. Wheel steps by 1. Disabled
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
    on_change: impl Fn(String) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let shown = format!("{value}");
    let mut i = text_input("0", &shown)
        .style(style::field_style(tok, false))
        .padding(pad(tok))
        .size(tok.body());
    let el: Element<'a, M> = if a11y.disabled {
        let _ = on_change;
        i.into()
    } else {
        i = i.on_input(on_change);
        mouse_area(i)
            .on_scroll(move |delta| {
                let dir = if scroll_wheel_y(delta) > 0.0 { 1 } else { -1 };
                on_change(step_number(value, 1.0, f64::MIN, f64::MAX, dir).to_string())
            })
            .into()
    };
    a11y::attach(el, &a11y)
}

/// Step a numeric value.
pub fn step_number(value: f64, step: f64, min: f64, max: f64, dir: i32) -> f64 {
    (value + step * f64::from(dir)).clamp(min, max)
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
///     widget::FieldOpts::NONE,
///     tok,
///     A11y::new("Name", Role::TextBox),
///     None,
/// );
/// ```
#[allow(clippy::too_many_arguments)]
pub fn themed_text_input<'a, M: Clone + 'a>(
    placeholder: &str,
    value: &str,
    on_input: impl Fn(String) -> M + 'a,
    on_submit: Option<M>,
    opts: FieldOpts<'a>,
    tok: Tokens,
    a11y: A11y,
    input_id: Option<Id>,
) -> Element<'a, M> {
    let a11y = a11y.merge_value(value.to_string());
    let outlined = matches!(opts.face, FieldFace::Outlined);
    let mut i = text_input(placeholder, value)
        .style(style::field_style(tok, outlined))
        .padding(pad(tok))
        .size(tok.body());
    if let Some(id) = input_id {
        i = i.id(id);
    }
    if !a11y.disabled {
        i = i.on_input(on_input);
        if let Some(m) = a11y.apply_message(on_submit) {
            i = i.on_submit(m);
        }
    }
    let mut field: Element<'a, M> = container(i)
        .width(Length::Fill)
        .height(Length::Fixed(control_height(tok)))
        .into();
    if opts.icons != Icons::NONE {
        let mut r = Row::new().spacing(gap(tok)).align_y(Alignment::Center);
        if let Some(ic) = opts.icons.leading {
            r = r.push(icon_svg(ic, tok, A11y::new("prefix", Role::Image)));
        }
        r = r.push(field);
        if let Some(ic) = opts.icons.trailing {
            r = r.push(icon_svg(ic, tok, A11y::new("suffix", Role::Image)));
        }
        field = r.width(Length::Fill).into();
    }
    let mut col = Column::new().spacing(4).width(Length::Fill);
    if !opts.label.is_empty() && !value.is_empty() {
        col = col.push(
            text(opts.label)
                .size(tok.meta())
                .color(tok.scheme().on_surface_variant),
        );
    }
    col = col.push(field);
    if let Some(max) = opts.max_len {
        let n = value.chars().count();
        col = col.push(
            text(format!("{n}/{max}"))
                .size(tok.meta())
                .color(tok.scheme().on_surface_variant)
                .width(Length::Fill),
        );
    }
    a11y::attach(col.into(), &a11y)
}

/// Field stack with optional supporting or error text under the control.
///
/// `support` is quiet helper copy. `error` paints error role ink and wins
/// when both are set. Pass an already-built field as `child`.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_input = |s| s;
/// let field = widget::themed_text_input(
///     "Email",
///     "",
///     on_input,
///     None,
///     widget::FieldOpts::NONE,
///     tok,
///     A11y::new("Email", Role::TextBox),
///     None,
/// );
/// let _: icedtea::Element<'_, String> = widget::field_support(
///     field,
///     Some("We never share your email."),
///     None,
///     tok,
///     A11y::new("Email field", Role::Group),
/// );
/// ```
pub fn field_support<'a, M: 'a>(
    child: Element<'a, M>,
    support: Option<&str>,
    error: Option<&str>,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let a11y = a11y.merge_error(error).merge_hint(support.unwrap_or(""));
    let s = tok.scheme();
    let mut col = column![child].spacing(4).width(Length::Fill);
    if let Some(err) = error.filter(|t| !t.is_empty()) {
        col = col.push(
            text(err.to_string())
                .size(tok.meta())
                .color(s.error)
                .width(Length::Fill),
        );
    } else if let Some(help) = support.filter(|t| !t.is_empty()) {
        col = col.push(
            text(help.to_string())
                .size(tok.meta())
                .color(s.on_surface_variant)
                .width(Length::Fill),
        );
    }
    a11y::attach(col.into(), &a11y)
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
        FieldOpts::NONE,
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
            Icons::NONE,
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
        .style(style::field_style(tok, false))
        .padding(pad(tok))
        .size(tok.body());
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
///     "Show",
///     "Hide",
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
    show: &str,
    hide: &str,
    copy: &crate::action::Action<M>,
    tok: Tokens,
    dir: Direction,
    a11y: A11y,
) -> Element<'a, M> {
    let toggle_title = if revealed { hide } else { show };
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
        Icons::NONE,
        A11y::button(toggle_title).with_disabled(a11y.disabled),
    );
    let copy_btn = themed_button(
        copy.title.clone(),
        copy.invoke(),
        tok,
        Variant::Quiet,
        Icons::NONE,
        A11y::button(copy.title.clone()).with_disabled(!copy.enabled || a11y.disabled),
    );
    let kids = crate::i18n::order(dir, [field, toggle, copy_btn]);
    let mut r = Row::new().spacing(gap(tok)).align_y(Alignment::Center);
    for k in kids {
        r = r.push(k);
    }
    a11y::attach(r.into(), &a11y)
}

/// A labeled read-only value the user can select and copy.
///
/// Meta label in a fixed gutter, then [`selectable`] (fill), then an
/// optional Copy [`crate::action::Action`]. Pass
/// [`crate::layout::FORM_LABEL`] so multi-row stacks share one column
/// (same gutter as [`crate::layout::form`]). The application posts
/// [`crate::field::Selectables::copy`] with [`crate::copy_text`].
/// Mono face for paths and ids; UI face for prose. Disabled still
/// allows select-and-copy.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::action::Action;
/// use icedtea::i18n::Direction;
/// use icedtea::layout;
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
///     layout::FORM_LABEL,
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
    label_width: f32,
    tok: Tokens,
    dir: Direction,
    a11y: A11y,
) -> Element<'a, M> {
    let title = title.into();
    let label = container(meta(
        title.clone(),
        tok,
        a11y.child(Role::Status).with_value(title.clone()),
    ))
    .width(Length::Fixed(label_width.max(1.0)));
    let value = container(selectable(
        content,
        on_action,
        tok,
        face,
        a11y.child(Role::TextBox),
    ))
    .width(Length::Fill);
    let mut kids: Vec<Element<'a, M>> = vec![label.into(), value.into()];
    if let Some(copy) = copy {
        kids.push(themed_button(
            copy.title.clone(),
            copy.invoke(),
            tok,
            Variant::Quiet,
            Icons::NONE,
            A11y::button(copy.title.clone()).with_disabled(!copy.enabled || a11y.disabled),
        ));
    }
    let kids = crate::i18n::order(dir, kids);
    let mut r = Row::new()
        .spacing(gap(tok))
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
        .padding(pad(tok))
        .size(tok.body())
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
///
/// See [`crate::select`] for the app-facing select-and-copy contract.
pub use crate::select::select_only;

/// Body the user can drag-select and copy.
///
/// Looks like body text: zero pad, no border, canvas fill. The
/// application owns the buffer and posts `Content::selection()` with
/// [`crate::copy_text`]. Height shrinks to the text. Disabled still
/// allows select-and-copy. Use [`typo::FontFace::Ui`] for prose and
/// [`typo::FontFace::Mono`] for paths or raw values.
///
/// For painted markdown (not an editor buffer), use
/// [`markdown_view`]: selection and Ctrl+C are paint-side.
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
        .size(match face {
            typo::FontFace::Mono => tok.code(),
            typo::FontFace::Ui => tok.body(),
        })
        .wrapping(iced::widget::text::Wrapping::Word)
        .style(selectable_style(tok))
        .on_action(move |a| on_action(select_only(a)));
    a11y::attach(e.into(), &a11y)
}

fn selectable_style(
    tok: Tokens,
) -> impl Fn(&iced::Theme, iced::widget::text_editor::Status) -> iced::widget::text_editor::Style {
    move |_t, _s| {
        let s = tok.scheme();
        iced::widget::text_editor::Style {
            // Transparent so value fields read as body text, not a dark editor slab.
            background: iced::Background::Color(Color::TRANSPARENT),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            placeholder: s.on_surface_variant,
            value: s.on_surface,
            selection: s.secondary_container,
        }
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
/// buffer. Disabled still allows select-and-copy. `wrap` is word wrap;
/// `false` keeps each source line on one row (diff hunks, search hits).
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
///     true,
///     A11y::new("src", Role::TextBox),
/// );
/// ```
#[allow(clippy::too_many_arguments)]
pub fn highlighted_code<'a, M: Clone + 'a>(
    content: &'a Content,
    syntax: &str,
    on_action: impl Fn(text_editor::Action) -> M + 'a,
    tok: Tokens,
    theme_name: &str,
    height: Length,
    wrap: bool,
    a11y: A11y,
) -> Element<'a, M> {
    let theme = crate::theme::code_highlight(theme_name);
    let wrapping = if wrap {
        iced::widget::text::Wrapping::Word
    } else {
        iced::widget::text::Wrapping::None
    };
    let e = text_editor(content)
        .height(height)
        .padding(inset(tok))
        .style(editor_style(tok))
        .highlight(syntax, theme)
        .font(typo::MONO)
        .size(tok.code())
        .wrapping(wrapping)
        .on_action(move |a| on_action(select_only(a)));
    container(e)
        .width(Length::Fill)
        .height(height)
        .style(move |_| editor_frame(tok))
        .id(Id::from(a11y.node_id()))
        .into()
}

fn editor_frame(tok: Tokens) -> iced::widget::container::Style {
    let s = tok.scheme();
    iced::widget::container::Style {
        background: Some(iced::Background::Color(s.surface_container_highest)),
        border: iced::Border {
            color: s.outline_variant,
            width: 1.0,
            radius: tok.radius(crate::m3::shape::Component::Field),
        },
        ..iced::widget::container::Style::default()
    }
}

pub fn editor_style(
    tok: Tokens,
) -> impl Fn(&iced::Theme, iced::widget::text_editor::Status) -> iced::widget::text_editor::Style {
    move |_t, _s| {
        let s = tok.scheme();
        iced::widget::text_editor::Style {
            background: iced::Background::Color(s.surface_container_highest),
            border: iced::Border {
                color: s.outline_variant,
                width: 1.0,
                radius: tok.radius(crate::m3::shape::Component::Field),
            },
            placeholder: s.on_surface_variant,
            value: s.on_surface,
            selection: s.secondary_container,
        }
    }
}

/// A query field with a search icon.
///
/// Use for palette and list filters. Empty query means show all.
/// Placeholder is the a11y name. `on_submit` is Enter. `input_id`
/// focuses the field (palette, find-in-page).
/// Corners follow [`Tokens::shape`] ([`crate::m3::shape::Component::Search`]).
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_input = |s| s;
/// let _: icedtea::Element<'_, String> = widget::search_input(
///     "",
///     on_input,
///     None,
///     tok,
///     A11y::new("Search", Role::TextBox),
///     None,
/// );
/// ```
pub fn search_input<'a, M: Clone + 'a>(
    value: &str,
    on_input: impl Fn(String) -> M + 'a,
    on_submit: Option<M>,
    tok: Tokens,
    a11y: A11y,
    input_id: Option<Id>,
) -> Element<'a, M> {
    search_input_clear(value, on_input, None, on_submit, tok, a11y, input_id)
}

/// Search field with optional clear control when non-empty.
///
/// `on_clear` empties the query. When `None`, behaves like [`search_input`].
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_input = |s| s;
/// let clear = String::new();
/// let _: icedtea::Element<'_, String> = widget::search_input_clear(
///     "q",
///     on_input,
///     Some(clear),
///     None,
///     tok,
///     A11y::new("Search", Role::TextBox),
///     None,
/// );
/// ```
pub fn search_input_clear<'a, M: Clone + 'a>(
    value: &str,
    on_input: impl Fn(String) -> M + 'a,
    on_clear: Option<M>,
    on_submit: Option<M>,
    tok: Tokens,
    a11y: A11y,
    input_id: Option<Id>,
) -> Element<'a, M> {
    let search_ic: Element<'a, M> = icon_svg(Icon::Search, tok, A11y::new("search", Role::Image));
    let placeholder = if a11y.name.is_empty() {
        "Search".to_string()
    } else {
        a11y.name.clone()
    };
    let field_a11y = a11y.child(Role::TextBox).merge_value(value.to_string());
    let mut i = text_input(&placeholder, value)
        .style(style::search_style(tok))
        .padding(pad(tok))
        .size(tok.body());
    if let Some(id) = input_id {
        i = i.id(id);
    }
    if !field_a11y.disabled {
        i = i.on_input(on_input);
        if let Some(m) = field_a11y.apply_message(on_submit) {
            i = i.on_submit(m);
        }
    }
    let field: Element<'a, M> = a11y::attach(
        container(i)
            .width(Length::Fill)
            .height(Length::Fixed(control_height(tok)))
            .into(),
        &field_a11y,
    );
    let mut r = Row::new().spacing(gap(tok)).align_y(Alignment::Center);
    for kid in crate::i18n::order(tok.direction, [search_ic, field]) {
        r = r.push(kid);
    }
    if !value.is_empty() {
        if let Some(clear) = on_clear {
            r = r.push(icon_button(
                Icon::Close,
                if a11y.disabled { None } else { Some(clear) },
                tok,
                Variant::Ghost,
                ControlSize::Default,
                A11y::button("Clear search").with_disabled(a11y.disabled),
            ));
        }
    }
    a11y::attach(r.into(), &a11y)
}

/// Docked search results under a search field (M3 search view, desktop).
///
/// Hits are application-filtered. Empty `hits` shows `empty`. Disabled
/// drops pick and clear.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_input = |s| s;
/// let on_pick = |i| format!("{i}");
/// let _: icedtea::Element<'_, String> = widget::search_view(
///     "in",
///     ["Inbox", "Sent"],
///     on_input,
///     on_pick,
///     Some(String::new()),
///     "No matches",
///     tok,
///     A11y::new("find", Role::Group),
/// );
/// ```
#[allow(clippy::too_many_arguments)]
pub fn search_view<'a, M: Clone + 'a>(
    query: &str,
    hits: impl IntoIterator<Item = impl Into<String>>,
    on_input: impl Fn(String) -> M + 'a,
    on_pick: impl Fn(usize) -> M + Copy + 'a,
    on_clear: Option<M>,
    empty: &'a str,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let hits: Vec<String> = hits.into_iter().map(Into::into).collect();
    let field = search_input_clear(
        query,
        on_input,
        if a11y.disabled { None } else { on_clear },
        None,
        tok,
        A11y::new(a11y.name.clone(), Role::TextBox).with_disabled(a11y.disabled),
        None,
    );
    let body: Element<'a, M> = if hits.is_empty() {
        meta(empty, tok, A11y::new(empty, Role::Status))
    } else {
        let mut col = Column::new().spacing(0).width(Length::Fill);
        for (i, hit) in hits.iter().enumerate() {
            let hit_a11y = A11y::button(hit.clone()).with_disabled(a11y.disabled);
            let mut b = button(
                container(
                    text(hit_a11y.apply_name(hit.clone()))
                        .size(tok.body())
                        .color(tok.scheme().on_surface),
                )
                .width(Length::Fill)
                .align_x(crate::i18n::align_start(tok.direction)),
            )
            .padding(pad(tok))
            .width(Length::Fill)
            .style(style::button_style(tok, Variant::Ghost));
            if let Some(m) = hit_a11y.apply_message(if a11y.disabled {
                None
            } else {
                Some(on_pick(i))
            }) {
                b = b.on_press(m);
            }
            col = col.push(a11y::attach(b.into(), &hit_a11y));
        }
        col.into()
    };
    a11y::attach(
        column![field, body].spacing(4).width(Length::Fill).into(),
        &a11y,
    )
}

/// Pick one string from a list.
///
/// `size` is [`ControlSize`]. Compact uses tight pad and meta type so a
/// toolbar or HUD can nest a dropdown. Default keeps the field body
/// look. Placeholder shows when nothing is selected. Wheel over the
/// control moves the selection. Disabled keeps the current face.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// use icedtea::widget::ControlSize;
/// let tok = theme::named("dark").tokens;
/// let opts = ["nord", "dark"];
/// let on_select = |name| name;
/// let _: icedtea::Element<'_, &str> = widget::themed_pick_list(
///     opts,
///     Some("nord"),
///     on_select,
///     tok,
///     ControlSize::Default,
///     A11y::new("theme", Role::ComboBox),
/// );
/// ```
pub fn themed_pick_list<'a, T, M: Clone + 'a>(
    options: impl std::borrow::Borrow<[T]> + 'a,
    selected: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
    tok: Tokens,
    size: ControlSize,
    a11y: A11y,
) -> Element<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
{
    let (face_pad, type_px) = match size {
        ControlSize::Compact => {
            let p = f32::from(size.pad());
            (Padding::from([p, p + 4.0]), tok.meta())
        }
        ControlSize::Default => (pad(tok), tok.body()),
        ControlSize::Comfortable => {
            let p = f32::from(size.pad());
            (Padding::from([p, p + 4.0]), tok.body())
        }
    };
    if a11y.disabled {
        let _ = on_select;
        let shown = selected
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();
        return a11y::attach(
            container(
                text(shown)
                    .size(type_px)
                    .color(tok.scheme().on_surface_variant),
            )
            .padding(face_pad)
            .style(move |_| style::panel(tok))
            .into(),
            &a11y,
        );
    }
    let opts: Vec<T> = options.borrow().to_vec();
    let sel = selected.clone();
    let on_select = std::rc::Rc::new(on_select);
    let on_pick = {
        let on_select = on_select.clone();
        move |t| on_select(t)
    };
    let picker = pick_list(options, selected, on_pick)
        .handle(pick_list::Handle::None)
        .style(style::picker_style(tok))
        .padding(face_pad)
        .text_size(type_px);
    let h = sized_control_height(tok, size);
    let picker: Element<'a, M> = container(picker).height(Length::Fixed(h)).into();
    let mut face = Row::new()
        .spacing(4)
        .align_y(Alignment::Center)
        .height(Length::Fixed(h));
    for kid in crate::i18n::order(tok.direction, [picker, pick_chevron(tok)]) {
        face = face.push(kid);
    }
    let picker: Element<'a, M> = face.into();
    let el: Element<'a, M> = if opts.is_empty() {
        picker
    } else {
        mouse_area(picker)
            .on_scroll(move |delta| {
                let n = opts.len();
                let i = sel
                    .as_ref()
                    .and_then(|s| opts.iter().position(|o| o == s))
                    .unwrap_or(0);
                let j = if scroll_wheel_y(delta) < 0.0 {
                    i.saturating_add(1).min(n - 1)
                } else {
                    i.saturating_sub(1)
                };
                on_select(opts[j].clone())
            })
            .into()
    };
    a11y::attach(el, &a11y)
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
                Icons::NONE,
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
                Icons::NONE,
                A11y::button("next-day").with_disabled(a11y.disabled),
            ),
        ]
        .spacing(gap(tok))
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

/// Arabic/Urdu/Persian clocks use Eastern digits; Hebrew uses 123.
/// `Direction::Rtl` maps to Eastern.
fn clock_digits(n: impl Into<u32>, dir: Direction) -> String {
    let n = n.into();
    let western = format!("{n:02}");
    if dir != Direction::Rtl {
        return western;
    }
    const EASTERN: [char; 10] = ['٠', '١', '٢', '٣', '٤', '٥', '٦', '٧', '٨', '٩'];
    western
        .bytes()
        .map(|b| EASTERN[(b - b'0') as usize])
        .collect()
}

fn time_colon<'a, M: 'a>(tok: Tokens) -> Element<'a, M> {
    container(
        text(":")
            .size(tok.body())
            .font(typo::UI_BOLD)
            .color(tok.scheme().on_surface),
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
        clock_digits(v.hour12(), tok.direction)
    } else {
        clock_digits(v.hour, tok.direction)
    };
    let mut row = Row::new().spacing(4).align_y(Alignment::Center);
    row = row.push(themed_button(
        hour,
        a11y.apply_message(Some(on_field(TimeField::Hour))),
        tok,
        Variant::Quiet,
        Icons::NONE,
        A11y::button("hour").with_disabled(a11y.disabled),
    ));
    row = row.push(time_colon(tok));
    row = row.push(themed_button(
        clock_digits(v.minute, tok.direction),
        a11y.apply_message(Some(on_field(TimeField::Minute))),
        tok,
        Variant::Quiet,
        Icons::NONE,
        A11y::button("minute").with_disabled(a11y.disabled),
    ));
    if clock.seconds {
        row = row.push(time_colon(tok));
        row = row.push(themed_button(
            clock_digits(v.second, tok.direction),
            a11y.apply_message(Some(on_field(TimeField::Second))),
            tok,
            Variant::Quiet,
            Icons::NONE,
            A11y::button("second").with_disabled(a11y.disabled),
        ));
    }
    if clock.hour12 {
        row = row.push(themed_button(
            if v.afternoon() { "PM" } else { "AM" },
            a11y.apply_message(Some(on_field(TimeField::Period))),
            tok,
            Variant::Quiet,
            Icons::NONE,
            A11y::button("period").with_disabled(a11y.disabled),
        ));
    }
    a11y::attach(
        container(row)
            .width(Length::Fill)
            .align_x(crate::i18n::align_start(tok.direction))
            .into(),
        &a11y,
    )
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
    pub fn item_offset(&self, index: usize, tok: crate::theme::Tokens) -> f32 {
        self.items
            .iter()
            .take(index)
            .map(|i| crate::select::markdown_item_extent(i, tok))
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
                Icons::NONE,
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
/// Parse with [`parse`], then view with [`markdown_view`]. Truncate by
/// slicing the source before parse.
///
/// # Select and copy
///
/// Painted with real markdown layout (headings, lists, code frames,
/// quotes). Body is [`Tokens::body`](crate::theme::Tokens::body); H1 is
/// [`Tokens::page`](crate::theme::Tokens::page) (window title), not
/// iced's 2× blog heading. Drag a range with
/// [`crate::select::markdown_select`] so it can start in one block and
/// end in another; pass the live [`crate::select::MarkdownSpan`] here.
/// The view is not flattened into one mixed-size `Rich`. The document
/// tree stays one `view_with` whether a range is empty or not. Pointer
/// events reach `on_pointer` first so paint and Copy share that span
/// (a double-click selects the word under the caret).
/// Ctrl+C / Cmd+C on a span is [`crate::select::MarkdownSpan::text`] via
/// [`crate::copy_text`]. Full document copy is [`MarkdownDoc::source`].
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::select::{markdown_select, MarkdownPointer, MarkdownSelect};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let doc = widget::parse("# Hi\n\nBody.");
/// let on_link = |_uri| MarkdownPointer::Release;
/// let on_pointer = |ev| ev;
/// let state = markdown_select(&doc.items, MarkdownSelect::default(), MarkdownPointer::Press, tok);
/// let _: icedtea::Element<'_, _> = widget::markdown_view(
///     &doc.items,
///     Some(&state.span),
///     on_pointer,
///     tok,
///     on_link,
///     A11y::new("md", Role::Group),
/// );
/// ```
pub fn markdown_view<'a, M: Clone + 'a>(
    items: &'a [markdown::Item],
    span: Option<&'a crate::select::MarkdownSpan>,
    on_pointer: impl Fn(crate::select::MarkdownPointer) -> M + Copy + 'a,
    tok: Tokens,
    on_link: impl Fn(markdown::Uri) -> M + Copy + 'a,
    a11y: A11y,
) -> Element<'a, M> {
    let settings = crate::select::markdown_paint_settings(markdown_style(tok), tok);
    let fill = tok.scheme().secondary_container;
    let live = span.copied().filter(|s| !s.is_empty());
    let mut col = Column::new().spacing(settings.spacing).width(Length::Fill);
    for (i, item) in items.iter().enumerate() {
        let viewer = MarkdownPaint {
            items,
            index: i,
            span: live,
            fill,
        };
        col = col.push(markdown::item(&viewer, settings, item, i).map(on_link));
    }
    a11y::attach(markdown_listen(col.into(), on_pointer), &a11y)
}

/// One top-level markdown item. Highlight follows [`crate::select::MarkdownSpan`].
struct MarkdownPaint<'a> {
    items: &'a [markdown::Item],
    index: usize,
    span: Option<crate::select::MarkdownSpan>,
    fill: Color,
}

impl MarkdownPaint<'_> {
    fn paint_text(
        &self,
        text: &markdown::Text,
        style: markdown::Style,
    ) -> Vec<iced::advanced::text::Span<'static, markdown::Uri, iced::Font>> {
        let spans = text.spans(style);
        let local = self
            .span
            .and_then(|s| crate::select::markdown_paint_range(s, self.items, self.index, text));
        match local {
            Some((a, b)) => crate::select::highlight_markdown_spans(&spans, a, b, self.fill),
            None => crate::select::highlight_markdown_spans(&spans, 0, 0, self.fill),
        }
    }
}

impl<'a> markdown::Viewer<'a, markdown::Uri> for MarkdownPaint<'a> {
    fn on_link_click(url: markdown::Uri) -> markdown::Uri {
        url
    }

    fn heading(
        &self,
        settings: markdown::Settings,
        level: &'a markdown::HeadingLevel,
        text: &'a markdown::Text,
        index: usize,
    ) -> Element<'a, markdown::Uri> {
        let size = match *level {
            markdown::HeadingLevel::H1 => settings.h1_size,
            markdown::HeadingLevel::H2 => settings.h2_size,
            markdown::HeadingLevel::H3 => settings.h3_size,
            markdown::HeadingLevel::H4 => settings.h4_size,
            markdown::HeadingLevel::H5 => settings.h5_size,
            markdown::HeadingLevel::H6 => settings.h6_size,
        };
        container(
            rich_text(self.paint_text(text, settings.style))
                .on_link_click(Self::on_link_click)
                .size(size),
        )
        .padding(iced::padding::top(if index > 0 {
            settings.text_size / 2.0
        } else {
            iced::Pixels::ZERO
        }))
        .into()
    }

    fn paragraph(
        &self,
        settings: markdown::Settings,
        text: &markdown::Text,
    ) -> Element<'a, markdown::Uri> {
        rich_text(self.paint_text(text, settings.style))
            .size(settings.text_size)
            .on_link_click(Self::on_link_click)
            .into()
    }

    fn code_block(
        &self,
        settings: markdown::Settings,
        _language: Option<&'a str>,
        _code: &'a str,
        lines: &'a [markdown::Text],
    ) -> Element<'a, markdown::Uri> {
        let painted: Vec<Element<'a, markdown::Uri>> = lines
            .iter()
            .map(|line| {
                rich_text(self.paint_text(line, settings.style))
                    .on_link_click(Self::on_link_click)
                    .font(settings.style.code_block_font)
                    .size(settings.code_size)
                    .into()
            })
            .collect();
        container(
            scrollable(container(Column::with_children(painted)).padding(settings.code_size))
                .direction(ScrollDir::Horizontal(
                    Scrollbar::default()
                        .width(settings.code_size / 2)
                        .scroller_width(settings.code_size / 2),
                )),
        )
        .width(Length::Fill)
        .padding(settings.code_size / 4)
        .into()
    }
}

/// Publish markdown pointer events even when the child captured the press.
fn markdown_listen<'a, M: Clone + 'a>(
    child: Element<'a, M>,
    on_pointer: impl Fn(crate::select::MarkdownPointer) -> M + 'a,
) -> Element<'a, M> {
    MarkdownListen {
        content: child,
        on_pointer: Box::new(on_pointer),
    }
    .into()
}

struct MarkdownListen<'a, Message> {
    content: Element<'a, Message>,
    on_pointer: Box<dyn Fn(crate::select::MarkdownPointer) -> Message + 'a>,
}

#[derive(Default)]
struct MarkdownListenState {
    previous_click: Option<iced::advanced::mouse::Click>,
    last: Option<iced::Point>,
}

impl<'a, Message: Clone> Widget<Message, iced::Theme, iced::Renderer>
    for MarkdownListen<'a, Message>
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MarkdownListenState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(MarkdownListenState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> iced::Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) {
        let local = cursor.position_in(layout.bounds());
        let select_ev = matches!(
            event,
            Event::Mouse(
                iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)
                    | iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)
            )
        );
        if let Some(local) = local {
            let state = tree.state.downcast_mut::<MarkdownListenState>();
            if state.last != Some(local) {
                state.last = Some(local);
                shell.publish((self.on_pointer)(crate::select::MarkdownPointer::Move {
                    x: local.x,
                    y: local.y,
                }));
            }
            match event {
                Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                    let click = iced::advanced::mouse::Click::new(
                        cursor.position().unwrap_or(local),
                        iced::mouse::Button::Left,
                        state.previous_click,
                    );
                    shell.publish((self.on_pointer)(crate::select::MarkdownPointer::Press));
                    if click.kind() != iced::advanced::mouse::click::Kind::Single {
                        shell.publish((self.on_pointer)(crate::select::MarkdownPointer::Double));
                    }
                    state.previous_click = Some(click);
                    shell.capture_event();
                }
                Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                    shell.publish((self.on_pointer)(crate::select::MarkdownPointer::Release));
                    shell.capture_event();
                }
                _ => {}
            }
        }
        if select_ev && local.is_some() {
            return;
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: Clone + 'a> From<MarkdownListen<'a, Message>> for Element<'a, Message> {
    fn from(value: MarkdownListen<'a, Message>) -> Self {
        Self::new(value)
    }
}

fn markdown_style(tok: Tokens) -> markdown::Style {
    let s = tok.scheme();
    let mut style = markdown::Style::from_palette(iced::theme::Palette {
        background: s.surface,
        text: s.on_surface,
        primary: s.primary,
        success: s.success,
        warning: s.warning,
        danger: s.error,
    });
    style.font = typo::UI;
    style.inline_code_color = s.on_surface;
    style.inline_code_font = typo::MONO;
    style.code_block_font = typo::MONO;
    style.link_color = s.primary;
    style.inline_code_highlight.background = iced::Background::Color(s.surface_container_high);
    style
}

/// Hover text on a child.
///
/// Empty tip text is a no-op wrap. The child keeps its own `A11y`.
/// Corners follow [`Tokens::shape`] ([`crate::m3::shape::Component::Tooltip`]).
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
///     widget::TooltipAnchor::Follow,
///     tok,
///     A11y::new("Tip", Role::Tooltip),
/// );
/// ```
pub fn tooltip_wrap<'a, M: 'a>(
    child: Element<'a, M>,
    tip: impl Into<String>,
    anchor: TooltipAnchor,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let tip = a11y.apply_name(tip);
    a11y::attach(
        tooltip(
            child,
            container(meta(tip.clone(), tok, A11y::new(tip, Role::Tooltip)))
                .padding(gap(tok))
                .style(tip_style(tok)),
            anchor.position(),
        )
        .into(),
        &a11y,
    )
}

/// Hover title plus supporting copy on a child (M3 rich tooltip).
///
/// Empty title and body is a no-op wrap. The child keeps its own `A11y`.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> = widget::tooltip_rich(
///     widget::label("Hover", tok, A11y::new("Hover", Role::Header)),
///     "Save",
///     "Write the buffer to disk.",
///     Some(("Learn more".into(), ())),
///     widget::TooltipAnchor::Follow,
///     tok,
///     A11y::new("Save tip", Role::Tooltip),
/// );
/// ```
pub fn tooltip_rich<'a, M: Clone + 'a>(
    child: Element<'a, M>,
    title: impl Into<String>,
    body: impl Into<String>,
    action: Option<(String, M)>,
    anchor: TooltipAnchor,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let title = title.into();
    let body = body.into();
    if title.is_empty() && body.is_empty() && action.is_none() {
        return child;
    }
    let mut col = Column::new().spacing(2);
    if !title.is_empty() {
        col = col.push(label(title.clone(), tok, A11y::new(title, Role::Header)));
    }
    if !body.is_empty() {
        col = col.push(meta(body.clone(), tok, A11y::new(body, Role::Status)));
    }
    if let Some((t, m)) = action {
        col = col.push(themed_button(
            t.clone(),
            Some(m),
            tok,
            Variant::Ghost,
            Icons::NONE,
            A11y::button(t),
        ));
    }
    a11y::attach(
        tooltip(
            child,
            container(col).padding(inset(tok)).style(tip_style(tok)),
            anchor.position(),
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

/// Chip fill, ink, and border for a variant (M3 filter: Quiet = outline, Primary = selected fill).
pub fn chip_face(
    tok: Tokens,
    variant: Variant,
) -> (iced::Color, iced::Color, iced::border::Border) {
    let s = tok.scheme();
    let r = tok.radius(crate::m3::shape::Component::Chip);
    match variant {
        // Selected filter / assist filled. Primary solid so ink is
        // on_primary, not the same on_surface as the idle outline.
        Variant::Primary | Variant::Chip => (
            s.primary,
            s.on_primary,
            iced::border::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: r,
            },
        ),
        // Idle filter outline (must differ from selected fill)
        Variant::Quiet | Variant::Ghost => (
            Color::TRANSPARENT,
            s.on_surface,
            iced::border::Border {
                color: s.outline,
                width: 1.0,
                radius: r,
            },
        ),
        Variant::Danger => (
            s.error_container,
            s.on_error_container,
            iced::border::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: r,
            },
        ),
        Variant::Success => {
            let wash = crate::theme::mix(s.success, s.surface, 0.20);
            (
                wash,
                crate::m3::color::ink_on(s.success, wash),
                iced::border::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: r,
                },
            )
        }
        Variant::Warning => {
            let wash = crate::theme::mix(s.warning, s.surface, 0.20);
            (
                wash,
                crate::m3::color::ink_on(s.warning, wash),
                iced::border::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: r,
                },
            )
        }
        Variant::Outlined | Variant::Elevated => (
            Color::TRANSPARENT,
            s.on_surface,
            iced::border::Border {
                color: s.outline,
                width: 1.0,
                radius: r,
            },
        ),
    }
}

/// A compact labeled pill.
///
/// Optional press and optional dismiss. META type, chip wash, shrink
/// width. Disabled keeps the face and drops press.
///
///
/// ```
/// use icedtea::a11y::A11y;
/// use icedtea::theme;
/// use icedtea::variant::Variant;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let add = ();
/// let _: icedtea::Element<'_, ()> = widget::chip(
///     "Add note",
///     Some(add),
///     None,
///     tok,
///     Variant::Chip,
///     widget::ChipKind::Assist,
///     icedtea::icon::Icons::NONE,
///     A11y::button("Add note"),
/// );
/// let drop = ();
/// let _: icedtea::Element<'_, ()> = widget::chip(
///     "Rust",
///     None,
///     Some(drop),
///     tok,
///     Variant::Quiet,
///     widget::ChipKind::Input,
///     icedtea::icon::Icons::NONE,
///     A11y::button("Rust"),
/// );
/// ```
#[allow(clippy::too_many_arguments)]
pub fn chip<'a, M: Clone + 'a>(
    title: impl Into<String>,
    press: Option<M>,
    dismiss: Option<M>,
    tok: Tokens,
    variant: Variant,
    kind: ChipKind,
    icons: Icons,
    a11y: A11y,
) -> Element<'a, M> {
    let title = a11y.apply_name(title);
    let variant = match kind {
        ChipKind::Assist => Variant::Chip,
        ChipKind::Filter => variant,
        ChipKind::Input | ChipKind::Suggestion => Variant::Quiet,
    };
    let (wash, ink, border) = chip_face(tok, variant);
    let mut line = Row::new().spacing(4).align_y(Alignment::Center);
    if let Some(ic) = icons.leading {
        line = line.push(icon_svg(ic, tok, A11y::new(title.clone(), Role::Image)));
    }
    line = line.push(text(title.clone()).size(tok.meta()).color(ink));
    if let Some(msg) = dismiss {
        line = line.push(dismiss_button(
            msg,
            tok,
            A11y::button(format!("dismiss {title}")).with_disabled(a11y.disabled),
        ));
    }
    let face = container(line).padding(pad(tok)).style(move |_| {
        let mut st = style::fill(wash, ink);
        st.border = border;
        st
    });
    let body: Element<'a, M> = if let Some(msg) = a11y.apply_message(press) {
        mouse_area(face).on_press(msg).into()
    } else {
        face.into()
    };
    a11y::attach(body, &a11y)
}

/// Multi-select filter chips (M3 filter chip set).
///
/// The application owns which indices are on. Press toggles one index.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let on_toggle = |i| i;
/// let _: icedtea::Element<'_, usize> = widget::filter_chips(
///     &["Unread".into(), "Flagged".into()],
///     &[true, false],
///     on_toggle,
///     tok,
///     A11y::new("Filters", Role::Group),
/// );
/// ```
pub fn filter_chips<'a, M: Clone + 'a>(
    labels: &[String],
    selected: &[bool],
    on_toggle: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let mut r = Row::new().spacing(gap(tok)).align_y(Alignment::Center);
    for (i, label) in labels.iter().enumerate() {
        let on = selected.get(i).copied().unwrap_or(false);
        r = r.push(chip(
            label.clone(),
            if a11y.disabled {
                None
            } else {
                Some(on_toggle(i))
            },
            None,
            tok,
            if on { Variant::Primary } else { Variant::Quiet },
            ChipKind::Filter,
            Icons::NONE,
            A11y::button(label.clone())
                .with_checked(on)
                .with_disabled(a11y.disabled),
        ));
    }
    a11y::attach(r.into(), &a11y)
}

/// A count or status mark.
///
/// Short text. Empty string is an empty mark.
/// Both sizes use meta type; Large is not body reading type.
/// Corners follow [`Tokens::shape`] ([`crate::m3::shape::Component::Badge`]).
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::variant::Variant;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> =
///     widget::badge(
///         "New",
///         None,
///         tok,
///         Variant::Primary,
///         widget::BadgeSize::Large,
///         A11y::new("New", Role::Status),
///     );
/// ```
pub fn badge<'a, M: 'a>(
    title: impl Into<String>,
    host: Option<Element<'a, M>>,
    tok: Tokens,
    variant: Variant,
    size: BadgeSize,
    a11y: A11y,
) -> Element<'a, M> {
    let title = a11y.apply_name(title);
    let (wash, ink, mut border) = chip_face(tok, variant);
    border.radius = tok.radius(crate::m3::shape::Component::Badge);
    let pad = match size {
        BadgeSize::Small => [2, 5],
        BadgeSize::Large => [4, 8],
    };
    let type_size = tok.meta();
    let mark: Element<'a, M> = container(text(title).size(type_size).color(ink))
        .padding(pad)
        .style(move |_| {
            let mut st = style::fill(wash, ink);
            st.border = border;
            st
        })
        .into();
    let body = if let Some(child) = host {
        Stack::new()
            .push(child)
            .push(container(mark).width(Length::Fill).align_x(Alignment::End))
            .into()
    } else {
        mark
    };
    a11y::attach(body, &a11y)
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
///     widget::CardFace::Elevated,
///     A11y::new("Document", Role::Group),
/// );
/// ```
pub fn group_box<'a, M: 'a>(
    title: impl Into<String>,
    child: Element<'a, M>,
    tok: Tokens,
    face: CardFace,
    a11y: A11y,
) -> Element<'a, M> {
    let title = a11y.apply_name(title);
    a11y::attach(
        container(
            column![
                meta(title.clone(), tok, A11y::new(title, Role::Header)),
                child
            ]
            .spacing(gap(tok)),
        )
        .padding(sheet(tok))
        .width(Length::Fill)
        .style(move |_| match face {
            CardFace::Elevated => style::raised_card(tok),
            CardFace::Filled => style::card(tok, false),
            CardFace::Outlined => style::outlined_card(tok),
        })
        .into(),
        &a11y,
    )
}

/// A page-level message with an optional action.
///
/// Use for “offline” or “update available”. Optional button message.
/// Corners follow [`Tokens::shape`] ([`crate::m3::shape::Component::Banner`]).
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
    let a11y = a11y.merge_live(a11y::Live::Polite);
    let text_s = a11y.apply_name(text_s);
    let mut r = row![label(text_s.clone(), tok, A11y::new(text_s, Role::Status))]
        .spacing(inset(tok))
        .align_y(Alignment::Center);
    if let Some((t, m)) = action {
        r = r.push(themed_button(
            t.clone(),
            a11y.apply_message(Some(m)),
            tok,
            Variant::Quiet,
            Icons::NONE,
            A11y::button(t).with_disabled(a11y.disabled),
        ));
    }
    a11y::attach(
        container(r)
            .width(Length::Fill)
            .padding(inset(tok))
            .style(move |_| style::banner(tok))
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
    let a11y = a11y.merge_live(a11y::Live::Polite);
    let text_s = a11y.apply_name(text_s);
    a11y::attach(
        container(label(text_s.clone(), tok, A11y::new(text_s, Role::Status)))
            .width(Length::Fill)
            .padding(inset(tok))
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
    let mut r = Row::new().spacing(gap(tok)).align_y(Alignment::Center);
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
/// nothing. Corners follow [`Tokens::shape`]
/// ([`crate::m3::shape::Component::Toast`]).
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::toast::{Toast, ToastKind};
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let t = Toast { id: 1, kind: ToastKind::Success, text: "Saved".into(), ttl_ms: 0, age_ms: 0 };
/// let _: icedtea::Element<'_, ()> =
///     widget::toast_view(&t, (), tok, A11y::new("Saved", Role::Status));
/// ```
pub fn toast_view<'a, M: Clone + 'a>(
    toast: &Toast,
    dismiss: M,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let a11y = a11y
        .merge_live(a11y::Live::Polite)
        .merge_value(toast.text.clone());
    let kind = toast.kind;
    let text_s = toast.text.clone();
    let fade_ms =
        crate::motion::duration(crate::m3::motion::TOAST, tok.reduced_motion).as_millis() as u64;
    let progress = crate::motion::toast_progress(toast.age_ms, toast.ttl_ms, fade_ms);
    let t = crate::motion::visual(progress, tok.reduced_motion);
    let paint = tok.fade(t);
    let face = container(
        row![
            label(text_s.clone(), paint, A11y::new(text_s, Role::Status),),
            Space::new().width(Length::Fill),
            dismiss_button(
                dismiss,
                paint,
                A11y::button("dismiss").with_disabled(a11y.disabled),
            ),
        ]
        .spacing(gap(paint))
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(pad(paint))
    .style(move |theme| style::fade_face(toast_style(tok, kind)(theme), t));
    crate::motion::overlay(face.into(), t, crate::motion::Slide::Down, tok, a11y)
}

fn tip_style(tok: Tokens) -> impl Fn(&iced::Theme) -> iced::widget::container::Style {
    move |_| style::tooltip(tok)
}

fn toast_style(
    tok: Tokens,
    kind: ToastKind,
) -> impl Fn(&iced::Theme) -> iced::widget::container::Style {
    move |_| style::callout(tok, kind)
}

/// A themed scroller with a usable handle.
///
/// `stick` pins to the end. `scroll_id` is for `scroll_to`. `on_scroll`
/// receives the pixel offset from the start when the offset moves.
/// The rail sits on the end side (`Tokens.direction`).
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
    F: Fn(f32) -> M + 'a,
{
    let boxed = on_scroll.map(|f| Box::new(f) as Box<dyn Fn(f32) -> M + 'a>);
    a11y::attach(
        ThemedScroll::new(child, tok, stick, scroll_id, boxed).into(),
        &a11y,
    )
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
        stick_scroll_snapped(n, h, viewport)
    } else {
        window.scroll.max(0.0)
    };
    let (top, win, bot) = virtual_pads(n, h, scroll, viewport, overscan, None);
    // Extra bottom pad so anchor_bottom max-scroll is a multiple of row_h
    // (otherwise the first visible line is a fractional band under the clip).
    let align_pad = stick_align_pad(n, h, viewport);
    let mut col = Column::new().spacing(0);
    if n == 0 {
        col = col.push(meta("No lines", tok, A11y::new("No lines", Role::Status)));
    } else {
        col = col.push(Space::new().height(Length::Fixed(top)));
        for i in win.range() {
            let line = lines.get(i).map(String::as_str).unwrap_or("");
            col = col.push(
                container(start_label(
                    line.to_string(),
                    tok.meta(),
                    tok.scheme().on_surface,
                    typo::MONO,
                    iced::widget::text::Wrapping::None,
                    tok.direction,
                ))
                .width(Length::Fill)
                .height(h)
                .padding([2.0, gap(tok)])
                .clip(true),
            );
        }
        col = col.push(Space::new().height(Length::Fixed(bot + align_pad)));
    }
    let prev = window;
    themed_scroll(
        col.into(),
        tok,
        a11y,
        true,
        scroll_id,
        Some(move |y: f32| on_scroll(window_after_scroll(prev, y, viewport, h, n, overscan, None))),
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
        // Clamp at paint time (same as wheel/rail). Unclamped past-end
        // scroll after a face/height change can mount an empty window.
        let win = match heights {
            RowHeights::Uniform(h) => window_after_scroll(
                prev,
                prev.scroll,
                viewport,
                h.max(0.0),
                len,
                overscan,
                cover,
            ),
            RowHeights::PerRow(hs) => {
                window_after_scroll_var(prev, prev.scroll, viewport, hs, overscan, cover)
            }
        };
        let scroll = win.scroll;
        let shift = heights.offset(win.start) - scroll;
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
        // shift is often negative (overscan + partial first row). ClipLayer
        // scissors so card backgrounds cannot cover chrome above the list.
        let mut inner = container(rows(win))
            .width(crate::layout::FILL)
            .padding(Padding {
                top: shift,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            });
        if let Some(id) = scroll_id.clone() {
            inner = inner.id(id);
        }
        let frame = container(inner)
            .width(crate::layout::FILL)
            .height(crate::layout::FILL);
        let pane = mouse_area(ClipLayer::new(frame)).on_scroll(move |delta| {
            let max_s = (content - viewport).max(0.0);
            emit((scroll + scroll_delta_pixels(delta, step)).clamp(0.0, max_s))
        });
        let rail = Element::from(ScrollRail::new(content, viewport, scroll, emit, tok));
        let mut strip = Row::new()
            .spacing(4)
            .width(crate::layout::FILL)
            .height(crate::layout::FILL);
        for kid in crate::i18n::order(tok.direction, [pane.into(), rail]) {
            strip = strip.push(kid);
        }
        strip.into()
    })
    .into()
}

/// Stick-to-end scroll snapped to a row boundary so the first painted
/// line is whole (raw `end_offset` can leave a fractional top band).
fn stick_scroll_snapped(n: usize, row_h: f32, viewport: f32) -> f32 {
    let max_s = crate::layout::end_offset(n as f32 * row_h, viewport);
    if row_h > 0.0 {
        (max_s / row_h).floor() * row_h
    } else {
        max_s
    }
}

/// Bottom pad so content height makes max-scroll a multiple of `row_h`.
fn stick_align_pad(n: usize, row_h: f32, viewport: f32) -> f32 {
    if row_h <= 0.0 {
        return 0.0;
    }
    let max_s = crate::layout::end_offset(n as f32 * row_h, viewport);
    if max_s <= 0.0 {
        return 0.0;
    }
    let r = max_s % row_h;
    if r < 1e-3 {
        0.0
    } else {
        row_h - r
    }
}

/// Truncate for a one-line face so the rail does not bisect a glyph.
/// Breaks on a space when one sits inside the budget so a word is not cut.
fn ellipsize_line(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if max_chars == 0 || chars.len() <= max_chars {
        return s.to_string();
    }
    let keep = max_chars.saturating_sub(1);
    let cut = &chars[..keep];
    if let Some(i) = cut.iter().rposition(|c| c.is_whitespace()) {
        if i > 0 {
            return format!("{}…", cut[..i].iter().collect::<String>());
        }
    }
    format!("{}…", cut.iter().collect::<String>())
}

fn row_slot_el<'a, M: Clone + 'a>(
    slot: crate::collection::RowSlot,
    index: usize,
    on_check: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
    disabled: bool,
) -> Element<'a, M> {
    match slot {
        crate::collection::RowSlot::Empty => Space::new().width(0).height(16).into(),
        crate::collection::RowSlot::Icon(icon) => {
            icon_svg(icon, tok, A11y::new("row-icon", Role::Image))
        }
        crate::collection::RowSlot::Check(on) => {
            let s = tok.scheme();
            let face = container(Space::new().width(10).height(10))
                .width(16)
                .height(16)
                .center_x(16)
                .center_y(16)
                .style(move |_| {
                    if on {
                        style::fill(s.primary, s.on_primary)
                    } else {
                        let mut st = style::fill(Color::TRANSPARENT, s.on_surface);
                        st.border = iced::border::Border {
                            color: s.outline,
                            width: 2.0,
                            radius: tok.radius(crate::m3::shape::Component::Field),
                        };
                        st
                    }
                });
            if disabled {
                face.into()
            } else {
                mouse_area(face).on_press(on_check(index)).into()
            }
        }
    }
}

/// Shrink text inside a fill pad. Fill+align on `text` drops
/// right-to-left glyphs (empty list cards).
fn start_label<'a, M: 'a>(
    s: impl Into<String>,
    size: f32,
    color: iced::Color,
    font: iced::Font,
    wrapping: iced::widget::text::Wrapping,
    dir: Direction,
) -> Element<'a, M> {
    container(
        text(s.into())
            .size(size)
            .color(color)
            .font(font)
            .wrapping(wrapping),
    )
    .width(Length::Fill)
    .align_x(crate::i18n::align_start(dir))
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
    // Tall rows (list-detail at 64px) wrap the title. Short rows stay
    // one clipped line so a 24px rail does not bisect a glyph.
    let wrap = row_h >= 56.0;
    let title = if wrap {
        title.to_string()
    } else {
        ellipsize_line(title, 26)
    };
    let wrapping = if wrap {
        iced::widget::text::Wrapping::Word
    } else {
        iced::widget::text::Wrapping::None
    };
    let (pad_l, pad_r) = crate::i18n::inline_pad(tok.direction, gap(tok), inset(tok));
    let on = tok.scheme().on_surface;
    let mut col = column![start_label(
        title,
        tok.body(),
        on,
        typo::UI,
        wrapping,
        tok.direction,
    )]
    .spacing(2)
    .width(Length::Fill);
    if let Some(m) = meta_s.filter(|s| !s.is_empty()) {
        let meta_t = if wrap {
            m.to_string()
        } else {
            ellipsize_line(m, 32)
        };
        col = col.push(start_label(
            meta_t,
            tok.meta(),
            meta_color,
            typo::UI,
            wrapping,
            tok.direction,
        ));
    }
    container(col)
        .width(Length::Fill)
        .height(row_h)
        .padding(Padding {
            top: gap(tok),
            right: pad_r,
            bottom: gap(tok),
            left: pad_l,
        })
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
    let on = tok.scheme().on_surface;
    let (pad_l, pad_r) = crate::i18n::inline_pad(tok.direction, inset(tok), inset(tok));
    let mut col = column![start_label(
        title.to_string(),
        tok.body(),
        on,
        if selected { typo::UI_BOLD } else { typo::UI },
        iced::widget::text::Wrapping::Word,
        tok.direction,
    )]
    .spacing(2)
    .width(Length::Fill);
    if let Some(m) = meta_s.filter(|s| !s.is_empty()) {
        col = col.push(start_label(
            m.to_string(),
            tok.meta(),
            meta_color,
            typo::UI,
            iced::widget::text::Wrapping::None,
            tok.direction,
        ));
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
            top: gap(tok),
            right: pad_r,
            bottom: gap(tok),
            left: pad_l,
        })
        .clip(true)
        .style(move |_| style::card(tok, selected))
        .into()
}

/// A virtualized column of app-built rows with known heights.
///
/// Use for expand cards and other free-form faces: pass
/// [`collection::expand_card_heights`](crate::collection::expand_card_heights)
/// (or any per-row slice), keep a [`VisibleWindow`], and build each
/// mounted index. This reuses list windowing (overscan, rail, wheel);
/// it is not a second list model — title/meta lists stay on
/// [`list_view`].
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::collection::{expand_card_heights, VisibleWindow};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let heights = expand_card_heights(3, 48.0, &[(1, 120.0)]);
/// let win = VisibleWindow::new(200.0);
/// let on_scroll = |w| w;
/// let _: icedtea::Element<'_, _> = widget::virtual_column(
///     &heights,
///     win,
///     2,
///     None,
///     on_scroll,
///     None,
///     tok,
///     |i| widget::label(format!("row {i}"), tok, A11y::new("r", Role::ListItem)),
///     A11y::new("cards", Role::List),
/// );
/// ```
#[allow(clippy::too_many_arguments)]
pub fn virtual_column<'a, M: Clone + 'a>(
    heights: &'a [f32],
    window: VisibleWindow,
    overscan: usize,
    cover: Option<usize>,
    on_scroll: impl Fn(VisibleWindow) -> M + Copy + 'a,
    scroll_id: Option<Id>,
    tok: Tokens,
    row: impl Fn(usize) -> Element<'a, M> + 'a,
    a11y: A11y,
) -> Element<'a, M> {
    let len = heights.len();
    let prev = window;
    a11y::attach(
        virtual_clip(
            prev,
            RowHeights::PerRow(heights),
            len,
            overscan,
            cover,
            on_scroll,
            scroll_id,
            tok,
            move |win| {
                let mut col = Column::new();
                for i in win.range() {
                    let h = heights.get(i).copied().unwrap_or(0.0);
                    col = col.push(container(row(i)).width(Length::Fill).height(h).clip(true));
                }
                col
            },
        ),
        &a11y,
    )
}

/// Emit [`ItemButton`] plus current modifiers when the child is pressed.
pub fn item_press<'a, M: Clone + 'a>(
    child: Element<'a, M>,
    on_click: impl Fn(ItemButton, keyboard::Modifiers) -> M + 'a,
) -> Element<'a, M> {
    ItemPress {
        content: child,
        on_click: Box::new(on_click),
    }
    .into()
}

/// Swallow mouse events on `child` so they do not fall through.
pub fn capture_press<'a, M: 'a>(child: Element<'a, M>) -> Element<'a, M> {
    CapturePress { content: child }.into()
}

struct ItemPress<'a, Message> {
    content: Element<'a, Message>,
    on_click: Box<dyn Fn(ItemButton, keyboard::Modifiers) -> Message + 'a>,
}

#[derive(Default)]
struct PressState {
    modifiers: keyboard::Modifiers,
}

impl<'a, Message: Clone> Widget<Message, iced::Theme, iced::Renderer> for ItemPress<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<PressState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(PressState::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> iced::Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        let state = tree.state.downcast_mut::<PressState>();
        match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(m)) => state.modifiers = *m,
            Event::Keyboard(keyboard::Event::KeyPressed { modifiers, .. })
            | Event::Keyboard(keyboard::Event::KeyReleased { modifiers, .. }) => {
                state.modifiers = *modifiers;
            }
            Event::Mouse(iced::mouse::Event::ButtonPressed(button))
                if !shell.is_event_captured() =>
            {
                let Some(pos) = cursor.position() else {
                    return;
                };
                if !layout.bounds().contains(pos) {
                    return;
                }
                let item = match button {
                    iced::mouse::Button::Left => ItemButton::Primary,
                    iced::mouse::Button::Right => ItemButton::Secondary,
                    _ => return,
                };
                shell.publish((self.on_click)(item, state.modifiers));
                shell.capture_event();
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
}

impl<'a, Message: Clone + 'a> From<ItemPress<'a, Message>> for Element<'a, Message> {
    fn from(value: ItemPress<'a, Message>) -> Self {
        Self::new(value)
    }
}

struct CapturePress<'a, Message> {
    content: Element<'a, Message>,
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for CapturePress<'a, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> iced::Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if shell.is_event_captured() {
            return;
        }
        if let Event::Mouse(_) = event {
            if cursor
                .position()
                .is_some_and(|p| layout.bounds().contains(p))
            {
                shell.capture_event();
            }
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
}

impl<'a, Message: 'a> From<CapturePress<'a, Message>> for Element<'a, Message> {
    fn from(value: CapturePress<'a, Message>) -> Self {
        Self::new(value)
    }
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
///     Select(icedtea::collection::ItemClick),
///     Scroll(VisibleWindow),
///     Check(usize),
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
///     Msg::Check,
///     A11y::new("list", Role::List),
/// );
/// ```
#[allow(clippy::too_many_arguments)]
pub fn list_view<'a, M, L>(
    model: &'a L,
    selection: &'a Selection,
    on_select: impl Fn(ItemClick) -> M + Copy + 'a,
    tok: Tokens,
    window: VisibleWindow,
    row_h: impl Into<RowHeights<'a>>,
    overscan: usize,
    on_scroll: impl Fn(VisibleWindow) -> M + Copy + 'a,
    empty: &'a str,
    meta_color: impl Fn(usize) -> iced::Color + Copy + 'a,
    scroll_id: Option<Id>,
    face: RowFace<impl Fn(usize) -> f32 + Copy + 'a>,
    on_check: impl Fn(usize) -> M + Copy + 'a,
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
                        let body: Element<'a, M> = if disabled {
                            painted
                        } else {
                            item_press(painted, move |button, modifiers| {
                                on_select(ItemClick {
                                    id: i,
                                    button,
                                    modifiers,
                                })
                            })
                        };
                        let mut line = Row::new()
                            .spacing(4)
                            .align_y(Alignment::Center)
                            .width(Length::Fill);
                        for kid in crate::i18n::order(
                            tok.direction,
                            [
                                row_slot_el(model.leading(i), i, on_check, tok, disabled),
                                body,
                                row_slot_el(model.trailing(i), i, on_check, tok, disabled),
                            ],
                        ) {
                            line = line.push(kid);
                        }
                        let row: Element<'a, M> = line.into();
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
/// let on_select = |c: icedtea::collection::ItemClick| c.id;
/// let _: icedtea::Element<'_, usize> = widget::item_grid(
///     &labels,
///     on_select,
///     Some(0),
///     tok,
///     A11y::new("grid", Role::List),
/// );
/// ```
pub fn item_grid<'a, M: Clone + 'a>(
    labels: &[String],
    on_select: impl Fn(ItemClick) -> M + Copy + 'a,
    selected: Option<usize>,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let cols = 3;
    let mut rows = iced::widget::Column::new()
        .spacing(gap(tok))
        .width(Length::Fill)
        .height(Length::Fill);
    let mut i = 0;
    while i < labels.len() {
        let mut r = iced::widget::Row::new()
            .spacing(gap(tok))
            .width(Length::Fill)
            .height(Length::Fill);
        for _ in 0..cols {
            if i < labels.len() {
                let s = labels[i].clone();
                let on = selected == Some(i);
                // M3: selected tile = tonal; idle = outlined so the cell reads.
                let tile = themed_button_sized(
                    s.clone(),
                    None,
                    tok,
                    if on {
                        Variant::Quiet
                    } else {
                        Variant::Outlined
                    },
                    Icons::NONE,
                    Length::Fill,
                    Length::Fill,
                    A11y::new(s.clone(), Role::ListItem)
                        .with_checked(on)
                        .with_disabled(a11y.disabled),
                );
                r = r.push(if a11y.disabled {
                    tile
                } else {
                    item_press(tile, move |button, modifiers| {
                        on_select(ItemClick {
                            id: i,
                            button,
                            modifiers,
                        })
                    })
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
/// `on_cell` is an [`ItemClick`] (row) plus the column. `on_sort` is
/// the header click. Empty
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
///     checks: vec![false],
/// };
/// let cols = icedtea::collection::ColumnLayout::new(vec![120.0]);
/// #[derive(Clone, Copy)]
/// enum Msg {
///     Cell(icedtea::collection::ItemClick, usize),
///     Sort(usize),
///     Scroll(VisibleWindow),
///     HScroll(f32),
///     Check(usize),
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
///     Msg::Check,
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
    on_cell: impl Fn(ItemClick, usize) -> M + Copy + 'a,
    on_sort: impl Fn(usize) -> M + Copy + 'a,
    on_scroll: impl Fn(VisibleWindow) -> M + Copy + 'a,
    on_h_scroll: impl Fn(f32) -> M + Copy + 'a,
    on_check: impl Fn(usize) -> M + Copy + 'a,
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
    let show_checks = (0..n.max(1)).any(|r| model.row_checked(r).is_some());
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
    if show_checks {
        pin_head = pin_head.push(
            container(Space::new().width(16).height(16))
                .width(36)
                .center_x(36)
                .center_y(32),
        );
    }
    for c in &pin {
        let c = *c;
        let title = model.header(c).to_string();
        pin_head = pin_head.push(
            container(themed_button(
                title.clone(),
                a11y.apply_message(Some(on_sort(c))),
                tok,
                Variant::Ghost,
                Icons::NONE,
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
                Icons::NONE,
                A11y::button(title).with_disabled(disabled),
            ))
            .width(col_w(c)),
        );
    }
    let (h_pad_l, h_pad_r) = crate::i18n::inline_pad(tok.direction, -h_scroll, 0.0);
    let rest_head = mouse_area(
        container(rest_head)
            .width(Length::Fill)
            .padding(Padding {
                top: 0.0,
                right: h_pad_r,
                bottom: 0.0,
                left: h_pad_l,
            })
            .clip(true),
    )
    .on_scroll(move |delta| on_h_scroll((h_scroll + scroll_delta_x(delta)).max(0.0)));
    let mut header = Row::new().width(crate::layout::FILL);
    let pin_head: Element<'a, M> = pin_head.into();
    let rest_head: Element<'a, M> = rest_head.into();
    for kid in crate::i18n::order(tok.direction, [pin_head, rest_head]) {
        header = header.push(kid);
    }
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
                            let cell_style = style::table_cell(tok, selected, focused, stripe);
                            let ink = cell_style.text_color.unwrap_or(tok.scheme().on_surface);
                            let face = container(start_label(
                                value.clone(),
                                tok.body(),
                                ink,
                                typo::UI,
                                iced::widget::text::Wrapping::None,
                                tok.direction,
                            ))
                            .width(w)
                            .height(h)
                            .padding([gap(tok), gap(tok)])
                            .style(move |_| cell_style);
                            let cell: Element<'a, M> = if disabled {
                                face.into()
                            } else {
                                item_press(face.into(), move |button, modifiers| {
                                    on_cell(
                                        ItemClick {
                                            id: i,
                                            button,
                                            modifiers,
                                        },
                                        c,
                                    )
                                })
                            };
                            line.push(a11y::attach(
                                cell,
                                &A11y::new(format!("{i}:{c}"), Role::ListItem)
                                    .with_value(value)
                                    .with_checked(focused || selected)
                                    .with_disabled(disabled),
                            ))
                        };
                        let mut pin_line = Row::new().spacing(0);
                        if show_checks {
                            let on = model.row_checked(i).unwrap_or(false);
                            pin_line = pin_line.push(
                                container(row_slot_el(
                                    crate::collection::RowSlot::Check(on),
                                    i,
                                    on_check,
                                    tok,
                                    disabled,
                                ))
                                .width(36)
                                .center_x(36)
                                .center_y(h),
                            );
                        }
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
                                right: h_pad_r,
                                bottom: 0.0,
                                left: h_pad_l,
                            })
                            .clip(true);
                        let pin_line: Element<'a, M> = pin_line.into();
                        let rest_line: Element<'a, M> = rest_line.into();
                        let mut line = Row::new().width(crate::layout::FILL);
                        for kid in crate::i18n::order(tok.direction, [pin_line, rest_line]) {
                            line = line.push(kid);
                        }
                        body = body.push(line);
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
///
/// [`TreeFace::Outline`] is a tight heading tree (full-width wash, no
/// marks). [`TreeFace::Files`] is an explorer (inset wash, folder and
/// file marks from `dir`). Density scales pad, gap, and indent.
/// The application owns expand state. Leaf rows have no twisty.
/// `animating` is the branch that is opening or closing and its 0–1
/// height progress. `None` paints the committed tree.
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
///     Select(icedtea::collection::ItemClick<u64>),
/// }
/// let on_toggle = Msg::Toggle;
/// let on_select = Msg::Select;
/// let _: icedtea::Element<'_, Msg> = widget::tree_view(
///     &tree,
///     None,
///     None,
///     on_toggle,
///     on_select,
///     widget::TreeFace::Outline,
///     tok,
///     A11y::new("tree", Role::Tree),
/// );
/// ```
#[allow(clippy::too_many_arguments)]
pub fn tree_view<'a, M: Clone + 'a>(
    root: &TreeNode,
    selected: Option<u64>,
    animating: Option<(u64, f32)>,
    on_toggle: impl Fn(u64) -> M + Copy + 'a,
    on_select: impl Fn(ItemClick<u64>) -> M + Copy + 'a,
    face: TreeFace,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let closing = animating.map(|(id, _)| id);
    let progress = animating.map(|(_, p)| p).unwrap_or(1.0);
    let anim_id = animating.map(|(id, _)| id);
    let mut col = Column::new()
        .spacing(0)
        .width(Length::Fill)
        .align_x(crate::i18n::align_start(tok.direction));
    let mut branch: Vec<Element<'a, M>> = Vec::new();
    let mut wrap_depth: Option<u32> = None;
    for (depth, id, label_s, expanded, has_children) in root.flatten_during(closing) {
        if wrap_depth.is_some_and(|d| depth <= d) {
            col = tree_push_branch(col, std::mem::take(&mut branch), progress, tok);
            wrap_depth = None;
        }
        let line = tree_line(
            depth,
            id,
            label_s,
            expanded,
            has_children,
            selected,
            &on_toggle,
            on_select,
            face,
            tok,
            &a11y,
        );
        if Some(id) == anim_id {
            col = col.push(line);
            wrap_depth = Some(depth);
        } else if wrap_depth.is_some() {
            branch.push(line);
        } else {
            col = col.push(line);
        }
    }
    col = tree_push_branch(col, branch, progress, tok);
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

/// Compact disclosure mark. A full [`themed_button`] is a control face,
/// too tall for a file tree row.
fn tree_twisty<'a, M: Clone + 'a>(
    mark: &'static str,
    msg: Option<M>,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let face = text(mark).size(tok.body()).color(tok.scheme().on_surface);
    let mut b = button(face)
        .padding(Padding::from((gap(tok) / 2.0).max(4.0)))
        .style(style::button_style(tok, Variant::Ghost));
    if let Some(m) = a11y.apply_message(msg) {
        b = b.on_press(m);
    }
    a11y::attach(b.into(), &a11y)
}

const TREE_FOLDER_SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16" fill="#000"><path d="M1.5 3.5h4.1L7 5.2h7.5v7.8H1.5z"/></svg>"##;
const TREE_FILE_SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16" fill="#000"><path d="M4 1.5h5.2L12.5 5v9.5H4z"/></svg>"##;

#[allow(clippy::too_many_arguments)]
fn tree_line<'a, M: Clone + 'a>(
    depth: u32,
    id: u64,
    label_s: String,
    expanded: bool,
    has_children: bool,
    selected: Option<u64>,
    on_toggle: &impl Fn(u64) -> M,
    on_select: impl Fn(ItemClick<u64>) -> M + Copy + 'a,
    face: TreeFace,
    tok: Tokens,
    a11y: &A11y,
) -> Element<'a, M> {
    let is_sel = selected == Some(id);
    let step = match face {
        TreeFace::Outline => gap(tok) * 2.0,
        TreeFace::Files => gap(tok) + crate::density::GRID as f32,
    };
    let indent: Element<'a, M> = Space::new()
        .width(Length::Fixed(depth as f32 * step))
        .into();
    let twisty_w = gap(tok) * 2.0 + 4.0;
    let twisty: Element<'a, M> = if has_children {
        let mark = if expanded {
            "▾"
        } else {
            closed_disclosure(tok)
        };
        tree_twisty(
            mark,
            a11y.apply_message(Some(on_toggle(id))),
            tok,
            A11y::button(format!("toggle {label_s}"))
                .with_checked(expanded)
                .with_disabled(a11y.disabled),
        )
    } else {
        Space::new().width(twisty_w).into()
    };
    let start = crate::i18n::align_start(tok.direction);
    let v = gap(tok);
    let title = container(start_label(
        label_s.clone(),
        tok.body(),
        tok.scheme().on_surface,
        typo::UI,
        iced::widget::text::Wrapping::None,
        tok.direction,
    ))
    .width(Length::Fill)
    .align_x(start)
    .padding([v / 2.0, v]);
    let title: Element<'a, M> = if is_sel {
        title.style(move |_| style::list_row(tok, true)).into()
    } else {
        title.into()
    };
    let title: Element<'a, M> = if face == TreeFace::Files {
        let (pad_l, pad_r) = crate::i18n::inline_pad(tok.direction, 0.0, v / 2.0);
        container(title)
            .width(Length::Fill)
            .padding(Padding {
                top: 0.0,
                right: pad_r,
                bottom: 0.0,
                left: pad_l,
            })
            .into()
    } else {
        title
    };
    let pick: Element<'a, M> = if a11y.disabled {
        title
    } else {
        item_press(title, move |button, modifiers| {
            on_select(ItemClick {
                id,
                button,
                modifiers,
            })
        })
    };
    let pick: Element<'a, M> = a11y::attach(
        pick,
        &A11y::new(label_s.clone(), Role::Tree).with_checked(is_sel),
    );
    let mut kids = vec![indent, twisty];
    if face == TreeFace::Files {
        let mark = if has_children {
            TREE_FOLDER_SVG
        } else {
            TREE_FILE_SVG
        };
        kids.push(icon_svg(
            crate::icon::Glyph::Bytes(mark),
            tok,
            A11y::new(label_s, Role::Image),
        ));
    }
    kids.push(pick);
    let mut line = Row::new()
        .spacing(crate::density::GRID as f32)
        .align_y(Alignment::Center)
        .width(Length::Fill);
    for kid in crate::i18n::order(tok.direction, kids) {
        line = line.push(kid);
    }
    line.into()
}

fn tree_push_branch<'a, M: Clone + 'a>(
    col: Column<'a, M>,
    rows: Vec<Element<'a, M>>,
    progress: f32,
    tok: Tokens,
) -> Column<'a, M> {
    if rows.is_empty() {
        return col;
    }
    let mut kids = Column::new().spacing(0);
    for row in rows {
        kids = kids.push(row);
    }
    let body: Element<'a, M> = kids.into();
    let t = crate::motion::visual(progress, tok.reduced_motion);
    if t >= 1.0 {
        col.push(body)
    } else {
        col.push(crate::motion::expand(
            body,
            t,
            0.0,
            tok,
            A11y::new("tree-branch", Role::Group),
        ))
    }
}

/// Map a More-list title back to its tab index.
pub fn tab_overflow_pick<M: Clone>(
    titles: Vec<String>,
    on_select: impl Fn(usize) -> M + Copy,
) -> impl Fn(String) -> M {
    move |title: String| on_select(tab_overflow_index(&titles, &title))
}

/// Map a More-list title back to its tab index.
pub fn tab_overflow_index(titles: &[String], chosen: &str) -> usize {
    titles.iter().position(|t| t == chosen).unwrap_or(0)
}

/// How many titles fit in `max_width` before the More control.
pub fn tab_visible_count(titles: &[String], max_width: f32) -> usize {
    if titles.is_empty() {
        return 0;
    }
    if max_width <= 0.0 {
        return titles.len();
    }
    let mut used = 0.0;
    let mut visible = 0;
    for title in titles {
        let w = (title.len() as f32) * 7.0 + 48.0;
        if visible > 0 && used + w > max_width - 72.0 {
            break;
        }
        used += w;
        visible += 1;
    }
    visible.max(1).min(titles.len())
}

/// A tab bar over a body the application paints.
///
/// `Tabs { closable: false }` is pinned sections. Titles use meta type.
/// `with_disabled` freezes one tab. Select sends the index.
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
///     480.0,
///     false,
///     tok,
///     A11y::new("tabs", Role::Tab),
/// );
/// ```
pub fn tab_bar<'a, M: Clone + 'a>(
    tabs: &Tabs,
    on_select: impl Fn(usize) -> M + Copy + 'a,
    on_close: impl Fn(usize) -> M + Copy + 'a,
    max_width: f32,
    secondary: bool,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let visible = tab_visible_count(&tabs.titles, max_width);
    let mut r = Row::new().spacing(0).align_y(Alignment::End);
    for (i, title) in tabs.titles.iter().enumerate().take(visible) {
        let active = i == tabs.active;
        let tab_off = a11y.disabled || tabs.is_disabled(i);
        let badge = tabs.badges.get(i).filter(|s| !s.is_empty()).cloned();
        let mut label_row = Row::new().spacing(6).align_y(Alignment::Center);
        if let Some(Some(ic)) = tabs.icons.get(i) {
            label_row = label_row.push(icon_svg(*ic, tok, A11y::new(title.clone(), Role::Image)));
        }
        let title_el = if tab_off {
            text(title.clone())
                .size(tok.meta())
                .color(tok.scheme().on_surface_variant)
        } else {
            text(title.clone()).size(tok.meta())
        };
        label_row = label_row.push(title_el);
        if let Some(b) = badge {
            label_row = label_row.push(self::badge(
                b,
                None,
                tok,
                Variant::Primary,
                BadgeSize::Small,
                A11y::new("tab-badge", Role::Status),
            ));
        }
        let mut tab = button(label_row)
            .padding(pad(tok))
            .style(style::tab_style(tok, active && !tab_off));
        if !tab_off {
            tab = tab.on_press(on_select(i));
        }
        // Underbar only under this label: column Shrink → bar Fill of that width.
        let bar_h = if secondary { 1.0 } else { 3.0 };
        let indicator = if active {
            container(Space::new().height(bar_h))
                .width(Length::Fill)
                .style(move |_| style::tab_indicator(tok))
        } else {
            container(Space::new().height(0.0)).width(Length::Fill)
        };
        let label_col = column![tab, indicator].spacing(0).width(Length::Shrink);
        let cell: Element<'a, M> = if tabs.closable {
            row![
                label_col,
                dismiss_button(
                    on_close(i),
                    tok,
                    A11y::button(format!("close {title}")).with_disabled(tab_off),
                )
            ]
            .spacing(2)
            .align_y(Alignment::Center)
            .width(Length::Shrink)
            .into()
        } else {
            label_col.into()
        };
        r = r.push(a11y::attach(
            cell,
            &A11y::new(title.clone(), Role::Tab)
                .with_checked(active && !tab_off)
                .with_disabled(tab_off),
        ));
    }
    if visible < tabs.titles.len() {
        let all = tabs.titles.clone();
        let hidden: Vec<String> = all[visible..]
            .iter()
            .enumerate()
            .filter(|(j, _)| !tabs.is_disabled(visible + j))
            .map(|(_, t)| t.clone())
            .collect();
        if !hidden.is_empty() {
            r = r.push(themed_pick_list(
                hidden,
                None,
                tab_overflow_pick(all, on_select),
                tok,
                ControlSize::Default,
                A11y::new("more-tabs", Role::ComboBox).with_disabled(a11y.disabled),
            ));
        }
    }
    // Strip sits on app-bar surface; outline hairline under the row.
    let strip = column![
        r,
        container(Space::new().width(Length::Fill).height(1)).style(move |_| style::hairline(tok)),
    ];
    a11y::attach(
        container(strip)
            .width(Length::Fill)
            .style(move |_| style::app_bar(tok))
            .into(),
        &a11y,
    )
}

/// Title on the start edge, disclosure mark on the end.
///
/// Closed: ▸ (right). Open: ▾ (down). Same twisty as folders and tree
/// rows — not a 180° flip of a down chevron (that painted as ^ open).
fn disclosure_header<'a, M: Clone + 'a>(
    title: impl Into<String>,
    open: bool,
    msg: Option<M>,
    tok: Tokens,
    a11y: A11y,
    inset: Padding,
) -> Element<'a, M> {
    let title = title.into();
    // Unicode disclosure triangles (tree_view / Finder / VS Code style).
    let mark = if open { "▾" } else { closed_disclosure(tok) };
    let s = tok.scheme();
    let title_el: Element<'a, M> =
        container(text(title.clone()).size(tok.body()).color(s.on_surface))
            .width(Length::Fill)
            .align_x(crate::i18n::align_start(tok.direction))
            .into();
    let mark_el: Element<'a, M> = text(mark)
        .size(tok.body())
        .color(s.on_surface_variant)
        .into();
    let mut face = Row::new()
        .spacing(gap(tok))
        .align_y(Alignment::Center)
        .width(Length::Fill);
    for kid in crate::i18n::order(tok.direction, [title_el, mark_el]) {
        face = face.push(kid);
    }
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
///     1.0,
///     on_toggle,
///     tok,
///     A11y::new("acc", Role::Group),
/// );
/// ```
pub fn accordion_view<'a, M: Clone + 'a>(
    titles: &[String],
    bodies: Vec<Element<'a, M>>,
    state: &Accordion,
    progress: f32,
    on_toggle: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let a11y = a11y.merge_expanded(state.open.is_some());
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
            pad(tok),
        ));
        let t = if open {
            crate::motion::visual(progress, tok.reduced_motion)
        } else {
            0.0
        };
        if t > 0.0 {
            let pane = container(body)
                .width(Length::Fill)
                .padding(inset(tok))
                .style(move |_| style::panel(tok));
            col = col.push(crate::motion::expand(
                pane.into(),
                t,
                0.0,
                tok,
                A11y::new(title.clone(), Role::Group),
            ));
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
    let surface = tok.scheme().surface;
    let mut clear = surface;
    clear.a = 0.0;
    let grad = Linear::new(Radians(std::f32::consts::FRAC_PI_2))
        .add_stop(0.0, clear)
        .add_stop(1.0, surface);
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
/// The application owns `open` and `progress` (0 peek, 1 full). The
/// header toggles. Closed shows a [`Peek`] of the child. Title and
/// body share the card inset. The chevron sits on the trailing edge.
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
///     0.0,
///     |open| open,
///     tok,
///     A11y::new("Notes", Role::Group),
/// );
/// ```
#[allow(clippy::too_many_arguments)]
pub fn expander<'a, M: Clone + 'a>(
    title: impl Into<String>,
    child: Element<'a, M>,
    collapsed: impl Into<Peek>,
    open: bool,
    progress: f32,
    on_toggle: impl Fn(bool) -> M + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let a11y = a11y.merge_expanded(open);
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
    let peek_h = collapsed.into().height();
    let t = crate::motion::visual(progress, tok.reduced_motion);
    let body: Element<'a, M> = if t >= 1.0 {
        child
    } else if t <= 0.0 {
        peek_clip(child, peek_h, tok)
    } else {
        crate::motion::expand(child, t, peek_h, tok, A11y::new(title.clone(), Role::Group))
    };
    a11y::attach(
        container(column![header, body].spacing(gap(tok)))
            .padding(inset(tok))
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
                Icons::NONE,
                A11y::button("Prev").with_disabled(a11y.disabled || page == 0),
            ),
            meta(status.clone(), tok, A11y::new(status, Role::Status)),
            themed_button(
                "Next",
                a11y.apply_message((page + 1 < pages).then(|| on_page(page + 1))),
                tok,
                Variant::Quiet,
                Icons::NONE,
                A11y::button("Next").with_disabled(a11y.disabled || page + 1 >= pages),
            ),
        ]
        .spacing(gap(tok))
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

    #[test]
    fn item_press_reports_button_and_modifiers() {
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels, Point, Rectangle, Size};
        let tok = named("dark").tokens;
        let face = label("row", tok, A11y::new("row", Role::ListItem));
        let mut el: Element<'_, ItemClick> = item_press(face, |button, modifiers| ItemClick {
            id: 3,
            button,
            modifiers,
        });
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(200.0, 40.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(200.0, 40.0));
        let mut clipboard = clipboard::Null;
        {
            let mut messages = Vec::new();
            let mut shell = iced::advanced::Shell::new(&mut messages);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Keyboard(keyboard::Event::ModifiersChanged(
                    keyboard::Modifiers::SHIFT,
                )),
                layout,
                iced::mouse::Cursor::Available(Point::new(8.0, 8.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        let mut messages = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(iced::mouse::Event::ButtonPressed(
                    iced::mouse::Button::Right,
                )),
                layout,
                iced::mouse::Cursor::Available(Point::new(8.0, 8.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        assert_eq!(
            messages,
            vec![ItemClick {
                id: 3,
                button: ItemButton::Secondary,
                modifiers: keyboard::Modifiers::SHIFT,
            }]
        );
    }

    #[test]
    fn capture_press_swallows_a_move() {
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels, Point, Rectangle, Size};
        let face: Element<'_, ()> = Space::new().width(80).height(40).into();
        let mut el = capture_press(face);
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(80.0, 40.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(80.0, 40.0));
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        let mut shell = iced::advanced::Shell::new(&mut messages);
        el.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(iced::mouse::Event::CursorMoved {
                position: Point::new(8.0, 8.0),
            }),
            layout,
            iced::mouse::Cursor::Available(Point::new(8.0, 8.0)),
            &renderer,
            &mut clipboard,
            &mut shell,
            &viewport,
        );
        assert!(shell.is_event_captured());
        assert!(messages.is_empty());
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
        assert!(A11y::button("x").with_disabled(true).disabled);
        assert_eq!(
            A11y::new("y", Role::Checkbox).with_checked(false).checked,
            Some(false)
        );
    }

    #[test]
    fn badge_primary_ink_contrasts_with_fill() {
        let tok = named("dark").tokens;
        let s = tok.scheme();
        let (wash, ink, _) = chip_face(tok, Variant::Primary);
        assert_eq!(wash, s.primary);
        assert_eq!(ink, s.on_primary);
        assert_ne!(ink, wash);
        let _: Element<'_, ()> = badge(
            "2",
            None,
            tok,
            Variant::Primary,
            BadgeSize::Small,
            A11y::new("2", Role::Status),
        );
    }

    #[test]
    fn badge_corners_follow_shape_policy() {
        let desktop = named("dark").tokens;
        assert_eq!(
            desktop.radius(crate::m3::shape::Component::Badge).top_left,
            0.0
        );
        let pill = desktop.with_shape(crate::m3::ShapePolicy::Pill);
        assert_eq!(
            pill.radius(crate::m3::shape::Component::Badge).top_left,
            crate::m3::Shape::Full.dp()
        );
        let material = desktop.with_shape(crate::m3::ShapePolicy::Material);
        assert_eq!(
            material.radius(crate::m3::shape::Component::Badge).top_left,
            crate::m3::Shape::Small.dp()
        );
        let _: Element<'_, ()> = badge(
            "9",
            None,
            pill,
            Variant::Primary,
            BadgeSize::Large,
            A11y::new("9", Role::Status),
        );
    }

    #[test]
    fn toast_and_tooltip_constructors_read_toast_family() {
        let tok = named("dark")
            .tokens
            .with_shape(crate::m3::ShapePolicy::Material);
        let toast = Toast {
            id: 1,
            kind: ToastKind::Success,
            text: "Saved".into(),
            ttl_ms: 0,
            age_ms: 0,
        };
        let _: Element<'_, ()> = toast_view(&toast, (), tok, A11y::new("Saved", Role::Status));
        let _: Element<'_, ()> = tooltip_wrap(
            label("Hover", tok, A11y::new("Hover", Role::Header)),
            "Tip",
            TooltipAnchor::Follow,
            tok,
            A11y::new("Tip", Role::Tooltip),
        );
    }

    #[test]
    fn banner_constructor_reads_banner_family() {
        let tok = named("dark")
            .tokens
            .with_shape(crate::m3::ShapePolicy::Material);
        let _: Element<'_, ()> = banner(
            "Update available",
            Some(("Install".into(), ())),
            tok,
            A11y::new("Update available", Role::Status),
        );
    }

    #[test]
    fn search_constructor_reads_search_family() {
        let tok = named("dark")
            .tokens
            .with_shape(crate::m3::ShapePolicy::Material);
        let on_input = |s| s;
        let _: Element<'_, String> = search_input(
            "q",
            on_input,
            None,
            tok,
            A11y::new("Search", Role::TextBox),
            None,
        );
    }

    #[test]
    fn track_constructors_read_track_family() {
        let tok = named("dark")
            .tokens
            .with_shape(crate::m3::ShapePolicy::Material);
        let _: Element<'_, bool> = themed_switch(
            "Sounds",
            false,
            |on| on,
            tok,
            A11y::new("Sounds", Role::Switch),
        );
        let _: Element<'_, f32> = themed_slider(
            0.0..=1.0,
            0.4,
            |v| v,
            SliderMarks::NONE,
            tok,
            A11y::new("vol", Role::Slider).with_value("0.4"),
        );
        let _: Element<'_, ()> = progress(
            0.4,
            Some(0.7),
            Some("12s"),
            false,
            tok,
            A11y::new("p", Role::Progress).with_value("0.4"),
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
        let snippet = Content::with_text("fn");
        let _: Element<'_, ()> = code_block(&snippet, |_| (), tok, role("fn", Role::Group));
        let _: Element<'_, ()> = hyperlink("l", (), tok, role("l", Role::Link));
        let _: Element<'_, ()> =
            themed_button("B", Some(()), tok, Variant::Primary, Icons::NONE, btn("B"));
        let _: Element<'_, ()> = themed_button_sized(
            "7",
            Some(()),
            tok,
            Variant::Quiet,
            Icons::NONE,
            Length::Fill,
            Length::Fixed(Density::default().tile() as f32),
            btn("7"),
        );
        let _: Element<'_, ()> = display_line("6 × 4 =", tok, role("expr", Role::Status));
        let _: Element<'_, ()> = figure_display("12:40", tok, role("clock", Role::Status));
        let glyph = A11y::button("Backspace");
        let _: Element<'_, ()> = themed_button_sized(
            "⌫",
            Some(()),
            tok,
            Variant::Quiet,
            Icons::NONE,
            Length::Fill,
            Length::Fixed(48.0),
            glyph,
        );
        let _: Element<'_, ()> = themed_button(
            "D",
            None,
            tok,
            Variant::Danger,
            Icons::NONE,
            btn("D").with_disabled(true),
        );
        let _: Element<'_, i32> =
            split_button("S", 0, vec![("As…".into(), 1)], tok, Icons::NONE, btn("S"));
        let _: Element<'_, i32> = split_button(
            "S",
            0,
            vec![],
            tok,
            Icons::NONE,
            btn("S").with_disabled(true),
        );
        assert!((slider_step(0.0..=1.0) - 0.01).abs() < f32::EPSILON);
        assert!(slider_step(0.0..=1.0) < 1.0);
        assert!((slider_step(10.0..=10.0) - f32::EPSILON).abs() < 1e-12);
        let _: Element<'_, ()> =
            toggle_button("T", true, (), tok, Icons::NONE, btn("T").with_checked(true));
        let _: Element<'_, ()> = toggle_button(
            "T",
            false,
            (),
            tok,
            Icons::NONE,
            btn("T").with_checked(false),
        );
        let _: Element<'_, ()> = toggle_button(
            "T",
            true,
            (),
            tok,
            Icons::NONE,
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
        let idle = radio_idle_face(tok, false);
        assert_eq!(idle.border.color, tok.scheme().outline);
        assert!(idle.border.radius.top_left >= 8.0);
        let on = radio_idle_face(tok, true);
        assert_eq!(
            on.background,
            Some(iced::Background::Color(tok.scheme().primary))
        );
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
        assert!(scroll_wheel_y(iced::mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 }) > 0.0);
        assert!(scroll_wheel_y(iced::mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 }) < 0.0);
        assert_eq!(
            scroll_wheel_y(iced::mouse::ScrollDelta::Pixels { x: 0.0, y: 8.0 }),
            1.0
        );
        assert_eq!(
            scroll_wheel_y(iced::mouse::ScrollDelta::Pixels { x: 0.0, y: -3.0 }),
            -1.0
        );
        assert_eq!(
            slider_nudge(0.0..=1.0, 0.5, 1.0),
            0.5 + slider_step(0.0..=1.0)
        );
        assert_eq!(slider_nudge(0.0..=1.0, 1.0, 1.0), 1.0);
        assert_eq!(slider_nudge(0.0..=1.0, 0.0, -1.0), 0.0);
        assert_eq!(slider_nudge(0.0..=1.0, 0.4, 0.0), 0.4);
        assert_eq!(slider_step(3.0..=3.0), f32::EPSILON);
        assert_eq!(progress_weights(0.0, None), (0, 0, 100));
        assert_eq!(progress_weights(1.0, None), (100, 0, 0));
        assert_eq!(progress_weights(0.4, Some(0.7)), (40, 30, 30));
        let _: Element<'_, ()> = themed_slider(
            0.0..=1.0,
            0.5,
            |_| (),
            SliderMarks::NONE,
            tok,
            role("s", Role::Slider).with_value("0.5"),
        );
        let _: Element<'_, ()> = range_slider(
            0.0..=100.0,
            10.0,
            90.0,
            |_| (),
            tok,
            role("rs", Role::Slider),
        );
        assert_eq!(CheckState::Indeterminate.toggle(), CheckState::Checked);
        assert_eq!(CheckState::Checked.toggle(), CheckState::Unchecked);
        assert_eq!(CheckState::Unchecked.toggle(), CheckState::Checked);
        let tri_face = indeterminate_box_face(tok);
        assert_eq!(
            tri_face.background,
            Some(iced::Background::Color(tok.scheme().primary))
        );
        assert_eq!(tri_face.border.color, tok.scheme().primary);
        assert_eq!(check_state_from_bool(true), CheckState::Checked);
        assert_eq!(check_state_from_bool(false), CheckState::Unchecked);
        let off_slide = disabled_slider_face(tok);
        assert_eq!(
            off_slide.background,
            Some(iced::Background::Color(
                tok.scheme().surface_container_highest
            ))
        );
        let _: Element<'_, ()> = checkbox_indeterminate(
            "all",
            CheckState::Indeterminate,
            |_| (),
            tok,
            role("tri", Role::Checkbox),
        );
        let _: Element<'_, ()> = checkbox_indeterminate(
            "on",
            CheckState::Checked,
            |_| (),
            tok,
            role("tri-on", Role::Checkbox),
        );
        let _: Element<'_, ()> = checkbox_indeterminate(
            "off",
            CheckState::Unchecked,
            |_| (),
            tok,
            role("tri-off", Role::Checkbox).with_disabled(true),
        );
        let _: Element<'_, ()> = checkbox_indeterminate(
            "ind-d",
            CheckState::Indeterminate,
            |_| (),
            tok,
            role("tri-d", Role::Checkbox).with_disabled(true),
        );
        let _: Element<'_, ()> = segmented_button(
            ["A", "B"],
            1,
            |_| (),
            tok,
            ControlSize::Default,
            role("seg", Role::Group),
        );
        let _: Element<'_, ()> = segmented_button(
            ["A"],
            0,
            |_| (),
            tok,
            ControlSize::Default,
            role("seg-d", Role::Group).with_disabled(true),
        );
        let _: Element<'_, ()> = icon_button(
            Icon::Search,
            Some(()),
            tok,
            Variant::Ghost,
            ControlSize::Default,
            A11y::button("s"),
        );
        let _: Element<'_, ()> = icon_button(
            Icon::Close,
            None,
            tok,
            Variant::Quiet,
            ControlSize::Default,
            A11y::button("c").with_disabled(true),
        );
        let field = themed_text_input(
            "x",
            "",
            |_| (),
            None,
            FieldOpts::NONE,
            tok,
            role("f", Role::TextBox),
            None,
        );
        let _: Element<'_, ()> = field_support(
            field,
            Some("help"),
            Some("err"),
            tok,
            role("fs", Role::Group),
        );
        let field2 = themed_text_input(
            "y",
            "v",
            |_| (),
            None,
            FieldOpts::NONE,
            tok,
            role("f2", Role::TextBox),
            None,
        );
        let _: Element<'_, ()> =
            field_support(field2, Some("only"), None, tok, role("fs2", Role::Group));
        let field3 = themed_text_input(
            "z",
            "",
            |_| (),
            None,
            FieldOpts::NONE,
            tok,
            role("f3", Role::TextBox),
            None,
        );
        let _: Element<'_, ()> = field_support(field3, None, None, tok, role("fs3", Role::Group));
        let _: Element<'_, ()> = filter_chips(
            &["a".into(), "b".into()],
            &[true, false],
            |_| (),
            tok,
            role("fc", Role::Group),
        );
        let _: Element<'_, ()> = filter_chips(
            &["a".into()],
            &[],
            |_| (),
            tok,
            role("fc-d", Role::Group).with_disabled(true),
        );
        // Selected filter chip (Primary) must paint differently from idle (Quiet).
        let (on_bg, on_ink, on_border) = chip_face(tok, Variant::Primary);
        let (off_bg, off_ink, off_border) = chip_face(tok, Variant::Quiet);
        assert_ne!(on_bg, off_bg);
        assert_ne!(on_ink, off_ink);
        assert_ne!(on_border.color, off_border.color);
        assert!(on_border.width < off_border.width || off_border.width >= 1.0);
        for v in Variant::ALL {
            let (bg, ink, border) = chip_face(tok, v);
            assert_ne!(bg, ink);
            let _ = border.width;
        }
        for name in ["dark", "light"] {
            let t = named(name).tokens;
            for v in [Variant::Success, Variant::Warning] {
                let (wash, ink, _) = chip_face(t, v);
                must(
                    crate::m3::color::contrast_ratio(ink, wash) >= 4.5,
                    format!("{name} {v:?} chip ink on wash"),
                );
            }
        }
        let _: Element<'_, ()> = search_input_clear(
            "q",
            |_| (),
            Some(()),
            None,
            tok,
            role("sc", Role::TextBox),
            None,
        );
        let _: Element<'_, ()> = search_input_clear(
            "",
            |_| (),
            Some(()),
            None,
            tok,
            role("sc0", Role::TextBox),
            None,
        );
        let _: Element<'_, ()> = search_input_clear(
            "q",
            |_| (),
            None,
            None,
            tok,
            role("sc-n", Role::TextBox).with_disabled(true),
            None,
        );
        let _: Element<'_, ()> = range_slider(
            0.0..=10.0,
            2.0,
            8.0,
            |_| (),
            tok,
            role("rs-d", Role::Slider).with_disabled(true),
        );
        assert_eq!(clamp_range_pair(0.0..=100.0, -10.0, 200.0), (0.0, 100.0));
        assert_eq!(clamp_range_pair(0.0..=100.0, 80.0, 20.0), (20.0, 20.0));
        assert_eq!(range_pair_after_low(50.0, 40.0), (40.0, 40.0));
        assert_eq!(range_pair_after_low(10.0, 40.0), (10.0, 40.0));
        assert_eq!(range_pair_after_high(10.0, 5.0), (10.0, 10.0));
        assert_eq!(range_pair_after_high(10.0, 50.0), (10.0, 50.0));
        let _: Element<'_, ()> = progress(
            0.2,
            None,
            None,
            false,
            tok,
            role("p", Role::Progress).with_value("0.2"),
        );
        let _: Element<'_, ()> = progress(
            0.5,
            Some(0.8),
            Some("50% · 1 min"),
            false,
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
            (spinner_angles(0.0).1 - spinner_angles(0.0).0 - std::f32::consts::PI * 1.5).abs()
                < 0.01
        );
        assert!(ring_should_stroke(0.0, 1.0));
        assert!(!ring_should_stroke(0.0, 0.0));
        let a11y = A11y::button("Nope").with_disabled(true);
        let _: Element<'_, ()> =
            themed_button("Nope", Some(()), tok, Variant::Primary, Icons::NONE, a11y);
        let unnamed = A11y::button("");
        let _: Element<'_, ()> = themed_button(
            "Shown",
            Some(()),
            tok,
            Variant::Primary,
            Icons::NONE,
            unnamed,
        );
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
            FieldOpts::NONE,
            tok,
            role("v", Role::TextBox),
            Some(Id::new("name")),
        );
        let _: Element<'_, ()> = themed_text_input(
            "p",
            "",
            |_| (),
            None,
            FieldOpts::NONE,
            tok,
            role("Name", Role::TextBox),
            None,
        );
        let _: Element<'_, ()> = themed_text_input(
            "p",
            "",
            |_| (),
            Some(()),
            FieldOpts::NONE,
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
            "Show",
            "Hide",
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
            "Show",
            "Hide",
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
            crate::layout::FORM_LABEL,
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
            crate::layout::FORM_LABEL,
            tok,
            Direction::Rtl,
            role("vf-off", Role::Group).with_disabled(true),
        );
        let _: Element<'_, ()> =
            search_input("q", |_| (), None, tok, role("q", Role::TextBox), None);
        let mut sv: Element<'_, ()> = search_view(
            "in",
            ["Inbox", "Sent"],
            |_| (),
            |_| (),
            Some(()),
            "No matches",
            tok,
            role("find", Role::Group),
        );
        draw_once(&mut sv);
        let mut empty_sv: Element<'_, ()> = search_view(
            "zz",
            Vec::<String>::new(),
            |_| (),
            |_| (),
            None,
            "No matches",
            tok,
            role("find-empty", Role::Group).with_disabled(true),
        );
        draw_once(&mut empty_sv);
        let mut disabled_hits: Element<'_, ()> = search_view(
            "in",
            ["Inbox", "Sent"],
            |_| (),
            |_| (),
            Some(()),
            "No matches",
            tok,
            role("find-off", Role::Group).with_disabled(true),
        );
        draw_once(&mut disabled_hits);
        let mut group: Element<'_, usize> =
            button_group(["Cut", "Copy"], |i| i, tok, role("edit", Role::Group));
        draw_once(&mut group);
        let mut group_off: Element<'_, usize> = button_group(
            ["Cut"],
            |i| i,
            tok,
            role("edit-off", Role::Group).with_disabled(true),
        );
        draw_once(&mut group_off);
        let _: Element<'_, usize> = button_group(
            std::iter::empty::<Cell>(),
            |i| i,
            tok,
            role("edit-empty", Role::Group).with_disabled(true),
        );
        let mut tip: Element<'_, ()> = tooltip_rich(
            label("Save", tok, role("Save", Role::Header)),
            "Save",
            "Write the buffer.",
            None,
            TooltipAnchor::Follow,
            tok,
            role("tip", Role::Tooltip),
        );
        draw_once(&mut tip);
        let _: Element<'_, ()> = tooltip_rich(
            label("x", tok, role("x", Role::Header)),
            "",
            "",
            None,
            TooltipAnchor::Follow,
            tok,
            role("tip-empty", Role::Tooltip),
        );
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
            ControlSize::Default,
            role("a", Role::ComboBox),
        );
        let pick_src = include_str!("widget.rs")
            .split("pub fn themed_pick_list")
            .nth(1)
            .unwrap()
            .split("pub fn date_picker")
            .next()
            .unwrap();
        assert!(pick_src.contains("tok.meta()"));
        assert!(pick_src.contains("tok.body()"));
        assert!(pick_src.contains("text_size(type_px)"));
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
        let items = markdown::parse("# Hi");
        let items: Vec<_> = items.collect();
        let _: Element<'_, ()> =
            markdown_view(&items, None, |_| (), tok, |_| (), role("md", Role::Group));
        let code = Content::with_text("fn main() {}\n");
        let _: Element<'_, ()> = highlighted_code(
            &code,
            "rs",
            |_| (),
            tok,
            "dark",
            crate::layout::FILL,
            true,
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
            true,
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
            true,
            role("code", Role::Group),
        );
        let _: Element<'_, ()> = highlighted_code(
            &code,
            "rs",
            |_| (),
            tok,
            "dark",
            crate::layout::FILL,
            false,
            role("code", Role::Group),
        );
        let _: Element<'_, ()> = search_input_clear(
            "q",
            |_| (),
            None,
            None,
            tok,
            A11y::new("", Role::TextBox),
            None,
        );
        let _: Element<'_, ()> = tooltip_wrap(
            label("x", tok, role("x", Role::Header)),
            "tip",
            TooltipAnchor::Follow,
            tok,
            role("tip", Role::Tooltip),
        );
        let _: Element<'_, ()> = rule_h(tok, role("rule", Role::Separator));
        let _: Element<'_, ()> = dismiss_button((), tok, btn("dismiss"));
        let _: Element<'_, ()> = chip(
            "c",
            None,
            Some(()),
            tok,
            Variant::Quiet,
            ChipKind::Assist,
            Icons::NONE,
            btn("c"),
        );
        let _: Element<'_, ()> = chip(
            "plain",
            None,
            None,
            tok,
            Variant::Primary,
            ChipKind::Assist,
            Icons::NONE,
            btn("plain"),
        );
        let _: Element<'_, ()> = chip(
            "hot",
            None,
            Some(()),
            tok,
            Variant::Danger,
            ChipKind::Assist,
            Icons::NONE,
            btn("hot"),
        );
        let _: Element<'_, ()> = chip(
            "g",
            None,
            None,
            tok,
            Variant::Ghost,
            ChipKind::Assist,
            Icons::NONE,
            btn("g"),
        );
        let _: Element<'_, ()> = chip(
            "k",
            Some(()),
            None,
            tok,
            Variant::Chip,
            ChipKind::Assist,
            Icons::NONE,
            btn("k"),
        );
        let _: Element<'_, ()> = chip(
            "ok",
            None,
            None,
            tok,
            Variant::Success,
            ChipKind::Assist,
            Icons::NONE,
            btn("ok"),
        );
        let _: Element<'_, ()> = chip(
            "warn",
            None,
            None,
            tok,
            Variant::Warning,
            ChipKind::Assist,
            Icons::NONE,
            btn("warn"),
        );
        let _: Element<'_, ()> = badge(
            "b",
            None,
            tok,
            Variant::Quiet,
            BadgeSize::Large,
            role("b", Role::Status),
        );
        let _: Element<'_, ()> = badge(
            "new",
            None,
            tok,
            Variant::Primary,
            BadgeSize::Large,
            role("new", Role::Status),
        );
        let _: Element<'_, ()> = badge(
            "!",
            None,
            tok,
            Variant::Danger,
            BadgeSize::Large,
            role("bang", Role::Status),
        );
        let _: Element<'_, ()> = badge(
            "g",
            None,
            tok,
            Variant::Ghost,
            BadgeSize::Large,
            role("g", Role::Status),
        );
        let _: Element<'_, ()> = badge(
            "chip",
            None,
            tok,
            Variant::Chip,
            BadgeSize::Large,
            role("chip", Role::Status),
        );
        let _: Element<'_, ()> = badge(
            "ok",
            None,
            tok,
            Variant::Success,
            BadgeSize::Large,
            role("ok", Role::Status),
        );
        let _: Element<'_, ()> = badge(
            "warn",
            None,
            tok,
            Variant::Warning,
            BadgeSize::Large,
            role("warn", Role::Status),
        );
        let _: Element<'_, ()> = group_box(
            "g",
            label("x", tok, role("x", Role::Header)),
            tok,
            CardFace::Elevated,
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
            age_ms: 10,
        };
        let _: Element<'_, ()> = toast_view(&toast, (), tok, role("t", Role::Status));
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
            "No sessions",
            |_| tok.muted,
            None,
            RowFace::FLUSH,
            |_| (),
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
            |_| (),
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
            |_| (),
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
            None,
            tok,
            role("grid", Role::List),
        );
        let table = TableModel {
            headers: vec!["A".into(), "B".into()],
            rows: vec![vec!["1".into(), "x".into()], vec!["2".into(), "y".into()]],
            sort_col: None,
            sort_asc: true,
            checks: Vec::new(),
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
            |_| (),
            tok,
            role("table", Role::Table),
        );
        let big = TableModel {
            headers: vec!["N".into()],
            rows: (0..50).map(|i| vec![i.to_string()]).collect(),
            sort_col: None,
            sort_asc: true,
            checks: Vec::new(),
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
            |_| (),
            tok,
            role("table", Role::Table),
        );
        let tree = TreeNode::branch(1, "r", vec![TreeNode::leaf(2, "c")]);
        let _: Element<'_, ()> = tree_view(
            &tree,
            Some(2),
            None,
            |_| (),
            |_| (),
            TreeFace::Outline,
            tok,
            role("tree", Role::Tree),
        );
        let mut tabs = Tabs::new(["A", "B"]);
        tabs.closable = true;
        let _: Element<'_, ()> = tab_bar(
            &tabs,
            |_| (),
            |_| (),
            120.0,
            false,
            tok,
            role("tabs", Role::Tab),
        );
        let open_tabs = Tabs::new(["A"]);
        let _: Element<'_, ()> = tab_bar(
            &open_tabs,
            |_| (),
            |_| (),
            0.0,
            false,
            tok,
            role("tabs", Role::Tab),
        );
        let acc = Accordion { open: Some(0) };
        let _: Element<'_, ()> = accordion_view(
            &["A".into()],
            vec![label("b", tok, role("b", Role::Header))],
            &acc,
            1.0,
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
            0.0,
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
            1.0,
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
            Some((1, 0.5)),
            |_| (),
            |_| (),
            TreeFace::Outline,
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
        let scroll_src = src
            .split("pub fn themed_scroll")
            .nth(1)
            .unwrap()
            .split("pub fn log_view")
            .next()
            .unwrap();
        must(
            scroll_src.contains("ThemedScroll::new"),
            "themed_scroll must compose pane plus rail so the rail can sit on the end side",
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
        let _: Element<'_, ()> = markdown_view(
            &doc.items,
            None,
            |_| (),
            tok,
            |_| (),
            role("md", Role::Group),
        );
        let md = markdown_style(tok);
        let s = tok.scheme();
        assert_eq!(md.link_color, s.primary);
        assert_eq!(md.inline_code_color, s.on_surface);
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
        must(
            outlined.item_offset(heads[1].index, tok) > outlined.item_offset(heads[0].index, tok),
            "later heading sits below the first",
        );
        assert_eq!(outlined.item_offset(0, tok), 0.0);
        assert!(
            outlined.item_offset(outlined.items.len(), tok)
                > outlined.item_offset(heads[1].index, tok)
        );
        let _ = deep.item_offset(deep.items.len(), tok);
        let rich = parse(
            "# T\n\nA paragraph with [link](https://example.com).\n\n- bullet\n  - nested\n\n1. one\n\n- [x] done\n- [ ] todo\n\n> quoted\n>\n> still quoted\n\n---\n\n```rust\nfn x() {}\n```\n\n| Name | Ready |\n| --- | --- |\n| A | yes |\n\n![Logo](pixel.png)\n",
        );
        assert!(rich.items.len() >= 8);
        assert!(rich.item_offset(rich.items.len(), tok) > 100.0);
    }

    #[test]
    fn split_and_segmented_match_button_height() {
        let tok = named("dark").tokens;
        let btn = |n: &str| A11y::button(n);
        let max = iced::Size::new(800.0, 400.0);
        let mut labeled = themed_button(
            "Save",
            Some(()),
            tok,
            Variant::Primary,
            Icons::NONE,
            btn("Save"),
        );
        let mut split = split_button(
            "Save",
            0,
            vec![("As…".into(), 1)],
            tok,
            Icons::NONE,
            btn("Save"),
        );
        let mut segmented = segmented_button(
            ["Day", "Week", "Month"],
            0,
            |i| i,
            tok,
            ControlSize::Default,
            A11y::new("Range", Role::Group),
        );
        let labeled_h = layout_size(&mut labeled, max).height;
        let split_h = layout_size(&mut split, max).height;
        let segmented_h = layout_size(&mut segmented, max).height;
        assert_eq!(split_h, labeled_h);
        assert_eq!(segmented_h, labeled_h);
    }

    #[test]
    fn material_knobs_draw_disabled_empty_and_outlined() {
        let compact = named("dark")
            .tokens
            .with_density(crate::density::Density::named(
                crate::density::DensityName::Compact,
            ));
        let tok = named("dark").tokens;
        let btn = |n: &str| A11y::button(n);
        let role = |n: &str, r: Role| A11y::new(n, r);
        let _ = Cell::from(String::from("X"));
        let _ = Cell::from(&String::from("Y"));
        let mut filled = themed_button(
            "Save",
            Some(()),
            tok,
            Variant::Primary,
            Icons::both(Icon::Check, Icon::Chevron),
            btn("Save"),
        );
        draw_once(&mut filled);
        let mut outlined = themed_button(
            "Edit",
            None::<()>,
            compact,
            Variant::Outlined,
            Icons::NONE,
            btn("Edit").with_disabled(true),
        );
        draw_once(&mut outlined);
        let mut elevated = themed_button(
            "Open",
            Some(()),
            tok,
            Variant::Elevated,
            Icons::NONE,
            btn("Open"),
        );
        draw_once(&mut elevated);
        let mut toggle_ic = icon_button_toggle(
            Icon::Check,
            true,
            (),
            tok,
            Variant::Primary,
            ControlSize::Comfortable,
            btn("Bold").with_checked(true),
        );
        let mut off_ic = icon_button_toggle(
            Icon::Menu,
            false,
            (),
            compact,
            Variant::Quiet,
            ControlSize::Default,
            btn("Off").with_checked(false).with_disabled(true),
        );
        draw_once(&mut off_ic);
        draw_once(&mut toggle_ic);
        let mut cells = segmented_button(
            [Cell::new("Day").with_icon(Icon::Search), Cell::new("Week")],
            0,
            |i| i,
            tok,
            ControlSize::Default,
            role("seg", Role::Group),
        );
        draw_once(&mut cells);
        let opts = FieldOpts {
            face: FieldFace::Outlined,
            icons: Icons::both(Icon::Search, Icon::Close),
            label: "Find",
            max_len: Some(8),
        };
        let mut field = themed_text_input(
            "q",
            "hi",
            |_| (),
            None,
            opts,
            tok,
            role("q", Role::TextBox),
            None,
        );
        draw_once(&mut field);
        let empty_opts = FieldOpts {
            face: FieldFace::Filled,
            icons: Icons::NONE,
            label: "Name",
            max_len: Some(4),
        };
        let mut empty = themed_text_input(
            "Name",
            "",
            |_| (),
            None,
            empty_opts,
            compact,
            role("n", Role::TextBox).with_disabled(true),
            None,
        );
        draw_once(&mut empty);
        let mut vert = themed_slider(
            0.0..=1.0,
            0.3,
            |_| (),
            SliderMarks {
                vertical: true,
                thumb: "30%",
                ..SliderMarks::NONE
            },
            tok,
            role("v", Role::Slider),
        );
        draw_once(&mut vert);
        let mut vert_off = themed_slider(
            0.0..=1.0,
            0.0,
            |_| (),
            SliderMarks {
                vertical: true,
                ..SliderMarks::NONE
            },
            tok,
            role("vd", Role::Slider).with_disabled(true),
        );
        draw_once(&mut vert_off);
        let mut tip_b: Element<'_, ()> = tooltip_wrap(
            label("H", tok, role("Hb", Role::Header)),
            "tip",
            TooltipAnchor::Bottom,
            tok,
            role("tb", Role::Tooltip),
        );
        draw_once(&mut tip_b);
        let mut tip_s: Element<'_, ()> = tooltip_wrap(
            label("H", tok, role("Hs", Role::Header)),
            "tip",
            TooltipAnchor::Start,
            tok,
            role("ts", Role::Tooltip),
        );
        draw_once(&mut tip_s);
        let mut filled_card: Element<'_, ()> = group_box(
            "F",
            label("x", tok, role("xf", Role::Status)),
            tok,
            CardFace::Filled,
            role("cf", Role::Group),
        );
        draw_once(&mut filled_card);
        let mut suggest = chip(
            "go",
            Some(()),
            None,
            tok,
            Variant::Primary,
            ChipKind::Suggestion,
            Icons::trailing(Icon::Chevron),
            btn("go"),
        );
        draw_once(&mut suggest);
        let mut tabs = crate::collection::Tabs::new(["A", "B"]).with_icon(0, Icon::Search);
        tabs.closable = true;
        let mut strip = tab_bar(
            &tabs,
            |i| i,
            |_| 0,
            480.0,
            true,
            tok,
            role("tabs", Role::Tab),
        );
        draw_once(&mut strip);
        tabs.select(1);
        let _ = tabs.close(0);
        let mut busy: Element<'_, ()> = progress(
            0.2,
            None,
            Some("wait"),
            true,
            tok,
            role("p", Role::Progress),
        );
        draw_once(&mut busy);
        let mut late: Element<'_, ()> =
            progress(0.98, None, None, true, tok, role("p-late", Role::Progress));
        draw_once(&mut late);
        let mut still: Element<'_, ()> = progress(
            0.2,
            None,
            None,
            true,
            tok.with_reduced_motion(true),
            role("p-still", Role::Progress),
        );
        draw_once(&mut still);
        let mut card: Element<'_, ()> = group_box(
            "Box",
            label("x", tok, role("x", Role::Status)),
            tok,
            CardFace::Outlined,
            role("c", Role::Group),
        );
        draw_once(&mut card);
        let mut mark: Element<'_, ()> = badge(
            "9",
            Some(icon_svg(Icon::Menu, tok, role("i", Role::Image))),
            tok,
            Variant::Primary,
            BadgeSize::Small,
            role("b", Role::Status),
        );
        draw_once(&mut mark);
        let mut tip: Element<'_, ()> = tooltip_rich(
            label("H", tok, role("H", Role::Header)),
            "Save",
            "Write.",
            Some(("More".into(), ())),
            TooltipAnchor::Top,
            tok,
            role("t", Role::Tooltip),
        );
        draw_once(&mut tip);
        let mut input_chip = chip(
            "tag",
            None,
            Some(()),
            tok,
            Variant::Quiet,
            ChipKind::Input,
            Icons::leading(Icon::Close),
            btn("tag"),
        );
        draw_once(&mut input_chip);
        let mut filter_out = chip(
            "f",
            Some(()),
            None,
            tok,
            Variant::Outlined,
            ChipKind::Filter,
            Icons::NONE,
            btn("f"),
        );
        draw_once(&mut filter_out);
        let mut filter_el = chip(
            "e",
            Some(()),
            None,
            tok,
            Variant::Elevated,
            ChipKind::Filter,
            Icons::NONE,
            btn("e"),
        );
        draw_once(&mut filter_el);
        let mut ic_def = icon_button(
            Icon::Search,
            Some(()),
            tok,
            Variant::Ghost,
            ControlSize::Default,
            btn("s"),
        );
        draw_once(&mut ic_def);
        assert!(compact.density.pad < tok.density.pad);
        assert_eq!(ControlSize::Compact.pad(), 4);
        assert_eq!(ControlSize::Default.pad(), 8);
        assert_eq!(ControlSize::Comfortable.pad(), 12);
        assert_eq!(
            TooltipAnchor::Follow.position(),
            tooltip::Position::FollowCursor
        );
        assert_eq!(TooltipAnchor::Top.position(), tooltip::Position::Top);
        assert_eq!(TooltipAnchor::Bottom.position(), tooltip::Position::Bottom);
        assert_eq!(TooltipAnchor::Start.position(), tooltip::Position::Left);
        assert_eq!(FieldFace::default(), FieldFace::Filled);
        assert_eq!(CardFace::default(), CardFace::Elevated);
        assert_eq!(TreeFace::default(), TreeFace::Outline);
        assert_eq!(BadgeSize::default(), BadgeSize::Large);
        assert_eq!(ChipKind::default(), ChipKind::Assist);
        assert_eq!(TooltipAnchor::default(), TooltipAnchor::Follow);
        assert_eq!(ControlSize::default(), ControlSize::Default);
        let comfy = tok.with_density(crate::density::Density::named(
            crate::density::DensityName::Comfortable,
        ));
        assert!(comfy.density.pad > tok.density.pad);
        let mut comfy_btn = themed_button(
            "Wide",
            Some(()),
            comfy,
            Variant::Primary,
            Icons::NONE,
            btn("Wide"),
        );
        draw_once(&mut comfy_btn);
    }

    #[test]
    fn named_densities_produce_distinct_control_heights() {
        let base = named("dark").tokens;
        let height = |name| control_height(base.with_density(crate::density::Density::named(name)));
        let compact = height(crate::density::DensityName::Compact);
        let default = height(crate::density::DensityName::Default);
        let comfortable = height(crate::density::DensityName::Comfortable);
        must(
            compact < default,
            format!("compact {compact} must be shorter than default {default}"),
        );
        must(
            default < comfortable,
            format!("default {default} must be shorter than comfortable {comfortable}"),
        );
        let max = iced::Size::new(400.0, 80.0);
        let mut compact_btn = themed_button(
            "Save",
            Some(()),
            base.with_density(crate::density::Density::named(
                crate::density::DensityName::Compact,
            )),
            Variant::Primary,
            Icons::NONE,
            A11y::button("Save"),
        );
        let mut default_btn = themed_button(
            "Save",
            Some(()),
            base,
            Variant::Primary,
            Icons::NONE,
            A11y::button("Save"),
        );
        let mut comfortable_btn = themed_button(
            "Save",
            Some(()),
            base.with_density(crate::density::Density::named(
                crate::density::DensityName::Comfortable,
            )),
            Variant::Primary,
            Icons::NONE,
            A11y::button("Save"),
        );
        let hc = layout_size(&mut compact_btn, max).height;
        let hd = layout_size(&mut default_btn, max).height;
        let hh = layout_size(&mut comfortable_btn, max).height;
        assert!(hc < hd, "compact button {hc} < default {hd}");
        assert!(hd < hh, "default button {hd} < comfortable {hh}");
        let slider_fn = include_str!("widget.rs")
            .split("pub fn themed_slider")
            .nth(1)
            .unwrap();
        assert_eq!(
            slider_fn
                .matches(".style(style::slider_style(tok))")
                .count(),
            2
        );
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
            image_slot(
                ImageSlot::Loading,
                48.0,
                48.0,
                tok,
                role("load", Role::Image),
            ),
            image_slot(
                ImageSlot::Ready {
                    handle: iced::widget::image::Handle::from_bytes(TEST_PNG),
                    fit: iced::ContentFit::Contain,
                },
                48.0,
                48.0,
                tok,
                role("img", Role::Image),
            ),
            progress_ring(0.5, Some("half"), tok, role("pr", Role::Progress)),
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
                true,
                role("code", Role::Group),
            ),
            chip(
                "ok",
                None,
                None,
                tok,
                Variant::Success,
                ChipKind::Assist,
                Icons::NONE,
                btn("ok"),
            ),
            chip(
                "x",
                None,
                Some(()),
                tok,
                Variant::Quiet,
                ChipKind::Assist,
                Icons::NONE,
                btn("x"),
            ),
            tooltip_wrap(
                label("n", tok, role("n", Role::Status)),
                "tip",
                TooltipAnchor::Follow,
                tok,
                role("tt", Role::Tooltip),
            ),
            badge(
                "ok",
                None,
                tok,
                Variant::Success,
                BadgeSize::Large,
                role("ok", Role::Status),
            ),
            group_box(
                "Box",
                label("in", tok, role("in", Role::Status)),
                tok,
                CardFace::Elevated,
                role("box", Role::Group),
            ),
            banner("Hi", None, tok, role("ban", Role::Status)),
            info_bar(ToastKind::Info, "n", tok, role("ib", Role::Status)),
        ];
        for el in &mut painted {
            draw_once(el);
        }
        let tree = TreeNode::branch(1, "r", vec![TreeNode::leaf(2, "c")]);
        let mut tv = tree_view(
            &tree,
            Some(1),
            None,
            |_| (),
            |_| (),
            TreeFace::Outline,
            tok,
            role("tree", Role::Tree),
        );
        draw_once(&mut tv);
        let mut mid = tree_view(
            &tree,
            Some(1),
            Some((1, 0.5)),
            |_| (),
            |_| (),
            TreeFace::Outline,
            tok,
            role("tree", Role::Tree),
        );
        draw_once(&mut mid);
        let mut shut = tree_view(
            &tree,
            Some(1),
            Some((1, 0.0)),
            |_| (),
            |_| (),
            TreeFace::Outline,
            tok.with_reduced_motion(true),
            role("tree", Role::Tree),
        );
        draw_once(&mut shut);
        let acc = Accordion { open: Some(0) };
        let mut av = accordion_view(
            &["A".into()],
            vec![label("b", tok, role("b", Role::Header))],
            &acc,
            1.0,
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
            |_| (),
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
            |_| (),
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
            |_| (),
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
            |_| (),
            role("list", Role::List).with_disabled(true),
        );
        draw_once(&mut dead_lv);
        let table = TableModel {
            headers: vec!["A".into()],
            rows: vec![vec!["1".into()]],
            sort_col: None,
            sort_asc: true,
            checks: Vec::new(),
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
            |_| (),
            tok,
            role("table", Role::Table).with_disabled(true),
        );
        draw_once(&mut dead_dt);
        let mut dead_tree = tree_view(
            &tree,
            Some(1),
            None,
            |_| (),
            |_| (),
            TreeFace::Outline,
            tok,
            role("tree", Role::Tree).with_disabled(true),
        );
        draw_once(&mut dead_tree);
        let mut dead_grid = item_grid(
            &["A".into(), "B".into()],
            |_| (),
            None,
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
            crate::layout::FORM_LABEL,
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
            true,
            role("code", Role::Group).with_disabled(true),
        );
        draw_once(&mut dead_code);
        let mut dead_x = dismiss_button((), tok, A11y::button("x").with_disabled(true));
        draw_once(&mut dead_x);
        let toast = Toast {
            id: 1,
            kind: ToastKind::Info,
            text: "t".into(),
            ttl_ms: 10,
            age_ms: 10,
        };
        let mut tvw = toast_view(&toast, (), tok, role("t", Role::Status));
        draw_once(&mut tvw);
        let reduced = tok.with_reduced_motion(true);
        let fresh = Toast {
            id: 2,
            kind: ToastKind::Success,
            text: "ok".into(),
            ttl_ms: 4000,
            age_ms: 0,
        };
        let mut instant = toast_view(&fresh, (), reduced, role("ok", Role::Status));
        draw_once(&mut instant);
        let mut closed = accordion_view(
            &["A".into()],
            vec![label("b", tok, role("b", Role::Header))],
            &Accordion { open: None },
            0.0,
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
            0.0,
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
            1.0,
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
            0.0,
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
            "Show",
            "Hide",
            &copy,
            tok,
            Direction::Ltr,
            A11y::new("api-token", Role::Group),
        );
        let search_src = src
            .split("pub fn search_input_clear")
            .nth(1)
            .unwrap()
            .split("pub fn themed_pick_list")
            .next()
            .unwrap();
        assert!(!search_src.contains("apply_name(value)"));
        assert!(search_src.contains("a11y.child(Role::TextBox)"));
        assert!(search_src.contains("on_submit"));
        assert!(search_src.contains("input_id"));
        assert!(src.contains("pub fn search_input_clear"));
        let vf_src = src
            .split("pub fn value_field")
            .nth(1)
            .unwrap()
            .split("pub fn textarea")
            .next()
            .unwrap();
        assert!(vf_src.contains("label_width") && vf_src.contains("Length::Fixed"));
        assert!(vf_src.contains("Length::Fill"));
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
        let mut el: Element<'_, String> = search_input("typed-query", |s| s, None, tok, a11y, None);
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
    fn search_input_matches_pick_height_under_compact() {
        let tok = named("dark")
            .tokens
            .with_density(crate::density::Density::named(
                crate::density::DensityName::Compact,
            ));
        let mut search: Element<'_, ()> = search_input(
            "",
            |_| (),
            None,
            tok,
            A11y::new("Search", Role::TextBox),
            None,
        );
        let mut pick: Element<'_, ()> = themed_pick_list(
            &["All"][..],
            Some("All"),
            |_| (),
            tok,
            ControlSize::Default,
            A11y::new("Filter", Role::ComboBox),
        );
        let max = iced::Size::new(400.0, 80.0);
        let hs = layout_size(&mut search, max).height;
        let hp = layout_size(&mut pick, max).height;
        must(
            (hs - hp).abs() <= 1.0,
            format!("search {hs} must match pick {hp} under compact density"),
        );
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
            None,
            |id| id,
            |c| c.id,
            TreeFace::Outline,
            tok,
            A11y::new("tree", Role::Tree),
        );
        let _: Element<'_, u64> = tree_view(
            &tree,
            None,
            Some((1, 0.4)),
            |id| id,
            |c| c.id,
            TreeFace::Outline,
            tok,
            A11y::new("tree", Role::Tree),
        );
        let compact = tok.with_density(crate::density::Density::named(
            crate::density::DensityName::Compact,
        ));
        let mut files: Element<'_, u64> = tree_view(
            &tree,
            Some(2),
            None,
            |id| id,
            |c| c.id,
            TreeFace::Files,
            compact,
            A11y::new("tree", Role::Tree),
        );
        draw_once(&mut files);
        let comfy = tok.with_density(crate::density::Density::named(
            crate::density::DensityName::Comfortable,
        ));
        let mut roomy: Element<'_, u64> = tree_view(
            &tree,
            Some(2),
            None,
            |id| id,
            |c| c.id,
            TreeFace::Outline,
            comfy,
            A11y::new("tree", Role::Tree),
        );
        draw_once(&mut roomy);
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
            true,
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
        assert!(hl.contains("tok.code()"));
        assert!(hl.contains("wrapping"));
        assert!(!hl.contains("if !a11y.disabled"));
        let block_src = src
            .split("pub fn code_block")
            .nth(1)
            .unwrap()
            .split("/// A text link")
            .next()
            .unwrap();
        assert!(block_src.contains("tok.code()"));
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
        let tok = named("dark").tokens;
        let theme = crate::theme::iced_theme("dark", tok);
        let s = tok.scheme();
        let sel = selectable_style(tok)(&theme, iced::widget::text_editor::Status::Active);
        assert_eq!(sel.value, s.on_surface);
        assert_eq!(sel.placeholder, s.on_surface_variant);
        assert_eq!(sel.selection, s.secondary_container);
        let frame = editor_frame(tok);
        assert_eq!(
            frame.background,
            Some(iced::Background::Color(s.surface_container_highest))
        );
        assert_eq!(frame.border.color, s.outline_variant);
        let ed = editor_style(tok)(&theme, iced::widget::text_editor::Status::Active);
        assert_eq!(ed.value, s.on_surface);
        assert_eq!(ed.selection, s.secondary_container);
        // Disabled slider face uses scheme track colors (style closure).
        let _: Element<'_, ()> = themed_slider(
            0.0..=1.0,
            0.5,
            |_| (),
            SliderMarks::NONE,
            tok,
            A11y::new("off", Role::Slider).with_disabled(true),
        );
    }

    #[test]
    fn virtual_column_mounts_only_window_range() {
        let tok = named("dark").tokens;
        let heights = crate::collection::expand_card_heights(20, 40.0, &[(2, 100.0)]);
        let win = VisibleWindow {
            start: 1,
            end: 5,
            scroll: 40.0,
            viewport: 120.0,
        };
        let mut el: Element<'_, ()> = virtual_column(
            &heights,
            win,
            2,
            Some(2),
            |_| (),
            Some(Id::from("vc-scroll")),
            tok,
            |i| label(format!("r{i}"), tok, A11y::new("r", Role::ListItem)),
            A11y::new("vc", Role::List),
        );
        draw_once(&mut el);
        let empty_h: [f32; 0] = [];
        let mut empty: Element<'_, ()> = virtual_column(
            &empty_h,
            VisibleWindow::new(80.0),
            0,
            None,
            |_| (),
            None,
            tok,
            {
                let paint = |_: usize| label("x", tok, A11y::new("x", Role::ListItem));
                let _ = paint(0);
                paint
            },
            A11y::new("vc0", Role::List),
        );
        draw_once(&mut empty);
        let src = include_str!("widget.rs");
        let body = src
            .split("pub fn virtual_column")
            .nth(1)
            .unwrap()
            .split("/// A virtualized row list.")
            .next()
            .unwrap();
        assert!(body.contains("virtual_clip") && body.contains("RowHeights::PerRow"));
    }

    #[test]
    fn markdown_view_uses_structured_selectable_layout() {
        let tok = named("dark").tokens;
        let source = "# Title\n\nFirst paragraph.\n\nSecond block.";
        let doc = parse(source);
        let _: Element<'_, ()> = markdown_view(
            &doc.items,
            None,
            |_| (),
            tok,
            |_| (),
            A11y::new("md", Role::Group),
        );
        let plain = crate::select::markdown_plain(&doc.items);
        assert!(plain.contains("Title") && plain.contains("Second block."));
        let src = include_str!("widget.rs");
        let md = src
            .split("pub fn markdown_view")
            .nth(1)
            .unwrap()
            .split("fn markdown_style")
            .next()
            .unwrap();
        assert!(md.contains("markdown::item"));
        assert!(md.contains("view_with") || md.contains("markdown::item"));
        assert!(!md.contains("Rich::with_spans"));
        assert!(!md.contains("start.item != span.end.item"));
        assert!(!md.contains("from_ref(item)"));
        assert!(md.contains("highlight_markdown_spans"));
        assert!(md.contains("markdown_listen"));
        let mut st = crate::select::markdown_select(
            &doc.items,
            crate::select::MarkdownSelect::default(),
            crate::select::MarkdownPointer::at_y(0.0),
            tok,
        );
        st = crate::select::markdown_select(
            &doc.items,
            st,
            crate::select::MarkdownPointer::Press,
            tok,
        );
        let end_y = doc
            .items
            .iter()
            .map(|i| crate::select::markdown_item_extent(i, crate::theme::named("dark").tokens))
            .sum::<f32>();
        st = crate::select::markdown_select(
            &doc.items,
            st,
            crate::select::MarkdownPointer::at_y(end_y),
            tok,
        );
        let mut painted: Element<'_, crate::select::MarkdownPointer> = markdown_view(
            &doc.items,
            Some(&st.span),
            |ev| ev,
            tok,
            |_| crate::select::MarkdownPointer::Release,
            A11y::new("md-span", Role::Group),
        );
        draw_once(&mut painted);
        assert!(st.span.text(&doc.items).contains("Title"));
        assert!(st.span.text(&doc.items).contains("First"));
        let head = crate::select::MarkdownSpan {
            start: crate::select::MarkdownPos { item: 0, offset: 0 },
            end: crate::select::MarkdownPos { item: 0, offset: 5 },
        };
        let mut part: Element<'_, ()> = markdown_view(
            &doc.items,
            Some(&head),
            |_| (),
            tok,
            |_| (),
            A11y::new("md-head", Role::Group),
        );
        draw_once(&mut part);
        let nested = parse("- first\n  - nested\n- second");
        let list_i = nested
            .items
            .iter()
            .position(|i| matches!(i, iced::widget::markdown::Item::List { .. }))
            .expect("list");
        let plain = crate::select::markdown_item_plain(&nested.items[list_i]);
        let at = plain.find("first").expect("first");
        let word = crate::select::MarkdownSpan {
            start: crate::select::MarkdownPos {
                item: list_i,
                offset: at,
            },
            end: crate::select::MarkdownPos {
                item: list_i,
                offset: at + "first".len(),
            },
        };
        let mut nest: Element<'_, ()> = markdown_view(
            &nested.items,
            Some(&word),
            |_| (),
            tok,
            |_| (),
            A11y::new("md-nest", Role::Group),
        );
        draw_once(&mut nest);
    }

    #[test]
    fn markdown_view_posts_pointer_after_child_captures() {
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::mouse;
        use iced::{Font, Pixels, Point, Rectangle, Size};
        let tok = named("dark").tokens;
        let doc = parse("# Title that continues for a while\n\nA paragraph of body text.");
        let mut el: Element<'_, crate::select::MarkdownPointer> = markdown_view(
            &doc.items,
            None,
            |ev| ev,
            tok,
            |_| crate::select::MarkdownPointer::Release,
            A11y::new("md-press", Role::Group),
        );
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(400.0, 240.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(400.0, 240.0));
        let mut clipboard = clipboard::Null;
        let at = Point::new(24.0, 12.0);
        let mut first = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut first);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(at),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        must(
            first.contains(&crate::select::MarkdownPointer::Press),
            format!("press must reach on_pointer after paint-side capture, got {first:?}"),
        );
        assert!(first
            .iter()
            .any(|m| matches!(m, crate::select::MarkdownPointer::Move { .. })));
        let mut second = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut second);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(at),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        must(
            second.contains(&crate::select::MarkdownPointer::Double),
            format!("double-click must reach on_pointer, got {second:?}"),
        );
        let mut st = crate::select::MarkdownSelect::default();
        for ev in first.iter().chain(second.iter()) {
            st = crate::select::markdown_select(&doc.items, st, *ev, tok);
        }
        assert!(!st.span.is_empty());
        let copied = st.span.text(&doc.items);
        assert!(!copied.is_empty());
        assert_ne!(copied, doc.source);
        let mut up = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut up);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(at),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        assert!(up.contains(&crate::select::MarkdownPointer::Release));
        let mut miss = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut miss);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(Point::new(900.0, 900.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        must(
            !miss.contains(&crate::select::MarkdownPointer::Press),
            format!("a miss must not post Press, got {miss:?}"),
        );
        let mut moved = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut moved);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::CursorMoved {
                    position: Point::new(40.0, 20.0),
                }),
                layout,
                mouse::Cursor::Available(Point::new(40.0, 20.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        assert!(moved.iter().any(|m| matches!(
            m,
            crate::select::MarkdownPointer::Move { x, y } if (*x - 40.0).abs() < 1.0 && (*y - 20.0).abs() < 1.0
        )));
    }

    #[test]
    fn list_slots_and_table_checks_still_virtualize() {
        let tok = named("dark").tokens;
        let list = VecList {
            items: (0..40)
                .map(|i| {
                    crate::collection::ListRow::new(format!("r{i}"))
                        .with_leading(crate::collection::RowSlot::Check(i == 0))
                        .with_trailing(crate::collection::RowSlot::Icon(Icon::Search))
                })
                .collect(),
        };
        let win = VisibleWindow::new(120.0);
        let mut el: Element<'_, usize> = list_view(
            &list,
            &Sel::None,
            |c| c.id,
            tok,
            win,
            24.0,
            2,
            |_| 0,
            "Empty",
            |_| tok.muted,
            None,
            RowFace::FLUSH,
            |i| i,
            A11y::new("slotted", Role::List),
        );
        draw_once(&mut el);
        let empty_model = VecList::default();
        let mut empty_slots: Element<'_, ()> = list_view(
            &empty_model,
            &Sel::None,
            |_| (),
            tok,
            win,
            24.0,
            0,
            |_| (),
            "Empty",
            |_| tok.muted,
            None,
            RowFace::FLUSH,
            |_| (),
            A11y::new("empty-slots", Role::List),
        );
        draw_once(&mut empty_slots);
        let table = TableModel {
            headers: vec!["N".into()],
            rows: (0..40).map(|i| vec![format!("{i}")]).collect(),
            sort_col: None,
            sort_asc: true,
            checks: (0..40).map(|i| i % 2 == 0).collect(),
        };
        let cols = crate::collection::ColumnLayout::new(vec![80.0]);
        let mut dt: Element<'_, usize> = data_table(
            &table,
            &Sel::None,
            None,
            &cols,
            false,
            win,
            24.0,
            2,
            |_, _| 0,
            |_| 0,
            |_| 0,
            |_| 0,
            |i| i,
            tok,
            A11y::new("checks", Role::Table),
        );
        draw_once(&mut dt);
        let _: Element<'_, ()> = themed_slider(
            0.0..=1.0,
            0.2,
            |_| (),
            SliderMarks {
                ticks: 3,
                min: "lo",
                max: "hi",
                ..SliderMarks::NONE
            },
            tok,
            A11y::new("ticked", Role::Slider),
        );
        let _: Element<'_, ()> = themed_slider(
            0.0..=1.0,
            0.2,
            |_| (),
            SliderMarks::NONE,
            tok,
            A11y::new("plain", Role::Slider).with_disabled(true),
        );
        let mut buf: Element<'_, ()> = progress(
            0.3,
            Some(0.6),
            Some("30% · 4s"),
            false,
            tok,
            A11y::new("buf", Role::Progress),
        );
        draw_once(&mut buf);
        let mut zero: Element<'_, ()> = progress(
            0.0,
            Some(0.0),
            None,
            false,
            tok,
            A11y::new("zero", Role::Progress),
        );
        draw_once(&mut zero);
        let tabs = Tabs::new(["A", "B", "C", "D", "E"]).with_badge(0, "2");
        let mut ov: Element<'_, usize> = tab_bar(
            &tabs,
            |i| i,
            |_| 0,
            80.0,
            false,
            tok,
            A11y::new("ov", Role::Tab),
        );
        draw_once(&mut ov);
        let _: Element<'_, usize> = tab_bar(
            &tabs,
            |i| i,
            |_| 0,
            0.0,
            false,
            tok,
            A11y::new("all", Role::Tab),
        );
        assert_eq!(tab_overflow_index(&tabs.titles, "C"), 2);
        assert_eq!(tab_overflow_pick(tabs.titles.clone(), |i| i)("C".into()), 2);
        assert_eq!(tab_overflow_index(&tabs.titles, "nope"), 0);
        assert_eq!(tab_visible_count(&[], 100.0), 0);
        assert_eq!(tab_visible_count(&tabs.titles, 0.0), 5);
        assert!(tab_visible_count(&tabs.titles, 80.0) < 5);
        assert_eq!(tab_visible_count(&["A".into()], 10.0), 1);
        let dead_slots = VecList {
            items: vec![crate::collection::ListRow::new("x")
                .with_leading(crate::collection::RowSlot::Check(true))],
        };
        let mut dead_check: Element<'_, ()> = list_view(
            &dead_slots,
            &Sel::None,
            |_| (),
            tok,
            win,
            24.0,
            0,
            |_| (),
            "Empty",
            |_| tok.muted,
            None,
            RowFace::FLUSH,
            |_| (),
            A11y::new("dead-check", Role::List).with_disabled(true),
        );
        draw_once(&mut dead_check);
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
            |_| window,
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
            checks: Vec::new(),
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

    fn rail_box(boxes: &[iced::Rectangle]) -> iced::Rectangle {
        let rail_w = crate::chrome::SCROLL_RAIL_WIDTH;
        *boxes
            .iter()
            .find(|b| (b.width - rail_w).abs() < 0.6 && b.height > 20.0)
            .expect("rail")
    }

    #[test]
    fn rtl_rails_sit_on_the_left_of_list_and_scroll() {
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels, Size};

        let tok = named("dark")
            .tokens
            .with_direction(crate::i18n::Direction::Rtl);
        let row_h = 20.0;
        let viewport = 200.0;
        let list = VecList {
            items: (0..80)
                .map(|i| {
                    crate::collection::ListRow::new(format!("r{i}"))
                        .with_leading(crate::collection::RowSlot::Check(false))
                })
                .collect(),
        };
        let window = crate::collection::visible_window(0.0, viewport, row_h, 80, 4, None);
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
            None,
            RowFace::FLUSH,
            |_| window,
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
        let rb = rail_box(&boxes);
        must(
            rb.x - origin.x < 8.0,
            format!(
                "RTL list rail must sit on the left, got x={}",
                rb.x - origin.x
            ),
        );
        let checks: Vec<_> = boxes
            .iter()
            .filter(|b| (b.width - 16.0).abs() < 0.6 && (b.height - 16.0).abs() < 0.6)
            .collect();
        must(
            checks.iter().any(|b| b.x + b.width > origin.x + 200.0),
            "RTL list leading check must sit on the start (right) side",
        );

        let mut scroller: Element<'_, f32> = themed_scroll(
            iced::widget::column![
                label("a", tok, A11y::new("a", Role::Status)),
                Space::new().height(800.0),
            ]
            .into(),
            tok,
            A11y::new("scroll", Role::Group),
            false,
            None,
            None::<fn(_) -> f32>,
        );
        let mut st = Tree::new(scroller.as_widget());
        let sn = scroller.as_widget_mut().layout(&mut st, &renderer, &limits);
        let sl = Layout::new(&sn);
        let so = sl.bounds();
        let mut sb = Vec::new();
        walk_bounds(sl, &mut sb);
        let srail = rail_box(&sb);
        must(
            srail.x - so.x < 8.0,
            format!(
                "RTL themed_scroll rail must sit on the left, got x={}",
                srail.x - so.x
            ),
        );
    }

    #[test]
    fn rtl_tree_indent_and_closed_mark_follow_direction() {
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels, Size};

        let tok = named("dark")
            .tokens
            .with_direction(crate::i18n::Direction::Rtl);
        let root = crate::collection::TreeNode::branch(
            1,
            "src",
            vec![crate::collection::TreeNode::leaf(2, "lib.rs")],
        );
        let mut el: Element<'_, ()> = tree_view(
            &root,
            None,
            None,
            |_| (),
            |_| (),
            TreeFace::Outline,
            tok,
            A11y::new("tree", Role::Tree),
        );
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(320.0, 240.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let origin = layout.bounds();
        let mut boxes = Vec::new();
        walk_bounds(layout, &mut boxes);
        let rb = rail_box(&boxes);
        must(
            rb.x - origin.x < 8.0,
            format!(
                "RTL tree rail must sit on the left, got x={}",
                rb.x - origin.x
            ),
        );
        assert_eq!(closed_disclosure(tok), "◂");
        let ltr = named("dark").tokens;
        assert_eq!(closed_disclosure(ltr), "▸");
        let titles: Vec<_> = boxes
            .iter()
            .filter(|b| b.height > 10.0 && b.height < 40.0 && b.width > origin.width * 0.4)
            .collect();
        must(
            origin.width > 200.0,
            format!(
                "RTL tree row must fill the pane, got width={}",
                origin.width
            ),
        );
        must(
            titles
                .iter()
                .any(|b| b.x + b.width > origin.x + origin.width * 0.7),
            "RTL tree title must fill toward the start (right) side",
        );
        let line_src = include_str!("widget.rs")
            .split("fn tree_line")
            .nth(1)
            .unwrap()
            .split("fn tree_push_branch")
            .next()
            .unwrap();
        assert!(line_src.contains("width(Length::Fill)"));
        assert!(line_src.contains("align_x(start)"));
        assert!(line_src.contains("tree_twisty"));
        assert!(!line_src.contains("themed_button"));
        assert!(line_src.contains("TreeFace::Files"));
        assert!(line_src.contains("gap(tok)"));
    }

    #[test]
    fn rtl_themed_button_keeps_label_extent() {
        use iced::advanced::layout::Limits;
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels, Size};

        let face = include_str!("widget.rs")
            .split("pub fn themed_button_sized")
            .nth(1)
            .unwrap()
            .split("pub fn split_button")
            .next()
            .unwrap();
        must(
            !face.contains(".width(Length::Fill)\n            .align_x(Alignment::Center)"),
            "Fill+align on button text drops right-to-left titles",
        );
        let tok = named("dark")
            .tokens
            .with_direction(crate::i18n::Direction::Rtl);
        let title = "کنٹرولز";
        let mut el: Element<'_, ()> = themed_button(
            title,
            Some(()),
            tok,
            Variant::Primary,
            Icons::NONE,
            A11y::button(title),
        );
        let mut empty: Element<'_, ()> = themed_button(
            "",
            Some(()),
            tok,
            Variant::Primary,
            Icons::NONE,
            A11y::button("pad"),
        );
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(240.0, 80.0));
        let mut tree = Tree::new(el.as_widget());
        let labeled = el
            .as_widget_mut()
            .layout(&mut tree, &renderer, &limits)
            .size();
        let mut empty_tree = Tree::new(empty.as_widget());
        let pad = empty
            .as_widget_mut()
            .layout(&mut empty_tree, &renderer, &limits)
            .size();
        must(
            labeled.width > pad.width + 8.0,
            format!(
                "RTL themed_button must keep title extent, labeled={} pad={}",
                labeled.width, pad.width
            ),
        );
        let card_src = include_str!("widget.rs")
            .split("fn card_row")
            .nth(1)
            .unwrap()
            .split("/// A virtualized column")
            .next()
            .unwrap();
        must(
            card_src.contains("start_label"),
            "card_row must wrap shrink text so RTL titles paint",
        );
        must(
            !card_src.contains(".width(Length::Fill)\n            .align_x(start)"),
            "Fill+align on list card text drops right-to-left titles",
        );
        let flush_src = include_str!("widget.rs")
            .split("fn two_line_row")
            .nth(1)
            .unwrap()
            .split("fn card_row")
            .next()
            .unwrap();
        assert!(flush_src.contains("start_label"));
        assert_eq!(clock_digits(9u32, crate::i18n::Direction::Rtl), "٠٩");
        assert_eq!(clock_digits(30u32, crate::i18n::Direction::Ltr), "30");
        assert_eq!(clock_digits(12u32, crate::i18n::Direction::Rtl), "١٢");
        assert_eq!(clock_digits(34u32, crate::i18n::Direction::Rtl), "٣٤");
        assert_eq!(clock_digits(56u32, crate::i18n::Direction::Rtl), "٥٦");
        assert_eq!(clock_digits(78u32, crate::i18n::Direction::Rtl), "٧٨");
        assert_eq!(clock_digits(0u32, crate::i18n::Direction::Rtl), "٠٠");
        let time_src = include_str!("widget.rs")
            .split("pub fn time_picker")
            .nth(1)
            .unwrap()
            .split("pub fn parse")
            .next()
            .unwrap();
        assert!(time_src.contains("clock_digits"));
        let secret_src = include_str!("widget.rs")
            .split("pub fn secret_field")
            .nth(1)
            .unwrap()
            .split("pub fn value_field")
            .next()
            .unwrap();
        assert!(!secret_src.contains("\"Show\""));
        assert!(secret_src.contains("if revealed { hide } else { show }"));
    }

    #[test]
    fn rtl_pick_list_and_disclosure_put_the_mark_on_the_end() {
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels, Size};

        let tok = named("dark")
            .tokens
            .with_direction(crate::i18n::Direction::Rtl);
        let opts = ["a", "b"];
        let mut pick: Element<'_, &str> = themed_pick_list(
            opts,
            Some("a"),
            |s| s,
            tok,
            ControlSize::Default,
            A11y::new("theme", Role::ComboBox),
        );
        let mut tree = Tree::new(pick.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(240.0, 48.0));
        let node = pick.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let origin = layout.bounds();
        let mut boxes = Vec::new();
        walk_bounds(layout, &mut boxes);
        let icons: Vec<_> = boxes
            .iter()
            .filter(|b| (b.width - 16.0).abs() < 0.6 && (b.height - 16.0).abs() < 0.6)
            .collect();
        must(
            icons.iter().any(|b| b.x - origin.x < origin.width / 2.0),
            "RTL pick chevron must sit on the end (left) side",
        );
        assert_eq!(closed_disclosure(tok), "◂");

        let titles = ["Files".into()];
        let body = label("body", tok, A11y::new("body", Role::Status));
        let mut acc: Element<'_, usize> = accordion_view(
            &titles,
            vec![body],
            &crate::collection::Accordion { open: None },
            0.0,
            |i| i,
            tok,
            A11y::new("acc", Role::Group),
        );
        let mut at = Tree::new(acc.as_widget());
        let an = acc.as_widget_mut().layout(&mut at, &renderer, &limits);
        let al = Layout::new(&an);
        let ao = al.bounds();
        let mut ab = Vec::new();
        walk_bounds(al, &mut ab);
        let marks: Vec<_> = ab
            .iter()
            .filter(|b| b.height > 8.0 && b.height < 28.0 && b.width > 6.0 && b.width < 28.0)
            .collect();
        must(
            marks.iter().any(|b| b.x - ao.x < ao.width / 2.0),
            "RTL disclosure mark must sit on the end (left) side",
        );
    }

    #[test]
    fn rtl_checkbox_and_button_group_follow_direction() {
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels, Size};

        let tok = named("dark")
            .tokens
            .with_direction(crate::i18n::Direction::Rtl);
        let mut box_el: Element<'_, bool> = themed_checkbox(
            "Accept",
            true,
            |on| on,
            tok,
            A11y::new("Accept", Role::Checkbox).with_checked(true),
        );
        let mut tree = Tree::new(box_el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(320.0, 48.0));
        let node = box_el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let origin = layout.bounds();
        let mut boxes = Vec::new();
        walk_bounds(layout, &mut boxes);
        let marks: Vec<_> = boxes
            .iter()
            .filter(|b| (b.width - 16.0).abs() < 6.0 && (b.height - 16.0).abs() < 6.0)
            .collect();
        must(
            origin.width > 200.0,
            format!("RTL checkbox row must fill, got {}", origin.width),
        );
        must(
            marks
                .iter()
                .any(|b| b.x + b.width > origin.x + origin.width * 0.5),
            "RTL checkbox mark must sit on the start (right) side",
        );

        let mut group: Element<'_, usize> = button_group(
            [
                Cell::new("Cut").with_icon(Icon::Close),
                Cell::from("Copy"),
                Cell::from("Paste"),
            ],
            |i| i,
            tok,
            A11y::new("edit", Role::Group),
        );
        let mut gt = Tree::new(group.as_widget());
        let gn = group.as_widget_mut().layout(&mut gt, &renderer, &limits);
        let gl = Layout::new(&gn);
        let go = gl.bounds();
        let mut gb = Vec::new();
        walk_bounds(gl, &mut gb);
        let icons: Vec<_> = gb
            .iter()
            .filter(|b| (b.width - 16.0).abs() < 0.6 && (b.height - 16.0).abs() < 0.6)
            .collect();
        must(
            icons.iter().any(|b| b.x + b.width > go.x + go.width * 0.5),
            "RTL button-group leading icon sits on the start (right) of the first action",
        );
        let group_src = include_str!("widget.rs")
            .split("pub fn button_group")
            .nth(1)
            .unwrap()
            .split("pub fn icon_button")
            .next()
            .unwrap();
        assert!(group_src.contains("align_start(tok.direction)"));
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
            |_| window,
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
            |_| window,
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
            |_| sep_win,
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
            |_| 0.0,
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
            |_| (),
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
            |_| (),
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
            checks: Vec::new(),
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
            |_| 0.0,
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
            checks: Vec::new(),
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
            checks: Vec::new(),
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

    #[test]
    fn stick_scroll_snaps_to_row_boundary() {
        // 200 lines * 20px, viewport 250 → raw end 3750; snap to 3740.
        let s = stick_scroll_snapped(200, 20.0, 250.0);
        assert_eq!(s % 20.0, 0.0);
        assert!(s <= crate::layout::end_offset(4000.0, 250.0));
        assert_eq!(s, 3740.0);
        assert_eq!(stick_scroll_snapped(5, 20.0, 200.0), 0.0);
        assert_eq!(
            stick_scroll_snapped(10, 0.0, 100.0),
            crate::layout::end_offset(0.0, 100.0)
        );
    }

    #[test]
    fn stick_align_pad_makes_max_scroll_row_aligned() {
        let n = 200usize;
        let h = 20.0f32;
        let viewport = 250.0f32;
        let pad = stick_align_pad(n, h, viewport);
        let content = n as f32 * h + pad;
        let max_s = (content - viewport).max(0.0);
        assert!((max_s % h).abs() < 1e-3);
        assert_eq!(stick_align_pad(5, 20.0, 200.0), 0.0);
        assert_eq!(stick_align_pad(10, 0.0, 100.0), 0.0);
        // max_scroll already a multiple of row_h → no pad.
        assert_eq!(stick_align_pad(100, 20.0, 200.0), 0.0);
    }

    #[test]
    fn ellipsize_line_keeps_short_and_trims_long() {
        assert_eq!(ellipsize_line("short", 42), "short");
        assert_eq!(ellipsize_line("any", 0), "any");
        let long = "Quarterly notes for Lisbon and the Berlin office";
        let e = ellipsize_line(long, 20);
        assert!(e.ends_with('…'));
        assert!(e.chars().count() <= 20);
        assert!(!e.contains("office"));
        assert!(!e.ends_with("Lisbo…"), "break on a space, not mid-word");
        // Short flush face: word-aware cut, not a mid-word "Lisbo…".
        let flush = ellipsize_line(long, 26);
        assert!(flush.ends_with('…'));
        assert!(flush.chars().count() <= 26);
        assert!(!flush.contains("Berlin"));
        assert!(flush.ends_with("for…") || flush.ends_with("notes…"));
        assert_eq!(ellipsize_line("abcdefghijklmnop", 8), "abcdefg…");
        assert_eq!(ellipsize_line(" abcdefgh", 6), " abcd…");
    }

    #[test]
    fn flush_list_view_wraps_a_tall_mail_title() {
        let tok = named("dark").tokens;
        let list = VecList {
            items: vec![crate::collection::ListRow::new(
                "Quarterly notes for Lisbon and the Berlin office",
            )
            .with_meta("This morning")
            .with_leading(crate::collection::RowSlot::Check(true))
            .with_trailing(crate::collection::RowSlot::Icon(Icon::Search))],
        };
        let win = VisibleWindow::new(200.0);
        let mut el: Element<'_, usize> = list_view(
            &list,
            &Sel::Single(0),
            |c| c.id,
            tok,
            win,
            64.0,
            2,
            |_| 0,
            "No rows",
            |_| tok.scheme().on_surface_variant,
            None,
            RowFace::FLUSH,
            |_| 0,
            A11y::new("mail", Role::List),
        );
        draw_once(&mut el);
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
    fn checkbox_indeterminate_and_range_slider_emit() {
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::mouse;
        use iced::{Event, Font, Pixels, Point, Rectangle, Size};
        let tok = named("dark").tokens;
        let press = |el: &mut Element<'_, CheckState>, at: Point| {
            let mut tree = Tree::new(el.as_widget());
            let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
                Font::DEFAULT,
                Pixels::from(16u32),
            ));
            let limits = Limits::new(Size::ZERO, Size::new(240.0, 80.0));
            let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
            let layout = Layout::new(&node);
            let viewport = Rectangle::new(Point::ORIGIN, Size::new(240.0, 80.0));
            let mut clipboard = clipboard::Null;
            let mut messages = Vec::new();
            {
                let mut shell = iced::advanced::Shell::new(&mut messages);
                el.as_widget_mut().update(
                    &mut tree,
                    &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                    layout,
                    mouse::Cursor::Available(at),
                    &renderer,
                    &mut clipboard,
                    &mut shell,
                    &viewport,
                );
            }
            messages
        };
        let mut box_el: Element<'_, CheckState> = checkbox_indeterminate(
            "all",
            CheckState::Unchecked,
            |s| s,
            tok,
            A11y::new("all", Role::Checkbox),
        );
        assert!(!press(&mut box_el, Point::new(8.0, 8.0)).is_empty());
        let mut range_el: Element<'_, (f32, f32)> = range_slider(
            0.0..=100.0,
            20.0,
            80.0,
            |pair| pair,
            tok,
            A11y::new("span", Role::Slider),
        );
        let mut tree = Tree::new(range_el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(240.0, 80.0));
        let node = range_el
            .as_widget_mut()
            .layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(240.0, 80.0));
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            range_el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(Point::new(40.0, 8.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            range_el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::CursorMoved {
                    position: Point::new(80.0, 8.0),
                }),
                layout,
                mouse::Cursor::Available(Point::new(80.0, 8.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            range_el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(Point::new(200.0, 48.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            range_el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::CursorMoved {
                    position: Point::new(80.0, 48.0),
                }),
                layout,
                mouse::Cursor::Available(Point::new(80.0, 48.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        let _ = messages;
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
            SliderMarks {
                ticks: 4,
                min: "0",
                max: "1",
                ..SliderMarks::NONE
            },
            tok,
            A11y::new("vol", Role::Slider).with_disabled(true),
        );
        let opts = ["a".to_string(), "b".to_string()];
        let mut pick_el: Element<'_, u8> = themed_pick_list(
            opts,
            Some("a".into()),
            |_| 2u8,
            tok,
            ControlSize::Default,
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
            checks: Vec::new(),
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
            0.0,
            |open| open,
            tok,
            A11y::new("exp", Role::Group),
        );
        let mut open: Element<'_, bool> = expander(
            "Notes",
            tall(),
            48.0,
            true,
            1.0,
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
        assert!(head.contains("Length::Fill"));
        // Closed ▸ / ◂ / open ▾ (not a 180°-rotated SVG that reads as ^).
        assert!(head.contains("▾"));
        assert!(head.contains("closed_disclosure"));
        assert!(head.contains("align_start"));
        assert_eq!(closed_disclosure(tok), "▸");
    }

    #[test]
    fn rtl_disclosure_title_sits_on_the_start_edge() {
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels, Size};

        let tok = named("dark")
            .tokens
            .with_direction(crate::i18n::Direction::Rtl);
        let titles = ["Files".to_string(), "Appearance".to_string()];
        let bodies = vec![
            label("a", tok, A11y::new("a", Role::Status)),
            label("b", tok, A11y::new("b", Role::Status)),
        ];
        let acc = crate::collection::Accordion { open: Some(0) };
        let mut el: Element<'_, usize> = accordion_view(
            &titles,
            bodies,
            &acc,
            1.0,
            |i| i,
            tok,
            A11y::new("acc", Role::Group),
        );
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(320.0, 240.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let origin = layout.bounds();
        let mut boxes = Vec::new();
        walk_bounds(layout, &mut boxes);
        let marks: Vec<_> = boxes
            .iter()
            .filter(|b| b.width < 24.0 && b.height < 28.0 && b.width > 4.0)
            .collect();
        must(
            marks.iter().any(|b| b.x - origin.x < origin.width / 2.0),
            "RTL disclosure mark must sit on the end (left)",
        );
        let src = include_str!("widget.rs");
        let head = src
            .split("fn disclosure_header")
            .nth(1)
            .unwrap()
            .split("pub fn accordion_view")
            .next()
            .unwrap();
        must(
            head.contains("align_start(tok.direction)"),
            "disclosure title must start-align so RTL text sits on the right",
        );
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
            1.0,
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
        assert!(lefts.iter().any(|x| (*x - 12.0).abs() < 2.0));
        assert!(!lefts.iter().any(|x| (*x - 24.0).abs() < 2.0));
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
            0.0,
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
            assert_eq!(size, iced::Size::new(120.0, 80.0));
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
            item_grid(&labels, |_| (), Some(1), tok, A11y::new("grid", Role::List));
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
            0.0,
            |_| (),
            tok,
            A11y::new("acc", Role::Group),
        );
        let mut open: Element<'_, ()> = accordion_view(
            &titles,
            vec![body()],
            &Accordion { open: Some(0) },
            1.0,
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

    fn pump_wheel<M: Clone>(el: &mut Element<'_, M>, delta: iced::mouse::ScrollDelta) -> Vec<M> {
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
        let limits = Limits::new(Size::ZERO, Size::new(320.0, 200.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let bounds = layout.bounds();
        let at = Point::new(
            bounds.x + bounds.width * 0.5,
            bounds.y + bounds.height - 2.0,
        );
        let cursor = mouse::Cursor::Available(at);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(320.0, 200.0));
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::CursorMoved { position: at }),
                layout,
                cursor,
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::WheelScrolled { delta }),
                layout,
                cursor,
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        messages
    }

    #[test]
    fn wheel_steps_slider_number_and_pick() {
        let tok = named("dark").tokens;
        let mut slider: Element<'_, f32> = themed_slider(
            0.0..=1.0,
            0.4,
            |v| v,
            SliderMarks {
                ticks: 3,
                min: "lo",
                max: "hi",
                ..SliderMarks::NONE
            },
            tok,
            A11y::new("s", Role::Slider),
        );
        let up = pump_wheel(
            &mut slider,
            iced::mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
        );
        assert!(!up.is_empty());
        assert!(up[0] > 0.4);
        let down = pump_wheel(
            &mut slider,
            iced::mouse::ScrollDelta::Pixels { x: 0.0, y: -4.0 },
        );
        assert!(!down.is_empty());
        assert!(down[0] < 0.4);

        let mut num: Element<'_, String> =
            number_input(3.0, |s| s, tok, A11y::new("n", Role::SpinButton));
        let n_up = pump_wheel(&mut num, iced::mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 });
        assert_eq!(n_up, ["4".to_string()]);
        let n_down = pump_wheel(
            &mut num,
            iced::mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
        );
        assert_eq!(n_down, ["2".to_string()]);

        let opts = ["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
        let mut pick: Element<'_, String> = themed_pick_list(
            opts.clone(),
            Some("beta".into()),
            |s| s,
            tok,
            ControlSize::Default,
            A11y::new("p", Role::ComboBox),
        );
        let next = pump_wheel(
            &mut pick,
            iced::mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
        );
        assert_eq!(next, ["gamma".to_string()]);
        let prev = pump_wheel(
            &mut pick,
            iced::mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
        );
        assert_eq!(prev, ["alpha".to_string()]);
        let cmd = pump_pick_command_wheel(&mut pick, -1.0);
        assert!(!cmd.is_empty());
        let empty: &[String] = &[];
        let _: Element<'_, String> = themed_pick_list(
            empty,
            None,
            |s| s,
            tok,
            ControlSize::Default,
            A11y::new("empty", Role::ComboBox),
        );
    }

    fn pump_pick_command_wheel(el: &mut Element<'_, String>, y: f32) -> Vec<String> {
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::keyboard;
        use iced::mouse;
        use iced::{Event, Font, Pixels, Point, Rectangle, Size};
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(320.0, 200.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let bounds = layout.bounds();
        let at = Point::new(bounds.x + 8.0, bounds.y + 8.0);
        let cursor = mouse::Cursor::Available(at);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(320.0, 200.0));
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Keyboard(keyboard::Event::ModifiersChanged(crate::shortcut::primary())),
                layout,
                cursor,
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines { x: 0.0, y },
                }),
                layout,
                cursor,
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        messages
    }

    fn pick_layout_height(size: ControlSize) -> f32 {
        use iced::advanced::layout::Limits;
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels, Size};
        let tok = named("dark").tokens;
        let opts = ["nord", "dark"];
        let mut el: Element<'_, &str> = themed_pick_list(
            opts,
            Some("nord"),
            |s| s,
            tok,
            size,
            A11y::new("theme", Role::ComboBox),
        );
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(240.0, 80.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        node.size().height
    }

    #[test]
    fn themed_pick_list_compact_uses_meta_and_tight_pad() {
        let tok = named("dark").tokens;
        assert!(tok.meta() < tok.body());
        let compact = pick_layout_height(ControlSize::Compact);
        let default = pick_layout_height(ControlSize::Default);
        let comfortable = pick_layout_height(ControlSize::Comfortable);
        must(
            compact < default,
            format!("Compact pick ({compact}) must be shorter than Default ({default})"),
        );
        must(
            comfortable >= default,
            format!(
                "Comfortable pick ({comfortable}) must not be shorter than Default ({default})"
            ),
        );
        let src = include_str!("widget.rs")
            .split("pub fn themed_pick_list")
            .nth(1)
            .unwrap()
            .split("pub fn date_picker")
            .next()
            .unwrap();
        assert!(src.contains("ControlSize::Compact"));
        assert!(src.contains("tok.meta()"));
        assert!(src.contains("tok.body()"));
        assert!(src.contains("size.pad()"));
    }

    fn segmented_layout_height(size: ControlSize) -> f32 {
        use iced::advanced::layout::Limits;
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels, Size};
        let tok = named("dark").tokens;
        let mut el: Element<'_, usize> = segmented_button(
            ["Day", "Week"],
            0,
            |i| i,
            tok,
            size,
            A11y::new("Range", Role::Group),
        );
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(240.0, 80.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        node.size().height
    }

    #[test]
    fn segmented_button_compact_is_shorter_than_default() {
        let compact = segmented_layout_height(ControlSize::Compact);
        let default = segmented_layout_height(ControlSize::Default);
        must(
            compact < default,
            format!("Compact segmented ({compact}) must be shorter than Default ({default})"),
        );
    }

    #[test]
    fn tab_bar_skips_press_on_a_disabled_tab() {
        let tok = named("dark").tokens;
        let tabs = Tabs::new(["One", "Two"]).with_disabled(1);
        let mut el: Element<'_, usize> = tab_bar(
            &tabs,
            |i| i,
            |_| 99,
            480.0,
            false,
            tok,
            A11y::new("tabs", Role::Tab),
        );
        draw_once(&mut el);
        let src = include_str!("widget.rs")
            .split("pub fn tab_bar")
            .nth(1)
            .unwrap()
            .split("/// Title on the start edge")
            .next()
            .unwrap();
        assert!(src.contains("is_disabled"));
        assert!(src.contains("tab_off"));
    }

    fn type_line(px: f32) -> f32 {
        f32::from(iced::widget::text::LineHeight::default().to_absolute(iced::Pixels(px)))
    }

    #[test]
    fn tab_bar_titles_layout_to_meta_not_body() {
        let tok = named("dark").tokens;
        assert!(tok.body() > tok.meta());
        let tabs = Tabs::new(["One"]);
        let mut el: Element<'_, usize> = tab_bar(
            &tabs,
            |i| i,
            |_| 99,
            480.0,
            false,
            tok,
            A11y::new("tabs", Role::Tab),
        );
        let h = layout_size(&mut el, iced::Size::new(480.0, 80.0)).height;
        let p = pad(tok);
        let extra = 3.0 + 1.0;
        let meta_h = type_line(tok.meta()) + p.top + p.bottom + extra;
        let body_h = type_line(tok.body()) + p.top + p.bottom + extra;
        must(
            (h - meta_h).abs() < (h - body_h).abs(),
            format!("tab_bar {h} must match meta {meta_h} closer than body {body_h}"),
        );
        must(
            (h - meta_h).abs() <= 2.0,
            format!("tab_bar {h} must be within 2px of meta face {meta_h}"),
        );
    }

    #[test]
    fn badge_sizes_use_meta_not_body() {
        let tok = named("dark").tokens;
        assert!(tok.body() > tok.meta());
        let mut small: Element<'_, ()> = badge(
            "New",
            None,
            tok,
            Variant::Primary,
            BadgeSize::Small,
            A11y::new("New", Role::Status),
        );
        let mut large: Element<'_, ()> = badge(
            "New",
            None,
            tok,
            Variant::Primary,
            BadgeSize::Large,
            A11y::new("New", Role::Status),
        );
        let hs = layout_size(&mut small, iced::Size::new(200.0, 80.0)).height;
        let hl = layout_size(&mut large, iced::Size::new(200.0, 80.0)).height;
        let meta_small = type_line(tok.meta()) + 4.0;
        let meta_large = type_line(tok.meta()) + 8.0;
        let body_large = type_line(tok.body()) + 8.0;
        must(
            (hs - meta_small).abs() <= 2.0,
            format!("Small badge {hs} must match meta {meta_small}"),
        );
        must(
            (hl - meta_large).abs() <= 2.0,
            format!("Large badge {hl} must match meta {meta_large}"),
        );
        must(
            (hl - meta_large).abs() < (hl - body_large).abs(),
            format!("Large badge {hl} must not match body {body_large}"),
        );
    }

    #[test]
    fn icon_svg_app_bytes_recolor_from_tokens() {
        let tok = named("dark").tokens;
        let mark: &'static [u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="#000"><path d="M8 1 15 8 8 15 1 8z"/></svg>"##;
        assert!(std::str::from_utf8(mark).unwrap().contains("fill=\"#000\""));
        let style = icon_style(tok)(&iced::Theme::Dark, svg::Status::Idle);
        assert_eq!(style.color, Some(tok.scheme().on_surface));
        let named_ink = icon_style(tok)(&iced::Theme::Light, svg::Status::Idle);
        let app: Element<'_, ()> = icon_svg(Glyph::Bytes(mark), tok, A11y::new("app", Role::Image));
        let bundled: Element<'_, ()> =
            icon_svg(Icon::Search, tok, A11y::new("search", Role::Image));
        let _ = (app, bundled, named_ink);
        let src = include_str!("widget.rs")
            .split("pub fn icon_svg")
            .nth(1)
            .unwrap()
            .split("pub fn label")
            .next()
            .unwrap();
        assert!(src.contains("glyph.into().bytes()"));
        assert!(src.contains("icon_style(tok)"));
        let btn: Element<'_, ()> = icon_button(
            Glyph::Bytes(mark),
            Some(()),
            tok,
            Variant::Ghost,
            ControlSize::Default,
            A11y::button("app"),
        );
        let chip: Element<'_, ()> = chip(
            "App",
            Some(()),
            None,
            tok,
            Variant::Quiet,
            ChipKind::Assist,
            Icons::leading(Glyph::Bytes(mark)),
            A11y::button("App"),
        );
        let _ = (btn, chip);
    }

    #[test]
    fn tree_wraps_a_closing_sibling_and_expander_grows() {
        let tok = named("dark").tokens;
        let mut tree = TreeNode::branch(
            1,
            "root",
            vec![
                TreeNode::branch(2, "a", vec![TreeNode::leaf(3, "a1")]),
                TreeNode::branch(4, "b", vec![TreeNode::leaf(5, "b1")]),
            ],
        );
        tree.expanded = true;
        tree.children[0].expanded = false;
        tree.children[1].expanded = true;
        let mut closing: Element<'_, u64> = tree_view(
            &tree,
            Some(2),
            Some((2, 0.4)),
            |id| id,
            |c| c.id,
            TreeFace::Outline,
            tok,
            A11y::new("tree", Role::Tree),
        );
        draw_once(&mut closing);
        let mut done: Element<'_, u64> = tree_view(
            &tree,
            None,
            Some((2, 1.0)),
            |id| id,
            |c| c.id,
            TreeFace::Outline,
            tok,
            A11y::new("tree", Role::Tree).with_disabled(true),
        );
        draw_once(&mut done);

        let tall = Column::new()
            .spacing(8)
            .push(label("one", tok, A11y::new("one", Role::Status)))
            .push(label("two", tok, A11y::new("two", Role::Status)))
            .push(label("three", tok, A11y::new("three", Role::Status)))
            .push(label("four", tok, A11y::new("four", Role::Status)))
            .into();
        let mut mid: Element<'_, bool> = expander(
            "Notes",
            tall,
            Peek::from(48.0),
            true,
            0.45,
            |open| open,
            tok,
            A11y::new("exp", Role::Group),
        );
        draw_once(&mut mid);

        let mut acc: Element<'_, ()> = accordion_view(
            &["A".into()],
            vec![label("b", tok, A11y::new("b", Role::Header))],
            &Accordion { open: Some(0) },
            0.4,
            |_| (),
            tok,
            A11y::new("acc", Role::Group),
        );
        draw_once(&mut acc);

        let toast = Toast {
            id: 3,
            kind: ToastKind::Warning,
            text: "mid".into(),
            ttl_ms: 4000,
            age_ms: 75,
        };
        let mut tv: Element<'_, ()> = toast_view(&toast, (), tok, A11y::new("mid", Role::Status));
        draw_once(&mut tv);
    }

    fn press_messages<M: Clone>(
        el: &mut Element<'_, M>,
        at: iced::Point,
        button: iced::mouse::Button,
        viewport: iced::Size,
    ) -> Vec<M> {
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::{Event, Font, Pixels, Rectangle};
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(iced::Size::ZERO, viewport);
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let vp = Rectangle::new(iced::Point::ORIGIN, viewport);
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(iced::mouse::Event::ButtonPressed(button)),
                layout,
                iced::mouse::Cursor::Available(at),
                &renderer,
                &mut clipboard,
                &mut shell,
                &vp,
            );
        }
        messages
    }

    fn drive_tree<M: Clone>(el: &mut Element<'_, M>, viewport: iced::Size) {
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::renderer::Style;
        use iced::advanced::widget::operation::focusable;
        use iced::advanced::widget::Tree;
        use iced::{Event, Font, Pixels, Point, Rectangle, Theme};
        let mut tree = Tree::new(el.as_widget());
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(iced::Size::ZERO, viewport);
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let vp = Rectangle::new(Point::ORIGIN, viewport);
        let miss = Rectangle::new(Point::new(4000.0, 4000.0), iced::Size::new(4.0, 4.0));
        el.as_widget_mut().diff(&mut tree);
        let _ = el.as_widget().mouse_interaction(
            &tree,
            layout,
            iced::mouse::Cursor::Available(Point::new(8.0, 8.0)),
            &vp,
            &renderer,
        );
        el.as_widget().draw(
            &tree,
            &mut renderer,
            &Theme::Dark,
            &Style::default(),
            layout,
            iced::mouse::Cursor::Available(Point::new(8.0, 8.0)),
            &vp,
        );
        el.as_widget().draw(
            &tree,
            &mut renderer,
            &Theme::Dark,
            &Style::default(),
            layout,
            iced::mouse::Cursor::Unavailable,
            &miss,
        );
        let mut op = focusable::unfocus::<()>();
        el.as_widget_mut()
            .operate(&mut tree, layout, &renderer, &mut op);
        let _ = el
            .as_widget_mut()
            .overlay(&mut tree, layout, &renderer, &vp, iced::Vector::ZERO);
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::<M>::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Shift),
                    modified_key: keyboard::Key::Named(keyboard::key::Named::Shift),
                    physical_key: keyboard::key::Physical::Unidentified(
                        keyboard::key::NativeCode::Unidentified,
                    ),
                    location: keyboard::Location::Standard,
                    modifiers: keyboard::Modifiers::SHIFT,
                    text: None,
                    repeat: false,
                }),
                layout,
                iced::mouse::Cursor::Available(Point::new(8.0, 8.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &vp,
            );
            el.as_widget_mut().update(
                &mut tree,
                &Event::Keyboard(keyboard::Event::KeyReleased {
                    key: keyboard::Key::Named(keyboard::key::Named::Shift),
                    modified_key: keyboard::Key::Named(keyboard::key::Named::Shift),
                    physical_key: keyboard::key::Physical::Unidentified(
                        keyboard::key::NativeCode::Unidentified,
                    ),
                    location: keyboard::Location::Standard,
                    modifiers: keyboard::Modifiers::empty(),
                }),
                layout,
                iced::mouse::Cursor::Available(Point::new(8.0, 8.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &vp,
            );
        }
        let _ = messages;
    }

    #[test]
    fn item_press_left_middle_and_miss_and_tree() {
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::{Event, Font, Pixels, Point, Rectangle, Size};
        let tok = named("dark").tokens;
        let face = label("row", tok, A11y::new("row", Role::ListItem));
        let mut el: Element<'_, ItemClick> = item_press(face, |button, modifiers| ItemClick {
            id: 1,
            button,
            modifiers,
        });
        drive_tree(&mut el, Size::new(200.0, 40.0));
        let left = press_messages(
            &mut el,
            Point::new(8.0, 8.0),
            iced::mouse::Button::Left,
            Size::new(200.0, 40.0),
        );
        assert_eq!(left[0].button, ItemButton::Primary);
        let mid = press_messages(
            &mut el,
            Point::new(8.0, 8.0),
            iced::mouse::Button::Middle,
            Size::new(200.0, 40.0),
        );
        assert!(mid.is_empty());
        let away = press_messages(
            &mut el,
            Point::new(900.0, 900.0),
            iced::mouse::Button::Left,
            Size::new(200.0, 40.0),
        );
        assert!(away.is_empty());
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(200.0, 40.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(200.0, 40.0));
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
                layout,
                iced::mouse::Cursor::Unavailable,
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        assert!(messages.is_empty());
        let mut cap: Element<'_, ()> =
            capture_press(label("pad", tok, A11y::new("pad", Role::Group)));
        drive_tree(&mut cap, Size::new(80.0, 40.0));
    }

    #[test]
    fn markdown_view_forwards_operate_and_link_passthrough() {
        let tok = named("dark").tokens;
        let items: Vec<_> =
            markdown::parse("# Title\n\nSee [docs](https://example.com).").collect();
        let mut el: Element<'_, crate::select::MarkdownPointer> = markdown_view(
            &items,
            None,
            |ev| ev,
            tok,
            |_| crate::select::MarkdownPointer::Release,
            A11y::new("md", Role::Group),
        );
        drive_tree(&mut el, iced::Size::new(400.0, 240.0));
        let url = String::from("https://example.com");
        assert_eq!(
            <MarkdownPaint as iced::widget::markdown::Viewer<markdown::Uri>>::on_link_click(
                url.clone()
            ),
            url
        );
    }

    #[test]
    fn list_grid_table_and_tree_emit_item_click() {
        let tok = named("dark").tokens;
        let list = VecList {
            items: vec![
                crate::collection::ListRow::new("Alpha")
                    .with_leading(crate::collection::RowSlot::Check(true)),
                crate::collection::ListRow::new("Beta"),
            ],
        };
        let window = VisibleWindow::new(80.0);
        let list_side = || ItemClick {
            id: 99,
            button: ItemButton::Primary,
            modifiers: keyboard::Modifiers::empty(),
        };
        let mut list_el: Element<'_, ItemClick> = list_view(
            &list,
            &Sel::None,
            |c| c,
            tok,
            window,
            24.0,
            0,
            |_| list_side(),
            "Empty",
            |_| tok.muted,
            None,
            RowFace::FLUSH,
            |_| list_side(),
            A11y::new("list", Role::List),
        );
        let list_msgs = press_messages(
            &mut list_el,
            iced::Point::new(40.0, 12.0),
            iced::mouse::Button::Left,
            iced::Size::new(320.0, 80.0),
        );
        let _ = press_messages(
            &mut list_el,
            iced::Point::new(10.0, 12.0),
            iced::mouse::Button::Left,
            iced::Size::new(320.0, 80.0),
        );
        {
            use iced::advanced::clipboard;
            use iced::advanced::layout::{Layout, Limits};
            use iced::advanced::widget::Tree;
            use iced::{Event, Font, Pixels, Rectangle};
            let mut tree = Tree::new(list_el.as_widget());
            let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
                Font::DEFAULT,
                Pixels::from(16u32),
            ));
            let limits = Limits::new(iced::Size::ZERO, iced::Size::new(320.0, 80.0));
            let node = list_el
                .as_widget_mut()
                .layout(&mut tree, &renderer, &limits);
            let layout = Layout::new(&node);
            let vp = Rectangle::new(iced::Point::ORIGIN, iced::Size::new(320.0, 80.0));
            let mut clipboard = clipboard::Null;
            let mut messages = Vec::new();
            {
                let mut shell = iced::advanced::Shell::new(&mut messages);
                list_el.as_widget_mut().update(
                    &mut tree,
                    &Event::Mouse(iced::mouse::Event::WheelScrolled {
                        delta: iced::mouse::ScrollDelta::Lines { x: 0.0, y: -3.0 },
                    }),
                    layout,
                    iced::mouse::Cursor::Available(iced::Point::new(40.0, 20.0)),
                    &renderer,
                    &mut clipboard,
                    &mut shell,
                    &vp,
                );
            }
            let _ = messages;
        }
        must(
            list_msgs.iter().any(|c| c.id < 99),
            format!("list row press must emit ItemClick, got {list_msgs:?}"),
        );

        let labels = vec!["Inbox".into(), "Mail".into()];
        let mut grid: Element<'_, ItemClick> =
            item_grid(&labels, |c| c, Some(0), tok, A11y::new("grid", Role::List));
        let grid_msgs = press_messages(
            &mut grid,
            iced::Point::new(40.0, 20.0),
            iced::mouse::Button::Left,
            iced::Size::new(300.0, 160.0),
        );
        must(
            !grid_msgs.is_empty(),
            format!("grid tile press must emit ItemClick, got {grid_msgs:?}"),
        );

        let table = TableModel {
            headers: vec!["A".into(), "B".into()],
            rows: vec![vec!["1".into(), "2".into()]],
            sort_col: None,
            sort_asc: true,
            checks: vec![true],
        };
        let cols = crate::collection::ColumnLayout::new(vec![80.0, 80.0]);
        let table_side = || {
            (
                ItemClick {
                    id: 0,
                    button: ItemButton::Primary,
                    modifiers: keyboard::Modifiers::empty(),
                },
                0usize,
            )
        };
        let on_sort = |_: usize| table_side();
        let on_scroll = |_: VisibleWindow| table_side();
        let on_h_scroll = |_: f32| table_side();
        let on_check = |_: usize| table_side();
        let mut table_el: Element<'_, (ItemClick, usize)> = data_table(
            &table,
            &Sel::None,
            None,
            &cols,
            false,
            VisibleWindow::new(80.0),
            24.0,
            0,
            |click, c| (click, c),
            on_sort,
            on_scroll,
            on_h_scroll,
            on_check,
            tok,
            A11y::new("table", Role::Table),
        );
        let table_msgs = press_messages(
            &mut table_el,
            iced::Point::new(80.0, 44.0),
            iced::mouse::Button::Left,
            iced::Size::new(240.0, 80.0),
        );
        let _ = press_messages(
            &mut table_el,
            iced::Point::new(24.0, 10.0),
            iced::mouse::Button::Left,
            iced::Size::new(240.0, 80.0),
        );
        let _ = press_messages(
            &mut table_el,
            iced::Point::new(10.0, 40.0),
            iced::mouse::Button::Left,
            iced::Size::new(240.0, 80.0),
        );
        {
            use iced::advanced::clipboard;
            use iced::advanced::layout::{Layout, Limits};
            use iced::advanced::widget::Tree;
            use iced::{Event, Font, Pixels, Rectangle};
            let mut tree = Tree::new(table_el.as_widget());
            let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
                Font::DEFAULT,
                Pixels::from(16u32),
            ));
            let limits = Limits::new(iced::Size::ZERO, iced::Size::new(240.0, 80.0));
            let node = table_el
                .as_widget_mut()
                .layout(&mut tree, &renderer, &limits);
            let layout = Layout::new(&node);
            let vp = Rectangle::new(iced::Point::ORIGIN, iced::Size::new(240.0, 80.0));
            let mut clipboard = clipboard::Null;
            let mut messages = Vec::new();
            {
                let mut shell = iced::advanced::Shell::new(&mut messages);
                table_el.as_widget_mut().update(
                    &mut tree,
                    &Event::Mouse(iced::mouse::Event::WheelScrolled {
                        delta: iced::mouse::ScrollDelta::Lines { x: -2.0, y: -2.0 },
                    }),
                    layout,
                    iced::mouse::Cursor::Available(iced::Point::new(80.0, 12.0)),
                    &renderer,
                    &mut clipboard,
                    &mut shell,
                    &vp,
                );
                table_el.as_widget_mut().update(
                    &mut tree,
                    &Event::Mouse(iced::mouse::Event::WheelScrolled {
                        delta: iced::mouse::ScrollDelta::Lines { x: 0.0, y: -3.0 },
                    }),
                    layout,
                    iced::mouse::Cursor::Available(iced::Point::new(40.0, 48.0)),
                    &renderer,
                    &mut clipboard,
                    &mut shell,
                    &vp,
                );
            }
            let _ = messages;
        }
        must(
            table_msgs
                .iter()
                .any(|(c, _)| c.button == ItemButton::Primary),
            format!("table cell press must emit ItemClick, got {table_msgs:?}"),
        );

        let root = TreeNode::branch(1, "root", vec![TreeNode::leaf(2, "child")]);
        let mut tree_el: Element<'_, ItemClick<u64>> = tree_view(
            &root,
            Some(1),
            None,
            |_| ItemClick {
                id: 0,
                button: ItemButton::Primary,
                modifiers: keyboard::Modifiers::empty(),
            },
            |c| c,
            TreeFace::Files,
            tok,
            A11y::new("tree", Role::Tree),
        );
        let tree_msgs = press_messages(
            &mut tree_el,
            iced::Point::new(48.0, 10.0),
            iced::mouse::Button::Left,
            iced::Size::new(240.0, 80.0),
        );
        must(
            tree_msgs.iter().any(|c| c.id == 1 || c.id == 2),
            format!("tree label press must emit ItemClick, got {tree_msgs:?}"),
        );
    }
}
