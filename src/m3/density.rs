//! M3 spacing on a 4 dp grid.

use serde::{Deserialize, Serialize};

pub const GRID: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DensityName {
    Compact,
    Default,
    Comfortable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Density {
    pub name: DensityName,
    pub space: u32,
    pub pad: u32,
}

impl Density {
    pub fn named(name: DensityName) -> Self {
        match name {
            DensityName::Compact => Self {
                name,
                space: 4,
                pad: 8,
            },
            DensityName::Default => Self {
                name,
                space: 8,
                pad: 12,
            },
            DensityName::Comfortable => Self {
                name,
                space: 12,
                pad: 16,
            },
        }
    }

    pub fn touch_target(self) -> u32 {
        match self.name {
            DensityName::Compact => 40,
            DensityName::Default => 48,
            DensityName::Comfortable => 56,
        }
    }

    pub fn tile(self) -> u32 {
        self.touch_target()
    }

    /// Inter-item gap (Compact 4, Default 8, Comfortable 12).
    pub fn gap(self) -> f32 {
        self.space as f32
    }

    /// Container inset (Compact 8, Default 12, Comfortable 16).
    pub fn inset(self) -> f32 {
        self.pad as f32
    }

    /// Card / group pad: one grid step past inset (12 / 16 / 20).
    pub fn sheet(self) -> f32 {
        Self::snap(self.pad + GRID) as f32
    }

    pub fn snap(px: u32) -> u32 {
        let rem = px % GRID;
        if rem == 0 {
            px
        } else {
            px + (GRID - rem)
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
    fn density_on_grid() {
        for name in [
            DensityName::Compact,
            DensityName::Default,
            DensityName::Comfortable,
        ] {
            let d = Density::named(name);
            assert_eq!(d.space % GRID, 0);
            assert_eq!(d.pad % GRID, 0);
            assert_eq!(d.touch_target() % GRID, 0);
            assert_eq!(d.tile(), d.touch_target());
        }
        assert_eq!(Density::default().space, 8);
        assert_eq!(Density::snap(0), 0);
        assert_eq!(Density::snap(4), 4);
        assert_eq!(Density::snap(5), 8);
        assert_eq!(Density::named(DensityName::Compact).touch_target(), 40);
        assert_eq!(Density::named(DensityName::Comfortable).touch_target(), 56);
        let compact = Density::named(DensityName::Compact);
        let default = Density::named(DensityName::Default);
        let comfy = Density::named(DensityName::Comfortable);
        assert_eq!(compact.gap(), 4.0);
        assert_eq!(default.gap(), 8.0);
        assert_eq!(comfy.gap(), 12.0);
        assert_eq!(compact.inset(), 8.0);
        assert_eq!(default.inset(), 12.0);
        assert_eq!(comfy.inset(), 16.0);
        assert_eq!(compact.sheet(), 12.0);
        assert_eq!(default.sheet(), 16.0);
        assert_eq!(comfy.sheet(), 20.0);
        assert!(compact.gap() < default.gap() && default.gap() < comfy.gap());
        assert!(compact.inset() < default.inset() && default.inset() < comfy.inset());
    }
}
