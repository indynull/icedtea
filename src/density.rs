//! Spacing densities on a 4px grid.

use serde::{Deserialize, Serialize};

/// Named density presets. Default space is 8px.
///
/// ```
/// use icedtea::density::{Density, DensityName};
/// assert_eq!(Density::named(DensityName::Default).space, 8);
/// assert_eq!(Density::named(DensityName::Compact).space, 4);
/// assert_eq!(Density::named(DensityName::Comfortable).space, 12);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DensityName {
    Compact,
    Default,
    Comfortable,
}

/// Resolved spacing for a density.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Density {
    pub name: DensityName,
    /// Base gap (4 / 8 / 12).
    pub space: u32,
    /// Control padding (6 / 8 / 12).
    pub pad: u32,
}

impl Density {
    pub fn named(name: DensityName) -> Self {
        match name {
            DensityName::Compact => Self {
                name,
                space: 4,
                pad: 6,
            },
            DensityName::Default => Self {
                name,
                space: 8,
                pad: 8,
            },
            DensityName::Comfortable => Self {
                name,
                space: 12,
                pad: 12,
            },
        }
    }

    /// Snap a pixel value up to the 4px grid.
    pub fn snap(px: u32) -> u32 {
        let rem = px % 4;
        if rem == 0 {
            px
        } else {
            px + (4 - rem)
        }
    }
}

impl Default for Density {
    fn default() -> Self {
        Self::named(DensityName::Default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_sit_on_four_px_grid() {
        for name in [
            DensityName::Compact,
            DensityName::Default,
            DensityName::Comfortable,
        ] {
            let d = Density::named(name);
            assert_eq!(d.space % 4, 0);
            assert_eq!(d.name, name);
        }
        assert_eq!(Density::default().space, 8);
        assert_eq!(Density::snap(0), 0);
        assert_eq!(Density::snap(4), 4);
        assert_eq!(Density::snap(5), 8);
        assert_eq!(Density::snap(7), 8);
    }
}
