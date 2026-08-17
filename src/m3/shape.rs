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
    ExtraLarge,
    Full,
}

impl Shape {
    pub fn dp(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::ExtraSmall => 4.0,
            Self::Small => 8.0,
            Self::Medium => 12.0,
            Self::Large => 16.0,
            Self::ExtraLarge => 28.0,
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
/// M3 still documents Full for switches/FABs on mobile; here only the
/// switch *thumb* stays circular via iced geometry, not container radius.
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
    /// App bar / shell / tab strip (flush).
    AppBar,
    Shell,
    /// Tab *label* is flush; active indicator is a separate underbar.
    Tab,
}

/// How constructors pick a corner from [`Component`].
///
/// Default is [`Self::Desktop`] (every family 0 dp). Apps that want
/// Material corners, or one radius on every control, set this on
/// [`crate::theme::Tokens`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ShapePolicy {
    /// Every family is M3 None (0 dp). Desktop default.
    #[default]
    Desktop,
    /// Extra-small (4 dp) on every family.
    Tight,
    /// Medium (12 dp) on every family.
    Soft,
    /// Full pill on buttons, chips, and badges. Cards, menus, fields,
    /// dialogs, and chrome rows stay boxes (medium or flush).
    Pill,
    /// Documented Material map (buttons extra-small, chips and badges
    /// small, cards medium, dialogs extra-large, app bars flush).
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
            ShapePolicy::Tight => Shape::ExtraSmall,
            ShapePolicy::Soft => Shape::Medium,
            ShapePolicy::Pill => self.pill_shape(),
            ShapePolicy::Material => self.material_shape(),
        }
    }

    fn pill_shape(self) -> Shape {
        match self {
            Self::Button | Self::Chip | Self::Badge => Shape::Full,
            Self::AppBar | Self::Shell | Self::Tab => Shape::None,
            Self::Field | Self::Checkbox | Self::Card | Self::Menu | Self::Dialog => Shape::Medium,
        }
    }

    fn material_shape(self) -> Shape {
        match self {
            Self::Button | Self::Field | Self::Checkbox | Self::Menu => Shape::ExtraSmall,
            Self::Chip | Self::Badge => Shape::Small,
            Self::Card => Shape::Medium,
            Self::Dialog => Shape::ExtraLarge,
            Self::AppBar | Self::Shell | Self::Tab => Shape::None,
        }
    }

    pub fn radius(self) -> Radius {
        self.shape().radius()
    }

    pub fn radius_for(self, policy: ShapePolicy) -> Radius {
        self.shape_for(policy).radius()
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
        for s in [
            Shape::None,
            Shape::ExtraSmall,
            Shape::Small,
            Shape::Medium,
            Shape::Large,
            Shape::ExtraLarge,
            Shape::Full,
        ] {
            let _ = s.radius();
            assert!(s.dp() >= 0.0);
        }
        assert!(Shape::Small.dp() < Shape::Large.dp());
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
            Component::AppBar,
            Component::Shell,
            Component::Tab,
        ] {
            assert_eq!(c.shape(), Shape::None, "{c:?}");
            assert_eq!(c.shape().dp(), 0.0);
            assert_eq!(std::hint::black_box(c.radius()).top_left, 0.0);
            assert_eq!(c.shape_for(ShapePolicy::Desktop), Shape::None);
            assert_eq!(c.shape_for(ShapePolicy::Tight), Shape::ExtraSmall);
            assert_eq!(c.shape_for(ShapePolicy::Soft), Shape::Medium);
        }
        assert_eq!(Component::Button.shape_for(ShapePolicy::Pill), Shape::Full);
        assert_eq!(Component::Chip.shape_for(ShapePolicy::Pill), Shape::Full);
        assert_eq!(Component::Badge.shape_for(ShapePolicy::Pill), Shape::Full);
        assert_eq!(Component::Card.shape_for(ShapePolicy::Pill), Shape::Medium);
        assert_eq!(Component::Menu.shape_for(ShapePolicy::Pill), Shape::Medium);
        assert_eq!(
            Component::Dialog.shape_for(ShapePolicy::Pill),
            Shape::Medium
        );
        assert_eq!(Component::Field.shape_for(ShapePolicy::Pill), Shape::Medium);
        assert_eq!(
            Component::Checkbox.shape_for(ShapePolicy::Pill),
            Shape::Medium
        );
        assert_eq!(Component::AppBar.shape_for(ShapePolicy::Pill), Shape::None);
        assert_eq!(Component::Shell.shape_for(ShapePolicy::Pill), Shape::None);
        assert_eq!(Component::Tab.shape_for(ShapePolicy::Pill), Shape::None);
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
            Component::Shell.shape_for(ShapePolicy::Material),
            Shape::None
        );
        assert_eq!(Component::Tab.shape_for(ShapePolicy::Material), Shape::None);
        assert_eq!(
            Component::Field.radius_for(ShapePolicy::Tight).top_left,
            4.0
        );
        assert_eq!(ShapePolicy::default(), ShapePolicy::Desktop);
    }
}
