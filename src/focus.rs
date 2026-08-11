//! Roving tabindex, spatial arrows, landmarks, live regions.
//!
//! Use with [`crate::key::handle`] so arrow keys move between panels.

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
    }
}
