//! Focus target, roving tabindex, spatial arrows, landmarks, live regions.
//!
//! A [`target`] owns click-to-focus and the 2 dp primary ring. Use
//! [`rove`] / [`spatial_next`] with [`crate::key::handle`] so arrow
//! keys move between panels.

/// Named landmark region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landmark {
    Banner,
    Navigation,
    Main,
    Complementary,
    ContentInfo,
    Search,
    Status,
}

impl Landmark {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Banner => "banner",
            Self::Navigation => "navigation",
            Self::Main => "main",
            Self::Complementary => "complementary",
            Self::ContentInfo => "contentinfo",
            Self::Search => "search",
            Self::Status => "status",
        }
    }
}

/// Live region politeness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Live {
    Off,
    Polite,
    Assertive,
}

/// Roving tabindex: one `active` index owns focus in a group of `len`.
///
/// ```
/// use icedtea::focus::rove;
/// assert_eq!(rove(0, 1, 4), 1);
/// assert_eq!(rove(3, 1, 4), 0);
/// ```
pub fn rove(active: usize, delta: i32, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    (active as i32 + delta).rem_euclid(len as i32) as usize
}

/// Axis-aligned neighbor among panel centers. `None` when no neighbor.
///
/// ```
/// use icedtea::focus::spatial_next;
/// let boxes = [(0.0, 0.0), (200.0, 0.0), (0.0, 120.0)];
/// assert_eq!(spatial_next(0, icedtea::key::Press::ArrowRight, &boxes), Some(1));
/// assert_eq!(spatial_next(0, icedtea::key::Press::ArrowDown, &boxes), Some(2));
/// ```
pub fn spatial_next(
    from: usize,
    press: crate::key::Press,
    centers: &[(f32, f32)],
) -> Option<usize> {
    let (fx, fy) = *centers.get(from)?;
    let (dx, dy) = match press {
        crate::key::Press::ArrowLeft => (-1.0, 0.0),
        crate::key::Press::ArrowRight => (1.0, 0.0),
        crate::key::Press::ArrowUp => (0.0, -1.0),
        crate::key::Press::ArrowDown => (0.0, 1.0),
        _ => return None,
    };
    let mut best: Option<(usize, f32)> = None;
    for (i, (x, y)) in centers.iter().copied().enumerate() {
        if i == from {
            continue;
        }
        let vx = x - fx;
        let vy = y - fy;
        let along = vx * dx + vy * dy;
        if along <= 0.0 {
            continue;
        }
        let cross = (vx * dy - vy * dx).abs();
        let score = along + cross * 2.0;
        if best.is_none_or(|(_, s)| score < s) {
            best = Some((i, score));
        }
    }
    best.map(|(i, _)| i)
}

/// Modal focus trap: Escape leaves; other keys stay inside.
pub fn trap_escape(press: &crate::key::Press) -> bool {
    matches!(press, crate::key::Press::Escape)
}

/// 2 dp primary frame when the target is focused.
///
/// Same wash [`crate::widget::form_group`] uses on the active row.
pub fn ring(tok: crate::theme::Tokens, focused: bool) -> iced::widget::container::Style {
    let s = tok.scheme();
    iced::widget::container::Style {
        border: iced::Border {
            color: if focused {
                s.primary
            } else {
                iced::Color::TRANSPARENT
            },
            width: if focused { 2.0 } else { 0.0 },
            radius: tok.radius(crate::m3::shape::Component::Field),
        },
        ..iced::widget::container::Style::default()
    }
}

/// Wrap `child` so a click focuses it and a focused ring paints.
///
/// `can_focus` is false when the constructor is empty or disabled:
/// the child is not a [`iced::advanced::widget::operation::focusable::Focusable`]
/// and a click does not take focus.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let body = widget::label("rows", tok, A11y::new("rows", Role::List));
/// let _: icedtea::Element<'_, ()> = icedtea::focus::target(body, tok, true);
/// ```
pub fn target<'a, M: Clone + 'a>(
    child: iced::Element<'a, M>,
    tok: crate::theme::Tokens,
    can_focus: bool,
) -> iced::Element<'a, M> {
    Target {
        content: child,
        tok,
        can_focus,
        on_key: None,
    }
    .into()
}

/// [`target`] plus keys while focused (`Press::step_index` for lists).
pub fn target_keys<'a, M: Clone + 'a>(
    child: iced::Element<'a, M>,
    tok: crate::theme::Tokens,
    can_focus: bool,
    on_key: impl Fn(crate::key::Press) -> Option<M> + 'a,
) -> iced::Element<'a, M> {
    Target {
        content: child,
        tok,
        can_focus,
        on_key: Some(Box::new(on_key)),
    }
    .into()
}

struct Target<'a, Message> {
    content: iced::Element<'a, Message>,
    tok: crate::theme::Tokens,
    can_focus: bool,
    on_key: Option<Box<dyn Fn(crate::key::Press) -> Option<Message> + 'a>>,
}

#[derive(Default)]
struct TargetState {
    focused: bool,
}

impl iced::advanced::widget::operation::focusable::Focusable for TargetState {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}

impl<'a, Message: Clone> iced::advanced::Widget<Message, iced::Theme, iced::Renderer>
    for Target<'a, Message>
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<TargetState>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(TargetState::default())
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> iced::Size<iced::Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
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
        if !self.can_focus {
            return;
        }
        let state = tree.state.downcast_mut::<TargetState>();
        let over = cursor
            .position()
            .is_some_and(|p| layout.bounds().contains(p));
        match event {
            iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
            | iced::Event::Touch(iced::touch::Event::FingerPressed { .. }) => {
                let next = over;
                if state.focused != next {
                    state.focused = next;
                    shell.request_redraw();
                }
            }
            iced::Event::Keyboard(kev) if state.focused && !shell.is_event_captured() => {
                if let (Some(press), Some(on_key)) = (crate::key::press(kev), &self.on_key) {
                    if let Some(msg) = on_key(press) {
                        shell.publish(msg);
                        shell.capture_event();
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
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
        let focused = self.can_focus && tree.state.downcast_ref::<TargetState>().focused;
        if !focused {
            return;
        }
        let face = ring(self.tok, true);
        iced::advanced::Renderer::fill_quad(
            renderer,
            iced::advanced::renderer::Quad {
                bounds: layout.bounds(),
                border: face.border,
                ..iced::advanced::renderer::Quad::default()
            },
            iced::Background::Color(iced::Color::TRANSPARENT),
        );
    }

    fn operate(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        if self.can_focus {
            let state = tree.state.downcast_mut::<TargetState>();
            operation.focusable(None, layout.bounds(), state);
        }
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
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
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'b>,
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

impl<'a, Message: Clone + 'a> From<Target<'a, Message>> for iced::Element<'a, Message> {
    fn from(value: Target<'a, Message>) -> Self {
        Self::new(value)
    }
}

/// Publish `on_escape` when Escape is pressed and the child did not capture it.
pub fn dismiss_on_escape<'a, M: Clone + 'a>(
    child: iced::Element<'a, M>,
    on_escape: M,
) -> iced::Element<'a, M> {
    DismissEscape {
        content: child,
        on_escape,
    }
    .into()
}

struct DismissEscape<'a, Message> {
    content: iced::Element<'a, Message>,
    on_escape: Message,
}

impl<'a, Message: Clone> iced::advanced::Widget<Message, iced::Theme, iced::Renderer>
    for DismissEscape<'a, Message>
{
    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> iced::Size<iced::Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
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
        if matches!(
            event,
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                ..
            })
        ) {
            shell.publish(self.on_escape.clone());
            shell.capture_event();
        }
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
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
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'b>,
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

impl<'a, Message: Clone + 'a> From<DismissEscape<'a, Message>> for iced::Element<'a, Message> {
    fn from(value: DismissEscape<'a, Message>) -> Self {
        Self::new(value)
    }
}

/// Walk Tab / Shift+Tab across focus targets. First frame focuses
/// `first` when set, otherwise the first target.
///
/// [`crate::run!`] wraps the window view with this. `form_group`
/// captures Tab inside a mixed form so this does not also move.
pub fn cycle<'a, M: 'a>(
    child: iced::Element<'a, M>,
    first: Option<iced::widget::Id>,
) -> iced::Element<'a, M> {
    Cycle {
        content: child,
        first,
    }
    .into()
}

struct Cycle<'a, Message> {
    content: iced::Element<'a, Message>,
    first: Option<iced::widget::Id>,
}

#[derive(Default)]
struct CycleState {
    mounted: bool,
}

impl<'a, Message> iced::advanced::Widget<Message, iced::Theme, iced::Renderer>
    for Cycle<'a, Message>
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<CycleState>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(CycleState::default())
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        vec![iced::advanced::widget::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> iced::Size<iced::Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_mut::<CycleState>();
        if !state.mounted {
            state.mounted = true;
            apply_first_focus(
                &mut self.content,
                tree,
                layout,
                renderer,
                self.first.as_ref(),
            );
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
        if shell.is_event_captured() {
            return;
        }
        let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) = event
        else {
            return;
        };
        if !matches!(
            key,
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab)
        ) {
            return;
        }
        if modifiers.control() || modifiers.alt() || modifiers.logo() {
            return;
        }
        cycle_by(
            &mut self.content,
            tree,
            layout,
            renderer,
            if modifiers.shift() { -1 } else { 1 },
        );
        shell.capture_event();
        shell.request_redraw();
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
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
        tree: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
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
        tree: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'b>,
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

fn cycle_by<M>(
    content: &mut iced::Element<'_, M>,
    tree: &mut iced::advanced::widget::Tree,
    layout: iced::advanced::Layout<'_>,
    renderer: &iced::Renderer,
    delta: i32,
) {
    use iced::advanced::widget::operation::{focusable::Focusable, Operation};
    struct Count {
        focused: Option<usize>,
        n: usize,
    }
    impl Operation<()> for Count {
        fn focusable(
            &mut self,
            _id: Option<&iced::widget::Id>,
            _bounds: iced::Rectangle,
            state: &mut dyn Focusable,
        ) {
            if state.is_focused() {
                self.focused = Some(self.n);
            }
            self.n += 1;
        }
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
            operate(self);
        }
    }
    let mut count = Count {
        focused: None,
        n: 0,
    };
    content
        .as_widget_mut()
        .operate(&mut tree.children[0], layout, renderer, &mut count);
    if count.n == 0 {
        return;
    }
    let from = count
        .focused
        .unwrap_or(if delta > 0 { count.n - 1 } else { 0 });
    let want = rove(from, delta, count.n);
    struct Set {
        i: usize,
        want: usize,
    }
    impl Operation<()> for Set {
        fn focusable(
            &mut self,
            _id: Option<&iced::widget::Id>,
            _bounds: iced::Rectangle,
            state: &mut dyn Focusable,
        ) {
            if self.i == self.want {
                state.focus();
            } else {
                state.unfocus();
            }
            self.i += 1;
        }
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
            operate(self);
        }
    }
    let mut set = Set { i: 0, want };
    content
        .as_widget_mut()
        .operate(&mut tree.children[0], layout, renderer, &mut set);
}

fn apply_op<M>(
    content: &mut iced::Element<'_, M>,
    tree: &mut iced::advanced::widget::Tree,
    layout: iced::advanced::Layout<'_>,
    renderer: &iced::Renderer,
    mut op: impl iced::advanced::widget::Operation<()>,
) {
    use iced::advanced::widget::operation::{Operation, Outcome};
    content
        .as_widget_mut()
        .operate(&mut tree.children[0], layout, renderer, &mut op);
    if let Outcome::Chain(mut next) = Operation::finish(&op) {
        content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, next.as_mut());
    }
}

fn apply_first_focus<M>(
    content: &mut iced::Element<'_, M>,
    tree: &mut iced::advanced::widget::Tree,
    layout: iced::advanced::Layout<'_>,
    renderer: &iced::Renderer,
    first: Option<&iced::widget::Id>,
) {
    if let Some(id) = first {
        apply_op(
            content,
            tree,
            layout,
            renderer,
            iced::advanced::widget::operation::focusable::focus::<()>(id.clone()),
        );
    } else {
        apply_op(
            content,
            tree,
            layout,
            renderer,
            iced::advanced::widget::operation::focusable::focus_next::<()>(),
        );
    }
}

impl<'a, Message: 'a> From<Cycle<'a, Message>> for iced::Element<'a, Message> {
    fn from(value: Cycle<'a, Message>) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::Press;

    #[test]
    fn rove_spatial_and_trap() {
        assert_eq!(rove(0, 1, 3), 1);
        assert_eq!(rove(2, 1, 3), 0);
        assert_eq!(rove(0, -1, 3), 2);
        assert_eq!(rove(0, 1, 0), 0);
        let boxes = [(0.0, 0.0), (200.0, 0.0), (0.0, 120.0)];
        assert_eq!(spatial_next(0, Press::ArrowRight, &boxes), Some(1));
        assert_eq!(spatial_next(0, Press::ArrowDown, &boxes), Some(2));
        assert_eq!(spatial_next(2, Press::ArrowUp, &boxes), Some(0));
        assert_eq!(spatial_next(0, Press::ArrowLeft, &boxes), None);
        assert_eq!(spatial_next(9, Press::ArrowRight, &boxes), None);
        assert_eq!(spatial_next(0, Press::Enter, &boxes), None);
        assert!(trap_escape(&Press::Escape));
        assert!(!trap_escape(&Press::Enter));
        assert_eq!(Landmark::Main.as_str(), "main");
        assert_eq!(Landmark::Banner.as_str(), "banner");
        assert_eq!(Landmark::Navigation.as_str(), "navigation");
        assert_eq!(Landmark::Complementary.as_str(), "complementary");
        assert_eq!(Landmark::ContentInfo.as_str(), "contentinfo");
        assert_eq!(Landmark::Search.as_str(), "search");
        assert_eq!(Landmark::Status.as_str(), "status");
        assert_eq!(Live::Polite, Live::Polite);
        assert_ne!(Live::Off, Live::Assertive);
        let tok = crate::theme::named("dark").tokens;
        let on = ring(tok, true);
        assert_eq!(on.border.width, 2.0);
        assert_eq!(on.border.color, tok.scheme().primary);
        let off = ring(tok, false);
        assert_eq!(off.border.width, 0.0);
    }

    fn pump_click<M: Clone>(el: &mut iced::Element<'_, M>, can_focus: bool) -> bool {
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::operation::{focusable::Focusable, Operation};
        use iced::advanced::widget::Tree;
        use iced::{Event, Font, Pixels, Point, Rectangle, Size};
        struct Count {
            n: usize,
            focused: usize,
        }
        impl Operation<()> for Count {
            fn focusable(
                &mut self,
                _id: Option<&iced::widget::Id>,
                _bounds: Rectangle,
                state: &mut dyn Focusable,
            ) {
                self.n += 1;
                if state.is_focused() {
                    self.focused += 1;
                }
            }
            fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
                operate(self);
            }
        }
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(200.0, 80.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let at = Point::new(8.0, 8.0);
        let mut messages = Vec::<M>::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            let mut clipboard = iced::advanced::clipboard::Null;
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
                layout,
                iced::mouse::Cursor::Available(at),
                &renderer,
                &mut clipboard,
                &mut shell,
                &Rectangle::new(Point::ORIGIN, Size::new(200.0, 80.0)),
            );
        }
        let mut op = Count { n: 0, focused: 0 };
        el.as_widget_mut()
            .operate(&mut tree, layout, &renderer, &mut op);
        if can_focus {
            assert_eq!(op.n, 1, "enabled target is one focusable");
            op.focused == 1
        } else {
            assert_eq!(op.n, 0, "empty or disabled target is not focusable");
            false
        }
    }

    #[test]
    fn target_click_focuses_when_enabled() {
        let tok = crate::theme::named("dark").tokens;
        let body = crate::widget::label(
            "rows",
            tok,
            crate::a11y::A11y::new("rows", crate::a11y::Role::List),
        );
        let mut el: iced::Element<'_, ()> = target(body, tok, true);
        assert!(pump_click(&mut el, true));
    }

    #[test]
    fn target_click_skips_when_cannot_focus() {
        let tok = crate::theme::named("dark").tokens;
        let body = crate::widget::label(
            "rows",
            tok,
            crate::a11y::A11y::new("rows", crate::a11y::Role::List),
        );
        let mut el: iced::Element<'_, ()> = target(body, tok, false);
        assert!(!pump_click(&mut el, false));
    }

    fn two_targets() -> iced::Element<'static, ()> {
        let tok = crate::theme::named("dark").tokens;
        let a = target(
            crate::widget::label(
                "a",
                tok,
                crate::a11y::A11y::new("a", crate::a11y::Role::List),
            ),
            tok,
            true,
        );
        let b = target(
            crate::widget::label(
                "b",
                tok,
                crate::a11y::A11y::new("b", crate::a11y::Role::List),
            ),
            tok,
            true,
        );
        iced::widget::column![a, b].into()
    }

    fn focused_indices(el: &mut iced::Element<'_, ()>, tab_shift: Option<bool>) -> Vec<usize> {
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::operation::{focusable::Focusable, Operation};
        use iced::advanced::widget::Tree;
        use iced::{Event, Font, Pixels, Point, Rectangle, Size};
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(200.0, 160.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(200.0, 160.0));
        let mut messages = Vec::<()>::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            let mut clipboard = iced::advanced::clipboard::Null;
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(iced::mouse::Event::CursorMoved {
                    position: Point::new(1.0, 1.0),
                }),
                layout,
                iced::mouse::Cursor::Unavailable,
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        if let Some(shift) = tab_shift {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            let mut clipboard = iced::advanced::clipboard::Null;
            el.as_widget_mut().update(
                &mut tree,
                &Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab),
                    modified_key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab),
                    physical_key: iced::keyboard::key::Physical::Unidentified(
                        iced::keyboard::key::NativeCode::Unidentified,
                    ),
                    location: iced::keyboard::Location::Standard,
                    modifiers: if shift {
                        iced::keyboard::Modifiers::SHIFT
                    } else {
                        iced::keyboard::Modifiers::empty()
                    },
                    text: None,
                    repeat: false,
                }),
                layout,
                iced::mouse::Cursor::Unavailable,
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        struct Hit {
            i: usize,
            hits: Vec<usize>,
        }
        impl Operation<()> for Hit {
            fn focusable(
                &mut self,
                _id: Option<&iced::widget::Id>,
                _bounds: Rectangle,
                state: &mut dyn Focusable,
            ) {
                if state.is_focused() {
                    self.hits.push(self.i);
                }
                self.i += 1;
            }
            fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
                operate(self);
            }
        }
        let mut op = Hit {
            i: 0,
            hits: Vec::new(),
        };
        el.as_widget_mut()
            .operate(&mut tree, layout, &renderer, &mut op);
        op.hits
    }

    #[test]
    fn cycle_focuses_first_target_on_mount() {
        let mut el = cycle(two_targets(), None);
        assert_eq!(focused_indices(&mut el, None), vec![0]);
    }

    #[test]
    fn cycle_tab_walks_targets() {
        let mut el = cycle(two_targets(), None);
        assert_eq!(focused_indices(&mut el, Some(false)), vec![1]);
        let mut el = cycle(two_targets(), None);
        assert_eq!(focused_indices(&mut el, Some(true)), vec![1]);
    }
}
