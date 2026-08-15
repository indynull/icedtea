//! Overlay and disclosure paint from a 0..=1 progress.
//!
//! The application owns [`iced::Animation`] and the clock. Pass
//! `animation.interpolate(0.0, 1.0, now)` as `progress`. Reduced-motion
//! tokens snap that value to 0 or 1. [`bounce_out`], [`pulse`], and
//! [`shake`] are extra curves for the same hook.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::tree::Tree;
use iced::advanced::widget::Widget;
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::time::Duration;
use iced::{Element, Event, Length, Rectangle, Size, Vector};

use crate::a11y::{self, A11y};
use crate::m3::motion::{self as m3motion, DurationStep, Ease};
use crate::theme::Tokens;

/// Slide direction at progress 0 (restored at 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Slide {
    None,
    Up,
    Down,
    Start,
    End,
}

impl Slide {
    /// Pixel offset at progress 0.
    pub fn pixels(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Up | Self::Down => m3motion::OVERLAY_SLIDE,
            Self::Start | Self::End => m3motion::SHEET_SLIDE,
        }
    }

    fn delta(self, remain: f32) -> Vector {
        let d = self.pixels() * remain;
        match self {
            Self::None => Vector::ZERO,
            Self::Up => Vector::new(0.0, d),
            Self::Down => Vector::new(0.0, -d),
            Self::Start => Vector::new(-d, 0.0),
            Self::End => Vector::new(d, 0.0),
        }
    }
}

/// Snap progress when reduced motion is on.
pub fn visual(progress: f32, reduced: bool) -> f32 {
    let p = progress.clamp(0.0, 1.0);
    if reduced {
        if p >= 0.5 {
            1.0
        } else {
            0.0
        }
    } else {
        p
    }
}

/// Opacity for a given visual progress.
pub fn fade(progress: f32) -> f32 {
    progress.clamp(0.0, 1.0)
}

/// Remaining offset (progress 0 = `full`, 1 = 0).
pub fn offset(progress: f32, full: f32) -> f32 {
    (1.0 - progress.clamp(0.0, 1.0)) * full
}

/// Height between a closed peek and the open size.
pub fn height(progress: f32, peek: f32, open: f32) -> f32 {
    let t = progress.clamp(0.0, 1.0);
    peek + (open - peek) * t
}

/// Ease-out bounce. 0 and 1 are at rest; the approach hops.
///
/// Pass the result into [`overlay`] or [`expand`] the same way as
/// [`Ease::sample`](crate::m3::Ease::sample).
pub fn bounce_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;
    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        let t = t - 1.5 / D1;
        N1 * t * t + 0.75
    } else if t < 2.5 / D1 {
        let t = t - 2.25 / D1;
        N1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / D1;
        N1 * t * t + 0.984375
    }
}

/// One pulse cycle: 0 at the ends, 1 in the middle.
///
/// Loop `t` over 0..=1 (the same phase a spinner uses). Reduced
/// motion should hold 1.
pub fn pulse(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    0.5 - 0.5 * (t * std::f32::consts::TAU).cos()
}

/// Shake displacement in -1..=1. 0 at both ends.
///
/// Multiply by a pixel amount and shift padding, or scale a slide.
pub fn shake(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t <= 0.0 || t >= 1.0 {
        return 0.0;
    }
    (t * std::f32::consts::PI * 6.0).sin() * (1.0 - t)
}

/// Toast enter/exit from age and remaining TTL.
pub fn toast_progress(age_ms: u64, ttl_ms: u64, fade_ms: u64) -> f32 {
    if fade_ms == 0 {
        return if ttl_ms > 0 { 1.0 } else { 0.0 };
    }
    let fade = fade_ms as f32;
    let enter = (age_ms as f32 / fade).clamp(0.0, 1.0);
    let exit = if ttl_ms >= fade_ms {
        1.0
    } else {
        ttl_ms as f32 / fade
    };
    enter.min(exit)
}

/// Duration for an overlay / sheet / toast / expand job.
pub fn duration(step: DurationStep, reduced: bool) -> Duration {
    step.duration(reduced)
}

/// Configure an iced animation for overlay enter (emphasized decelerate).
pub fn overlay_animation(open: bool, reduced: bool) -> iced::Animation<bool> {
    iced::Animation::new(open)
        .duration(m3motion::OVERLAY.duration(reduced))
        .easing(Ease::EmphasizedDecelerate.lilt())
}

/// Configure an iced animation for expander height.
pub fn expand_animation(open: bool, reduced: bool) -> iced::Animation<bool> {
    iced::Animation::new(open)
        .duration(m3motion::EXPAND.duration(reduced))
        .easing(Ease::EmphasizedDecelerate.lilt())
}

/// Configure an iced animation for a determinate 0..=1 value.
pub fn value_animation(start: f32, reduced: bool) -> iced::Animation<f32> {
    iced::Animation::new(start.clamp(0.0, 1.0))
        .duration(m3motion::PROGRESS.duration(reduced))
        .easing(Ease::Standard.lilt())
}

/// Linear indeterminate run: `(lead, mid, tail)` portions that sum to 1.
///
/// `phase` is 0..=1 over one cycle. The chunk grows as it leaves the
/// start, then shrinks as it exits the end. Reduced motion holds a
/// static mid-track chunk.
pub fn progress_run(phase: f32, reduced: bool) -> (f32, f32, f32) {
    if reduced {
        return (0.35, 0.30, 0.35);
    }
    let p = phase.rem_euclid(1.0);
    let head = Ease::EmphasizedDecelerate.sample(p);
    let tail = if p < 0.38 {
        0.0
    } else {
        Ease::StandardAccelerate.sample(((p - 0.38) / 0.62).clamp(0.0, 1.0))
    };
    let start = tail.min(0.92);
    let end = head.max(start + 0.08).min(1.0);
    let mid = (end - start).clamp(0.08, 1.0);
    let lead = start;
    let rest = (1.0 - lead - mid).max(0.0);
    (lead, mid, rest)
}

/// Fade and slide a child for overlay enter/exit.
///
/// `progress` is 0 (gone) to 1 (at rest). The application owns
/// [`iced::Animation`] and passes `interpolate(0.0, 1.0, now)`.
/// Build the child with [`Tokens::fade`](`crate::theme::Tokens::fade`)
/// so fills, ink, and icons fade with the slide. [`Slide::None`]
/// skips the translate (fade only). Reduced-motion tokens snap to
/// 0 or 1. Empty progress still occupies layout so a closing frame
/// can run.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::motion::{self, Slide};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let paint = tok.fade(1.0);
/// let body = widget::label("Sheet", paint, A11y::new("Sheet", Role::Status));
/// let _: icedtea::Element<'_, ()> = motion::overlay(
///     body,
///     1.0,
///     Slide::Up,
///     tok,
///     A11y::new("motion", Role::Group),
/// );
/// ```
pub fn overlay<'a, M: 'a>(
    child: Element<'a, M>,
    progress: f32,
    slide: Slide,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    a11y::attach(
        OverlayLayer {
            content: child,
            progress,
            slide,
            reduced: tok.reduced_motion,
        }
        .into(),
        &a11y,
    )
}

/// Clip a child between a peek height and its open height.
///
/// `progress` 0 is `peek` pixels (0 hides). 1 is the child's laid-out
/// height. Reduced-motion tokens snap.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::motion;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let body = widget::label("more", tok, A11y::new("more", Role::Status));
/// let _: icedtea::Element<'_, ()> = motion::expand(
///     body,
///     1.0,
///     0.0,
///     tok,
///     A11y::new("expand", Role::Group),
/// );
/// ```
pub fn expand<'a, M: 'a>(
    child: Element<'a, M>,
    progress: f32,
    peek: f32,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    a11y::attach(
        ExpandLayer {
            content: child,
            progress,
            peek: peek.max(0.0),
            reduced: tok.reduced_motion,
        }
        .into(),
        &a11y,
    )
}

struct OverlayLayer<'a, Message> {
    content: Element<'a, Message>,
    progress: f32,
    slide: Slide,
    reduced: bool,
}

impl<Message> OverlayLayer<'_, Message> {
    fn slide_delta(&self) -> iced::Vector {
        let t = visual(self.progress, self.reduced);
        self.slide.delta(1.0 - t)
    }
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for OverlayLayer<'a, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
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
        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let size = child.size();
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
        let t = visual(self.progress, self.reduced);
        if t <= 0.0 {
            return;
        }
        // Draw translates by `slide_delta`; hit-test must match.
        let cursor = cursor - self.slide_delta();
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
        if visual(self.progress, self.reduced) <= 0.0 {
            return mouse::Interaction::None;
        }
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor - self.slide_delta(),
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
        use iced::advanced::Renderer as _;
        let t = visual(self.progress, self.reduced);
        if t <= 0.0 {
            return;
        }
        let remain = 1.0 - t;
        let delta = self.slide.delta(remain);
        // Surfaces, ink, and icons fade through `Tokens::fade` on the
        // child. This layer only translates; multiplying text_color
        // here would ghost labels on an opaque card.
        let child_layout = layout.children().next().unwrap();
        renderer.with_translation(delta, |renderer| {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                child_layout,
                cursor,
                viewport,
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
}

impl<'a, Message: 'a> From<OverlayLayer<'a, Message>> for Element<'a, Message> {
    fn from(value: OverlayLayer<'a, Message>) -> Self {
        Self::new(value)
    }
}

struct ExpandLayer<'a, Message> {
    content: Element<'a, Message>,
    progress: f32,
    peek: f32,
    reduced: bool,
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for ExpandLayer<'a, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child_limits =
            layout::Limits::new(Size::ZERO, Size::new(limits.max().width, f32::INFINITY));
        let child =
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &child_limits);
        let t = visual(self.progress, self.reduced);
        let h = height(t, self.peek, child.size().height);
        let size = Size::new(child.size().width, h.max(0.0));
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
        if visual(self.progress, self.reduced) <= 0.0 && self.peek <= 0.0 {
            return;
        }
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
        use iced::advanced::Renderer as _;
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
}

impl<'a, Message: 'a> From<ExpandLayer<'a, Message>> for Element<'a, Message> {
    fn from(value: ExpandLayer<'a, Message>) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::a11y::Role;
    use crate::theme::named;
    use crate::widget;

    #[test]
    fn visual_snaps_when_reduced() {
        assert_eq!(visual(0.0, false), 0.0);
        assert_eq!(visual(1.0, false), 1.0);
        assert!((visual(0.4, false) - 0.4).abs() < f32::EPSILON);
        assert_eq!(visual(0.4, true), 0.0);
        assert_eq!(visual(0.5, true), 1.0);
        assert_eq!(visual(0.9, true), 1.0);
    }

    #[test]
    fn fade_offset_height_and_toast_progress() {
        assert_eq!(fade(0.0), 0.0);
        assert_eq!(fade(1.0), 1.0);
        assert_eq!(offset(0.0, 12.0), 12.0);
        assert_eq!(offset(1.0, 12.0), 0.0);
        assert!((offset(0.5, 12.0) - 6.0).abs() < f32::EPSILON);
        assert_eq!(height(0.0, 10.0, 40.0), 10.0);
        assert_eq!(height(1.0, 10.0, 40.0), 40.0);
        assert!((height(0.5, 10.0, 40.0) - 25.0).abs() < f32::EPSILON);
        assert_eq!(bounce_out(0.0), 0.0);
        assert!((bounce_out(1.0) - 1.0).abs() < 1e-5);
        let bmid = bounce_out(0.5);
        assert!(bmid > 0.7 && bmid < 1.0);
        // Third hop (2/D1 .. 2.5/D1).
        let b3 = bounce_out(0.85);
        assert!(b3 > 0.9 && b3 < 1.0);
        assert_eq!(bounce_out(-1.0), 0.0);
        assert!((bounce_out(2.0) - 1.0).abs() < 1e-5);
        assert_eq!(pulse(-0.5), 0.0);
        assert_eq!(shake(-0.5), 0.0);
        assert_eq!(shake(2.0), 0.0);
        assert_eq!(pulse(0.0), 0.0);
        assert!((pulse(0.5) - 1.0).abs() < 1e-5);
        assert!((pulse(1.0)).abs() < 1e-5);
        assert_eq!(shake(0.0), 0.0);
        assert_eq!(shake(1.0), 0.0);
        assert!(shake(0.2).abs() > 0.1);
        assert_eq!(toast_progress(0, 4000, 150), 0.0);
        assert_eq!(toast_progress(200, 4000, 150), 1.0);
        assert!(toast_progress(200, 80, 150) < 1.0);
        assert_eq!(toast_progress(200, 0, 150), 0.0);
        let mid = toast_progress(75, 4000, 150);
        assert!(mid > 0.4 && mid < 0.6);
        // Reduced motion: DurationStep::duration(true) is 0 ms.
        // A live toast is at rest immediately; an expired one is gone.
        let fade0 = duration(m3motion::TOAST, true).as_millis() as u64;
        assert_eq!(fade0, 0);
        assert_eq!(toast_progress(0, 4000, fade0), 1.0);
        assert_eq!(toast_progress(0, 0, fade0), 0.0);
    }

    #[test]
    fn reduced_motion_durations_are_instant() {
        assert_eq!(duration(m3motion::OVERLAY, true), Duration::ZERO);
        assert!(duration(m3motion::OVERLAY, false) > Duration::ZERO);
        let instant = overlay_animation(false, true);
        assert!(!instant.is_animating(iced::time::Instant::now()));
        let expand_live = expand_animation(true, false);
        let expand_off = expand_animation(false, true);
        assert!(!expand_off.is_animating(iced::time::Instant::now()));
        let _ = expand_live;
        let snap = value_animation(0.4, true);
        assert!(!snap.is_animating(iced::time::Instant::now()));
        assert!((snap.value() - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn progress_run_travels_then_snaps_when_reduced() {
        let (l0, m0, t0) = progress_run(0.0, false);
        let (l1, m1, t1) = progress_run(0.98, false);
        let (lm, mm, tm) = progress_run(0.45, false);
        assert!((l0 + m0 + t0 - 1.0).abs() < 0.02);
        assert!((l1 + m1 + t1 - 1.0).abs() < 0.02);
        assert!((lm + mm + tm - 1.0).abs() < 0.02);
        assert!(m0 >= 0.08 && m1 >= 0.08);
        assert!(l0 < lm);
        assert!(l1 > lm);
        assert!(mm > m0);
        let (lr, mr, tr) = progress_run(0.1, true);
        assert_eq!((lr, mr, tr), progress_run(0.9, true));
        assert!((lr + mr + tr - 1.0).abs() < 0.02);
    }

    #[test]
    fn overlay_and_expand_constructors_build() {
        let tok = named("dark").tokens;
        let body = widget::label("Sheet", tok, A11y::new("Sheet", Role::Status));
        let _: Element<'_, ()> =
            overlay(body, 0.5, Slide::Up, tok, A11y::new("motion", Role::Group));
        let reduced = tok.with_reduced_motion(true);
        let body = widget::label("more", reduced, A11y::new("more", Role::Status));
        let _: Element<'_, ()> = expand(body, 0.3, 8.0, reduced, A11y::new("expand", Role::Group));
        assert_eq!(visual(0.3, true), 0.0);
    }

    #[test]
    fn slide_offsets_match_direction() {
        assert_eq!(Slide::None.pixels(), 0.0);
        assert_eq!(Slide::Up.pixels(), m3motion::OVERLAY_SLIDE);
        assert_eq!(Slide::Down.pixels(), m3motion::OVERLAY_SLIDE);
        assert_eq!(Slide::Start.pixels(), m3motion::SHEET_SLIDE);
        assert_eq!(Slide::End.pixels(), m3motion::SHEET_SLIDE);
        assert_eq!(Slide::None.delta(1.0), Vector::ZERO);
        assert_eq!(
            Slide::Up.delta(1.0),
            Vector::new(0.0, m3motion::OVERLAY_SLIDE)
        );
        assert_eq!(
            Slide::Down.delta(1.0),
            Vector::new(0.0, -m3motion::OVERLAY_SLIDE)
        );
        // Start/End offset toward that docked edge at progress 0.
        assert_eq!(
            Slide::Start.delta(1.0),
            Vector::new(-m3motion::SHEET_SLIDE, 0.0)
        );
        assert_eq!(
            Slide::End.delta(1.0),
            Vector::new(m3motion::SHEET_SLIDE, 0.0)
        );
    }

    #[test]
    fn end_sheet_enters_from_the_docked_edge() {
        // LTR trailing sheet: progress 0 sits further toward End (+x),
        // then slides to rest. Start-docked is the mirror (-x).
        assert!(Slide::End.delta(1.0).x > 0.0);
        assert!(Slide::Start.delta(1.0).x < 0.0);
        assert_eq!(Slide::End.delta(0.0), Vector::ZERO);
        let src = include_str!("pattern.rs");
        let sheet = src
            .split("pub fn side_sheet")
            .nth(1)
            .unwrap()
            .split("pub fn context_card_size")
            .next()
            .unwrap();
        assert!(sheet.contains("Slide::End"));
        assert!(sheet.contains("Slide::Start"));
        assert!(sheet.contains("if end"));
    }

    fn drive<M: Clone>(el: &mut Element<'_, M>, viewport: iced::Rectangle) {
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::renderer::Style;
        use iced::advanced::widget::operation::focusable;
        use iced::advanced::widget::Tree;
        use iced::{Event, Font, Pixels, Point, Size, Theme};
        let mut tree = Tree::new(el.as_widget());
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let _ = el.as_widget().size();
        let _ = el.as_widget().size_hint();
        let _ = el.as_widget().children();
        el.as_widget_mut().diff(&mut tree);
        let limits = Limits::new(Size::ZERO, Size::new(320.0, 240.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        el.as_widget().draw(
            &tree,
            &mut renderer,
            &Theme::Dark,
            &Style::default(),
            layout,
            mouse::Cursor::Available(Point::new(8.0, 8.0)),
            &viewport,
        );
        let _ = el.as_widget().mouse_interaction(
            &tree,
            layout,
            mouse::Cursor::Available(Point::new(8.0, 8.0)),
            &viewport,
            &renderer,
        );
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::<M>::new();
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
            el.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(mouse::Event::CursorMoved {
                    position: Point::new(8.0, 8.0),
                }),
                layout,
                mouse::Cursor::Available(Point::new(8.0, 8.0)),
                &renderer,
                &mut clipboard,
                &mut shell,
                &viewport,
            );
        }
        let mut op = focusable::unfocus::<()>();
        el.as_widget_mut()
            .operate(&mut tree, layout, &renderer, &mut op);
        let _ = messages;
    }

    #[test]
    fn overlay_and_expand_paint_at_zero_mid_and_one() {
        let tok = named("dark").tokens;
        let reduced = tok.with_reduced_motion(true);
        let vp = iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(320.0, 240.0));
        let miss =
            iced::Rectangle::new(iced::Point::new(800.0, 800.0), iced::Size::new(10.0, 10.0));
        let body = || widget::label("Sheet", tok, A11y::new("Sheet", Role::Status));
        for slide in [
            Slide::None,
            Slide::Up,
            Slide::Down,
            Slide::Start,
            Slide::End,
        ] {
            for p in [0.0, 0.5, 1.0] {
                let mut el: Element<'_, ()> =
                    overlay(body(), p, slide, tok, A11y::new("motion", Role::Group));
                drive(&mut el, vp);
            }
        }
        let mut gone: Element<'_, ()> =
            overlay(body(), 0.0, Slide::Up, reduced, A11y::new("r", Role::Group));
        drive(&mut gone, vp);
        let mut snap: Element<'_, ()> =
            overlay(body(), 0.3, Slide::Up, reduced, A11y::new("s", Role::Group));
        drive(&mut snap, vp);
        for (p, peek, t) in [
            (0.0, 0.0, tok),
            (0.0, 12.0, tok),
            (0.5, 8.0, tok),
            (1.0, 0.0, tok),
            (0.3, 8.0, reduced),
        ] {
            let mut el: Element<'_, ()> =
                expand(body(), p, peek, t, A11y::new("expand", Role::Group));
            drive(&mut el, vp);
            drive(&mut el, miss);
        }
        let far = iced::Rectangle::new(iced::Point::new(4000.0, 4000.0), iced::Size::new(4.0, 4.0));
        let mut hidden: Element<'_, ()> =
            expand(body(), 0.0, 0.0, tok, A11y::new("gone", Role::Group));
        drive(&mut hidden, far);
        let mut clipped: Element<'_, ()> =
            expand(body(), 1.0, 0.0, tok, A11y::new("full", Role::Group));
        drive(&mut clipped, far);
    }

    #[test]
    fn overlay_press_hits_the_slid_child() {
        use crate::variant::Variant;
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::{Event, Font, Pixels, Point, Size};
        let tok = named("dark").tokens;
        let face = widget::themed_button(
            "Copy",
            Some(()),
            tok,
            Variant::Ghost,
            crate::icon::Icons::NONE,
            A11y::button("Copy"),
        );
        let progress = 0.5;
        let mut el: Element<'_, ()> = overlay(
            face,
            progress,
            Slide::Up,
            tok,
            A11y::new("menu", Role::Group),
        );
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(320.0, 240.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let delta = Slide::Up.delta(1.0 - visual(progress, false));
        let at = Point::new(24.0, 12.0) + delta;
        let viewport = iced::Rectangle::new(Point::ORIGIN, Size::new(320.0, 240.0));
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
        {
            let mut shell = iced::advanced::Shell::new(&mut messages);
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
        assert_eq!(messages, vec![()]);
    }

    #[test]
    fn overlay_spacer_press_is_not_captured() {
        use crate::variant::Variant;
        use iced::advanced::clipboard;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::widget::{column, row, Space};
        use iced::{Event, Font, Pixels, Point, Size};
        let tok = named("dark").tokens;
        let face = widget::themed_button(
            "Copy",
            Some(()),
            tok,
            Variant::Ghost,
            crate::icon::Icons::NONE,
            A11y::button("Copy"),
        );
        let placed = column![
            Space::new().height(Length::Fixed(80.0)),
            row![Space::new().width(Length::Fixed(80.0)), face],
        ];
        let mut el: Element<'_, ()> = overlay(
            placed.into(),
            1.0,
            Slide::Up,
            tok,
            A11y::new("place", Role::Group),
        );
        let mut tree = Tree::new(el.as_widget());
        let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(320.0, 240.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let viewport = iced::Rectangle::new(Point::ORIGIN, Size::new(320.0, 240.0));
        let mut clipboard = clipboard::Null;
        let mut messages = Vec::new();
        let captured;
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
            captured = shell.is_event_captured();
        }
        assert!(
            !captured,
            "a press on the placement spacer must reach the dismiss surface"
        );
        assert!(messages.is_empty(), "spacer press must not run the row");
    }

    #[test]
    fn expand_draw_returns_when_viewport_misses() {
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::renderer::Style;
        use iced::advanced::widget::{Tree, Widget};
        use iced::{Font, Pixels, Point, Size, Theme};

        let tok = named("dark").tokens;
        let body = widget::label("more", tok, A11y::new("more", Role::Status));
        let mut layer = ExpandLayer {
            content: body,
            progress: 1.0,
            peek: 0.0,
            reduced: false,
        };
        let mut tree = Tree::new(&layer as &dyn Widget<(), Theme, iced::Renderer>);
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(320.0, 240.0));
        let node =
            Widget::<(), Theme, iced::Renderer>::layout(&mut layer, &mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let miss = Rectangle::new(Point::new(800.0, 800.0), Size::new(10.0, 10.0));
        Widget::<(), Theme, iced::Renderer>::draw(
            &layer,
            &tree,
            &mut renderer,
            &Theme::Dark,
            &Style::default(),
            layout,
            mouse::Cursor::Unavailable,
            &miss,
        );
    }
}
