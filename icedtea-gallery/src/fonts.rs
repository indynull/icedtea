//! Gallery-only Noto Sans faces. Not library API.

use std::borrow::Cow;

/// Family name registered by the Regular and Bold files.
pub const NOTO_SANS: &str = "Noto Sans";

/// Regular and Bold bytes for [`icedtea::Boot::fonts`].
pub fn bytes() -> Vec<Cow<'static, [u8]>> {
    vec![
        Cow::Borrowed(include_bytes!("../assets/fonts/NotoSans-Regular.ttf")),
        Cow::Borrowed(include_bytes!("../assets/fonts/NotoSans-Bold.ttf")),
    ]
}

/// Look-strip face. Noto is the gallery default; System is the host sans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiFace {
    Noto,
    System,
}

impl UiFace {
    pub fn bind(self, system: &str) {
        let name = match self {
            Self::Noto => NOTO_SANS,
            Self::System => system,
        };
        icedtea::typo::bind_sans_family(name);
    }
}
