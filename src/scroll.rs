//! Scroll rail with a usable minimum handle.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::operation::{self, Operation};
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::widget::{Id, Widget};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::mouse;
use iced::{Background, Color, Element, Event, Length, Point, Rectangle, Size, Vector};

use crate::i18n::Direction;

use crate::chrome::{SCROLL_HANDLE_MIN, SCROLL_RAIL_WIDTH};
use crate::collection::{scroll_from_rail, scroller_span};
use crate::theme::Tokens;

#[derive(Default)]
struct State {
    dragging: Option<f32>,
}

/// Vertical rail. `on_scroll` is the content offset (pixels).
pub struct ScrollRail<'a, Message> {
    content: f32,
    viewport: f32,
    scroll: f32,
    on_scroll: Box<dyn Fn(f32) -> Message + 'a>,
    tok: Tokens,
}

impl<'a, Message> ScrollRail<'a, Message> {
    pub fn new(
        content: f32,
        viewport: f32,
        scroll: f32,
        on_scroll: impl Fn(f32) -> Message + 'a,
        tok: Tokens,
    ) -> Self {
        Self {
            content,
            viewport,
            scroll,
            on_scroll: Box::new(on_scroll),
            tok,
        }
    }
}

fn thumb(content: f32, viewport: f32, scroll: f32, rail: f32) -> (f32, f32) {
    scroller_span(content, viewport, scroll, rail, SCROLL_HANDLE_MIN)
}

impl<Message, Renderer> Widget<Message, iced::Theme, Renderer> for ScrollRail<'_, Message>
where
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(SCROLL_RAIL_WIDTH), Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(
            Length::Fixed(SCROLL_RAIL_WIDTH),
            Length::Fill,
            Size::new(SCROLL_RAIL_WIDTH, 0.0),
        );
        layout::Node::new(size)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State>();
        let rail = bounds.height;
        let (off, len) = thumb(self.content, self.viewport, self.scroll, rail);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(pos) = cursor.position() else {
                    return;
                };
                if !bounds.contains(pos) {
                    return;
                }
                let y = pos.y - bounds.y;
                if y >= off && y <= off + len {
                    state.dragging = Some(y - off);
                } else {
                    let thumb_y = (y - len / 2.0).clamp(0.0, (rail - len).max(0.0));
                    state.dragging = Some(y - thumb_y);
                    shell.publish((self.on_scroll)(scroll_from_rail(
                        self.content,
                        self.viewport,
                        thumb_y,
                        rail,
                        SCROLL_HANDLE_MIN,
                    )));
                }
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let Some(grab) = state.dragging else {
                    return;
                };
                let Some(pos) = cursor.position() else {
                    return;
                };
                let y = pos.y - bounds.y;
                let thumb_y = (y - grab).clamp(0.0, (rail - len).max(0.0));
                shell.publish((self.on_scroll)(scroll_from_rail(
                    self.content,
                    self.viewport,
                    thumb_y,
                    rail,
                    SCROLL_HANDLE_MIN,
                )));
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.dragging.take().is_some() {
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if !cursor.is_over(bounds) {
                    return;
                }
                let dy = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => -y * 24.0,
                    mouse::ScrollDelta::Pixels { y, .. } => -y,
                };
                let max_scroll = (self.content - self.viewport).max(0.0);
                shell.publish((self.on_scroll)((self.scroll + dy).clamp(0.0, max_scroll)));
                shell.capture_event();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        if state.dragging.is_some() {
            return mouse::Interaction::Grabbing;
        }
        let bounds = layout.bounds();
        let Some(pos) = cursor.position() else {
            return mouse::Interaction::default();
        };
        if !bounds.contains(pos) {
            return mouse::Interaction::default();
        }
        let (off, len) = thumb(self.content, self.viewport, self.scroll, bounds.height);
        let y = pos.y - bounds.y;
        if y >= off && y <= off + len {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::Pointer
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let scheme = self.tok.scheme();
        let rail_r = self.tok.radius(crate::m3::shape::Component::Button);
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: rail_r,
                },
                ..renderer::Quad::default()
            },
            Background::Color(scheme.surface_container_low),
        );
        let (off, len) = thumb(self.content, self.viewport, self.scroll, bounds.height);
        if len <= 0.0 {
            return;
        }
        let thumb_bounds = Rectangle {
            x: bounds.x,
            y: bounds.y + off,
            width: bounds.width,
            height: len,
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds: thumb_bounds,
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: rail_r,
                },
                ..renderer::Quad::default()
            },
            Background::Color(scheme.outline),
        );
    }
}

impl<'a, Message: 'a> From<ScrollRail<'a, Message>> for Element<'a, Message> {
    fn from(value: ScrollRail<'a, Message>) -> Self {
        Self::new(value)
    }
}

/// Hard-clip child paint to this widget's bounds.
///
/// iced's `container::clip` only tightens the draw *viewport* used for
/// culling. Card / row backgrounds still fill their full layout box, so
/// virtualized overscan paints over chrome above the list. This wrapper
/// uses `Renderer::with_layer`, the same scissor path as scrollable.
pub struct ClipLayer<'a, Message> {
    content: Element<'a, Message>,
}

impl<'a, Message> ClipLayer<'a, Message> {
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for ClipLayer<'a, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        // Follow the child: list panes are Fill×Fill; table unfrozen
        // strips are Fill×Shrink. Never force Fill height on a table row.
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let hint = self.content.as_widget().size();
        let max = limits.max();
        let mut child_max = max;
        if matches!(hint.width, Length::Fill) && max.width.is_finite() {
            child_max.width = max.width;
        } else if matches!(hint.width, Length::Fill) && !max.width.is_finite() {
            child_max.width = 1.0;
        }
        if matches!(hint.height, Length::Fill) && max.height.is_finite() {
            child_max.height = max.height;
        } else if matches!(hint.height, Length::Fill) && !max.height.is_finite() {
            child_max.height = 1.0;
        }
        let child_limits = layout::Limits::new(Size::ZERO, child_max);
        let child =
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &child_limits);
        let size = limits.resolve(hint.width, hint.height, child.size());
        if (size.width - child.size().width).abs() > 0.5
            || (size.height - child.size().height).abs() > 0.5
        {
            let child = self.content.as_widget_mut().layout(
                &mut tree.children[0],
                renderer,
                &layout::Limits::new(Size::ZERO, size),
            );
            return layout::Node::with_children(size, vec![child]);
        }
        layout::Node::with_children(size, vec![child])
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let Some(clipped) = bounds.intersection(viewport) else {
            return;
        };
        renderer.with_layer(clipped, |renderer| {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                &clipped,
            );
        });
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: 'a> From<ClipLayer<'a, Message>> for Element<'a, Message> {
    fn from(value: ClipLayer<'a, Message>) -> Self {
        Self::new(value)
    }
}

const RAIL_GAP: f32 = 4.0;

struct ThemedScrollState {
    scroll: f32,
    dragging: Option<f32>,
    /// Left button went down inside the pane (not the rail). Move and
    /// Release still reach the child after the cursor leaves.
    content_press: bool,
    pinned_end: bool,
    last_notified: Option<f32>,
    content_h: f32,
    viewport_h: f32,
}

impl Default for ThemedScrollState {
    fn default() -> Self {
        Self {
            scroll: 0.0,
            dragging: None,
            content_press: false,
            pinned_end: false,
            last_notified: None,
            content_h: 0.0,
            viewport_h: 0.0,
        }
    }
}

impl operation::Scrollable for ThemedScrollState {
    fn snap_to(&mut self, offset: operation::scrollable::RelativeOffset<Option<f32>>) {
        if let Some(y) = offset.y {
            let max = (self.content_h - self.viewport_h).max(0.0);
            self.scroll = (y.clamp(0.0, 1.0) * max).max(0.0);
            self.pinned_end = y >= 1.0;
            self.last_notified = None;
        }
    }

    fn scroll_to(&mut self, offset: operation::scrollable::AbsoluteOffset<Option<f32>>) {
        if let Some(y) = offset.y {
            self.scroll = y.max(0.0);
            self.pinned_end = false;
            self.last_notified = None;
        }
    }

    fn scroll_by(
        &mut self,
        offset: operation::scrollable::AbsoluteOffset,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        let max = (content_bounds.height - bounds.height).max(0.0);
        self.scroll = (self.scroll + offset.y).clamp(0.0, max);
        self.pinned_end = false;
        self.last_notified = None;
    }
}

/// Pane plus an explicit end-side rail. iced's `scrollable` always paints
/// its rail on the physical right; this compose follows [`Tokens::direction`].
pub struct ThemedScroll<'a, Message> {
    content: Element<'a, Message>,
    tok: crate::theme::Tokens,
    stick: bool,
    id: Option<Id>,
    on_scroll: Option<Box<dyn Fn(f32) -> Message + 'a>>,
}

impl<'a, Message> ThemedScroll<'a, Message> {
    pub fn new(
        content: impl Into<Element<'a, Message>>,
        tok: crate::theme::Tokens,
        stick: bool,
        id: Option<Id>,
        on_scroll: Option<Box<dyn Fn(f32) -> Message + 'a>>,
    ) -> Self {
        Self {
            content: content.into(),
            tok,
            stick,
            id,
            on_scroll,
        }
    }

    fn pane_width(total: f32) -> f32 {
        (total - SCROLL_RAIL_WIDTH - RAIL_GAP).max(0.0)
    }

    fn rail_x(rtl: bool, total: f32) -> f32 {
        if rtl {
            0.0
        } else {
            Self::pane_width(total) + RAIL_GAP
        }
    }

    fn pane_x(rtl: bool) -> f32 {
        if rtl {
            SCROLL_RAIL_WIDTH + RAIL_GAP
        } else {
            0.0
        }
    }
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for ThemedScroll<'a, Message>
where
    Message: 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ThemedScrollState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ThemedScrollState {
            pinned_end: self.stick,
            ..ThemedScrollState::default()
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(Length::Fill, Length::Fill, Size::ZERO);
        let rtl = self.tok.direction == Direction::Rtl;
        let pane_w = Self::pane_width(size.width);
        let viewport = size.height;
        // Min width is the pane so Shrink columns fill it. A zero min
        // left labels hugging the physical left in RTL.
        let child_min = Size::new(pane_w, 0.0);
        let child_max = Size::new(pane_w, f32::INFINITY);
        let mut child = self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(child_min, child_max),
        );
        if child.size().height < viewport {
            child = self.content.as_widget_mut().layout(
                &mut tree.children[0],
                renderer,
                &layout::Limits::new(child_min, Size::new(pane_w, viewport)),
            );
        }
        let content_h = child.size().height;
        let max_scroll = (content_h - viewport).max(0.0);
        let state = tree.state.downcast_mut::<ThemedScrollState>();
        state.content_h = content_h;
        state.viewport_h = viewport;
        if self.stick && state.pinned_end {
            state.scroll = max_scroll;
        }
        state.scroll = state.scroll.clamp(0.0, max_scroll);
        let child = child.move_to(Point::new(Self::pane_x(rtl), -state.scroll));
        let rail = layout::Node::new(Size::new(SCROLL_RAIL_WIDTH, viewport))
            .move_to(Point::new(Self::rail_x(rtl, size.width), 0.0));
        layout::Node::with_children(size, vec![child, rail])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        let bounds = layout.bounds();
        let mut children = layout.children();
        let content_layout = children.next().unwrap();
        let scroll = tree.state.downcast_ref::<ThemedScrollState>().scroll;
        let translation = Vector::new(0.0, -scroll);
        operation.scrollable(
            self.id.as_ref(),
            bounds,
            content_layout.bounds(),
            translation,
            tree.state.downcast_mut::<ThemedScrollState>(),
        );
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                content_layout,
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let mut kids = layout.children();
        let content_layout = kids.next().unwrap();
        let rail_layout = kids.next().unwrap();
        let content_h = content_layout.bounds().height;
        let view_h = bounds.height;
        let max_scroll = (content_h - view_h).max(0.0);
        let state = tree.state.downcast_mut::<ThemedScrollState>();
        let rail = rail_layout.bounds();
        let (off, len) = thumb(content_h, view_h, state.scroll, rail.height);

        let mut moved = false;
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position() {
                    if rail.contains(pos) {
                        let y = pos.y - rail.y;
                        if y >= off && y <= off + len {
                            state.dragging = Some(y - off);
                        } else {
                            let thumb_y = (y - len / 2.0).clamp(0.0, (rail.height - len).max(0.0));
                            state.dragging = Some(y - thumb_y);
                            state.scroll = scroll_from_rail(
                                content_h,
                                view_h,
                                thumb_y,
                                rail.height,
                                SCROLL_HANDLE_MIN,
                            );
                            state.pinned_end = false;
                            moved = true;
                        }
                        shell.capture_event();
                    } else if bounds.contains(pos) {
                        state.content_press = true;
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let (Some(grab), Some(pos)) = (state.dragging, cursor.position()) {
                    let y = pos.y - rail.y;
                    let thumb_y = (y - grab).clamp(0.0, (rail.height - len).max(0.0));
                    state.scroll = scroll_from_rail(
                        content_h,
                        view_h,
                        thumb_y,
                        rail.height,
                        SCROLL_HANDLE_MIN,
                    );
                    state.pinned_end = false;
                    moved = true;
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.dragging.take().is_some() {
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let over_pane = cursor.position().is_some_and(|p| bounds.contains(p));
                if over_pane && max_scroll > 0.0 {
                    let dy = match delta {
                        mouse::ScrollDelta::Lines { y, .. } => -y * 24.0,
                        mouse::ScrollDelta::Pixels { y, .. } => -y,
                    };
                    let next = (state.scroll + dy).clamp(0.0, max_scroll);
                    if (next - state.scroll).abs() > f32::EPSILON {
                        state.scroll = next;
                        state.pinned_end = self.stick && (max_scroll - next) < 1.0;
                        moved = true;
                    }
                    shell.capture_event();
                }
            }
            _ => {}
        }

        if moved || state.last_notified != Some(state.scroll) {
            if let Some(f) = &self.on_scroll {
                shell.publish(f(state.scroll));
            }
            state.last_notified = Some(state.scroll);
        }

        let over_pane = cursor.position().is_some_and(|p| bounds.contains(p));
        let pointer = matches!(event, Event::Mouse(_) | Event::Touch(_));
        let drag_out = state.content_press
            && matches!(
                event,
                Event::Mouse(
                    mouse::Event::CursorMoved { .. }
                        | mouse::Event::ButtonReleased(mouse::Button::Left)
                )
            );
        if !shell.is_event_captured() && (!pointer || over_pane || drag_out) {
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                content_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        ) {
            state.content_press = false;
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<ThemedScrollState>();
        if state.dragging.is_some() {
            return mouse::Interaction::Grabbing;
        }
        let mut kids = layout.children();
        let content_layout = kids.next().unwrap();
        let rail = kids.next().unwrap().bounds();
        if let Some(pos) = cursor.position() {
            if rail.contains(pos) {
                let (off, len) = thumb(
                    content_layout.bounds().height,
                    layout.bounds().height,
                    state.scroll,
                    rail.height,
                );
                let y = pos.y - rail.y;
                return if y >= off && y <= off + len {
                    mouse::Interaction::Grab
                } else {
                    mouse::Interaction::Pointer
                };
            }
            if !layout.bounds().contains(pos) {
                return mouse::Interaction::default();
            }
        }
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            content_layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let mut kids = layout.children();
        let content_layout = kids.next().unwrap();
        let rail_layout = kids.next().unwrap();
        let Some(clipped) = bounds.intersection(viewport) else {
            return;
        };
        renderer.with_layer(clipped, |renderer| {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                content_layout,
                cursor,
                &clipped,
            );
        });
        let state = tree.state.downcast_ref::<ThemedScrollState>();
        let rail_bounds = rail_layout.bounds();
        let scheme = self.tok.scheme();
        let rail_r = self.tok.radius(crate::m3::shape::Component::Button);
        renderer.fill_quad(
            renderer::Quad {
                bounds: rail_bounds,
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: rail_r,
                },
                ..renderer::Quad::default()
            },
            Background::Color(scheme.surface_container_low),
        );
        let (off, len) = thumb(
            content_layout.bounds().height,
            bounds.height,
            state.scroll,
            rail_bounds.height,
        );
        if len > 0.0 {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: rail_bounds.x,
                        y: rail_bounds.y + off,
                        width: rail_bounds.width,
                        height: len,
                    },
                    border: iced::Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: rail_r,
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(scheme.outline),
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: 'a> From<ThemedScroll<'a, Message>> for Element<'a, Message> {
    fn from(value: ThemedScroll<'a, Message>) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::named;
    use iced::advanced::clipboard;
    use iced::advanced::layout::Limits;
    use iced::advanced::widget::{Tree, Widget};
    use iced::{Font, Pixels, Point, Theme};

    #[test]
    fn clip_layer_uses_with_layer_scissor() {
        let src = include_str!("scroll.rs");
        let body = src.split("#[cfg(test)]").next().unwrap();
        assert!(body.contains("with_layer"));
        assert!(body.contains("struct ClipLayer"));
        assert!(body.contains("self.content.as_widget().size()"));
        assert!(body.contains("let child_min = Size::new(pane_w, 0.0)"));
    }

    #[test]
    fn themed_scroll_lays_shrink_content_at_pane_width() {
        use iced::widget::{column, Space};

        let tok = named("dark")
            .tokens
            .with_direction(crate::i18n::Direction::Rtl);
        let child: Element<'_, ()> = column![Space::new().width(40).height(16)].into();
        let mut scroll = ThemedScroll::new(child, tok, false, None, None);
        let mut tree = Tree::new(&scroll as &dyn Widget<(), iced::Theme, iced::Renderer>);
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(320.0, 240.0));
        let node = Widget::<(), iced::Theme, iced::Renderer>::layout(
            &mut scroll,
            &mut tree,
            &renderer,
            &limits,
        );
        let layout = Layout::new(&node);
        let origin = layout.bounds();
        let content = layout.children().next().unwrap().bounds();
        let fill_msg = format!(
            "RTL shrink content must fill the pane, got {} in {}",
            content.width, origin.width
        );
        assert!(content.width > origin.width * 0.85, "{fill_msg}");
    }

    #[test]
    fn clip_layer_fills_finite_parent_box() {
        use iced::widget::{container, Space};
        use iced::Length;

        let child = container(Space::new().width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill);
        let mut clip = ClipLayer::new(child);
        let mut tree = Tree::new(&clip as &dyn Widget<(), iced::Theme, iced::Renderer>);
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(320.0, 240.0));
        let node = Widget::<(), iced::Theme, iced::Renderer>::layout(
            &mut clip, &mut tree, &renderer, &limits,
        );
        assert!((node.size().height - 240.0).abs() < 0.5);
        assert!((node.size().width - 320.0).abs() < 0.5);
    }

    #[test]
    fn clip_layer_paints_and_forwards_child() {
        use iced::advanced::renderer::Style;
        use iced::widget::{container, text, Space};
        use iced::Length;

        // Fill child: parent-sized clip.
        let child = container(Space::new().width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill);
        let mut clip = ClipLayer::new(child);
        let mut tree = Tree::new(&clip as &dyn Widget<(), iced::Theme, iced::Renderer>);
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(200.0, 120.0));
        let node = Widget::<(), iced::Theme, iced::Renderer>::layout(
            &mut clip, &mut tree, &renderer, &limits,
        );
        let layout = Layout::new(&node);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(200.0, 120.0));
        let away = Rectangle::new(Point::new(500.0, 500.0), Size::new(10.0, 10.0));
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            Widget::<(), iced::Theme, iced::Renderer>::update(
                &mut clip,
                &mut tree,
                &Event::Mouse(mouse::Event::CursorMoved {
                    position: Point::new(10.0, 10.0),
                }),
                layout,
                mouse::Cursor::Available(Point::new(10.0, 10.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        let _ = Widget::<(), iced::Theme, iced::Renderer>::mouse_interaction(
            &clip,
            &tree,
            layout,
            mouse::Cursor::Available(Point::new(10.0, 10.0)),
            &viewport,
            &renderer,
        );
        Widget::<(), iced::Theme, iced::Renderer>::draw(
            &clip,
            &tree,
            &mut renderer,
            &Theme::Dark,
            &Style::default(),
            layout,
            mouse::Cursor::Available(Point::new(10.0, 10.0)),
            &viewport,
        );
        // Outside viewport: draw returns without painting.
        Widget::<(), iced::Theme, iced::Renderer>::draw(
            &clip,
            &tree,
            &mut renderer,
            &Theme::Dark,
            &Style::default(),
            layout,
            mouse::Cursor::Unavailable,
            &away,
        );
        {
            struct Nop;
            impl iced::advanced::widget::Operation<()> for Nop {
                fn traverse(
                    &mut self,
                    operate: &mut dyn FnMut(&mut dyn iced::advanced::widget::Operation<()>),
                ) {
                    operate(self);
                }
            }
            let mut op = Nop;
            Widget::<(), iced::Theme, iced::Renderer>::operate(
                &mut clip, &mut tree, layout, &renderer, &mut op,
            );
        }
        clip.diff(&mut tree);
        assert!(Widget::<(), iced::Theme, iced::Renderer>::overlay(
            &mut clip,
            &mut tree,
            layout,
            &renderer,
            &viewport,
            iced::Vector::ZERO,
        )
        .is_none());
        assert_eq!(
            Widget::<(), iced::Theme, iced::Renderer>::size_hint(&clip),
            Widget::<(), iced::Theme, iced::Renderer>::size(&clip)
        );

        // Shrink child: clip follows content size (table row path).
        let shrink = text("row").size(14);
        let mut clip2 = ClipLayer::new(shrink);
        let mut tree2 = Tree::new(&clip2 as &dyn Widget<(), iced::Theme, iced::Renderer>);
        let node2 = Widget::<(), iced::Theme, iced::Renderer>::layout(
            &mut clip2, &mut tree2, &renderer, &limits,
        );
        assert!(node2.size().height > 0.0 && node2.size().height < 120.0);

        // Infinite parent max: Fill child collapses to 1px probe then resolve.
        let fill = container(Space::new().width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill);
        let mut clip3 = ClipLayer::new(fill);
        let mut tree3 = Tree::new(&clip3 as &dyn Widget<(), iced::Theme, iced::Renderer>);
        let open = Limits::new(Size::ZERO, Size::INFINITE);
        let _ = Widget::<(), iced::Theme, iced::Renderer>::layout(
            &mut clip3, &mut tree3, &renderer, &open,
        );

        // Parent larger than shrink child: re-layout child into resolved size.
        let short = container(Space::new().width(40).height(20))
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(20.0));
        let mut clip4 = ClipLayer::new(container(short).width(Length::Fill).height(Length::Fill));
        let mut tree4 = Tree::new(&clip4 as &dyn Widget<(), iced::Theme, iced::Renderer>);
        let tight = Limits::new(Size::ZERO, Size::new(100.0, 80.0));
        let node4 = Widget::<(), iced::Theme, iced::Renderer>::layout(
            &mut clip4, &mut tree4, &renderer, &tight,
        );
        assert!((node4.size().width - 100.0).abs() < 0.5);
        assert!((node4.size().height - 80.0).abs() < 0.5);
    }

    #[test]
    fn rail_jump_and_drag_emit_scroll() {
        let tok = named("dark").tokens;
        let mut widget = ScrollRail::new(10_000.0, 400.0, 0.0, |y| y, tok);
        let mut tree = Tree::new(&widget as &dyn Widget<f32, Theme, iced_tiny_skia::Renderer>);
        let mut renderer = iced_tiny_skia::Renderer::new(Font::DEFAULT, Pixels::from(16u32));
        let limits = Limits::new(Size::ZERO, Size::new(12.0, 400.0));
        let node = Widget::<f32, Theme, iced_tiny_skia::Renderer>::layout(
            &mut widget,
            &mut tree,
            &renderer,
            &limits,
        );
        let layout = Layout::new(&node);
        let bounds = layout.bounds();
        assert!(bounds.height > 0.0);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(40.0, 400.0));
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        let mid = Point::new(bounds.x + 6.0, bounds.y + bounds.height / 2.0);
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::update(
                &mut widget,
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(mid),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        assert!(!messages.is_empty());
        assert!(*messages.last().unwrap() > 0.0);
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::update(
                &mut widget,
                &mut tree,
                &Event::Mouse(mouse::Event::CursorMoved { position: mid }),
                layout,
                mouse::Cursor::Available(Point::new(mid.x, bounds.y + bounds.height - 4.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::update(
                &mut widget,
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(mid),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::update(
                &mut widget,
                &mut tree,
                &Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines { x: 0.0, y: -3.0 },
                }),
                layout,
                mouse::Cursor::Available(mid),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        let top = Point::new(bounds.x + 6.0, bounds.y + 4.0);
        let grab = Widget::<f32, Theme, iced_tiny_skia::Renderer>::mouse_interaction(
            &widget,
            &tree,
            layout,
            mouse::Cursor::Available(top),
            &viewport,
            &renderer,
        );
        assert_eq!(grab, mouse::Interaction::Grab);
        assert_eq!(
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::mouse_interaction(
                &widget,
                &tree,
                layout,
                mouse::Cursor::Unavailable,
                &viewport,
                &renderer,
            ),
            mouse::Interaction::default()
        );
        Widget::<f32, Theme, iced_tiny_skia::Renderer>::draw(
            &widget,
            &tree,
            &mut renderer,
            &Theme::Dark,
            &iced::advanced::renderer::Style::default(),
            layout,
            mouse::Cursor::Available(mid),
            &viewport,
        );
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::update(
                &mut widget,
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(top),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        assert_eq!(
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::mouse_interaction(
                &widget,
                &tree,
                layout,
                mouse::Cursor::Available(top),
                &viewport,
                &renderer,
            ),
            mouse::Interaction::Grabbing
        );
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::update(
                &mut widget,
                &mut tree,
                &Event::Mouse(mouse::Event::CursorMoved { position: top }),
                layout,
                mouse::Cursor::Unavailable,
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::update(
                &mut widget,
                &mut tree,
                &Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                    modified_key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                    physical_key: iced::keyboard::key::Physical::Unidentified(
                        iced::keyboard::key::NativeCode::Unidentified,
                    ),
                    location: iced::keyboard::Location::Standard,
                    modifiers: iced::keyboard::Modifiers::default(),
                    text: None,
                    repeat: false,
                }),
                layout,
                mouse::Cursor::Available(top),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::update(
                &mut widget,
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Unavailable,
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::update(
                &mut widget,
                &mut tree,
                &Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels { x: 0.0, y: 20.0 },
                }),
                layout,
                mouse::Cursor::Unavailable,
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        let mut empty = ScrollRail::new(10.0, 400.0, 0.0, |y| y, tok);
        let mut etree = Tree::new(&empty as &dyn Widget<f32, Theme, iced_tiny_skia::Renderer>);
        let enode = Widget::<f32, Theme, iced_tiny_skia::Renderer>::layout(
            &mut empty, &mut etree, &renderer, &limits,
        );
        Widget::<f32, Theme, iced_tiny_skia::Renderer>::draw(
            &empty,
            &etree,
            &mut renderer,
            &Theme::Dark,
            &iced::advanced::renderer::Style::default(),
            Layout::new(&enode),
            mouse::Cursor::Unavailable,
            &viewport,
        );
        let outside = Point::new(bounds.x - 20.0, bounds.y + 10.0);
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::update(
                &mut widget,
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(mid),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::update(
                &mut widget,
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(outside),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::update(
                &mut widget,
                &mut tree,
                &Event::Mouse(mouse::Event::CursorMoved { position: mid }),
                layout,
                mouse::Cursor::Available(mid),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::update(
                &mut widget,
                &mut tree,
                &Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels { x: 0.0, y: 20.0 },
                }),
                layout,
                mouse::Cursor::Available(mid),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        assert_eq!(
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::mouse_interaction(
                &widget,
                &tree,
                layout,
                mouse::Cursor::Available(outside),
                &viewport,
                &renderer,
            ),
            mouse::Interaction::default()
        );
        assert_eq!(
            Widget::<f32, Theme, iced_tiny_skia::Renderer>::mouse_interaction(
                &widget,
                &tree,
                layout,
                mouse::Cursor::Available(mid),
                &viewport,
                &renderer,
            ),
            mouse::Interaction::Pointer
        );
        let limits0 = Limits::new(Size::ZERO, Size::new(12.0, 0.0));
        let mut flat = ScrollRail::new(10.0, 400.0, 0.0, |y| y, tok);
        let mut ftree = Tree::new(&flat as &dyn Widget<f32, Theme, iced_tiny_skia::Renderer>);
        let fnode = Widget::<f32, Theme, iced_tiny_skia::Renderer>::layout(
            &mut flat, &mut ftree, &renderer, &limits0,
        );
        Widget::<f32, Theme, iced_tiny_skia::Renderer>::draw(
            &flat,
            &ftree,
            &mut renderer,
            &Theme::Dark,
            &iced::advanced::renderer::Style::default(),
            Layout::new(&fnode),
            mouse::Cursor::Unavailable,
            &viewport,
        );
    }

    #[test]
    fn themed_scroll_rail_wheel_and_operate_drive_state() {
        use iced::advanced::renderer::Style;
        use iced::advanced::widget::operation;
        use iced::widget::{column, Space};
        use iced::Length;

        let tok = named("dark").tokens;
        let tall: Element<'_, f32> = column![
            Space::new().width(Length::Fill).height(400.0),
            Space::new().width(Length::Fill).height(400.0),
        ]
        .into();
        let id = Id::from("pane");
        let mut scroll =
            ThemedScroll::new(tall, tok, true, Some(id.clone()), Some(Box::new(|y| y)));
        let mut tree = Tree::new(&scroll as &dyn Widget<f32, iced::Theme, iced::Renderer>);
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(240.0, 160.0));
        let node = Widget::<f32, iced::Theme, iced::Renderer>::layout(
            &mut scroll,
            &mut tree,
            &renderer,
            &limits,
        );
        let layout = Layout::new(&node);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(240.0, 160.0));
        let miss = Rectangle::new(Point::new(4000.0, 4000.0), Size::new(4.0, 4.0));
        Widget::<f32, iced::Theme, iced::Renderer>::diff(&scroll, &mut tree);
        Widget::<f32, iced::Theme, iced::Renderer>::draw(
            &scroll,
            &tree,
            &mut renderer,
            &Theme::Dark,
            &Style::default(),
            layout,
            mouse::Cursor::Available(Point::new(8.0, 8.0)),
            &viewport,
        );
        Widget::<f32, iced::Theme, iced::Renderer>::draw(
            &scroll,
            &tree,
            &mut renderer,
            &Theme::Dark,
            &Style::default(),
            layout,
            mouse::Cursor::Unavailable,
            &miss,
        );
        let _ = Widget::<f32, iced::Theme, iced::Renderer>::overlay(
            &mut scroll,
            &mut tree,
            layout,
            &renderer,
            &viewport,
            iced::Vector::ZERO,
        );
        {
            let mut op = operation::scrollable::snap_to::<()>(
                id.clone(),
                operation::scrollable::RelativeOffset {
                    x: None,
                    y: Some(1.0),
                },
            );
            Widget::<f32, iced::Theme, iced::Renderer>::operate(
                &mut scroll,
                &mut tree,
                layout,
                &renderer,
                &mut op,
            );
            let mut op = operation::scrollable::scroll_to::<()>(
                id.clone(),
                operation::scrollable::AbsoluteOffset {
                    x: None,
                    y: Some(40.0),
                },
            );
            Widget::<f32, iced::Theme, iced::Renderer>::operate(
                &mut scroll,
                &mut tree,
                layout,
                &renderer,
                &mut op,
            );
            let mut op = operation::scrollable::scroll_by::<()>(
                id.clone(),
                operation::scrollable::AbsoluteOffset { x: 0.0, y: 12.0 },
            );
            Widget::<f32, iced::Theme, iced::Renderer>::operate(
                &mut scroll,
                &mut tree,
                layout,
                &renderer,
                &mut op,
            );
            let mut none = operation::scrollable::snap_to::<()>(
                id.clone(),
                operation::scrollable::RelativeOffset { x: None, y: None },
            );
            Widget::<f32, iced::Theme, iced::Renderer>::operate(
                &mut scroll,
                &mut tree,
                layout,
                &renderer,
                &mut none,
            );
            let mut none = operation::scrollable::scroll_to::<()>(
                id,
                operation::scrollable::AbsoluteOffset { x: None, y: None },
            );
            Widget::<f32, iced::Theme, iced::Renderer>::operate(
                &mut scroll,
                &mut tree,
                layout,
                &renderer,
                &mut none,
            );
        }
        let origin = layout.bounds();
        let rail = layout.children().nth(1).unwrap().bounds();
        let thumb_mid = Point::new(rail.x + rail.width / 2.0, rail.y + rail.height / 2.0);
        let track = Point::new(rail.x + rail.width / 2.0, rail.y + 4.0);
        let pane = Point::new(origin.x + 12.0, origin.y + 20.0);
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            Widget::<f32, iced::Theme, iced::Renderer>::update(
                &mut scroll,
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(thumb_mid),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        assert_eq!(
            Widget::<f32, iced::Theme, iced::Renderer>::mouse_interaction(
                &scroll,
                &tree,
                layout,
                mouse::Cursor::Available(thumb_mid),
                &viewport,
                &renderer,
            ),
            mouse::Interaction::Grabbing
        );
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            Widget::<f32, iced::Theme, iced::Renderer>::update(
                &mut scroll,
                &mut tree,
                &Event::Mouse(mouse::Event::CursorMoved {
                    position: Point::new(thumb_mid.x, thumb_mid.y + 20.0),
                }),
                layout,
                mouse::Cursor::Available(Point::new(thumb_mid.x, thumb_mid.y + 20.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            Widget::<f32, iced::Theme, iced::Renderer>::update(
                &mut scroll,
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(thumb_mid),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            Widget::<f32, iced::Theme, iced::Renderer>::update(
                &mut scroll,
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(track),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            Widget::<f32, iced::Theme, iced::Renderer>::update(
                &mut scroll,
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(track),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            Widget::<f32, iced::Theme, iced::Renderer>::update(
                &mut scroll,
                &mut tree,
                &Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -30.0 },
                }),
                layout,
                mouse::Cursor::Available(pane),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            Widget::<f32, iced::Theme, iced::Renderer>::update(
                &mut scroll,
                &mut tree,
                &Event::Mouse(mouse::Event::CursorMoved { position: pane }),
                layout,
                mouse::Cursor::Available(pane),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        let _ = Widget::<f32, iced::Theme, iced::Renderer>::mouse_interaction(
            &scroll,
            &tree,
            layout,
            mouse::Cursor::Available(thumb_mid),
            &viewport,
            &renderer,
        );
        let _ = Widget::<f32, iced::Theme, iced::Renderer>::mouse_interaction(
            &scroll,
            &tree,
            layout,
            mouse::Cursor::Available(track),
            &viewport,
            &renderer,
        );
        let _ = Widget::<f32, iced::Theme, iced::Renderer>::mouse_interaction(
            &scroll,
            &tree,
            layout,
            mouse::Cursor::Available(pane),
            &viewport,
            &renderer,
        );
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            Widget::<f32, iced::Theme, iced::Renderer>::update(
                &mut scroll,
                &mut tree,
                &Event::Keyboard(iced::keyboard::Event::ModifiersChanged(
                    iced::keyboard::Modifiers::empty(),
                )),
                layout,
                mouse::Cursor::Available(pane),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        let content_h = layout.children().next().unwrap().bounds().height;
        let view_h = layout.bounds().height;
        let (off, len) = crate::collection::scroller_span(
            content_h,
            view_h,
            0.0,
            rail.height,
            crate::chrome::SCROLL_HANDLE_MIN,
        );
        let on_thumb = Point::new(rail.x + rail.width / 2.0, rail.y + off + len / 2.0);
        let fresh_body: Element<'_, f32> = column![
            Space::new().width(Length::Fill).height(400.0),
            Space::new().width(Length::Fill).height(400.0),
        ]
        .into();
        let mut fresh = ThemedScroll::new(fresh_body, tok, false, None, Some(Box::new(|y| y)));
        let mut ftree = Tree::new(&fresh as &dyn Widget<f32, iced::Theme, iced::Renderer>);
        let fnode = Widget::<f32, iced::Theme, iced::Renderer>::layout(
            &mut fresh, &mut ftree, &renderer, &limits,
        );
        let flayout = Layout::new(&fnode);
        let frail = flayout.children().nth(1).unwrap().bounds();
        let (foff, flen) = crate::collection::scroller_span(
            flayout.children().next().unwrap().bounds().height,
            flayout.bounds().height,
            0.0,
            frail.height,
            crate::chrome::SCROLL_HANDLE_MIN,
        );
        let fthumb = Point::new(frail.x + frail.width / 2.0, frail.y + foff + flen / 2.0);
        let fpane = Point::new(flayout.bounds().x + 12.0, flayout.bounds().y + 24.0);
        assert!(!frail.contains(fpane));
        let _ = Widget::<f32, iced::Theme, iced::Renderer>::mouse_interaction(
            &fresh,
            &ftree,
            flayout,
            mouse::Cursor::Unavailable,
            &viewport,
            &renderer,
        );
        let over_pane = Widget::<f32, iced::Theme, iced::Renderer>::mouse_interaction(
            &fresh,
            &ftree,
            flayout,
            mouse::Cursor::Available(fpane),
            &viewport,
            &renderer,
        );
        assert_ne!(over_pane, mouse::Interaction::Grab);
        assert_ne!(over_pane, mouse::Interaction::Grabbing);
        assert_eq!(
            Widget::<f32, iced::Theme, iced::Renderer>::mouse_interaction(
                &fresh,
                &ftree,
                flayout,
                mouse::Cursor::Available(fthumb),
                &viewport,
                &renderer,
            ),
            mouse::Interaction::Grab
        );
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            Widget::<f32, iced::Theme, iced::Renderer>::update(
                &mut fresh,
                &mut ftree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                flayout,
                mouse::Cursor::Available(fthumb),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            Widget::<f32, iced::Theme, iced::Renderer>::update(
                &mut fresh,
                &mut ftree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                flayout,
                mouse::Cursor::Unavailable,
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
            Widget::<f32, iced::Theme, iced::Renderer>::update(
                &mut fresh,
                &mut ftree,
                &Event::Mouse(mouse::Event::WheelScrolled {
                    delta: mouse::ScrollDelta::Lines { x: 0.0, y: -2.0 },
                }),
                flayout,
                mouse::Cursor::Available(Point::new(900.0, 900.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        let _ = Widget::<f32, iced::Theme, iced::Renderer>::mouse_interaction(
            &fresh,
            &ftree,
            flayout,
            mouse::Cursor::Unavailable,
            &viewport,
            &renderer,
        );
        let _ = on_thumb;
        assert!(!messages.is_empty());
    }

    #[test]
    fn themed_scroll_drops_pointer_outside_the_pane() {
        use iced::widget::mouse_area;
        use iced::widget::Space;

        let tok = named("dark").tokens;
        let kid: Element<'_, bool> = mouse_area(
            Space::new()
                .width(Length::Fill)
                .height(Length::Fixed(400.0)),
        )
        .on_press(true)
        .into();
        let mut scroll = ThemedScroll::new(kid, tok, false, None, None);
        let mut tree = Tree::new(&scroll as &dyn Widget<bool, iced::Theme, iced::Renderer>);
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(200.0, 80.0));
        let node = Widget::<bool, iced::Theme, iced::Renderer>::layout(
            &mut scroll,
            &mut tree,
            &renderer,
            &limits,
        );
        let layout = Layout::new(&node);
        let pane = layout.bounds();
        let outside = Point::new(pane.x + 12.0, pane.y + pane.height + 40.0);
        let inside = Point::new(pane.x + 12.0, pane.y + 12.0);
        assert!(!pane.contains(outside));
        assert!(pane.contains(inside));
        let viewport = pane;
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            Widget::<bool, iced::Theme, iced::Renderer>::update(
                &mut scroll,
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(outside),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        assert!(messages.is_empty());
        assert_eq!(
            Widget::<bool, iced::Theme, iced::Renderer>::mouse_interaction(
                &scroll,
                &tree,
                layout,
                mouse::Cursor::Available(outside),
                &viewport,
                &renderer,
            ),
            mouse::Interaction::default()
        );
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            Widget::<bool, iced::Theme, iced::Renderer>::update(
                &mut scroll,
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(inside),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        assert_eq!(messages.as_slice(), &[true]);
    }

    #[test]
    fn themed_scroll_forwards_move_and_release_after_an_in_pane_press() {
        use iced::widget::mouse_area;
        use iced::widget::Space;

        let tok = named("dark").tokens;
        let kid: Element<'_, &'static str> = mouse_area(
            Space::new()
                .width(Length::Fill)
                .height(Length::Fixed(400.0)),
        )
        .on_press("press")
        .on_move(|_| "move")
        .on_release("release")
        .into();
        let mut scroll = ThemedScroll::new(kid, tok, false, None, None);
        let mut tree = Tree::new(&scroll as &dyn Widget<&str, iced::Theme, iced::Renderer>);
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(200.0, 80.0));
        let node = Widget::<&str, iced::Theme, iced::Renderer>::layout(
            &mut scroll,
            &mut tree,
            &renderer,
            &limits,
        );
        let layout = Layout::new(&node);
        let pane = layout.bounds();
        let outside = Point::new(pane.x + 12.0, pane.y + pane.height + 40.0);
        let inside = Point::new(pane.x + 12.0, pane.y + 12.0);
        let viewport = pane;
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            Widget::<&str, iced::Theme, iced::Renderer>::update(
                &mut scroll,
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(inside),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        assert!(messages.contains(&"press"));
        messages.clear();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            Widget::<&str, iced::Theme, iced::Renderer>::update(
                &mut scroll,
                &mut tree,
                &Event::Mouse(mouse::Event::CursorMoved { position: outside }),
                layout,
                mouse::Cursor::Available(outside),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        assert!(messages.contains(&"move"));
        messages.clear();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            Widget::<&str, iced::Theme, iced::Renderer>::update(
                &mut scroll,
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(outside),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        assert!(messages.contains(&"release"));
        messages.clear();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            Widget::<&str, iced::Theme, iced::Renderer>::update(
                &mut scroll,
                &mut tree,
                &Event::Mouse(mouse::Event::CursorMoved { position: outside }),
                layout,
                mouse::Cursor::Available(outside),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        assert!(!messages.contains(&"move"));
    }

    #[test]
    fn themed_scroll_forwards_keys_when_the_cursor_is_outside() {
        use iced::advanced::layout::{Limits as LayLimits, Node};
        use iced::advanced::renderer::Style;
        use std::cell::Cell;
        use std::rc::Rc;

        struct KeyHit {
            n: Rc<Cell<u32>>,
        }
        impl Widget<(), iced::Theme, iced::Renderer> for KeyHit {
            fn size(&self) -> Size<Length> {
                Size::new(Length::Fill, Length::Fixed(40.0))
            }
            fn layout(
                &mut self,
                _tree: &mut Tree,
                _renderer: &iced::Renderer,
                limits: &LayLimits,
            ) -> Node {
                Node::new(limits.resolve(Length::Fill, Length::Fixed(40.0), Size::new(40.0, 40.0)))
            }
            fn draw(
                &self,
                _tree: &Tree,
                _renderer: &mut iced::Renderer,
                _theme: &iced::Theme,
                _style: &Style,
                _layout: Layout<'_>,
                _cursor: mouse::Cursor,
                _viewport: &Rectangle,
            ) {
            }
            fn update(
                &mut self,
                _tree: &mut Tree,
                event: &Event,
                _layout: Layout<'_>,
                _cursor: mouse::Cursor,
                _renderer: &iced::Renderer,
                _clipboard: &mut dyn clipboard::Clipboard,
                _shell: &mut iced::advanced::Shell<'_, ()>,
                _viewport: &Rectangle,
            ) {
                if matches!(event, Event::Keyboard(_)) {
                    self.n.set(self.n.get() + 1);
                }
            }
        }

        let tok = named("dark").tokens;
        let hits = Rc::new(Cell::new(0));
        let probe = KeyHit { n: hits.clone() };
        let _ = Widget::<(), iced::Theme, iced::Renderer>::size(&probe);
        let kid: Element<'_, ()> = Element::new(probe);
        let mut scroll = ThemedScroll::new(kid, tok, false, None, None);
        let mut tree = Tree::new(&scroll as &dyn Widget<(), iced::Theme, iced::Renderer>);
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(200.0, 80.0));
        let node = Widget::<(), iced::Theme, iced::Renderer>::layout(
            &mut scroll,
            &mut tree,
            &renderer,
            &limits,
        );
        let layout = Layout::new(&node);
        let pane = layout.bounds();
        let outside = Point::new(pane.x + 12.0, pane.y + pane.height + 40.0);
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        let mut shell = iced::advanced::Shell::new(&mut messages);
        Widget::<(), iced::Theme, iced::Renderer>::update(
            &mut scroll,
            &mut tree,
            &Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                modified_key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                physical_key: iced::keyboard::key::Physical::Unidentified(
                    iced::keyboard::key::NativeCode::Unidentified,
                ),
                location: iced::keyboard::Location::Standard,
                modifiers: iced::keyboard::Modifiers::default(),
                text: None,
                repeat: false,
            }),
            layout,
            mouse::Cursor::Available(outside),
            &renderer,
            &mut clipboard,
            &mut shell,
            &pane,
        );
        assert_eq!(hits.get(), 1);
        let _ = Widget::<(), iced::Theme, iced::Renderer>::size(&scroll);
        Widget::<(), iced::Theme, iced::Renderer>::draw(
            &scroll,
            &tree,
            &mut renderer,
            &iced::Theme::Dark,
            &Style::default(),
            layout,
            mouse::Cursor::Available(outside),
            &pane,
        );
    }
}
