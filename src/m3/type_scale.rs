//! M3 type scale.

use iced::Font;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeRole {
    DisplayLarge,
    DisplayMedium,
    DisplaySmall,
    HeadlineLarge,
    HeadlineMedium,
    HeadlineSmall,
    TitleLarge,
    TitleMedium,
    TitleSmall,
    BodyLarge,
    BodyMedium,
    BodySmall,
    LabelLarge,
    LabelMedium,
    LabelSmall,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeScale {
    pub size: f32,
    pub line_height: f32,
    pub weight: u16,
}

impl TypeRole {
    pub fn scale(self) -> TypeScale {
        match self {
            Self::DisplayLarge => TypeScale {
                size: 57.0,
                line_height: 64.0,
                weight: 400,
            },
            Self::DisplayMedium => TypeScale {
                size: 45.0,
                line_height: 52.0,
                weight: 400,
            },
            Self::DisplaySmall => TypeScale {
                size: 36.0,
                line_height: 44.0,
                weight: 400,
            },
            Self::HeadlineLarge => TypeScale {
                size: 32.0,
                line_height: 40.0,
                weight: 400,
            },
            Self::HeadlineMedium => TypeScale {
                size: 28.0,
                line_height: 36.0,
                weight: 400,
            },
            Self::HeadlineSmall => TypeScale {
                size: 24.0,
                line_height: 32.0,
                weight: 400,
            },
            Self::TitleLarge => TypeScale {
                size: 22.0,
                line_height: 28.0,
                weight: 400,
            },
            Self::TitleMedium => TypeScale {
                size: 16.0,
                line_height: 24.0,
                weight: 500,
            },
            Self::TitleSmall => TypeScale {
                size: 14.0,
                line_height: 20.0,
                weight: 500,
            },
            Self::BodyLarge => TypeScale {
                size: 16.0,
                line_height: 24.0,
                weight: 400,
            },
            Self::BodyMedium => TypeScale {
                size: 14.0,
                line_height: 20.0,
                weight: 400,
            },
            Self::BodySmall => TypeScale {
                size: 12.0,
                line_height: 16.0,
                weight: 400,
            },
            Self::LabelLarge => TypeScale {
                size: 14.0,
                line_height: 20.0,
                weight: 500,
            },
            Self::LabelMedium => TypeScale {
                size: 12.0,
                line_height: 16.0,
                weight: 500,
            },
            Self::LabelSmall => TypeScale {
                size: 11.0,
                line_height: 16.0,
                weight: 500,
            },
        }
    }

    pub fn font(self) -> Font {
        if self.scale().weight >= 500 {
            Font {
                weight: iced::font::Weight::Medium,
                ..Font::DEFAULT
            }
        } else {
            Font::DEFAULT
        }
    }

    pub const ALL: [TypeRole; 15] = [
        Self::DisplayLarge,
        Self::DisplayMedium,
        Self::DisplaySmall,
        Self::HeadlineLarge,
        Self::HeadlineMedium,
        Self::HeadlineSmall,
        Self::TitleLarge,
        Self::TitleMedium,
        Self::TitleSmall,
        Self::BodyLarge,
        Self::BodyMedium,
        Self::BodySmall,
        Self::LabelLarge,
        Self::LabelMedium,
        Self::LabelSmall,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_scale_covers_all_roles() {
        for role in TypeRole::ALL {
            let s = role.scale();
            assert!(s.size > 0.0 && s.line_height >= s.size);
            let _ = role.font();
            assert!(s.size <= 57.0);
        }
        assert!(TypeRole::DisplayLarge.scale().size > TypeRole::BodyMedium.scale().size);
        assert!(TypeRole::LabelLarge.scale().weight >= 500);
        assert_eq!(TypeRole::BodySmall.font(), Font::DEFAULT);
    }
}
