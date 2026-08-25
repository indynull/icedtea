//! Layout recipe helpers: clamp, wrap, dock, form, overlay, scroll stick.

use iced::widget::{column, container, mouse_area, row, stack, Column, Row, Space, Stack};
use iced::{Alignment, Element, Length, Padding};

/// Fill the parent on this axis.
///
/// ```
/// assert_eq!(icedtea::layout::FILL, icedtea::iced::Length::Fill);
/// assert_eq!(icedtea::layout::SHRINK, icedtea::iced::Length::Shrink);
/// ```
pub const FILL: Length = Length::Fill;
/// Hug content on this axis.
pub const SHRINK: Length = Length::Shrink;

/// Default form label gutter (px). Stacked [`crate::widget::value_field`]
/// rows and [`form`] share this so multi-row labels align.
///
/// ```
/// assert_eq!(icedtea::layout::FORM_LABEL, 140.0);
/// ```
pub const FORM_LABEL: f32 = 140.0;

/// List pane width for [`crate::pattern::list_detail`] (px).
///
/// Fits a two-line mail title beside a checkbox and peek at body size.
///
/// ```
/// assert_eq!(icedtea::layout::LIST_PANE, 360.0);
/// ```
pub const LIST_PANE: f32 = 360.0;

/// Fixed length in pixels.
///
/// ```
/// assert_eq!(icedtea::layout::fixed(260.0), icedtea::iced::Length::Fixed(260.0));
/// ```
pub fn fixed(px: f32) -> Length {
    Length::Fixed(px)
}

fn stretches(len: Length) -> bool {
    matches!(len, Length::Fill | Length::FillPortion(_))
}

use super::size::{allocate, SizePolicy};
use super::span::{cell_geometry, grid_extent, GridCell};
use super::split::{Axis, SashEvent, SplitState};
use crate::i18n::Direction;
use crate::style;
use crate::theme::Tokens;

/// Clamp a child width to `max` while remaining centered when extra space exists.
///
/// ```
/// assert_eq!(icedtea::layout::clamp_width(400.0, 320.0), 320.0);
/// assert_eq!(icedtea::layout::clamp_width(200.0, 320.0), 200.0);
/// ```
pub fn clamp_width(available: f32, max: f32) -> f32 {
    available.min(max).max(0.0)
}

/// Side padding when clamping.
pub fn clamp_pad(available: f32, max: f32) -> f32 {
    ((available - clamp_width(available, max)) / 2.0).max(0.0)
}

/// Dock slot occupancy → default window size.
///
/// ```
/// let d = icedtea::layout::DockSpec::default();
/// assert!(d.header > 0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockSpec {
    pub header: f32,
    pub footer: f32,
    pub left: f32,
    pub right: f32,
    pub center_min: (f32, f32),
}

impl Default for DockSpec {
    fn default() -> Self {
        Self {
            header: 40.0,
            footer: 28.0,
            left: 220.0,
            right: 0.0,
            center_min: (320.0, 240.0),
        }
    }
}

/// Default and minimum window size from docked chrome + content.
///
/// ```
/// let (def, min) = icedtea::layout::window_size_from_dock(icedtea::layout::DockSpec::default());
/// assert!(def.width >= min.width);
/// ```
pub fn window_size_from_dock(spec: DockSpec) -> (iced::Size, iced::Size) {
    let min_w = spec.left + spec.right + spec.center_min.0;
    let min_h = spec.header + spec.footer + spec.center_min.1;
    let def = iced::Size::new((min_w + 120.0).max(640.0), (min_h + 80.0).max(440.0));
    let min = iced::Size::new(min_w.max(320.0), min_h.max(240.0));
    (def, min)
}

/// Form row: label width + field stretch.
pub fn form_columns(total: f32, label_pref: f32) -> (f32, f32) {
    let sizes = allocate(
        total,
        &[
            SizePolicy::between(80.0, label_pref, 220.0, 0.0),
            SizePolicy::expand(1.0),
        ],
    );
    (sizes[0], sizes[1])
}

/// Overlay card size inside a window.
pub fn overlay_card(window: iced::Size, max_w: f32, max_h: f32) -> iced::Size {
    iced::Size::new(
        (window.width * 0.72)
            .min(max_w)
            .max(280.0)
            .min(window.width),
        (window.height * 0.78)
            .min(max_h)
            .max(160.0)
            .min(window.height),
    )
}

/// Scroll stick-to-end: true when the viewport is at the tail.
pub fn stick_to_end(offset: f32, content: f32, viewport: f32, slop: f32) -> bool {
    content <= viewport || offset + viewport >= content - slop
}

/// Next offset that pins the viewport to the end.
pub fn end_offset(content: f32, viewport: f32) -> f32 {
    (content - viewport).max(0.0)
}

/// Whether a stack shows child `i`.
pub fn stack_visible(active: usize, i: usize) -> bool {
    active == i
}

/// Split first/second sizes for a recipe view.
pub fn split_sizes(state: SplitState, total: f32) -> (f32, f32, f32) {
    (
        state.first_size(total),
        state.sash,
        state.second_size(total),
    )
}

/// Dock a header, optional side panes, center, and footer.
///
/// The outer column fills its parent so a [`Length::Fill`] center
/// receives leftover height after header and footer.
pub fn dock<'a, Message: 'a, Theme, Renderer>(
    header: Option<iced::Element<'a, Message, Theme, Renderer>>,
    footer: Option<iced::Element<'a, Message, Theme, Renderer>>,
    left: Option<iced::Element<'a, Message, Theme, Renderer>>,
    right: Option<iced::Element<'a, Message, Theme, Renderer>>,
    center: iced::Element<'a, Message, Theme, Renderer>,
) -> iced::Element<'a, Message, Theme, Renderer>
where
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    let mut mid = Row::new();
    if let Some(l) = left {
        mid = mid.push(l);
    }
    mid = mid.push(center);
    if let Some(r) = right {
        mid = mid.push(r);
    }
    let mid = mid.width(Length::Fill).height(Length::Fill);
    let mut col = Column::new();
    if let Some(h) = header {
        col = col.push(h);
    }
    col = col.push(mid);
    if let Some(f) = footer {
        col = col.push(f);
    }
    col.width(Length::Fill).height(Length::Fill).into()
}

/// Center a child with optional dim overlay fill.
pub fn overlay_center<'a, M: 'a>(backdrop: Element<'a, M>, card: Element<'a, M>) -> Element<'a, M> {
    Stack::new()
        .push(backdrop)
        .push(
            container(card)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        )
        .into()
}

/// Horizontal box. RTL reverses child order.
///
/// When `width` fills, children share the row. When `height` fills,
/// each child stretches to the row height.
pub fn row_box<'a, M: 'a>(
    children: impl IntoIterator<Item = Element<'a, M>>,
    spacing: u32,
    pad: u32,
    width: Length,
    height: Length,
    dir: Direction,
) -> Element<'a, M> {
    let kids = crate::i18n::order(dir, children);
    // Equal-share columns only when the row is a filling pane, not a chrome strip.
    let kids: Vec<Element<'a, M>> = if stretches(width) && stretches(height) {
        kids.into_iter()
            .map(|c| container(c).width(FILL).height(height).into())
            .collect()
    } else {
        kids
    };
    let mut r = row(kids)
        .spacing(spacing)
        .padding(pad as f32)
        .width(width)
        .height(height);
    if !stretches(height) {
        r = r.align_y(Alignment::Center);
    }
    r.into()
}

/// Vertical box. Fill-height children take leftover space after shrink
/// siblings (a caption above a filling editor).
pub fn column_box<'a, M: 'a>(
    children: impl IntoIterator<Item = Element<'a, M>>,
    spacing: u32,
    pad: u32,
    width: Length,
    height: Length,
) -> Element<'a, M> {
    column(children)
        .spacing(spacing)
        .padding(pad as f32)
        .width(width)
        .height(height)
        .into()
}

/// Equal-fill tile pad: each cell shares the row width.
/// Equal-fill tiles. Pair with [`crate::widget::button`].
///
/// `columns` is the row length.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::layout;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let cell = widget::label("7", tok, A11y::new("7", Role::Header));
/// let _: icedtea::Element<'_, ()> = layout::pad(vec![cell], 1, 8);
/// ```
pub fn pad<'a, M: 'a>(cells: Vec<Element<'a, M>>, columns: usize, spacing: u32) -> Element<'a, M> {
    let filled: Vec<Element<'a, M>> = cells
        .into_iter()
        .map(|c| container(c).width(Length::Fill).into())
        .collect();
    grid(filled, columns, spacing)
}

/// Grid as wrapped rows of `columns` cells.
pub fn grid<'a, M: 'a>(cells: Vec<Element<'a, M>>, columns: usize, spacing: u32) -> Element<'a, M> {
    let cols = columns.max(1);
    let mut rows = Column::new().spacing(spacing);
    let mut iter = cells.into_iter().peekable();
    while iter.peek().is_some() {
        let mut r = Row::new().spacing(spacing);
        for _ in 0..cols {
            if let Some(c) = iter.next() {
                r = r.push(c);
            } else {
                r = r.push(Space::new().width(Length::Fill).height(Length::Shrink));
            }
        }
        rows = rows.push(r);
    }
    rows.into()
}

/// Form: label/field pairs stacked. RTL puts the field first.
///
/// Labels use [`FORM_LABEL`] so stacked rows share one gutter.
pub fn form<'a, M: 'a>(
    rows_in: impl IntoIterator<Item = (Element<'a, M>, Element<'a, M>)>,
    spacing: u32,
    dir: Direction,
) -> Element<'a, M> {
    let mut col = Column::new().spacing(spacing);
    for (label, field) in rows_in {
        let label = container(label)
            .width(Length::Fixed(FORM_LABEL))
            .align_x(crate::i18n::align_start(dir));
        let field = container(field).width(Length::Fill);
        let pair = match dir {
            Direction::Ltr => Row::new().push(label).push(field),
            Direction::Rtl => Row::new().push(field).push(label),
        };
        col = col.push(pair.spacing(spacing).align_y(Alignment::Center));
    }
    col.into()
}

/// Visible child of a stack.
pub fn stack_child<'a, M: 'a>(children: Vec<Element<'a, M>>, active: usize) -> Element<'a, M> {
    let i = active.min(children.len().saturating_sub(1));
    children
        .into_iter()
        .nth(i)
        .unwrap_or_else(|| Space::new().into())
}

/// Split view. The sash grip emits [`SashEvent::Press`]; Move/Release come from
/// [`super::split::listen_sash`] (window-space pointer) while pressed.
///
/// The sash is a 6 px strip: a hairline and a short centered handle, painted
/// from `tok`. An empty [`Space`] sits over the paint so hover shows the
/// resize cursor.
///
/// On a horizontal split, `first` is the start pane: left in
/// [`Direction::Ltr`], right in [`Direction::Rtl`].
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::layout::{self, Axis, SashEvent, SplitState};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// #[derive(Clone, Copy)]
/// enum Msg {
///     Sash(SashEvent),
/// }
/// let on_sash = Msg::Sash;
/// let _: icedtea::Element<'_, Msg> = layout::split_view(
///     widget::label("nav", tok, A11y::new("nav", Role::Status)),
///     widget::label("body", tok, A11y::new("body", Role::Status)),
///     SplitState::new(Axis::Horizontal, 0.3),
///     400.0,
///     on_sash,
///     tok.direction,
///     tok,
/// );
/// ```
pub fn split_view<'a, M: Clone + 'a>(
    first: Element<'a, M>,
    second: Element<'a, M>,
    state: SplitState,
    total: f32,
    on_sash: impl Fn(SashEvent) -> M + 'a,
    dir: crate::i18n::Direction,
    tok: Tokens,
) -> Element<'a, M> {
    let (a, sash, b) = split_sizes(state, total);
    let grip = sash_grip(state.axis, sash, on_sash, tok);
    match state.axis {
        Axis::Horizontal => {
            let start = container(first).width(Length::Fixed(a.max(1.0)));
            let end = container(second)
                .width(Length::Fixed(b.max(1.0)))
                .width(Length::Fill);
            let mut row = Row::new();
            match dir {
                crate::i18n::Direction::Ltr => {
                    row = row.push(start).push(grip).push(end);
                }
                crate::i18n::Direction::Rtl => {
                    row = row.push(end).push(grip).push(start);
                }
            }
            row.into()
        }
        Axis::Vertical => Column::new()
            .push(container(first).height(Length::Fixed(a.max(1.0))))
            .push(grip)
            .push(container(second).height(Length::Fill))
            .into(),
    }
}

fn sash_face<'a, M: 'a>(axis: Axis, sash: f32, tok: Tokens) -> Element<'a, M> {
    let outline = tok.scheme().outline;
    let handle_len = (tok.density.gap() * 3.0).max(crate::density::GRID as f32 * 4.0);
    match axis {
        Axis::Horizontal => {
            let line = container(Space::new())
                .width(Length::Fixed(1.0))
                .height(Length::Fill)
                .style(move |_| style::fill(outline, outline));
            let handle = container(Space::new())
                .width(Length::Fixed(2.0))
                .height(Length::Fixed(handle_len))
                .style(move |_| style::fill(outline, outline));
            Stack::new()
                .width(Length::Fixed(sash))
                .height(Length::Fill)
                .push(
                    container(line)
                        .width(Length::Fixed(sash))
                        .height(Length::Fill)
                        .center_x(Length::Fill),
                )
                .push(
                    container(handle)
                        .width(Length::Fixed(sash))
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill),
                )
                .into()
        }
        Axis::Vertical => {
            let line = container(Space::new())
                .width(Length::Fill)
                .height(Length::Fixed(1.0))
                .style(move |_| style::fill(outline, outline));
            let handle = container(Space::new())
                .width(Length::Fixed(handle_len))
                .height(Length::Fixed(2.0))
                .style(move |_| style::fill(outline, outline));
            Stack::new()
                .width(Length::Fill)
                .height(Length::Fixed(sash))
                .push(
                    container(line)
                        .width(Length::Fill)
                        .height(Length::Fixed(sash))
                        .center_y(Length::Fill),
                )
                .push(
                    container(handle)
                        .width(Length::Fill)
                        .height(Length::Fixed(sash))
                        .center_x(Length::Fill)
                        .center_y(Length::Fill),
                )
                .into()
        }
    }
}

fn sash_grip<'a, M: Clone + 'a>(
    axis: Axis,
    sash: f32,
    on_sash: impl Fn(SashEvent) -> M + 'a,
    tok: Tokens,
) -> Element<'a, M> {
    let face = sash_face(axis, sash, tok);
    let hit = match axis {
        Axis::Horizontal => Space::new().width(Length::Fixed(sash)).height(Length::Fill),
        Axis::Vertical => Space::new().height(Length::Fixed(sash)).width(Length::Fill),
    };
    let grip = mouse_area(hit)
        .on_press(on_sash(SashEvent::Press))
        .interaction(match axis {
            Axis::Horizontal => iced::mouse::Interaction::ResizingHorizontally,
            Axis::Vertical => iced::mouse::Interaction::ResizingVertically,
        });
    let (w, h) = match axis {
        Axis::Horizontal => (Length::Fixed(sash), Length::Fill),
        Axis::Vertical => (Length::Fill, Length::Fixed(sash)),
    };
    Stack::new().width(w).height(h).push(face).push(grip).into()
}

/// Place children on a grid using [`GridCell`] spans (pixel offsets).
pub fn grid_spanned<'a, M: 'a>(
    items: Vec<(GridCell, Element<'a, M>)>,
    cell_w: f32,
    cell_h: f32,
    gap: f32,
) -> Element<'a, M> {
    let cells: Vec<GridCell> = items.iter().map(|(c, _)| *c).collect();
    let (tw, th) = grid_extent(&cells, cell_w, cell_h, gap);
    let mut layers = Vec::new();
    for (cell, child) in items {
        let (x, y, w, h) = cell_geometry(&cell, cell_w, cell_h, gap);
        layers.push(
            container(container(child).width(w).height(h))
                .padding(Padding {
                    top: y,
                    right: 0.0,
                    bottom: 0.0,
                    left: x,
                })
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        );
    }
    if layers.is_empty() {
        return Space::new().width(tw.max(1.0)).height(th.max(1.0)).into();
    }
    container(stack(layers))
        .width(tw.max(1.0))
        .height(th.max(1.0))
        .into()
}

/// Clamp container: centered max width.
pub fn clamp<'a, M: 'a>(child: Element<'a, M>, max: f32) -> Element<'a, M> {
    container(child)
        .width(Length::Fill)
        .max_width(max)
        .center_x(Length::Fill)
        .into()
}

/// Uniform padding.
pub fn padding(all: u32) -> Padding {
    Padding::new(all as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::text;

    #[test]
    fn recipe_math() {
        assert_eq!(clamp_width(400.0, 320.0), 320.0);
        assert_eq!(clamp_width(10.0, 320.0), 10.0);
        assert_eq!(clamp_pad(400.0, 320.0), 40.0);

        let (def, min) = window_size_from_dock(DockSpec::default());
        assert!(def.width >= min.width);
        assert_eq!(FORM_LABEL, 140.0);
        let (l, f) = form_columns(400.0, FORM_LABEL);
        assert!(l >= 80.0 && f > 0.0);
        let card = overlay_card(iced::Size::new(800.0, 600.0), 640.0, 480.0);
        assert!(card.width <= 640.0);
        assert!(stick_to_end(0.0, 10.0, 20.0, 4.0));
        assert!(!stick_to_end(0.0, 100.0, 20.0, 4.0));
        assert!(stick_to_end(80.0, 100.0, 20.0, 4.0));
        assert_eq!(end_offset(100.0, 20.0), 80.0);
        assert_eq!(end_offset(10.0, 20.0), 0.0);
        assert!(stack_visible(1, 1));
        assert!(!stack_visible(1, 0));
        assert!(!crate::layout::Breakpoint::from_width(500.0).sidebar_beside());
        assert!(crate::layout::Breakpoint::from_width(900.0).sidebar_beside());
        let st = SplitState::new(Axis::Horizontal, 0.4);
        let (a, sash, b) = split_sizes(st, 206.0);
        assert!((a + sash + b - 206.0).abs() < 0.01);
        let _ = padding(8);
        let _ = DockSpec::default();
        assert_eq!(FILL, Length::Fill);
        assert_eq!(SHRINK, Length::Shrink);
        assert_eq!(fixed(260.0), Length::Fixed(260.0));
        assert_eq!(LIST_PANE, 360.0);
        assert_eq!(fixed(LIST_PANE), Length::Fixed(360.0));
        assert!(stretches(FILL));
        assert!(stretches(Length::FillPortion(1)));
        assert!(!stretches(SHRINK));
        assert!(!stretches(fixed(16.0)));
    }

    #[test]
    fn recipe_widgets_build() {
        let t = || text("x");
        let _: Element<'_, ()> = dock(
            Some(t().into()),
            Some(t().into()),
            Some(t().into()),
            Some(t().into()),
            t().into(),
        );
        let _: Element<'_, ()> = dock(None, None, None, None, t().into());
        let split_src = include_str!("recipes.rs");
        let grip = split_src
            .split("pub fn split_view")
            .nth(1)
            .unwrap()
            .split("pub fn grid_spanned")
            .next()
            .unwrap();
        assert!(grip.contains(".on_press(on_sash(SashEvent::Press))"));
        assert!(!grip.contains("on_move"), "grip must not drive sash move");
        assert!(grip.contains("tok: Tokens"));
        assert!(grip.contains("Space::new()"));
        let _: Element<'_, ()> = overlay_center(t().into(), t().into());
        let _: Element<'_, ()> = row_box(
            [t().into(), t().into()],
            8,
            8,
            FILL,
            SHRINK,
            crate::i18n::Direction::Ltr,
        );
        let _: Element<'_, ()> = row_box(
            [t().into(), t().into()],
            8,
            8,
            FILL,
            FILL,
            crate::i18n::Direction::Rtl,
        );
        let _: Element<'_, ()> = row_box(
            [t().into()],
            8,
            8,
            fixed(120.0),
            SHRINK,
            crate::i18n::Direction::Ltr,
        );
        let _: Element<'_, ()> = column_box([t().into()], 8, 8, FILL, FILL);
        let _: Element<'_, ()> = column_box([t().into()], 8, 8, SHRINK, SHRINK);
        let _: Element<'_, ()> = grid(vec![t().into(), t().into(), t().into()], 2, 8);
        let _: Element<'_, ()> = pad(vec![t().into(), t().into(), t().into(), t().into()], 4, 8);
        let _: Element<'_, ()> = pad(vec![], 4, 8);
        let _: Element<'_, ()> = crate::layout::wrap(
            [
                crate::layout::Slot::hug(t()),
                crate::layout::Slot::hug(t()),
                crate::layout::Slot::hug(t()),
            ],
            crate::layout::BoxOpts {
                gap: 8.0,
                line_gap: 8.0,
                ..crate::layout::BoxOpts::new()
            },
            crate::i18n::Direction::Ltr,
        );
        let _: Element<'_, ()> = crate::layout::wrap(
            [crate::layout::Slot::hug(t()), crate::layout::Slot::hug(t())],
            crate::layout::BoxOpts {
                gap: 8.0,
                line_gap: 8.0,
                ..crate::layout::BoxOpts::new()
            },
            crate::i18n::Direction::Rtl,
        );
        let _: Element<'_, ()> = grid(vec![], 2, 8);
        let _: Element<'_, ()> = form([(t().into(), t().into())], 8, crate::i18n::Direction::Ltr);
        let _: Element<'_, ()> = form([(t().into(), t().into())], 8, crate::i18n::Direction::Rtl);
        let form_src = include_str!("recipes.rs")
            .split("pub fn form<")
            .nth(1)
            .unwrap()
            .split("/// Visible child")
            .next()
            .unwrap();
        assert!(form_src.contains("align_start(dir)"));
        let _: Element<'_, ()> = stack_child(vec![t().into(), t().into()], 1);
        let _: Element<'_, ()> = stack_child(vec![], 3);
        let tok = crate::theme::named("dark").tokens;
        let _: Element<'_, ()> = split_view(
            t().into(),
            t().into(),
            SplitState::new(Axis::Horizontal, 0.3),
            400.0,
            |_| (),
            crate::i18n::Direction::Ltr,
            tok,
        );
        let _: Element<'_, ()> = split_view(
            t().into(),
            t().into(),
            SplitState::new(Axis::Horizontal, 0.3),
            400.0,
            |_| (),
            crate::i18n::Direction::Rtl,
            tok,
        );
        let _: Element<'_, ()> = split_view(
            t().into(),
            t().into(),
            SplitState::new(Axis::Vertical, 0.3),
            400.0,
            |_| (),
            crate::i18n::Direction::Ltr,
            tok,
        );
        let _: Element<'_, ()> = grid_spanned(
            vec![
                (GridCell::new(0, 0).span(2, 1), t().into()),
                (GridCell::new(0, 1), t().into()),
            ],
            40.0,
            20.0,
            4.0,
        );
        let _: Element<'_, ()> = grid_spanned(vec![], 40.0, 20.0, 4.0);
        let _: Element<'_, ()> = clamp(t().into(), 480.0);
        fn paint(el: &mut Element<'_, ()>) {
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
            let limits = Limits::new(Size::ZERO, Size::new(400.0, 300.0));
            let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
            let layout = Layout::new(&node);
            let viewport = Rectangle::new(Point::ORIGIN, Size::new(400.0, 300.0));
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
        let mut d = dock(
            Some(t().into()),
            Some(t().into()),
            Some(t().into()),
            Some(t().into()),
            t().into(),
        );
        paint(&mut d);
        let mut ov = overlay_center(t().into(), t().into());
        paint(&mut ov);
        let mut sv = split_view(
            t().into(),
            t().into(),
            SplitState::new(Axis::Vertical, 0.3),
            400.0,
            |_| (),
            crate::i18n::Direction::Ltr,
            tok,
        );
        paint(&mut sv);
        let mut sv_h = split_view(
            t().into(),
            t().into(),
            SplitState::new(Axis::Horizontal, 0.3),
            400.0,
            |_| (),
            crate::i18n::Direction::Ltr,
            tok,
        );
        paint(&mut sv_h);
        let mut gs = grid_spanned(vec![], 40.0, 20.0, 4.0);
        paint(&mut gs);
        let mut g = grid(vec![t().into(), t().into(), t().into()], 2, 8);
        paint(&mut g);
        let mut sc = stack_child(vec![], 3);
        paint(&mut sc);
    }

    #[test]
    fn dock_center_takes_leftover_height() {
        use iced::advanced::layout::Limits;
        use iced::advanced::widget::{Tree, Widget};
        use iced::widget::{container, text, Space};
        use iced::{Font, Pixels, Size, Theme};

        type El<'a> = iced::Element<'a, (), Theme, iced_tiny_skia::Renderer>;
        let header: El<'_> = text("H").into();
        let footer: El<'_> = text("F").into();
        let center: El<'_> = container(Space::new()).width(FILL).height(FILL).into();
        let mut docked: El<'_> = dock(Some(header), Some(footer), None, None, center);
        let mut tree = Tree::new(docked.as_widget());
        let renderer = iced_tiny_skia::Renderer::new(Font::DEFAULT, Pixels::from(16u32));
        let max = Size::new(400.0, 300.0);
        let node = Widget::<(), Theme, iced_tiny_skia::Renderer>::layout(
            docked.as_widget_mut(),
            &mut tree,
            &renderer,
            &Limits::new(Size::ZERO, max),
        );
        assert!((node.size().width - 400.0).abs() < 0.5);
        assert!((node.size().height - 300.0).abs() < 0.5);
        let kids = node.children();
        assert_eq!(kids.len(), 3);
        let header_h = kids[0].size().height;
        let mid_h = kids[1].size().height;
        let footer_h = kids[2].size().height;
        assert!(header_h > 0.0 && header_h < 80.0);
        assert!(footer_h > 0.0 && footer_h < 80.0);
        assert!((header_h + mid_h + footer_h - 300.0).abs() < 1.0);
        assert!(mid_h > 200.0);

        let header: El<'_> = text("H").into();
        let footer: El<'_> = text("F").into();
        let left: El<'_> = text("L").into();
        let right: El<'_> = text("R").into();
        let center: El<'_> = container(Space::new()).width(FILL).height(FILL).into();
        let mut sided: El<'_> = dock(Some(header), Some(footer), Some(left), Some(right), center);
        let mut tree = Tree::new(sided.as_widget());
        let node = Widget::<(), Theme, iced_tiny_skia::Renderer>::layout(
            sided.as_widget_mut(),
            &mut tree,
            &renderer,
            &Limits::new(Size::ZERO, max),
        );
        assert!((node.size().height - 300.0).abs() < 0.5);
        assert!(node.children()[1].children().len() >= 3);
    }

    #[test]
    fn fill_editor_in_dock_takes_leftover_height() {
        use iced::advanced::layout::Limits;
        use iced::advanced::widget::{Tree, Widget};
        use iced::widget::text_editor::Content;
        use iced::widget::{column, container, text};
        use iced::{Font, Pixels, Size, Theme};

        type El<'a> = iced::Element<'a, (), Theme, iced_tiny_skia::Renderer>;
        let content = Content::<iced_tiny_skia::Renderer>::with_text("# hi\n\nbody\n");
        let editor: El<'_> = {
            let e = iced::widget::text_editor(&content).height(FILL);
            container(e).width(FILL).height(FILL).into()
        };
        let source: El<'_> = column![text("Source"), editor]
            .spacing(4)
            .padding(8)
            .width(FILL)
            .height(FILL)
            .into();
        let mut docked: El<'_> = dock(
            Some(text("menu").into()),
            Some(text("status").into()),
            None,
            None,
            source,
        );
        let mut tree = Tree::new(docked.as_widget());
        let renderer = iced_tiny_skia::Renderer::new(Font::DEFAULT, Pixels::from(16u32));
        let max = Size::new(400.0, 300.0);
        let node = Widget::<(), Theme, iced_tiny_skia::Renderer>::layout(
            docked.as_widget_mut(),
            &mut tree,
            &renderer,
            &Limits::new(Size::ZERO, max),
        );
        assert!((node.size().height - 300.0).abs() < 0.5);
        let kids = node.children();
        assert_eq!(kids.len(), 3);
        let mid = &kids[1];
        assert!(mid.size().height > 200.0);
        let col_kids = mid.children();
        assert!(!col_kids.is_empty());
        let source_kids = col_kids[0].children();
        assert!(source_kids.len() >= 2);
        assert!(source_kids[1].size().height > 150.0);
    }
}
