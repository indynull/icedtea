//! Radius and elevation presets.

use iced::border::Radius;
use iced::Shadow;
use iced::{Color, Vector};

/// Corner rounding. [`Corner::None`] is flush (no radius).
///
/// ```
/// use icedtea::chrome::Corner;
/// assert_eq!(Corner::Tight.radius_px(), 4.0);
/// assert_eq!(Corner::None.radius_px(), 0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Corner {
    #[default]
    None,
    Tight,
    Soft,
}

impl Corner {
    pub fn radius_px(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Tight => 4.0,
            Self::Soft => 8.0,
        }
    }

    pub fn radius(self) -> Radius {
        Radius::from(self.radius_px())
    }
}

/// Minimum scrollbar handle on the 4px grid. Iced's scroller floors at 2px.
pub const SCROLL_HANDLE_MIN: f32 = 24.0;
/// Rail and handle width.
pub const SCROLL_RAIL_WIDTH: f32 = 12.0;

/// Shadow / lift.
///
/// ```
/// use icedtea::chrome::Elevation;
/// assert!(Elevation::Raised.shadow().blur_radius > 0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Elevation {
    #[default]
    Flat,
    Raised,
}

impl Elevation {
    pub fn shadow(self) -> Shadow {
        match self {
            Self::Flat => Shadow::default(),
            Self::Raised => Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.24),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_and_elevation_presets() {
        assert_eq!(Corner::default().radius_px(), 0.0);
        assert_eq!(Corner::Tight.radius_px(), 4.0);
        assert_eq!(Corner::Soft.radius_px(), 8.0);
        assert_eq!(Corner::Tight.radius().top_left, 4.0);
        assert_eq!(Elevation::default().shadow().blur_radius, 0.0);
        assert!(Elevation::Raised.shadow().blur_radius > 0.0);
        assert_eq!(SCROLL_HANDLE_MIN, 24.0);
        assert_eq!(SCROLL_RAIL_WIDTH, 12.0);
    }
}
