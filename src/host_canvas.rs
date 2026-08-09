//! iced canvas stroke for rings. Renderer glue; excluded from coverage fail-under.

use iced::mouse;
use iced::widget::canvas::{self, Path, Stroke};
use iced::{Color, Radians, Rectangle, Renderer, Theme};

use crate::widget::ring_should_stroke;

/// Arc + track drawn by [`iced::widget::canvas`].
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
        frame.stroke(
            &Path::circle(center, radius),
            Stroke::default().with_width(4.0).with_color(self.track),
        );
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
