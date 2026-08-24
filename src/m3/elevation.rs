//! M3 elevation.

use super::color::Scheme;
use iced::{Color, Shadow, Vector};
use serde::{Deserialize, Serialize};

/// Whether constructors paint the documented shadow, or none.
///
/// Surfaces stay on their tonal container. Default is [`Self::Desktop`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ElevationPolicy {
    /// Per-component M3 levels (cards 1, menus 2, dialogs 3).
    #[default]
    Desktop,
    /// No drop shadow.
    Flat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Elevation {
    #[default]
    Level0,
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
    /// Desktop alias for Level2.
    Raised,
    Flat,
}

impl Elevation {
    /// Material resting height in dp (snapshot `styles/elevation`).
    pub fn dp(self) -> f32 {
        match self {
            Self::Level0 | Self::Flat => 0.0,
            Self::Level1 => 1.0,
            Self::Level2 | Self::Raised => 3.0,
            Self::Level3 => 6.0,
            Self::Level4 => 8.0,
            Self::Level5 => 12.0,
        }
    }

    /// One Material hover/focus step (level + 1, capped at 5).
    pub fn raise(self) -> Self {
        match self {
            Self::Level0 | Self::Flat => Self::Level1,
            Self::Level1 => Self::Level2,
            Self::Level2 => Self::Level3,
            Self::Raised => Self::Level3,
            Self::Level3 => Self::Level4,
            Self::Level4 | Self::Level5 => Self::Level5,
        }
    }

    pub fn shadow(self) -> Shadow {
        let (blur, y, a) = match self {
            Self::Level0 | Self::Flat => (0.0, 0.0, 0.0),
            // Offset y is the snapshot dp height. Blur and alpha are the
            // desktop paint so Level1 is visible on a dark surface.
            Self::Level1 => (3.0, 1.0, 0.32),
            Self::Level2 | Self::Raised => (6.0, 3.0, 0.30),
            Self::Level3 => (12.0, 6.0, 0.28),
            Self::Level4 => (16.0, 8.0, 0.26),
            Self::Level5 => (24.0, 12.0, 0.24),
        };
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, a),
            offset: Vector::new(0.0, y),
            blur_radius: blur,
        }
    }

    pub fn surface(self, scheme: Scheme) -> Color {
        match self {
            Self::Level0 | Self::Flat => scheme.surface,
            Self::Level1 => scheme.surface_container_low,
            Self::Level2 | Self::Raised => scheme.surface_container,
            Self::Level3 => scheme.surface_container_high,
            Self::Level4 | Self::Level5 => scheme.surface_container_highest,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m3::scheme_light;
    #[test]
    fn elevation_steps() {
        let s = scheme_light();
        assert_eq!(Elevation::Level0.surface(s), s.surface);
        assert_eq!(Elevation::Flat.surface(s), s.surface);
        assert_eq!(Elevation::Level1.surface(s), s.surface_container_low);
        assert_eq!(Elevation::Level2.surface(s), s.surface_container);
        assert_eq!(Elevation::Raised.surface(s), s.surface_container);
        assert_eq!(Elevation::Level3.surface(s), s.surface_container_high);
        assert_eq!(Elevation::Level4.surface(s), s.surface_container_highest);
        assert_eq!(Elevation::Level5.surface(s), s.surface_container_highest);
        assert_eq!(Elevation::Level0.dp(), 0.0);
        assert_eq!(Elevation::Level1.dp(), 1.0);
        assert_eq!(Elevation::Level2.dp(), 3.0);
        assert_eq!(Elevation::Level3.dp(), 6.0);
        assert_eq!(Elevation::Level4.dp(), 8.0);
        assert_eq!(Elevation::Level5.dp(), 12.0);
        assert_eq!(Elevation::Level0.shadow().blur_radius, 0.0);
        assert_eq!(Elevation::Flat.shadow().blur_radius, 0.0);
        assert!(Elevation::Level1.shadow().blur_radius > 0.0);
        assert!(Elevation::Level1.shadow().color.a >= 0.28);
        assert_ne!(
            Elevation::Level1.shadow().blur_radius,
            Elevation::Flat.shadow().blur_radius
        );
        assert_eq!(Elevation::Level1.shadow().offset.y, 1.0);
        assert_eq!(Elevation::Level2.shadow().offset.y, 3.0);
        assert_eq!(Elevation::Level3.shadow().offset.y, 6.0);
        assert!(Elevation::Raised.shadow().blur_radius > 0.0);
        assert!(Elevation::Level5.shadow().blur_radius >= Elevation::Level3.shadow().blur_radius);
        assert_eq!(Elevation::Level1.raise(), Elevation::Level2);
        assert_eq!(Elevation::Level2.raise(), Elevation::Level3);
        assert_eq!(Elevation::Raised.raise(), Elevation::Level3);
        assert_eq!(Elevation::Level5.raise(), Elevation::Level5);
        assert_eq!(Elevation::Level0.raise(), Elevation::Level1);
        for e in [
            Elevation::Level0,
            Elevation::Flat,
            Elevation::Level1,
            Elevation::Level2,
            Elevation::Raised,
            Elevation::Level3,
            Elevation::Level4,
            Elevation::Level5,
        ] {
            let _ = std::hint::black_box(e).raise();
        }
        let _ = Elevation::Level4.shadow();
        assert_eq!(ElevationPolicy::default(), ElevationPolicy::Desktop);
    }
}
