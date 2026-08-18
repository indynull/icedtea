//! Radius and elevation presets (Material Design 3).

pub use crate::m3::elevation::{Elevation, ElevationPolicy};
pub use crate::m3::shape::{Corner, ShapePolicy};

/// Minimum scrollbar handle on the 4 dp grid.
pub const SCROLL_HANDLE_MIN: f32 = 24.0;
/// Rail and handle width.
pub const SCROLL_RAIL_WIDTH: f32 = 12.0;
/// Content pixels per mouse-wheel line. Same step as iced's `scrollable`.
pub const SCROLL_LINE: f32 = 60.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_and_elevation_presets() {
        assert_eq!(Corner::default().radius_px(), 0.0);
        assert_eq!(Corner::Tight.radius_px(), 4.0);
        assert_eq!(Corner::Soft.radius_px(), 12.0);
        assert!(Elevation::Raised.shadow().blur_radius > 0.0);
        assert_eq!(SCROLL_HANDLE_MIN, 24.0);
        assert_eq!(SCROLL_LINE, 60.0);
    }
}
