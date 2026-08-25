//! Measuring box and wrap: hug, share leftover, pack, reflow.

use iced::advanced::layout::{Layout, Limits, Node};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::{Element, Event, Length, Padding, Point, Rectangle, Size, Vector};

use super::size::{allocate, SizePolicy};
use super::split::Axis;
use crate::i18n::Direction;

/// How leftover space sits after children take their share.
///
/// ```
/// use icedtea::layout::Pack;
/// assert_ne!(Pack::Start, Pack::Between);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pack {
    /// Children sit on the start edge.
    #[default]
    Start,
    /// Children sit on the end edge.
    End,
    /// Leftover is split before the first and after the last child.
    Center,
    /// Leftover is split between children.
    Between,
}

/// How children sit on the cross axis.
///
/// ```
/// use icedtea::layout::Cross;
/// assert_ne!(Cross::Stretch, Cross::Start);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cross {
    /// Fill the box cross size (the line, or the whole pack when that
    /// axis is [`Length::Fill`]).
    #[default]
    Stretch,
    /// Sit on the start of the cross axis.
    Start,
    /// Sit in the middle of the cross axis.
    Center,
    /// Sit on the end of the cross axis.
    End,
}

/// Optional extras for [`pack`] and [`wrap`].
///
/// ```
/// use icedtea::layout::{BoxOpts, Pack};
/// assert_eq!(BoxOpts::new().pack, Pack::Start);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxOpts {
    pub axis: Axis,
    pub pack: Pack,
    pub cross: Cross,
    pub gap: f32,
    pub line_gap: f32,
    pub padding: Padding,
    pub width: Length,
    pub height: Length,
}

impl BoxOpts {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for BoxOpts {
    fn default() -> Self {
        Self {
            axis: Axis::Horizontal,
            pack: Pack::Start,
            cross: Cross::Stretch,
            gap: 0.0,
            line_gap: 0.0,
            padding: Padding::ZERO,
            width: Length::Fill,
            height: Length::Shrink,
        }
    }
}

/// One child of [`pack`] or [`wrap`].
pub struct Slot<'a, Message> {
    child: Element<'a, Message>,
    policy: SizePolicy,
}

impl<'a, Message: 'a> Slot<'a, Message> {
    /// Hug content on the main axis. Leftover goes to [`Slot::share`] siblings
    /// or to [`Pack`].
    pub fn hug(child: impl Into<Element<'a, Message>>) -> Self {
        Self {
            child: child.into(),
            policy: SizePolicy::between(0.0, 0.0, f32::INFINITY, 0.0),
        }
    }

    /// Take a share of leftover after hug siblings measure.
    pub fn share(child: impl Into<Element<'a, Message>>) -> Self {
        Self {
            child: child.into(),
            policy: SizePolicy::expand(1.0),
        }
    }

    /// Take `stretch` shares of leftover.
    pub fn share_by(child: impl Into<Element<'a, Message>>, stretch: f32) -> Self {
        Self {
            child: child.into(),
            policy: SizePolicy::expand(stretch),
        }
    }

    /// Use an explicit min / preferred / max / stretch.
    pub fn sized(child: impl Into<Element<'a, Message>>, policy: SizePolicy) -> Self {
        Self {
            child: child.into(),
            policy,
        }
    }
}

/// Measuring row or column. Children hug or share leftover; [`Pack`] places
/// what stretch does not take. Empty slots yield an empty box.
///
/// Call this for a chrome strip (search that grows between two marks) or a
/// caption above a filling editor. Disabled / empty: no slots is a zero-size
/// box. Direction only mirrors a horizontal box.
///
/// ```
/// use icedtea::iced::widget::{container, Space};
/// use icedtea::iced::Length;
/// use icedtea::i18n::Direction;
/// use icedtea::layout::{self, BoxOpts, Slot};
/// let hug = container(Space::new())
///     .width(Length::Fixed(24.0))
///     .height(Length::Fixed(24.0));
/// let grow = container(Space::new())
///     .width(Length::Fill)
///     .height(Length::Fixed(24.0));
/// let _: icedtea::Element<'_, ()> = layout::pack(
///     [Slot::hug(hug), Slot::share(grow)],
///     BoxOpts::new(),
///     Direction::Ltr,
/// );
/// ```
pub fn pack<'a, Message: 'a>(
    slots: impl IntoIterator<Item = Slot<'a, Message>>,
    opts: BoxOpts,
    dir: Direction,
) -> Element<'a, Message> {
    MeasureFlow::new(slots, opts, dir, false).into()
}

/// Measuring wrap. Each child is measured; a new line starts when the next
/// child does not fit. Unequal children are allowed. Window direction puts
/// the first child on the start edge.
///
/// Pass slots, not a uniform child width or the parent width. Empty slots
/// yield an empty box. Share slots on a line take leftover after hug
/// siblings, so a tile wall reflows when the parent crosses a column count.
///
/// ```
/// use icedtea::iced::widget::{container, Space};
/// use icedtea::iced::Length;
/// use icedtea::i18n::Direction;
/// use icedtea::layout::{self, BoxOpts, Slot};
/// let chip = container(Space::new())
///     .width(Length::Fixed(48.0))
///     .height(Length::Fixed(24.0));
/// let _: icedtea::Element<'_, ()> = layout::wrap(
///     [Slot::hug(chip)],
///     BoxOpts {
///         gap: 8.0,
///         line_gap: 8.0,
///         ..BoxOpts::new()
///     },
///     Direction::Ltr,
/// );
/// ```
pub fn wrap<'a, Message: 'a>(
    slots: impl IntoIterator<Item = Slot<'a, Message>>,
    opts: BoxOpts,
    dir: Direction,
) -> Element<'a, Message> {
    MeasureFlow::new(slots, opts, dir, true).into()
}

struct MeasureFlow<'a, Message> {
    children: Vec<Element<'a, Message>>,
    policies: Vec<SizePolicy>,
    opts: BoxOpts,
    dir: Direction,
    wrap: bool,
}

impl<'a, Message: 'a> MeasureFlow<'a, Message> {
    fn new(
        slots: impl IntoIterator<Item = Slot<'a, Message>>,
        opts: BoxOpts,
        dir: Direction,
        wrap: bool,
    ) -> Self {
        let mut children = Vec::new();
        let mut policies = Vec::new();
        for slot in slots {
            children.push(slot.child);
            policies.push(slot.policy);
        }
        Self {
            children,
            policies,
            opts,
            dir,
            wrap,
        }
    }
}

impl<'a, Message: 'a> From<MeasureFlow<'a, Message>> for Element<'a, Message> {
    fn from(value: MeasureFlow<'a, Message>) -> Self {
        Self::new(value)
    }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for MeasureFlow<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.opts.width, self.opts.height)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &Limits) -> Node {
        flow_layout(
            &mut self.children,
            &self.policies,
            &mut tree.children,
            renderer,
            limits,
            self.opts,
            self.dir,
            self.wrap,
        )
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
        viewport: &Rectangle,
    ) {
        for ((child, state), child_layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                state,
                event,
                child_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((child, state), child_layout) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            child.as_widget().draw(
                state,
                renderer,
                theme,
                style,
                child_layout,
                cursor,
                viewport,
            );
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        for ((child, state), child_layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child
                .as_widget_mut()
                .operate(state, child_layout, renderer, operation);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), child_layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, child_layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn flow_layout<Message>(
    children: &mut [Element<'_, Message>],
    policies: &[SizePolicy],
    trees: &mut [Tree],
    renderer: &iced::Renderer,
    limits: &Limits,
    opts: BoxOpts,
    dir: Direction,
    wrap: bool,
) -> Node {
    let limits = limits
        .width(opts.width)
        .height(opts.height)
        .shrink(opts.padding);
    let max = limits.max();
    let pad = opts.padding;
    if children.is_empty() {
        let size = limits.resolve(opts.width, opts.height, Size::ZERO);
        return Node::new(size.expand(pad));
    }

    let horizontal = opts.axis == Axis::Horizontal;
    let main_max = if horizontal { max.width } else { max.height };
    let cross_max = if horizontal { max.height } else { max.width };
    let gap = opts.gap.max(0.0);
    let line_gap = opts.line_gap.max(0.0);

    let mut intrinsic = Vec::with_capacity(children.len());
    for (i, child) in children.iter_mut().enumerate() {
        let policy = policies[i];
        let measure_main = if policy.stretch > 0.0 {
            if policy.min > 0.0 {
                policy.min
            } else {
                main_max
            }
        } else {
            main_max
        };
        let (mw, mh) = if horizontal {
            (measure_main, cross_max)
        } else {
            (cross_max, measure_main)
        };
        let node = child.as_widget_mut().layout(
            &mut trees[i],
            renderer,
            &Limits::new(Size::ZERO, Size::new(mw.max(0.0), mh.max(0.0))),
        );
        let size = node.size();
        let measured_main = if horizontal { size.width } else { size.height };
        let measured_cross = if horizontal { size.height } else { size.width };
        // Share with no preferred (Fill) must not report the parent width as
        // its basis — that overflows allocate() and shrinks hug siblings.
        let preferred = if policy.stretch > 0.0 && policy.preferred <= 0.0 {
            policy.min
        } else if policy.preferred > 0.0 {
            policy.preferred
        } else {
            measured_main
        };
        let base = preferred.clamp(policy.min, policy.max);
        let mut policy = policy;
        if policy.stretch <= 0.0 {
            policy.min = base;
            policy.max = base.max(policy.min);
        }
        intrinsic.push(Intrinsic {
            base,
            cross: measured_cross,
            policy,
        });
    }

    let lines = if wrap {
        bin_lines(&intrinsic, main_max, gap)
    } else {
        vec![(0..children.len()).collect()]
    };

    let mut assigned_main = vec![0.0; children.len()];
    let mut assigned_cross = vec![0.0; children.len()];
    let mut pos_main = vec![0.0; children.len()];
    let mut pos_cross = vec![0.0; children.len()];
    let mut cursor_cross = 0.0;
    let mut content_main: f32 = 0.0;

    for (line_i, line) in lines.iter().enumerate() {
        if line_i > 0 {
            cursor_cross += line_gap;
        }
        let gaps = gap * line.len().saturating_sub(1) as f32;
        let policies_line: Vec<SizePolicy> = line
            .iter()
            .map(|&i| {
                let mut p = intrinsic[i].policy;
                p.preferred = intrinsic[i].base;
                if p.stretch <= 0.0 {
                    p.max = intrinsic[i].base.max(p.min);
                }
                p
            })
            .collect();
        let sizes = allocate(main_max - gaps, &policies_line);
        let used: f32 = sizes.iter().sum::<f32>() + gaps;
        let leftover = (main_max - used).max(0.0);
        let (offset, between) = pack_offset(opts.pack, leftover, line.len());
        let line_cross = line
            .iter()
            .map(|&i| intrinsic[i].cross)
            .fold(0.0_f32, f32::max);
        let mut x = offset;
        for (k, &i) in line.iter().enumerate() {
            if k > 0 {
                x += gap + between;
            }
            assigned_main[i] = sizes[k];
            assigned_cross[i] = match opts.cross {
                Cross::Stretch => line_cross.max(intrinsic[i].cross),
                _ => intrinsic[i].cross,
            };
            pos_main[i] = x;
            let extra_cross = (line_cross - assigned_cross[i]).max(0.0);
            pos_cross[i] = cursor_cross
                + match opts.cross {
                    Cross::Start | Cross::Stretch => 0.0,
                    Cross::Center => extra_cross / 2.0,
                    Cross::End => extra_cross,
                };
            x += sizes[k];
        }
        content_main = content_main.max(x);
        cursor_cross += line_cross;
    }

    let content_cross = cursor_cross;
    let (content_w, content_h) = if horizontal {
        (content_main, content_cross)
    } else {
        (content_cross, content_main)
    };
    let resolved = limits.resolve(opts.width, opts.height, Size::new(content_w, content_h));
    if !wrap && opts.cross == Cross::Stretch {
        let box_cross = if horizontal {
            resolved.height
        } else {
            resolved.width
        };
        assigned_cross.fill(box_cross);
    }
    let box_w = resolved.width;

    let mut nodes = Vec::with_capacity(children.len());
    for (i, child) in children.iter_mut().enumerate() {
        let (cw, ch) = if horizontal {
            (assigned_main[i], assigned_cross[i])
        } else {
            (assigned_cross[i], assigned_main[i])
        };
        let node = child.as_widget_mut().layout(
            &mut trees[i],
            renderer,
            &Limits::new(Size::ZERO, Size::new(cw.max(0.0), ch.max(0.0)))
                .width(Length::Fixed(cw.max(0.0)))
                .height(Length::Fixed(ch.max(0.0))),
        );
        let (x, y) = if horizontal {
            let logical = pos_main[i];
            let px = match dir {
                Direction::Ltr => logical,
                Direction::Rtl => (box_w - logical - assigned_main[i]).max(0.0),
            };
            (px + pad.left, pos_cross[i] + pad.top)
        } else {
            (pos_cross[i] + pad.left, pos_main[i] + pad.top)
        };
        nodes.push(node.move_to(Point::new(x, y)));
    }

    Node::with_children(resolved.expand(pad), nodes)
}

struct Intrinsic {
    base: f32,
    cross: f32,
    policy: SizePolicy,
}

fn bin_lines(items: &[Intrinsic], main_max: f32, gap: f32) -> Vec<Vec<usize>> {
    let mut lines = Vec::new();
    let mut cur = Vec::new();
    let mut used = 0.0;
    for (i, item) in items.iter().enumerate() {
        let need = item.base;
        let extra = if cur.is_empty() { 0.0 } else { gap };
        if !cur.is_empty() && used + extra + need > main_max + 0.01 {
            lines.push(cur);
            cur = Vec::new();
            used = 0.0;
        }
        if !cur.is_empty() {
            used += gap;
        }
        used += need;
        cur.push(i);
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

fn pack_offset(pack: Pack, leftover: f32, count: usize) -> (f32, f32) {
    if leftover <= 0.0 || count == 0 {
        return (0.0, 0.0);
    }
    match pack {
        Pack::Start => (0.0, 0.0),
        Pack::End => (leftover, 0.0),
        Pack::Center => (leftover / 2.0, 0.0),
        Pack::Between => {
            if count < 2 {
                (0.0, 0.0)
            } else {
                (0.0, leftover / (count - 1) as f32)
            }
        }
    }
}

/// Test-only sized child. Production callers pass real widgets.
#[cfg(test)]
fn block<'a, M: 'a>(w: f32, h: f32) -> Element<'a, M> {
    iced::widget::container(iced::widget::Space::new())
        .width(Length::Fixed(w))
        .height(Length::Fixed(h))
        .into()
}

#[cfg(test)]
fn layout_box(el: &mut Element<'_, ()>, max: Size) -> Node {
    use iced::advanced::widget::Tree;
    use iced::{Font, Pixels};
    let mut tree = Tree::new(el.as_widget());
    let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
        Font::DEFAULT,
        Pixels::from(16u32),
    ));
    el.as_widget_mut()
        .layout(&mut tree, &renderer, &Limits::new(Size::ZERO, max))
}

#[cfg(test)]
fn boxes(node: &Node) -> Vec<Rectangle> {
    node.children().iter().map(|c| c.bounds()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_hug_share_hug_grows_middle_at_two_widths() {
        let build = || {
            pack(
                [
                    Slot::hug(block::<()>(40.0, 10.0)),
                    Slot::share(block::<()>(10.0, 10.0)),
                    Slot::hug(block::<()>(20.0, 10.0)),
                ],
                BoxOpts {
                    gap: 0.0,
                    height: Length::Shrink,
                    ..BoxOpts::new()
                },
                Direction::Ltr,
            )
        };
        let mut narrow = build();
        let n = layout_box(&mut narrow, Size::new(200.0, 40.0));
        let b = boxes(&n);
        assert_eq!(b.len(), 3);
        assert!((b[0].width - 40.0).abs() < 0.5);
        assert!((b[2].width - 20.0).abs() < 0.5);
        assert!((b[1].width - 140.0).abs() < 1.0);
        assert!((b[0].x - 0.0).abs() < 0.5);
        assert!((b[2].x + b[2].width - 200.0).abs() < 1.0);

        let mut wide = build();
        let w = layout_box(&mut wide, Size::new(400.0, 40.0));
        let b = boxes(&w);
        assert!((b[0].width - 40.0).abs() < 0.5);
        assert!((b[2].width - 20.0).abs() < 0.5);
        assert!((b[1].width - 340.0).abs() < 1.0);
        assert!((b[1].width - 140.0).abs() > 50.0);
    }

    #[test]
    fn pack_fill_share_between_fixed_hugs_keeps_hug_widths() {
        let fill = || -> Element<'static, ()> {
            iced::widget::container(iced::widget::Space::new())
                .width(Length::Fill)
                .height(Length::Fixed(10.0))
                .into()
        };
        let build = || {
            pack(
                [
                    Slot::hug(block::<()>(40.0, 10.0)),
                    Slot::share(fill()),
                    Slot::hug(block::<()>(20.0, 10.0)),
                ],
                BoxOpts {
                    gap: 0.0,
                    height: Length::Shrink,
                    ..BoxOpts::new()
                },
                Direction::Ltr,
            )
        };
        let mut narrow = build();
        let b = boxes(&layout_box(&mut narrow, Size::new(200.0, 40.0)));
        assert_eq!(b.len(), 3);
        assert!((b[0].width - 40.0).abs() < 0.5);
        assert!((b[2].width - 20.0).abs() < 0.5);
        assert!((b[1].width - 140.0).abs() < 1.0);
        let mut wide = build();
        let b = boxes(&layout_box(&mut wide, Size::new(400.0, 40.0)));
        assert!((b[0].width - 40.0).abs() < 0.5);
        assert!((b[2].width - 20.0).abs() < 0.5);
        assert!((b[1].width - 340.0).abs() < 1.0);
    }

    #[test]
    fn pack_places_leftover_at_start_end_center_and_between() {
        let kids = || {
            [
                Slot::hug(block::<()>(20.0, 8.0)),
                Slot::hug(block::<()>(20.0, 8.0)),
            ]
        };
        let run = |pack_mode: Pack| {
            let mut el = pack(
                kids(),
                BoxOpts {
                    pack: pack_mode,
                    height: Length::Shrink,
                    ..BoxOpts::new()
                },
                Direction::Ltr,
            );
            boxes(&layout_box(&mut el, Size::new(100.0, 20.0)))
        };
        let start = run(Pack::Start);
        assert!((start[0].x - 0.0).abs() < 0.5);
        assert!((start[1].x - 20.0).abs() < 0.5);

        let end = run(Pack::End);
        assert!((end[0].x - 60.0).abs() < 1.0);
        assert!((end[1].x - 80.0).abs() < 1.0);

        let center = run(Pack::Center);
        assert!((center[0].x - 30.0).abs() < 1.0);

        let between = run(Pack::Between);
        assert!((between[0].x - 0.0).abs() < 0.5);
        assert!((between[1].x - 80.0).abs() < 1.0);
    }

    #[test]
    fn wrap_uses_more_lines_when_parent_narrows() {
        let chips = || {
            wrap(
                [
                    Slot::hug(block::<()>(50.0, 10.0)),
                    Slot::hug(block::<()>(70.0, 10.0)),
                    Slot::hug(block::<()>(40.0, 10.0)),
                    Slot::hug(block::<()>(60.0, 10.0)),
                ],
                BoxOpts {
                    gap: 8.0,
                    line_gap: 4.0,
                    height: Length::Shrink,
                    ..BoxOpts::new()
                },
                Direction::Ltr,
            )
        };
        let mut wide = chips();
        let w = layout_box(&mut wide, Size::new(400.0, 80.0));
        let wb = boxes(&w);
        let line_count = |kids: &[Rectangle]| {
            let mut ys: Vec<i32> = kids.iter().map(|r| (r.y * 2.0).round() as i32).collect();
            ys.sort_unstable();
            ys.dedup();
            ys.len()
        };
        let wide_lines = line_count(&wb);
        assert_eq!(wide_lines, 1);

        let mut narrow = chips();
        let n = layout_box(&mut narrow, Size::new(130.0, 80.0));
        let nb = boxes(&n);
        let narrow_lines = line_count(&nb);
        assert!(narrow_lines > wide_lines);
    }

    #[test]
    fn wrap_first_child_sits_on_start_edge_in_rtl() {
        let mut el = wrap(
            [
                Slot::hug(block::<()>(40.0, 10.0)),
                Slot::hug(block::<()>(30.0, 10.0)),
            ],
            BoxOpts {
                gap: 8.0,
                height: Length::Shrink,
                ..BoxOpts::new()
            },
            Direction::Rtl,
        );
        let node = layout_box(&mut el, Size::new(200.0, 40.0));
        let b = boxes(&node);
        assert!(b[0].x > b[1].x);
        assert!((b[0].x + b[0].width - 200.0).abs() < 1.0);
    }

    #[test]
    fn wrap_share_tiles_drop_columns_when_narrow() {
        let tiles = || {
            wrap(
                (0..4).map(|_| {
                    Slot::sized(
                        block::<()>(20.0, 20.0),
                        SizePolicy::between(80.0, 80.0, f32::INFINITY, 1.0),
                    )
                }),
                BoxOpts {
                    gap: 8.0,
                    line_gap: 8.0,
                    height: Length::Shrink,
                    ..BoxOpts::new()
                },
                Direction::Ltr,
            )
        };
        let mut wide = tiles();
        let w = layout_box(&mut wide, Size::new(400.0, 200.0));
        let wb = boxes(&w);
        let wide_across = wb.iter().filter(|r| r.y < 1.0).count();
        assert_eq!(wide_across, 4);

        let mut narrow = tiles();
        let n = layout_box(&mut narrow, Size::new(180.0, 200.0));
        let nb = boxes(&n);
        let narrow_across = nb.iter().filter(|r| r.y < 1.0).count();
        assert!(narrow_across < wide_across);
        assert!(narrow_across >= 1);
    }

    #[test]
    fn pack_column_gives_leftover_height_to_share() {
        let mut el = pack(
            [
                Slot::hug(block::<()>(10.0, 20.0)),
                Slot::share(block::<()>(10.0, 10.0)),
            ],
            BoxOpts {
                axis: Axis::Vertical,
                width: Length::Fill,
                height: Length::Fill,
                ..BoxOpts::new()
            },
            Direction::Ltr,
        );
        let node = layout_box(&mut el, Size::new(80.0, 200.0));
        let b = boxes(&node);
        assert!((b[0].height - 20.0).abs() < 0.5);
        assert!((b[1].height - 180.0).abs() < 1.0);
    }

    #[test]
    fn empty_pack_and_wrap_resolve_to_parent_width() {
        let mut empty_pack = pack(
            [],
            BoxOpts {
                height: Length::Shrink,
                ..BoxOpts::new()
            },
            Direction::Ltr,
        );
        let p = layout_box(&mut empty_pack, Size::new(120.0, 40.0));
        assert!((p.size().width - 120.0).abs() < 0.5);
        assert!(p.children().is_empty());
        let mut empty_wrap = wrap(
            [],
            BoxOpts {
                height: Length::Shrink,
                ..BoxOpts::new()
            },
            Direction::Rtl,
        );
        let w = layout_box(&mut empty_wrap, Size::new(80.0, 40.0));
        assert!((w.size().width - 80.0).abs() < 0.5);
    }

    #[test]
    fn share_by_gives_more_to_the_heavier_child() {
        let mut el = pack(
            [
                Slot::share_by(block::<()>(10.0, 10.0), 1.0),
                Slot::share_by(block::<()>(10.0, 10.0), 3.0),
            ],
            BoxOpts {
                height: Length::Shrink,
                ..BoxOpts::new()
            },
            Direction::Ltr,
        );
        let b = boxes(&layout_box(&mut el, Size::new(200.0, 20.0)));
        assert!(b[1].width > b[0].width * 2.0);
    }

    #[test]
    fn pack_stretch_fills_box_cross_when_height_is_fill() {
        let mut el = pack(
            [
                Slot::hug(block::<()>(20.0, 8.0)),
                Slot::hug(block::<()>(20.0, 8.0)),
            ],
            BoxOpts {
                cross: Cross::Stretch,
                height: Length::Fill,
                ..BoxOpts::new()
            },
            Direction::Ltr,
        );
        let b = boxes(&layout_box(&mut el, Size::new(80.0, 40.0)));
        assert!((b[0].height - 40.0).abs() < 0.5);
        assert!((b[1].height - 40.0).abs() < 0.5);
    }

    #[test]
    fn pack_cross_center_and_end_sit_shorter_child() {
        let run = |cross: Cross| {
            let mut el = pack(
                [
                    Slot::hug(block::<()>(20.0, 8.0)),
                    Slot::hug(block::<()>(20.0, 20.0)),
                ],
                BoxOpts {
                    cross,
                    height: Length::Shrink,
                    ..BoxOpts::new()
                },
                Direction::Ltr,
            );
            boxes(&layout_box(&mut el, Size::new(80.0, 40.0)))
        };
        let start = run(Cross::Start);
        assert!((start[0].y - 0.0).abs() < 0.5);
        let center = run(Cross::Center);
        assert!((center[0].y - 6.0).abs() < 1.0);
        let end = run(Cross::End);
        assert!((end[0].y - 12.0).abs() < 1.0);
    }

    #[test]
    fn pack_between_one_child_stays_on_start() {
        let mut el = pack(
            [Slot::hug(block::<()>(20.0, 8.0))],
            BoxOpts {
                pack: Pack::Between,
                height: Length::Shrink,
                ..BoxOpts::new()
            },
            Direction::Ltr,
        );
        let b = boxes(&layout_box(&mut el, Size::new(100.0, 20.0)));
        assert!((b[0].x - 0.0).abs() < 0.5);
    }

    #[test]
    fn pack_padding_offsets_children() {
        let mut el = pack(
            [Slot::hug(block::<()>(20.0, 8.0))],
            BoxOpts {
                padding: Padding::new(4.0),
                height: Length::Shrink,
                ..BoxOpts::new()
            },
            Direction::Ltr,
        );
        let b = boxes(&layout_box(&mut el, Size::new(80.0, 40.0)));
        assert!((b[0].x - 4.0).abs() < 0.5);
        assert!((b[0].y - 4.0).abs() < 0.5);
    }

    #[test]
    fn pack_and_wrap_draw_and_forward_pointer() {
        use iced::advanced::layout::Layout;
        use iced::advanced::renderer::Style;
        use iced::advanced::widget::Tree;
        use iced::mouse;
        use iced::{Font, Pixels, Point, Theme};
        fn paint(el: &mut Element<'_, ()>) {
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
            let _ = el.as_widget().mouse_interaction(
                &tree,
                layout,
                mouse::Cursor::Unavailable,
                &viewport,
                &renderer,
            );
            let mut clipboard = iced::advanced::clipboard::Null;
            let mut messages = Vec::<()>::new();
            let mut shell = iced::advanced::Shell::new(&mut messages);
            el.as_widget_mut().update(
                &mut tree,
                &iced::Event::Mouse(mouse::Event::CursorLeft),
                layout,
                mouse::Cursor::Unavailable,
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            el.as_widget_mut().operate(
                &mut tree,
                layout,
                &renderer,
                &mut iced::advanced::widget::operation::focusable::unfocus::<()>(),
            );
            let _ = el.as_widget_mut().overlay(
                &mut tree,
                layout,
                &renderer,
                &viewport,
                iced::Vector::ZERO,
            );
        }
        let mut p = pack(
            [
                Slot::hug(block::<()>(20.0, 8.0)),
                Slot::sized(
                    block::<()>(10.0, 8.0),
                    SizePolicy::between(10.0, 10.0, 40.0, 1.0),
                ),
            ],
            BoxOpts::new(),
            Direction::Ltr,
        );
        {
            let mut tree = Tree::new(p.as_widget());
            p.as_widget().diff(&mut tree);
        }
        paint(&mut p);
        let mut w = wrap(
            [Slot::hug(block::<()>(20.0, 8.0))],
            BoxOpts::new(),
            Direction::Rtl,
        );
        paint(&mut w);
        let _ = BoxOpts::default();
        let _ = Pack::default();
        let _ = Cross::default();
    }
}
