//! iced canvas stroke for rings. Renderer glue; excluded from coverage fail-under.

use iced::mouse;
use iced::widget::canvas::{self, Path, Stroke};
use iced::{Color, Point, Radians, Rectangle, Renderer, Theme};

use crate::widget::ring_should_stroke;

/// Arc + track drawn by [`iced::widget::canvas()`].
#[derive(Clone, Copy)]
pub(crate) struct ArcRing {
    pub start: f32,
    pub end: f32,
    pub color: Color,
    pub track: Color,
}

impl<Message> canvas::Program<Message> for ArcRing {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let center = frame.center();
        let radius = bounds.width.min(bounds.height) / 2.0 - 4.0;
        if self.track.a > 0.001 {
            frame.stroke(
                &Path::circle(center, radius),
                Stroke::default().with_width(4.0).with_color(self.track),
            );
        }
        if ring_should_stroke(self.start, self.end) {
            let arc = Path::new(|b| {
                b.arc(canvas::path::Arc {
                    center,
                    radius,
                    start_angle: Radians(self.start),
                    end_angle: Radians(self.end),
                });
            });
            frame.stroke(
                &arc,
                Stroke::default().with_width(4.0).with_color(self.color),
            );
        }
        vec![frame.into_geometry()]
    }
}

/// Eight dots; `phase` (0..=1) lights them in turn.
#[derive(Clone, Copy)]
pub(crate) struct SpinnerDots {
    pub phase: f32,
    pub color: Color,
}

impl<Message> canvas::Program<Message> for SpinnerDots {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let center = frame.center();
        let radius = bounds.width.min(bounds.height) / 2.0 - 6.0;
        let n = 8;
        for i in 0..n {
            let t = i as f32 / n as f32;
            let ang = t * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
            let p = Point::new(center.x + radius * ang.cos(), center.y + radius * ang.sin());
            let delta = (self.phase.rem_euclid(1.0) - t).rem_euclid(1.0);
            let a = 0.18 + 0.82 * (1.0 - delta).powi(2);
            let c = Color {
                a: self.color.a * a,
                ..self.color
            };
            frame.fill(&Path::circle(p, 3.4), c);
        }
        vec![frame.into_geometry()]
    }
}
