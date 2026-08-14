//! Bundled chrome icons.
//!
//! SVGs are solid black fills (`fill="#000"`). iced's svg `color` style
//! recolors non-transparent pixels, so tokens tint them on every host.
//! Stroke/`currentColor` icons are avoided — they often rasterize empty
//! under iced's Metal/wgpu path on macOS.

/// Chrome icon set. Applications may add their own SVG bytes beside these.
///
/// Bytes are filled black paths for token recolor via [`crate::widget::icon_svg`].
///
/// ```
/// assert_eq!(icedtea::icon::Icon::Search.slug(), "search");
/// assert!(icedtea::icon::Icon::Close.svg().contains("<svg"));
/// assert!(icedtea::icon::Icon::Close.svg().contains("fill=\"#000\""));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    Close,
    Back,
    Search,
    Menu,
    Chevron,
    Check,
    Warning,
}

impl Icon {
    pub const ALL: [Icon; 7] = [
        Icon::Close,
        Icon::Back,
        Icon::Search,
        Icon::Menu,
        Icon::Chevron,
        Icon::Check,
        Icon::Warning,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Close => "close",
            Self::Back => "back",
            Self::Search => "search",
            Self::Menu => "menu",
            Self::Chevron => "chevron",
            Self::Check => "check",
            Self::Warning => "warning",
        }
    }

    pub fn svg(self) -> &'static str {
        match self {
            Self::Close => include_str!("../assets/icons/close.svg"),
            Self::Back => include_str!("../assets/icons/back.svg"),
            Self::Search => include_str!("../assets/icons/search.svg"),
            Self::Menu => include_str!("../assets/icons/menu.svg"),
            Self::Chevron => include_str!("../assets/icons/chevron.svg"),
            Self::Check => include_str!("../assets/icons/check.svg"),
            Self::Warning => include_str!("../assets/icons/warning.svg"),
        }
    }

    pub fn bytes(self) -> &'static [u8] {
        self.svg().as_bytes()
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "close" => Some(Self::Close),
            "back" => Some(Self::Back),
            "search" => Some(Self::Search),
            "menu" => Some(Self::Menu),
            "chevron" => Some(Self::Chevron),
            "check" => Some(Self::Check),
            "warning" => Some(Self::Warning),
            _ => None,
        }
    }
}

/// Optional leading and trailing chrome icons on a labeled control.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Icons {
    pub leading: Option<Icon>,
    pub trailing: Option<Icon>,
}

impl Icons {
    pub const NONE: Self = Self {
        leading: None,
        trailing: None,
    };

    pub fn leading(icon: Icon) -> Self {
        Self {
            leading: Some(icon),
            trailing: None,
        }
    }

    pub fn trailing(icon: Icon) -> Self {
        Self {
            leading: None,
            trailing: Some(icon),
        }
    }

    pub fn both(leading: Icon, trailing: Icon) -> Self {
        Self {
            leading: Some(leading),
            trailing: Some(trailing),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_has_svg_and_roundtrips() {
        for icon in Icon::ALL {
            let s = icon.svg();
            assert!(s.contains("<svg"));
            assert!(s.contains("fill=\"#000\""));
            assert!(!s.contains("currentColor"));
            assert_eq!(Icon::from_slug(icon.slug()), Some(icon));
            assert_eq!(icon.bytes(), s.as_bytes());
        }
        assert!(Icon::from_slug("nope").is_none());
        assert_eq!(Icons::trailing(Icon::Close).trailing, Some(Icon::Close));
        assert_eq!(
            Icons::both(Icon::Search, Icon::Menu).leading,
            Some(Icon::Search)
        );
    }
}
