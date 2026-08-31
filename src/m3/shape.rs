//! M3 shape scale and component → corner map.
//!
//! See <https://m3.material.io/styles/shape/overview>.

use iced::border::Radius;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    None,
    ExtraSmall,
    Small,
    Medium,
    Large,
    LargeIncreased,
    ExtraLarge,
    ExtraLargeIncreased,
    ExtraExtraLarge,
    Full,
}

impl Shape {
    /// Ten-step M3 corner scale plus Full (snapshot `styles/shape`).
    pub const STEPS: [(Self, f32); 10] = [
        (Self::None, 0.0),
        (Self::ExtraSmall, 4.0),
        (Self::Small, 8.0),
        (Self::Medium, 12.0),
        (Self::Large, 16.0),
        (Self::LargeIncreased, 20.0),
        (Self::ExtraLarge, 28.0),
        (Self::ExtraLargeIncreased, 32.0),
        (Self::ExtraExtraLarge, 48.0),
        (Self::Full, 9999.0),
    ];

    pub fn dp(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::ExtraSmall => 4.0,
            Self::Small => 8.0,
            Self::Medium => 12.0,
            Self::Large => 16.0,
            Self::LargeIncreased => 20.0,
            Self::ExtraLarge => 28.0,
            Self::ExtraLargeIncreased => 32.0,
            Self::ExtraExtraLarge => 48.0,
            Self::Full => 9999.0,
        }
    }
    pub fn radius(self) -> Radius {
        self.dp().into()
    }
}

/// Closed public surface → M3 corner shape (one path per control family).
///
/// icedtea is **desktop-first**: all families use M3 shape **None** (0 dp)
/// so chrome reads crisp and rectangular. Touch-pill Full / mobile Soft are
/// not the product default; apps that want round can restyle later.
///
/// Switch thumbs stay circular via iced geometry. The track uses
/// [`Component::Track`] (Full under Material and Pill).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Component {
    /// Filled / tonal / text buttons, chips.
    Button,
    Chip,
    /// Count or status mark (M3 Badge).
    Badge,
    /// Text fields, pickers, checkbox container.
    Field,
    Checkbox,
    /// Cards, menus, list card rows.
    Card,
    Menu,
    /// Dialog / modal sheet.
    Dialog,
    /// Transient notice (M3 snackbar).
    Toast,
    /// Hover tip.
    Tooltip,
    /// Page-level message (M3 banner). Flush under Material and Pill.
    Banner,
    /// Search field (M3 search). Extra-large under Material, full under Pill.
    Search,
    /// Switch track, slider rail, linear progress.
    Track,
    /// App bar / shell / tab strip (flush).
    AppBar,
    Shell,
    /// Tab *label* is flush; active indicator is a separate underbar.
    Tab,
    /// Exclusive in-pane segment (joined strip). Flush; not a stadium.
    Segment,
}

/// How constructors pick a corner from [`Component`].
///
/// Default is [`Self::Desktop`] (every family 0 dp). Apps that want
/// Material corners, or one radius on rounded families, set this on
/// [`crate::theme::Tokens`]. Flush chrome (tabs, app bars, banners,
/// exclusive segments) stays shape None under every policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ShapePolicy {
    /// Every family is M3 None (0 dp). Desktop default.
    #[default]
    Desktop,
    /// Extra-small (4 dp) on rounded families. Flush chrome stays None.
    Tight,
    /// Medium (12 dp) on rounded families. Flush chrome stays None.
    /// Checkbox stays extra-small so the box is not a circle.
    Soft,
    /// Full stadium on buttons, chips, badges, search, and tracks.
    /// Cards, fields, dialogs, toasts, and tooltips stay boxes.
    /// Menus stay extra-small: iced paints that radius on each selected
    /// row, so Medium reads as a stack of pills.
    /// Tabs, app bars, banners, and exclusive segments stay flush.
    Pill,
    /// Documented Material map (buttons extra-small, chips and badges
    /// small, cards medium, toasts and tooltips extra-small, dialogs
    /// and search extra-large, tracks full, flush chrome None).
    Material,
}

impl Component {
    /// Desktop flat: every family is M3 shape None (0 dp).
    pub fn shape(self) -> Shape {
        self.shape_for(ShapePolicy::Desktop)
    }

    /// Corner for this family under `policy`.
    pub fn shape_for(self, policy: ShapePolicy) -> Shape {
        match policy {
            ShapePolicy::Desktop => Shape::None,
            ShapePolicy::Tight => {
                if self.is_flush() {
                    Shape::None
                } else {
                    Shape::ExtraSmall
                }
            }
            ShapePolicy::Soft => {
                if self.is_flush() {
                    Shape::None
                } else if matches!(self, Self::Checkbox | Self::Menu) {
                    Shape::ExtraSmall
                } else {
                    Shape::Medium
                }
            }
            ShapePolicy::Pill => self.pill_shape(),
            ShapePolicy::Material => self.material_shape(),
        }
    }

    /// Tabs, app bars, banners, and exclusive segments stay rectangular.
    pub fn is_flush(self) -> bool {
        matches!(
            self,
            Self::AppBar | Self::Shell | Self::Tab | Self::Banner | Self::Segment
        )
    }

    fn pill_shape(self) -> Shape {
        match self {
            Self::Button | Self::Chip | Self::Badge | Self::Search | Self::Track => Shape::Full,
            Self::AppBar | Self::Shell | Self::Tab | Self::Banner | Self::Segment => Shape::None,
            Self::Checkbox => Shape::ExtraSmall,
            Self::Menu => Shape::ExtraSmall,
            Self::Field | Self::Card | Self::Dialog | Self::Toast | Self::Tooltip => Shape::Medium,
        }
    }

    fn material_shape(self) -> Shape {
        match self {
            Self::Button
            | Self::Field
            | Self::Checkbox
            | Self::Menu
            | Self::Toast
            | Self::Tooltip => Shape::ExtraSmall,
            Self::Chip | Self::Badge => Shape::Small,
            Self::Card => Shape::Medium,
            Self::Dialog | Self::Search => Shape::ExtraLarge,
            Self::Track => Shape::Full,
            Self::AppBar | Self::Shell | Self::Tab | Self::Banner | Self::Segment => Shape::None,
        }
    }

    pub fn radius(self) -> Radius {
        self.shape().radius()
    }

    pub fn radius_for(self, policy: ShapePolicy) -> Radius {
        self.shape_for(policy).radius()
    }

    /// Material resting elevation (snapshot component table).
    pub fn elevation(self) -> crate::m3::Elevation {
        use crate::m3::Elevation;
        match self {
            Self::Dialog | Self::Search => Elevation::Level3,
            Self::Menu | Self::Tooltip => Elevation::Level2,
            Self::Card => Elevation::Level1,
            Self::Banner => Elevation::Level1,
            _ => Elevation::Level0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Corner {
    #[default]
    None,
    Tight,
    Soft,
    Pill,
}

impl Corner {
    pub fn shape(self) -> Shape {
        match self {
            Self::None => Shape::None,
            Self::Tight => Shape::ExtraSmall,
            Self::Soft => Shape::Medium,
            Self::Pill => Shape::Full,
        }
    }
    pub fn radius_px(self) -> f32 {
        self.shape().dp()
    }
    pub fn radius(self) -> Radius {
        self.shape().radius()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shape_scale() {
        for (step, dp) in Shape::STEPS {
            assert_eq!(step.dp(), dp, "{step:?}");
            let _ = step.radius();
        }
        assert_eq!(
            Shape::STEPS.map(|(_, d)| d),
            [0.0, 4.0, 8.0, 12.0, 16.0, 20.0, 28.0, 32.0, 48.0, 9999.0]
        );
        assert!(Shape::Small.dp() < Shape::Large.dp());
        assert!(Shape::Large.dp() < Shape::LargeIncreased.dp());
        assert!(Shape::ExtraLarge.dp() < Shape::ExtraLargeIncreased.dp());
        assert_eq!(Corner::None.radius_px(), 0.0);
        assert_eq!(Corner::None.radius().top_left, 0.0);
        assert_eq!(Corner::Tight.radius_px(), 4.0);
        assert_eq!(Corner::Soft.radius_px(), 12.0);
        assert_eq!(Corner::Pill.shape(), Shape::Full);
        assert_eq!(Corner::default(), Corner::None);
        // Desktop flat policy: all components map to M3 None (0 dp).
        for c in [
            Component::Button,
            Component::Chip,
            Component::Badge,
            Component::Field,
            Component::Checkbox,
            Component::Card,
            Component::Menu,
            Component::Dialog,
            Component::Toast,
            Component::Tooltip,
            Component::Banner,
            Component::Search,
            Component::Track,
            Component::AppBar,
            Component::Shell,
            Component::Tab,
            Component::Segment,
        ] {
            assert_eq!(c.shape(), Shape::None, "{c:?}");
            assert_eq!(c.shape().dp(), 0.0);
            assert_eq!(std::hint::black_box(c.radius()).top_left, 0.0);
            assert_eq!(c.shape_for(ShapePolicy::Desktop), Shape::None);
            if c.is_flush() {
                assert_eq!(c.shape_for(ShapePolicy::Tight), Shape::None, "{c:?}");
                assert_eq!(c.shape_for(ShapePolicy::Soft), Shape::None, "{c:?}");
                assert_eq!(c.shape_for(ShapePolicy::Pill), Shape::None, "{c:?}");
                assert_eq!(c.shape_for(ShapePolicy::Material), Shape::None, "{c:?}");
            } else if matches!(c, Component::Checkbox | Component::Menu) {
                assert_eq!(c.shape_for(ShapePolicy::Tight), Shape::ExtraSmall);
                assert_eq!(c.shape_for(ShapePolicy::Soft), Shape::ExtraSmall);
            } else {
                assert_eq!(c.shape_for(ShapePolicy::Tight), Shape::ExtraSmall, "{c:?}");
                assert_eq!(c.shape_for(ShapePolicy::Soft), Shape::Medium, "{c:?}");
            }
        }
        assert_eq!(Component::Button.shape_for(ShapePolicy::Pill), Shape::Full);
        assert_eq!(Component::Chip.shape_for(ShapePolicy::Pill), Shape::Full);
        assert_eq!(Component::Badge.shape_for(ShapePolicy::Pill), Shape::Full);
        assert_eq!(Component::Card.shape_for(ShapePolicy::Pill), Shape::Medium);
        assert_eq!(
            Component::Menu.shape_for(ShapePolicy::Pill),
            Shape::ExtraSmall
        );
        assert_eq!(
            Component::Dialog.shape_for(ShapePolicy::Pill),
            Shape::Medium
        );
        assert_eq!(Component::Toast.shape_for(ShapePolicy::Pill), Shape::Medium);
        assert_eq!(
            Component::Tooltip.shape_for(ShapePolicy::Pill),
            Shape::Medium
        );
        assert_eq!(Component::Field.shape_for(ShapePolicy::Pill), Shape::Medium);
        assert_eq!(
            Component::Checkbox.shape_for(ShapePolicy::Pill),
            Shape::ExtraSmall
        );
        assert_eq!(Component::AppBar.shape_for(ShapePolicy::Pill), Shape::None);
        assert_eq!(Component::Shell.shape_for(ShapePolicy::Pill), Shape::None);
        assert_eq!(Component::Tab.shape_for(ShapePolicy::Pill), Shape::None);
        assert_eq!(Component::Banner.shape_for(ShapePolicy::Pill), Shape::None);
        assert_eq!(Component::Segment.shape_for(ShapePolicy::Pill), Shape::None);
        assert_eq!(Component::Search.shape_for(ShapePolicy::Pill), Shape::Full);
        assert_eq!(Component::Track.shape_for(ShapePolicy::Pill), Shape::Full);
        assert_eq!(
            Component::Button.shape_for(ShapePolicy::Material),
            Shape::ExtraSmall
        );
        assert_eq!(
            Component::Chip.shape_for(ShapePolicy::Material),
            Shape::Small
        );
        assert_eq!(
            Component::Badge.shape_for(ShapePolicy::Material),
            Shape::Small
        );
        assert_eq!(
            Component::Card.shape_for(ShapePolicy::Material),
            Shape::Medium
        );
        assert_eq!(
            Component::Dialog.shape_for(ShapePolicy::Material),
            Shape::ExtraLarge
        );
        assert_eq!(
            Component::AppBar.shape_for(ShapePolicy::Material),
            Shape::None
        );
        assert_eq!(
            Component::Field.shape_for(ShapePolicy::Material),
            Shape::ExtraSmall
        );
        assert_eq!(
            Component::Checkbox.shape_for(ShapePolicy::Material),
            Shape::ExtraSmall
        );
        assert_eq!(
            Component::Menu.shape_for(ShapePolicy::Material),
            Shape::ExtraSmall
        );
        assert_eq!(
            Component::Toast.shape_for(ShapePolicy::Material),
            Shape::ExtraSmall
        );
        assert_eq!(
            Component::Tooltip.shape_for(ShapePolicy::Material),
            Shape::ExtraSmall
        );
        assert_eq!(
            Component::Shell.shape_for(ShapePolicy::Material),
            Shape::None
        );
        assert_eq!(Component::Tab.shape_for(ShapePolicy::Material), Shape::None);
        assert_eq!(
            Component::Segment.shape_for(ShapePolicy::Material),
            Shape::None
        );
        assert_eq!(
            Component::Banner.shape_for(ShapePolicy::Material),
            Shape::None
        );
        assert_eq!(
            Component::Search.shape_for(ShapePolicy::Material),
            Shape::ExtraLarge
        );
        assert_eq!(
            Component::Track.shape_for(ShapePolicy::Material),
            Shape::Full
        );
        assert_eq!(Component::Dialog.elevation(), crate::m3::Elevation::Level3);
        assert_eq!(Component::Search.elevation(), crate::m3::Elevation::Level3);
        assert_eq!(Component::Menu.elevation(), crate::m3::Elevation::Level2);
        assert_eq!(Component::Tooltip.elevation(), crate::m3::Elevation::Level2);
        assert_eq!(Component::Card.elevation(), crate::m3::Elevation::Level1);
        assert_eq!(Component::Banner.elevation(), crate::m3::Elevation::Level1);
        assert_eq!(Component::Button.elevation(), crate::m3::Elevation::Level0);
        assert_eq!(Component::Field.elevation(), crate::m3::Elevation::Level0);
        assert_eq!(Component::AppBar.elevation(), crate::m3::Elevation::Level0);
        assert_eq!(
            Component::Field.radius_for(ShapePolicy::Tight).top_left,
            4.0
        );
        assert_eq!(ShapePolicy::default(), ShapePolicy::Desktop);
    }

    #[test]
    fn flush_chrome_stays_none_on_every_policy() {
        for c in [
            Component::Tab,
            Component::AppBar,
            Component::Shell,
            Component::Banner,
            Component::Segment,
        ] {
            assert!(c.is_flush(), "{c:?}");
            for policy in [
                ShapePolicy::Desktop,
                ShapePolicy::Tight,
                ShapePolicy::Soft,
                ShapePolicy::Pill,
                ShapePolicy::Material,
            ] {
                assert_eq!(c.shape_for(policy), Shape::None, "{c:?} {policy:?}");
                assert_eq!(c.radius_for(policy).top_left, 0.0, "{c:?} {policy:?}");
            }
        }
        assert!(!Component::Button.is_flush());
        assert!(!Component::Chip.is_flush());
        assert_eq!(Component::Button.shape_for(ShapePolicy::Pill), Shape::Full);
        assert_eq!(Component::Chip.shape_for(ShapePolicy::Pill), Shape::Full);
        assert_eq!(Component::Search.shape_for(ShapePolicy::Pill), Shape::Full);
        assert_eq!(Component::Track.shape_for(ShapePolicy::Pill), Shape::Full);
        assert_ne!(
            Component::Segment.shape_for(ShapePolicy::Pill),
            Component::Button.shape_for(ShapePolicy::Pill)
        );
    }
}
