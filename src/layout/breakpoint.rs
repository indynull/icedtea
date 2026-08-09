//! Named width breakpoints that swap recipes.

/// Width class for adaptive layouts.
///
/// ```
/// use icedtea::layout::Breakpoint;
/// assert_eq!(Breakpoint::from_width(500.0), Breakpoint::Compact);
/// assert_eq!(Breakpoint::from_width(800.0), Breakpoint::Medium);
/// assert_eq!(Breakpoint::from_width(1200.0), Breakpoint::Expanded);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Breakpoint {
    Compact,
    Medium,
    Expanded,
}

impl Breakpoint {
    pub const COMPACT_MAX: f32 = 600.0;
    pub const MEDIUM_MAX: f32 = 1000.0;

    pub fn from_width(width: f32) -> Self {
        if width < Self::COMPACT_MAX {
            Self::Compact
        } else if width < Self::MEDIUM_MAX {
            Self::Medium
        } else {
            Self::Expanded
        }
    }

    /// Compact uses a stacked recipe; medium+ keep sidebar beside content.
    pub fn sidebar_beside(self) -> bool {
        matches!(self, Self::Medium | Self::Expanded)
    }

    pub fn show_back(self) -> bool {
        matches!(self, Self::Compact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swaps_sidebar_recipe() {
        assert_eq!(Breakpoint::from_width(-1.0), Breakpoint::Compact);
        assert_eq!(Breakpoint::from_width(599.0), Breakpoint::Compact);
        assert_eq!(Breakpoint::from_width(600.0), Breakpoint::Medium);
        assert_eq!(Breakpoint::from_width(999.0), Breakpoint::Medium);
        assert_eq!(Breakpoint::from_width(1000.0), Breakpoint::Expanded);
        assert!(!Breakpoint::Compact.sidebar_beside());
        assert!(Breakpoint::Medium.sidebar_beside());
        assert!(Breakpoint::Expanded.sidebar_beside());
        assert!(Breakpoint::Compact.show_back());
        assert!(!Breakpoint::Expanded.show_back());
    }
}
