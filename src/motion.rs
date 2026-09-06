//! Overlay, disclose, switch, and attention paint from a 0..=1 progress.
//!
//! Pick a [`Job`], hold a [`Run`], and pass `progress` into the matching
//! constructor. The application owns the clock. Reduced-motion tokens
//! snap that value to 0 or 1. [`bounce_out`], [`pulse`], and [`shake`]
//! are extra curves for the same hook.

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::tree::Tree;
use iced::advanced::widget::Widget;
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::time::{Duration, Instant};
use iced::{Element, Event, Length, Rectangle, Size, Transformation, Vector};

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
        offset_delta(self, self.pixels() * remain)
    }

    /// Incoming `Up` leaves toward `Down`; `Start` leaves toward `End`.
    pub fn opposite(self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Start => Self::End,
            Self::End => Self::Start,
        }
    }
}

fn offset_delta(slide: Slide, d: f32) -> Vector {
    match slide {
        Slide::None => Vector::ZERO,
        Slide::Up => Vector::new(0.0, d),
        Slide::Down => Vector::new(0.0, -d),
        Slide::Start => Vector::new(-d, 0.0),
        Slide::End => Vector::new(d, 0.0),
    }
}

/// Disclose axis: block is height, inline is width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    Block,
    Inline,
}

/// Overlay chrome that enters and exits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Enter {
    Dialog,
    Menu,
    Sheet,
    Toast,
    Tooltip,
}

/// How [`switch`] replaces one child with another.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwitchFace {
    /// Unrelated destinations: fade the leaving child out, then fade
    /// the incoming child in. Tab bodies and settings groups.
    FadeThrough,
    /// Spatial siblings. `Slide` is the incoming direction. Next on Y
    /// is [`Slide::Up`]; previous is [`Slide::Down`].
    SharedAxis(Slide),
}

impl SwitchFace {
    /// Opacity for the leaving child. Shared axis is `1 - progress`.
    pub fn outgoing_fade(self, progress: f32) -> f32 {
        let p = progress.clamp(0.0, 1.0);
        match self {
            Self::FadeThrough => (1.0 - p / 0.35).clamp(0.0, 1.0),
            Self::SharedAxis(_) => 1.0 - p,
        }
    }

    /// Opacity for the incoming child. Shared axis is `progress`.
    pub fn incoming_fade(self, progress: f32) -> f32 {
        let p = progress.clamp(0.0, 1.0);
        match self {
            Self::FadeThrough => ((p - 0.35) / 0.65).clamp(0.0, 1.0),
            Self::SharedAxis(_) => p,
        }
    }

    fn incoming_slide(self) -> Slide {
        match self {
            Self::FadeThrough => Slide::None,
            Self::SharedAxis(slide) => slide,
        }
    }

    fn outgoing_slide(self) -> Slide {
        self.incoming_slide().opposite()
    }
}

/// How [`attention`] marks a child.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttentionFace {
    /// Decaying wiggle. Invalid field after a failed check.
    Shake,
    /// Scale pulse. Live or recording mark.
    Pulse,
}

/// Named motion job: duration, enter/exit ease, and which constructor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Job {
    Enter(Enter),
    Switch(SwitchFace),
    Disclose(Axis),
    Attention(AttentionFace),
    Value,
}

impl Job {
    /// Duration for this direction. Exit is shorter on overlay chrome.
    pub fn duration(self, entering: bool, reduced: bool) -> Duration {
        if reduced {
            return Duration::ZERO;
        }
        let step = match self {
            Self::Enter(Enter::Dialog | Enter::Sheet) => {
                if entering {
                    DurationStep::Short4
                } else {
                    DurationStep::Short3
                }
            }
            Self::Enter(Enter::Menu) => {
                if entering {
                    DurationStep::Short3
                } else {
                    DurationStep::Short2
                }
            }
            Self::Enter(Enter::Toast) => DurationStep::Short3,
            Self::Enter(Enter::Tooltip) => {
                if entering {
                    DurationStep::Short2
                } else {
                    DurationStep::Short1
                }
            }
            Self::Switch(SwitchFace::FadeThrough) => DurationStep::Medium1,
            Self::Switch(SwitchFace::SharedAxis(_)) => DurationStep::Medium2,
            Self::Disclose(_) => DurationStep::Medium1,
            Self::Attention(AttentionFace::Shake) => DurationStep::Short4,
            Self::Attention(AttentionFace::Pulse) => DurationStep::Medium2,
            Self::Value => DurationStep::Medium2,
        };
        step.duration(false)
    }

    /// Ease for this direction. Enter decelerates; exit accelerates.
    pub fn ease(self, entering: bool) -> Ease {
        match self {
            Self::Switch(SwitchFace::FadeThrough) => Ease::Standard,
            Self::Enter(_) | Self::Switch(_) if entering => Ease::EmphasizedDecelerate,
            Self::Enter(_) | Self::Switch(_) => Ease::EmphasizedAccelerate,
            Self::Disclose(_) => Ease::EmphasizedDecelerate,
            Self::Attention(_) | Self::Value => Ease::Standard,
        }
    }
}

/// One 0..=1 clock for a [`Job`]. Rebuilds duration and ease on [`Run::go`].
#[derive(Debug, Clone)]
pub struct Run {
    anim: iced::Animation<bool>,
    job: Job,
    reduced: bool,
    enter_hold: Option<Duration>,
}

impl Run {
    /// Rest at `open` (1) or closed (0).
    pub fn new(job: Job, open: bool, reduced: bool) -> Self {
        let mut run = Self {
            anim: iced::Animation::new(open),
            job,
            reduced,
            enter_hold: None,
        };
        run.anim = iced::Animation::new(open)
            .duration(run.span(open))
            .easing(job.ease(open).lilt());
        run
    }

    /// Hold this enter duration. Exit keeps the job's shorter ratio.
    ///
    /// Reduced motion still snaps to 0 ms. Use this from a gallery
    /// or when an application has felt a length; the default path is
    /// the job step.
    pub fn lasting(mut self, enter: Duration) -> Self {
        self.enter_hold = Some(enter);
        let open = self.anim.value();
        self.anim = iced::Animation::new(open)
            .duration(self.span(open))
            .easing(self.job.ease(open).lilt());
        self
    }

    /// Remember an enter length without snapping the current play.
    pub fn hold(&mut self, enter: Duration) {
        self.enter_hold = Some(enter);
    }

    fn span(&self, entering: bool) -> Duration {
        if self.reduced {
            return Duration::ZERO;
        }
        match self.enter_hold {
            None => self.job.duration(entering, false),
            Some(enter) => {
                if entering {
                    enter
                } else {
                    let je = self.job.duration(true, false).as_secs_f32().max(0.001);
                    let jx = self.job.duration(false, false).as_secs_f32();
                    Duration::from_secs_f32(enter.as_secs_f32() * (jx / je))
                }
            }
        }
    }

    /// Drive toward open or closed. Exit uses the job's shorter step.
    pub fn go(&mut self, open: bool, now: Instant) {
        let from = self.anim.value();
        self.anim = iced::Animation::new(from)
            .duration(self.span(open))
            .easing(self.job.ease(open).lilt());
        self.anim.go_mut(open, now);
    }

    /// Enter or exit length this run will use.
    pub fn duration(&self, entering: bool) -> Duration {
        self.span(entering)
    }

    /// Interpolated 0..=1 at `now`.
    pub fn progress(&self, now: Instant) -> f32 {
        self.anim.interpolate(0.0, 1.0, now)
    }

    /// True while the value is still moving.
    pub fn is_live(&self, now: Instant) -> bool {
        self.anim.is_animating(now)
    }
}

/// Configure a [`Run`] at rest.
pub fn run(job: Job, open: bool, reduced: bool) -> Run {
    Run::new(job, open, reduced)
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
/// Pass the result into [`overlay()`] or [`expand()`] the same way as
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

/// Clip a child between a peek size and its open size on one [`Axis`].
///
/// `progress` 0 is `peek` pixels (0 hides). 1 is the child's laid-out
/// size on that axis. [`Axis::Block`] is height; [`Axis::Inline`] is
/// width (drawer, folder rail). Reduced-motion tokens snap.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::motion::{self, Axis};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let body = widget::label("more", tok, A11y::new("more", Role::Status));
/// let _: icedtea::Element<'_, ()> = motion::expand(
///     body,
///     1.0,
///     0.0,
///     Axis::Block,
///     tok,
///     A11y::new("expand", Role::Group),
/// );
/// ```
pub fn expand<'a, M: 'a>(
    child: Element<'a, M>,
    progress: f32,
    peek: f32,
    axis: Axis,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    a11y::attach(
        ExpandLayer {
            content: child,
            progress,
            peek: peek.max(0.0),
            axis,
            reduced: tok.reduced_motion,
        }
        .into(),
        &a11y,
    )
}

/// Replace `outgoing` with `incoming` from a 0..=1 progress.
///
/// [`SwitchFace::SharedAxis`] is next/previous peers: incoming uses
/// `progress` on `slide`, leaving uses `1 - progress` on the opposite
/// slide, travel is [`m3::motion::OVERLAY_SLIDE`](crate::m3::motion::OVERLAY_SLIDE)
/// (12 dp). Build each child with [`Tokens::fade`](`crate::theme::Tokens::fade`)
/// from [`SwitchFace::incoming_fade`] / [`SwitchFace::outgoing_fade`].
/// [`SwitchFace::FadeThrough`] is tab bodies and other unrelated
/// destinations. Reduced-motion tokens snap. Child overlays (pick
/// lists) still open.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::motion::{self, Slide, SwitchFace};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let leaving = widget::label("one", tok, A11y::new("one", Role::Status));
/// let incoming = widget::label("two", tok, A11y::new("two", Role::Status));
/// let _: icedtea::Element<'_, ()> = motion::switch(
///     leaving,
///     incoming,
///     1.0,
///     SwitchFace::SharedAxis(Slide::Up),
///     tok,
///     A11y::new("step", Role::Group),
/// );
/// ```
pub fn switch<'a, M: 'a>(
    outgoing: Element<'a, M>,
    incoming: Element<'a, M>,
    progress: f32,
    face: SwitchFace,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    a11y::attach(
        SwitchLayer {
            outgoing,
            incoming,
            progress,
            face,
            reduced: tok.reduced_motion,
        }
        .into(),
        &a11y,
    )
}

/// Shake or pulse a child from a 0..=1 progress.
///
/// [`AttentionFace::Shake`] is a decaying wiggle that starts and ends
/// at rest (invalid field). [`AttentionFace::Pulse`] scales about the
/// center (live mark). Reduced-motion tokens hold rest.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::motion::{self, AttentionFace};
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let field = widget::label("Name", tok, A11y::new("Name", Role::Status));
/// let _: icedtea::Element<'_, ()> = motion::attention(
///     field,
///     0.0,
///     AttentionFace::Shake,
///     tok,
///     A11y::new("invalid", Role::Group),
/// );
/// ```
pub fn attention<'a, M: 'a>(
    child: Element<'a, M>,
    progress: f32,
    face: AttentionFace,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    a11y::attach(
        AttentionLayer {
            content: child,
            progress,
            face,
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

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        if visual(self.progress, self.reduced) <= 0.0 {
            return None;
        }
        let delta = self.slide_delta();
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation + delta,
        )
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
    axis: Axis,
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
        match self.axis {
            Axis::Block => Size::new(Length::Fill, Length::Shrink),
            Axis::Inline => Size::new(Length::Shrink, Length::Fill),
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let max = limits.max();
        let child_limits = match self.axis {
            Axis::Block => layout::Limits::new(Size::ZERO, Size::new(max.width, f32::INFINITY)),
            Axis::Inline => layout::Limits::new(Size::ZERO, Size::new(f32::INFINITY, max.height)),
        };
        let child =
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &child_limits);
        let t = visual(self.progress, self.reduced);
        let size = match self.axis {
            Axis::Block => Size::new(
                child.size().width,
                height(t, self.peek, child.size().height).max(0.0),
            ),
            Axis::Inline => Size::new(
                height(t, self.peek, child.size().width).max(0.0),
                child.size().height,
            ),
        };
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

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        if visual(self.progress, self.reduced) <= 0.0 && self.peek <= 0.0 {
            return None;
        }
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: 'a> From<ExpandLayer<'a, Message>> for Element<'a, Message> {
    fn from(value: ExpandLayer<'a, Message>) -> Self {
        Self::new(value)
    }
}

struct SwitchLayer<'a, Message> {
    outgoing: Element<'a, Message>,
    incoming: Element<'a, Message>,
    progress: f32,
    face: SwitchFace,
    reduced: bool,
}

impl<Message> SwitchLayer<'_, Message> {
    fn t(&self) -> f32 {
        visual(self.progress, self.reduced)
    }

    fn incoming_delta(&self) -> Vector {
        let remain = 1.0 - self.t();
        offset_delta(self.face.incoming_slide(), m3motion::OVERLAY_SLIDE * remain)
    }

    fn outgoing_delta(&self) -> Vector {
        offset_delta(
            self.face.outgoing_slide(),
            m3motion::OVERLAY_SLIDE * self.t(),
        )
    }
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for SwitchLayer<'a, Message> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.outgoing), Tree::new(&self.incoming)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.outgoing, &self.incoming]);
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
        let out = self
            .outgoing
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let incoming =
            self.incoming
                .as_widget_mut()
                .layout(&mut tree.children[1], renderer, limits);
        let size = Size::new(
            out.size().width.max(incoming.size().width),
            out.size().height.max(incoming.size().height),
        );
        layout::Node::with_children(size, vec![out, incoming])
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
        let t = self.t();
        let out_delta = self.outgoing_delta();
        let in_delta = self.incoming_delta();
        let mut kids = layout.children();
        let out_layout = kids.next().unwrap();
        let in_layout = kids.next().unwrap();
        if t < 1.0 {
            self.outgoing.as_widget_mut().update(
                &mut tree.children[0],
                event,
                out_layout,
                cursor - out_delta,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
        if t > 0.0 {
            self.incoming.as_widget_mut().update(
                &mut tree.children[1],
                event,
                in_layout,
                cursor - in_delta,
                renderer,
                clipboard,
                shell,
                viewport,
            );
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
        let t = self.t();
        let mut kids = layout.children();
        let out_layout = kids.next().unwrap();
        let in_layout = kids.next().unwrap();
        let incoming = if t > 0.0 {
            self.incoming.as_widget().mouse_interaction(
                &tree.children[1],
                in_layout,
                cursor - self.incoming_delta(),
                viewport,
                renderer,
            )
        } else {
            mouse::Interaction::None
        };
        if incoming != mouse::Interaction::None {
            return incoming;
        }
        if t < 1.0 {
            self.outgoing.as_widget().mouse_interaction(
                &tree.children[0],
                out_layout,
                cursor - self.outgoing_delta(),
                viewport,
                renderer,
            )
        } else {
            mouse::Interaction::None
        }
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
        let t = self.t();
        let mut kids = layout.children();
        let out_layout = kids.next().unwrap();
        let in_layout = kids.next().unwrap();
        if self.face.outgoing_fade(t) > 0.0 {
            renderer.with_translation(self.outgoing_delta(), |renderer| {
                self.outgoing.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    out_layout,
                    cursor,
                    viewport,
                );
            });
        }
        if self.face.incoming_fade(t) > 0.0 {
            renderer.with_translation(self.incoming_delta(), |renderer| {
                self.incoming.as_widget().draw(
                    &tree.children[1],
                    renderer,
                    theme,
                    style,
                    in_layout,
                    cursor,
                    viewport,
                );
            });
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        let mut kids = layout.children();
        let out_layout = kids.next().unwrap();
        let in_layout = kids.next().unwrap();
        self.outgoing.as_widget_mut().operate(
            &mut tree.children[0],
            out_layout,
            renderer,
            operation,
        );
        self.incoming.as_widget_mut().operate(
            &mut tree.children[1],
            in_layout,
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
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        let t = self.t();
        let out_delta = self.outgoing_delta();
        let in_delta = self.incoming_delta();
        let mut kids = layout.children();
        let out_layout = kids.next()?;
        let in_layout = kids.next()?;
        let (out_tree, rest) = tree.children.split_at_mut(1);
        let out = if t < 1.0 {
            self.outgoing.as_widget_mut().overlay(
                &mut out_tree[0],
                out_layout,
                renderer,
                viewport,
                translation + out_delta,
            )
        } else {
            None
        };
        let incoming = if t > 0.0 {
            self.incoming.as_widget_mut().overlay(
                &mut rest[0],
                in_layout,
                renderer,
                viewport,
                translation + in_delta,
            )
        } else {
            None
        };
        match (out, incoming) {
            (Some(a), Some(b)) => Some(overlay::Group::with_children(vec![a, b]).overlay()),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }
}

impl<'a, Message: 'a> From<SwitchLayer<'a, Message>> for Element<'a, Message> {
    fn from(value: SwitchLayer<'a, Message>) -> Self {
        Self::new(value)
    }
}

const ATTENTION_SHAKE: f32 = 8.0;
const ATTENTION_PULSE: f32 = 0.06;

struct AttentionLayer<'a, Message> {
    content: Element<'a, Message>,
    progress: f32,
    face: AttentionFace,
    reduced: bool,
}

impl<Message> AttentionLayer<'_, Message> {
    fn t(&self) -> f32 {
        visual(self.progress, self.reduced)
    }

    fn delta(&self) -> Vector {
        match self.face {
            AttentionFace::Shake if !self.reduced => {
                Vector::new(shake(self.t()) * ATTENTION_SHAKE, 0.0)
            }
            _ => Vector::ZERO,
        }
    }

    fn transform(&self, bounds: Rectangle) -> Transformation {
        match self.face {
            AttentionFace::Pulse if !self.reduced => {
                let s = 1.0 + ATTENTION_PULSE * pulse(self.t());
                let c = bounds.center();
                Transformation::translate(c.x, c.y)
                    * Transformation::scale(s)
                    * Transformation::translate(-c.x, -c.y)
            }
            _ => Transformation::translate(self.delta().x, self.delta().y),
        }
    }
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for AttentionLayer<'a, Message> {
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
        let delta = self.delta();
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor - delta,
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
            cursor - self.delta(),
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
        let child_layout = layout.children().next().unwrap();
        renderer.with_transformation(self.transform(layout.bounds()), |renderer| {
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

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        let delta = self.delta();
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation + delta,
        )
    }
}

impl<'a, Message: 'a> From<AttentionLayer<'a, Message>> for Element<'a, Message> {
    fn from(value: AttentionLayer<'a, Message>) -> Self {
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
        let _: Element<'_, ()> = expand(
            body,
            0.3,
            8.0,
            Axis::Block,
            reduced,
            A11y::new("expand", Role::Group),
        );
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
        let _ = el
            .as_widget_mut()
            .overlay(&mut tree, layout, &renderer, &viewport, Vector::ZERO);
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
            let mut el: Element<'_, ()> = expand(
                body(),
                p,
                peek,
                Axis::Block,
                t,
                A11y::new("expand", Role::Group),
            );
            drive(&mut el, vp);
            drive(&mut el, miss);
        }
        let far = iced::Rectangle::new(iced::Point::new(4000.0, 4000.0), iced::Size::new(4.0, 4.0));
        let mut hidden: Element<'_, ()> = expand(
            body(),
            0.0,
            0.0,
            Axis::Block,
            tok,
            A11y::new("gone", Role::Group),
        );
        drive(&mut hidden, far);
        let mut clipped: Element<'_, ()> = expand(
            body(),
            1.0,
            0.0,
            Axis::Block,
            tok,
            A11y::new("full", Role::Group),
        );
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
        let face = widget::button(
            "Copy",
            Some(()),
            tok,
            Variant::Ghost,
            crate::icon::Icons::NONE,
            crate::widget::ButtonOpts::SHRINK,
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
        let face = widget::button(
            "Copy",
            Some(()),
            tok,
            Variant::Ghost,
            crate::icon::Icons::NONE,
            crate::widget::ButtonOpts::SHRINK,
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
        let spacer_msg = "a press on the placement spacer must reach the dismiss surface";
        assert!(!captured, "{spacer_msg}");
        assert!(messages.is_empty(), "spacer press must not run the row");
    }

    #[test]
    fn lasting_scales_exit_with_the_job_ratio() {
        let job = Job::Enter(Enter::Dialog);
        let enter = job.duration(true, false);
        let exit = job.duration(false, false);
        assert!(exit < enter);
        let hold = Duration::from_millis(400);
        let mut clock = run(job, true, false).lasting(hold);
        let now = Instant::now();
        clock.go(false, now);
        let ratio = exit.as_secs_f32() / enter.as_secs_f32();
        let want = hold.as_secs_f32() * ratio;
        assert!((clock.duration(false).as_secs_f32() - want).abs() < 0.001);
        assert_eq!(clock.duration(true), hold);
        let snap = run(job, true, true).lasting(hold);
        assert_eq!(snap.span(true), Duration::ZERO);
    }

    #[test]
    fn lasting_fade_through_is_early_at_one_fifth() {
        let hold = Duration::from_millis(500);
        let mut clock = run(Job::Switch(SwitchFace::FadeThrough), false, false).lasting(hold);
        let now = Instant::now();
        clock.go(true, now);
        assert!(clock.is_live(now));
        let p = clock.progress(now + Duration::from_millis(100));
        assert!((0.02..0.35).contains(&p));
    }

    #[test]
    fn enter_exit_uses_shorter_exit() {
        let enter = Job::Enter(Enter::Dialog).duration(true, false);
        let exit = Job::Enter(Enter::Dialog).duration(false, false);
        assert!(exit < enter);
        assert_eq!(
            Job::Enter(Enter::Dialog).duration(true, true),
            Duration::ZERO
        );
        assert_eq!(
            Job::Switch(SwitchFace::SharedAxis(Slide::Up)).duration(true, false),
            DurationStep::Medium2.duration(false)
        );
        assert_eq!(
            Job::Enter(Enter::Dialog).ease(true),
            Ease::EmphasizedDecelerate
        );
        assert_eq!(
            Job::Enter(Enter::Dialog).ease(false),
            Ease::EmphasizedAccelerate
        );
        for (job, enter, exit) in [
            (
                Job::Enter(Enter::Menu),
                DurationStep::Short3,
                DurationStep::Short2,
            ),
            (
                Job::Enter(Enter::Toast),
                DurationStep::Short3,
                DurationStep::Short3,
            ),
            (
                Job::Enter(Enter::Tooltip),
                DurationStep::Short2,
                DurationStep::Short1,
            ),
            (
                Job::Disclose(Axis::Block),
                DurationStep::Medium1,
                DurationStep::Medium1,
            ),
            (
                Job::Attention(AttentionFace::Shake),
                DurationStep::Short4,
                DurationStep::Short4,
            ),
            (
                Job::Attention(AttentionFace::Pulse),
                DurationStep::Medium2,
                DurationStep::Medium2,
            ),
            (Job::Value, DurationStep::Medium2, DurationStep::Medium2),
        ] {
            assert_eq!(job.duration(true, false), enter.duration(false));
            assert_eq!(job.duration(false, false), exit.duration(false));
        }
        assert_eq!(
            Job::Disclose(Axis::Inline).ease(true),
            Ease::EmphasizedDecelerate
        );
        assert_eq!(
            Job::Attention(AttentionFace::Shake).ease(true),
            Ease::Standard
        );
        assert_eq!(Job::Value.ease(false), Ease::Standard);
    }

    #[test]
    fn switch_forwards_incoming_pointer_and_child_overlays() {
        use crate::variant::Variant;
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::widget::Tree;
        use iced::{Font, Pixels, Point, Size};
        let tok = named("dark").tokens;
        let face = || {
            widget::tooltip_wrap(
                widget::button(
                    "Go",
                    Some(()),
                    tok,
                    Variant::Primary,
                    crate::icon::Icons::NONE,
                    crate::widget::ButtonOpts::SHRINK,
                    A11y::button("Go"),
                ),
                "tip",
                crate::widget::TooltipAnchor::Follow,
                tok,
                A11y::new("tip", Role::Tooltip),
            )
        };
        let mut el: Element<'_, ()> = switch(
            face(),
            face(),
            0.5,
            SwitchFace::FadeThrough,
            tok,
            A11y::new("sw", Role::Group),
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
        let at = mouse::Cursor::Available(Point::new(16.0, 16.0));
        let hit = el
            .as_widget()
            .mouse_interaction(&tree, layout, at, &viewport, &renderer);
        assert_ne!(hit, mouse::Interaction::None);
        let _ = el
            .as_widget_mut()
            .overlay(&mut tree, layout, &renderer, &viewport, Vector::ZERO);
        let mut only_in: Element<'_, ()> = switch(
            face(),
            face(),
            1.0,
            SwitchFace::FadeThrough,
            tok,
            A11y::new("in", Role::Group),
        );
        drive(
            &mut only_in,
            iced::Rectangle::new(Point::ORIGIN, Size::new(320.0, 240.0)),
        );
        let mut only_out: Element<'_, ()> = switch(
            face(),
            face(),
            0.0,
            SwitchFace::FadeThrough,
            tok,
            A11y::new("out", Role::Group),
        );
        drive(
            &mut only_out,
            iced::Rectangle::new(Point::ORIGIN, Size::new(320.0, 240.0)),
        );

        struct OverlayChild;
        impl iced::advanced::widget::Widget<(), iced::Theme, iced::Renderer> for OverlayChild {
            fn size(&self) -> Size<Length> {
                Size::new(Length::Fill, Length::Shrink)
            }
            fn layout(
                &mut self,
                _tree: &mut Tree,
                _renderer: &iced::Renderer,
                limits: &Limits,
            ) -> iced::advanced::layout::Node {
                iced::advanced::layout::Node::new(Size::new(limits.max().width.min(80.0), 24.0))
            }
            fn draw(
                &self,
                _tree: &Tree,
                _renderer: &mut iced::Renderer,
                _theme: &iced::Theme,
                _style: &iced::advanced::renderer::Style,
                _layout: Layout<'_>,
                _cursor: mouse::Cursor,
                _viewport: &iced::Rectangle,
            ) {
            }
            fn overlay<'b>(
                &'b mut self,
                _tree: &'b mut Tree,
                _layout: Layout<'b>,
                _renderer: &iced::Renderer,
                _viewport: &iced::Rectangle,
                _translation: Vector,
            ) -> Option<iced::advanced::overlay::Element<'b, (), iced::Theme, iced::Renderer>>
            {
                Some(iced::advanced::overlay::Group::with_children(Vec::new()).overlay())
            }
        }
        let _ =
            iced::advanced::widget::Widget::<(), iced::Theme, iced::Renderer>::size(&OverlayChild);
        let kid = || Element::<()>::new(OverlayChild);
        let mut both: Element<'_, ()> = switch(
            kid(),
            kid(),
            0.5,
            SwitchFace::FadeThrough,
            tok,
            A11y::new("both", Role::Group),
        );
        drive(&mut both, viewport);
    }

    #[test]
    fn run_progress_snaps_when_reduced() {
        let now = Instant::now();
        let rest = run(Job::Enter(Enter::Dialog), true, true);
        assert!((rest.progress(now) - 1.0).abs() < 1e-5);
        assert!(!rest.is_live(now));
        let mut closing = run(Job::Enter(Enter::Dialog), true, true);
        closing.go(false, now);
        assert!((closing.progress(now) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn shared_axis_fades_cross_and_slides_opposite() {
        let face = SwitchFace::SharedAxis(Slide::Up);
        assert!((face.incoming_fade(0.0) - 0.0).abs() < 1e-5);
        assert!((face.incoming_fade(1.0) - 1.0).abs() < 1e-5);
        assert!((face.outgoing_fade(0.0) - 1.0).abs() < 1e-5);
        assert!((face.outgoing_fade(1.0) - 0.0).abs() < 1e-5);
        assert_eq!(face.incoming_slide(), Slide::Up);
        assert_eq!(face.outgoing_slide(), Slide::Down);
        let fade = SwitchFace::FadeThrough;
        assert!(fade.outgoing_fade(0.2) > 0.3);
        assert_eq!(fade.incoming_fade(0.2), 0.0);
        assert_eq!(fade.outgoing_fade(0.8), 0.0);
        assert!(fade.incoming_fade(0.8) > 0.5);
        assert_eq!(Slide::Start.opposite(), Slide::End);
        assert_eq!(Slide::None.opposite(), Slide::None);
    }

    #[test]
    fn switch_and_attention_constructors_build() {
        let tok = named("dark").tokens;
        let a = || widget::label("one", tok, A11y::new("one", Role::Status));
        let b = || widget::label("two", tok, A11y::new("two", Role::Status));
        let vp = iced::Rectangle::new(iced::Point::ORIGIN, iced::Size::new(320.0, 240.0));
        for face in [
            SwitchFace::FadeThrough,
            SwitchFace::SharedAxis(Slide::Up),
            SwitchFace::SharedAxis(Slide::Down),
            SwitchFace::SharedAxis(Slide::Start),
            SwitchFace::SharedAxis(Slide::End),
        ] {
            for p in [0.0, 0.5, 1.0] {
                let mut el: Element<'_, ()> =
                    switch(a(), b(), p, face, tok, A11y::new("step", Role::Group));
                drive(&mut el, vp);
            }
        }
        for face in [AttentionFace::Shake, AttentionFace::Pulse] {
            for p in [0.0, 0.3, 1.0] {
                let mut el: Element<'_, ()> =
                    attention(a(), p, face, tok, A11y::new("attn", Role::Group));
                drive(&mut el, vp);
            }
        }
        let mut wide: Element<'_, ()> = expand(
            a(),
            0.5,
            0.0,
            Axis::Inline,
            tok,
            A11y::new("rail", Role::Group),
        );
        drive(&mut wide, vp);
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
            axis: Axis::Block,
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
