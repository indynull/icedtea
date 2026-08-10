//! Layout recipe helpers: clamp, wrap, dock, form, overlay, scroll stick.

use iced::widget::{
    column, container, mouse_area, row, scrollable, stack, Column, Row, Space, Stack,
};
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

use super::breakpoint::Breakpoint;
use super::size::{distribute, SizePolicy};
use super::span::{cell_geometry, grid_extent, GridCell};
use super::split::{Axis, SashEvent, SplitState};
use crate::i18n::Direction;

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

/// How many items fit on one wrap line given child width, gap, and available width.
pub fn wrap_per_row(child_w: f32, gap: f32, width: f32) -> usize {
    if child_w <= 0.0 || width <= 0.0 {
        return 1;
    }
    ((width + gap) / (child_w + gap)).floor().max(1.0) as usize
}

/// How many wrap rows for `n` children of `child_w` in `width` with `gap`.
pub fn wrap_rows(n: usize, child_w: f32, gap: f32, width: f32) -> usize {
    if n == 0 {
        return 0;
    }
    if child_w <= 0.0 || width <= 0.0 {
        return 1;
    }
    n.div_ceil(wrap_per_row(child_w, gap, width))
}

/// Flow children to the next line from available `width` (not a fixed column count).
pub fn wrap<'a, M: 'a>(
    children: Vec<Element<'a, M>>,
    child_w: f32,
    gap: f32,
    width: f32,
) -> Element<'a, M> {
    let per = wrap_per_row(child_w, gap, width);
    grid(children, per, gap.max(0.0) as u32)
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
    let sizes = distribute(
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

/// Sidebar recipe given breakpoint: beside or stacked.
pub fn sidebar_mode(width: f32) -> &'static str {
    if Breakpoint::from_width(width).sidebar_beside() {
        "beside"
    } else {
        "stack"
    }
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

/// Vertical scroll with fill height.
pub fn scroll_y<'a, M: 'a>(child: Element<'a, M>) -> Element<'a, M> {
    scrollable(child).height(Length::Fill).into()
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
pub fn form<'a, M: 'a>(
    rows_in: impl IntoIterator<Item = (Element<'a, M>, Element<'a, M>)>,
    spacing: u32,
    dir: Direction,
) -> Element<'a, M> {
    let mut col = Column::new().spacing(spacing);
    for (label, field) in rows_in {
        let label = container(label).width(Length::Fixed(140.0));
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
pub fn split_view<'a, M: Clone + 'a>(
    first: Element<'a, M>,
    second: Element<'a, M>,
    state: SplitState,
    total: f32,
    on_sash: impl Fn(SashEvent) -> M + 'a,
) -> Element<'a, M> {
    let (a, sash, b) = split_sizes(state, total);
    let axis = state.axis;
    let grip = mouse_area(match axis {
        Axis::Horizontal => Space::new().width(Length::Fixed(sash)).height(Length::Fill),
        Axis::Vertical => Space::new().height(Length::Fixed(sash)).width(Length::Fill),
    })
    .on_press(on_sash(SashEvent::Press))
    .interaction(match axis {
        Axis::Horizontal => iced::mouse::Interaction::ResizingHorizontally,
        Axis::Vertical => iced::mouse::Interaction::ResizingVertically,
    });
    match axis {
        Axis::Horizontal => Row::new()
            .push(container(first).width(Length::Fixed(a.max(1.0))))
            .push(grip)
            .push(
                container(second)
                    .width(Length::Fixed(b.max(1.0)))
                    .width(Length::Fill),
            )
            .into(),
        Axis::Vertical => Column::new()
            .push(container(first).height(Length::Fixed(a.max(1.0))))
            .push(grip)
            .push(container(second).height(Length::Fill))
            .into(),
    }
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
        assert_eq!(wrap_rows(0, 10.0, 4.0, 100.0), 0);
        assert_eq!(wrap_per_row(20.0, 4.0, 100.0), 4);
        assert_eq!(wrap_rows(5, 20.0, 4.0, 100.0), 2);
        assert_eq!(wrap_rows(3, 0.0, 4.0, 100.0), 1);
        assert_eq!(wrap_per_row(0.0, 4.0, 100.0), 1);
        assert_eq!(wrap_per_row(20.0, 4.0, 0.0), 1);
        assert_eq!(wrap_rows(3, 20.0, 4.0, 0.0), 1);
        let (def, min) = window_size_from_dock(DockSpec::default());
        assert!(def.width >= min.width);
        let (l, f) = form_columns(400.0, 140.0);
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
        assert_eq!(sidebar_mode(500.0), "stack");
        assert_eq!(sidebar_mode(900.0), "beside");
        let st = SplitState::new(Axis::Horizontal, 0.4);
        let (a, sash, b) = split_sizes(st, 206.0);
        assert!((a + sash + b - 206.0).abs() < 0.01);
        let _ = padding(8);
        let _ = DockSpec::default();
        assert_eq!(FILL, Length::Fill);
        assert_eq!(SHRINK, Length::Shrink);
        assert_eq!(fixed(260.0), Length::Fixed(260.0));
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
        let _: Element<'_, ()> = overlay_center(t().into(), t().into());
        let _: Element<'_, ()> = scroll_y(t().into());
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
        let _: Element<'_, ()> = wrap(vec![t().into(), t().into(), t().into()], 40.0, 8.0, 100.0);
        let _: Element<'_, ()> = grid(vec![], 2, 8);
        let _: Element<'_, ()> = form([(t().into(), t().into())], 8, crate::i18n::Direction::Ltr);
        let _: Element<'_, ()> = form([(t().into(), t().into())], 8, crate::i18n::Direction::Rtl);
        let _: Element<'_, ()> = stack_child(vec![t().into(), t().into()], 1);
        let _: Element<'_, ()> = stack_child(vec![], 3);
        let _: Element<'_, ()> = split_view(
            t().into(),
            t().into(),
            SplitState::new(Axis::Horizontal, 0.3),
            400.0,
            |_| (),
        );
        let _: Element<'_, ()> = split_view(
            t().into(),
            t().into(),
            SplitState::new(Axis::Vertical, 0.3),
            400.0,
            |_| (),
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
        );
        paint(&mut sv);
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
