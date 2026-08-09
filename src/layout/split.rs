//! Split ratio and sash drag.

use iced::{mouse, Event, Point, Subscription};
use serde::{Deserialize, Serialize};

/// Axis of a split.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// Two-pane split with a persistable ratio (0–1 is the first pane share).
///
/// ```
/// use icedtea::layout::{Axis, SplitState};
/// let mut split = SplitState::new(Axis::Horizontal, 0.3);
/// let before = split.ratio;
/// split.drag(10.0, 200.0);
/// assert!(split.ratio > before);
/// assert_eq!(split.persist(), split.ratio);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SplitState {
    pub axis: Axis,
    pub ratio: f32,
    pub sash: f32,
    pub min_ratio: f32,
    pub max_ratio: f32,
}

impl SplitState {
    pub fn new(axis: Axis, ratio: f32) -> Self {
        Self {
            axis,
            ratio: ratio.clamp(0.05, 0.95),
            sash: 6.0,
            min_ratio: 0.12,
            max_ratio: 0.88,
        }
    }

    pub fn restore(axis: Axis, ratio: f32) -> Self {
        Self::new(axis, ratio)
    }

    pub fn persist(self) -> f32 {
        self.ratio
    }

    pub fn first_size(self, total: f32) -> f32 {
        let usable = (total - self.sash).max(0.0);
        usable * self.ratio
    }

    pub fn second_size(self, total: f32) -> f32 {
        let usable = (total - self.sash).max(0.0);
        usable - self.first_size(total)
    }

    pub fn drag(&mut self, delta_px: f32, total: f32) {
        let usable = (total - self.sash).max(1.0);
        self.ratio = (self.ratio + delta_px / usable).clamp(self.min_ratio, self.max_ratio);
    }
}

/// Pointer events on a split sash.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SashEvent {
    Press,
    Move(f32),
    Release,
}

/// Window-space pointer while a sash is pressed (not the 6px grip local point).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerDrive {
    Move(Point),
    Release,
}

/// Axis coordinate of a window-space pointer.
pub fn sash_pointer_pos(axis: Axis, p: Point) -> f32 {
    match axis {
        Axis::Horizontal => p.x,
        Axis::Vertical => p.y,
    }
}

impl PointerDrive {
    /// Convert window-space motion into a sash event on `axis`.
    pub fn into_event(self, axis: Axis) -> SashEvent {
        match self {
            Self::Move(p) => SashEvent::Move(sash_pointer_pos(axis, p)),
            Self::Release => SashEvent::Release,
        }
    }
}

/// Map a window mouse event to sash drive (move uses window position).
pub fn sash_from_window_event(event: &Event) -> Option<PointerDrive> {
    match event {
        Event::Mouse(mouse::Event::CursorMoved { position }) => Some(PointerDrive::Move(*position)),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
            Some(PointerDrive::Release)
        }
        _ => None,
    }
}

/// Listen for window cursor motion and left-button release (sash drag).
pub fn listen_sash() -> Subscription<PointerDrive> {
    iced::event::listen_with(|event, _status, _id| sash_from_window_event(&event))
}

/// Drag session for a sash (last pointer position while pressed).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SashDrag {
    pub pressed: bool,
    pub last: Option<f32>,
}

impl SashDrag {
    /// Apply a sash event; returns whether [`SplitState::drag`] ran.
    pub fn apply(&mut self, state: &mut SplitState, event: SashEvent, total: f32) -> bool {
        match event {
            SashEvent::Press => {
                self.pressed = true;
                self.last = None;
                false
            }
            SashEvent::Release => {
                self.pressed = false;
                self.last = None;
                false
            }
            SashEvent::Move(pos) => {
                let dragged = if self.pressed {
                    if let Some(prev) = self.last {
                        state.drag(pos - prev, total);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                self.last = Some(pos);
                dragged
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sash_math_and_persist() {
        let s = SplitState::new(Axis::Vertical, 2.0);
        assert!(s.ratio <= 0.95);
        let mut t = SplitState::restore(Axis::Horizontal, 0.25);
        assert_eq!(t.persist(), t.ratio);
        let total = 206.0;
        assert!((t.first_size(total) + t.second_size(total) + t.sash - total).abs() < 0.01);
        t.drag(1000.0, total);
        assert!((t.ratio - t.max_ratio).abs() < 0.001);
        t.drag(-1000.0, total);
        assert!((t.ratio - t.min_ratio).abs() < 0.001);
        let mut z = SplitState::new(Axis::Horizontal, 0.5);
        assert_eq!(z.first_size(0.0), 0.0);
        z.drag(1.0, 0.0);
        let mut drag = SashDrag::default();
        let mut st = SplitState::new(Axis::Horizontal, 0.3);
        assert!(!drag.apply(&mut st, SashEvent::Press, 200.0));
        assert!(!drag.apply(&mut st, SashEvent::Move(10.0), 200.0));
        let before = st.ratio;
        assert!(drag.apply(&mut st, SashEvent::Move(30.0), 200.0));
        assert!(st.ratio > before);
        assert!(!drag.apply(&mut st, SashEvent::Release, 200.0));
        assert!(!drag.pressed);
        assert!(!drag.apply(&mut st, SashEvent::Move(40.0), 200.0));
        assert!(!drag.pressed);

        let moved = Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(120.0, 40.0),
        });
        let drive = sash_from_window_event(&moved).expect("cursor");
        assert_eq!(drive.into_event(Axis::Horizontal), SashEvent::Move(120.0));
        assert_eq!(drive.into_event(Axis::Vertical), SashEvent::Move(40.0));
        let released = Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left));
        assert_eq!(
            sash_from_window_event(&released)
                .unwrap()
                .into_event(Axis::Horizontal),
            SashEvent::Release
        );
        assert!(sash_from_window_event(&Event::Mouse(mouse::Event::CursorLeft)).is_none());

        let mut drag = SashDrag::default();
        let mut st = SplitState::new(Axis::Horizontal, 0.3);
        let total = 400.0;
        assert!(!drag.apply(&mut st, SashEvent::Press, total));
        let first = Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(100.0, 8.0),
        });
        assert!(!drag.apply(
            &mut st,
            sash_from_window_event(&first)
                .unwrap()
                .into_event(Axis::Horizontal),
            total
        ));
        let before = st.ratio;
        let second = Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(180.0, 8.0),
        });
        assert!(drag.apply(
            &mut st,
            sash_from_window_event(&second)
                .unwrap()
                .into_event(Axis::Horizontal),
            total
        ));
        assert!(st.ratio > before);
        let window_delta = 180.0 - 100.0;
        assert!(window_delta > st.sash);
        assert!(!drag.apply(
            &mut st,
            sash_from_window_event(&released)
                .unwrap()
                .into_event(Axis::Horizontal),
            total
        ));
        let _ = listen_sash();
    }
}
