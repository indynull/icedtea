//! M3 shape scale.

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
        assert_eq!(Corner::Tight.radius_px(), 4.0);
        assert_eq!(Corner::Soft.radius_px(), 12.0);
        assert_eq!(Corner::Pill.shape(), Shape::Full);
        assert_eq!(Corner::default(), Corner::None);
    }
}
