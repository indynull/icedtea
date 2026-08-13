//! M3 elevation.

use super::color::Scheme;
use iced::{Color, Shadow, Vector};

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
    pub fn shadow(self) -> Shadow {
        let e = match self {
            Self::Flat | Self::Level0 => Self::Level0,
            Self::Raised => Self::Level2,
            other => other,
        };
        let (blur, y, a) = match e {
            Self::Level0 | Self::Flat => (0.0, 0.0, 0.0),
            Self::Level1 => (2.0, 1.0, 0.15),
            Self::Level2 | Self::Raised => (4.0, 2.0, 0.18),
            Self::Level3 => (8.0, 4.0, 0.20),
            Self::Level4 => (12.0, 6.0, 0.22),
            Self::Level5 => (16.0, 8.0, 0.24),
        };
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, a),
            offset: Vector::new(0.0, y),
            blur_radius: blur,
        }
    }

    pub fn surface(self, scheme: Scheme) -> Color {
        let e = match self {
            Self::Flat => Self::Level0,
            Self::Raised => Self::Level2,
            other => other,
        };
        match e {
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
        assert_eq!(Elevation::Level0.shadow().blur_radius, 0.0);
        assert!(Elevation::Level1.shadow().blur_radius > 0.0);
        assert!(Elevation::Raised.shadow().blur_radius > 0.0);
        assert!(Elevation::Level5.shadow().blur_radius >= Elevation::Level3.shadow().blur_radius);
    }
}
