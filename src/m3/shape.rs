//! M3 shape scale and component → corner map.
//!
//! See <https://m3.material.io/styles/shape/overview>.

use iced::border::Radius;

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

impl Component {
    /// Desktop flat: every family is M3 shape None (0 dp).
    pub fn shape(self) -> Shape {
        Shape::None
    }

    pub fn radius(self) -> Radius {
        self.shape().radius()
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
        }
    }
}
