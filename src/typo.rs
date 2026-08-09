//! Type scale. UI copy uses the platform sans; code uses bundled JetBrains Mono.

use iced::font::{Style, Weight};
use iced::Font;

/// Platform sans (`Family::SansSerif`). Applications may load their own face
/// and pass it to iced; icedtea does not ship a UI family.
pub const UI: Font = Font::DEFAULT;

/// Titles and selected labels.
pub const UI_BOLD: Font = Font {
    weight: Weight::Bold,
    ..Font::DEFAULT
};

/// Dim / thought prose.
pub const UI_ITALIC: Font = Font {
    style: Style::Italic,
    ..Font::DEFAULT
};

/// JetBrains Mono — ids, paths, code.
pub const MONO: Font = Font::with_name("JetBrains Mono");

/// Bundled JetBrains Mono Regular.
pub const MONO_BYTES: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");

/// Page title.
pub const PAGE: u32 = 18;
/// Section / card title.
pub const TITLE: u32 = 15;
/// Body copy (default text size).
pub const BODY: u32 = 14;
/// Meta, tabs, footer, keys.
pub const META: u32 = 12;
/// Code / monospace.
pub const CODE: u32 = 13;

/// Named step on the type scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeRole {
    Page,
    Title,
    Body,
    Meta,
    Code,
}

impl TypeRole {
    /// Pixel size for this role.
    ///
    /// ```
    /// assert_eq!(icedtea::typo::TypeRole::Body.size(), 14);
    /// ```
    pub fn size(self) -> u32 {
        match self {
            Self::Page => PAGE,
            Self::Title => TITLE,
            Self::Body => BODY,
            Self::Meta => META,
            Self::Code => CODE,
        }
    }

    pub fn font(self) -> Font {
        match self {
            Self::Code => MONO,
            Self::Title | Self::Page => UI_BOLD,
            Self::Body | Self::Meta => UI,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_is_ordered() {
        assert!(TypeRole::Page.size() > TypeRole::Title.size());
        assert!(TypeRole::Title.size() > TypeRole::Body.size());
        assert!(TypeRole::Body.size() > TypeRole::Meta.size());
        assert!(TypeRole::Code.size() >= TypeRole::Meta.size());
        assert_eq!(UI, Font::DEFAULT);
        assert!(MONO_BYTES.len() > 1000);
        assert_eq!(TypeRole::Page.size(), PAGE);
        assert_eq!(TypeRole::Title.size(), TITLE);
        assert_eq!(TypeRole::Body.size(), BODY);
        assert_eq!(TypeRole::Meta.size(), META);
        assert_eq!(TypeRole::Code.size(), CODE);
        assert_eq!(TypeRole::Code.font(), MONO);
        assert_eq!(TypeRole::Title.font(), UI_BOLD);
        assert_eq!(TypeRole::Page.font(), UI_BOLD);
        assert_eq!(TypeRole::Body.font(), UI);
        assert_eq!(TypeRole::Meta.font(), UI);
        let _ = UI_ITALIC;
    }
}
