//! Bootstrap: fonts, theme, locale, window settings for iced's application builder.

use iced::{Font, Pixels, Settings};

use crate::density::{Density, DensityName};
use crate::i18n::{Catalog, Locale};
use crate::theme::{self, ThemeCatalog, Tokens};
use crate::typo;
use crate::window::{self, WindowKind};

/// How an icedtea application boots.
///
/// ```
/// let boot = icedtea::app::Boot::new("demo", "dev.icedtea.demo");
/// let prep = icedtea::app::bootstrap(&boot);
/// assert_eq!(prep.tokens.canvas, icedtea::theme::named("dark").tokens.canvas);
/// assert!(!prep.iced_settings.fonts.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct Boot {
    pub title: String,
    pub application_id: String,
    pub window: WindowKind,
    pub theme_name: String,
    pub locale: Locale,
    pub density: DensityName,
}

impl Boot {
    pub fn new(title: impl Into<String>, application_id: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            application_id: application_id.into(),
            window: WindowKind::Application,
            theme_name: "dark".into(),
            locale: Locale::default(),
            density: DensityName::Default,
        }
    }

    pub fn overlay(mut self) -> Self {
        self.window = WindowKind::Overlay;
        self
    }

    pub fn dialog(mut self) -> Self {
        self.window = WindowKind::Dialog;
        self
    }

    pub fn theme(mut self, name: impl Into<String>) -> Self {
        self.theme_name = name.into();
        self
    }

    pub fn locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    pub fn density(mut self, density: DensityName) -> Self {
        self.density = density;
        self
    }
}

/// Resolved boot data used by `run`.
#[derive(Debug, Clone)]
pub struct Prepared {
    pub title: String,
    pub tokens: Tokens,
    pub theme_name: String,
    pub density: Density,
    pub locale: Locale,
    pub catalog: Catalog,
    pub iced_settings: Settings,
    pub window: iced::window::Settings,
    pub iced_theme: iced::Theme,
}

/// Load fonts, theme, and window settings. Does not open a window.
pub fn bootstrap(boot: &Boot) -> Prepared {
    bootstrap_with_catalog(boot, &ThemeCatalog::new())
}

pub fn bootstrap_with_catalog(boot: &Boot, themes: &ThemeCatalog) -> Prepared {
    let named = themes
        .get(&boot.theme_name)
        .or_else(|| themes.get("dark"))
        .expect("dark theme");
    let tokens = named.tokens;
    let iced_theme = theme::iced_theme(&named.name, tokens);
    let iced_settings = Settings {
        id: Some(boot.application_id.clone()),
        fonts: vec![std::borrow::Cow::Borrowed(typo::MONO_BYTES)],
        default_font: typo::UI,
        default_text_size: Pixels(typo::BODY as f32),
        antialiasing: true,
        vsync: true,
    };
    Prepared {
        title: boot.title.clone(),
        tokens,
        theme_name: named.name,
        density: Density::named(boot.density),
        locale: boot.locale.clone(),
        catalog: Catalog::for_locale(&boot.locale),
        iced_settings,
        window: window::settings(boot.window, &boot.application_id),
        iced_theme,
    }
}

impl Prepared {
    /// Text direction from [`Boot::locale`], used by chrome recipes.
    pub fn direction(&self) -> crate::i18n::Direction {
        self.locale.direction
    }
}

/// Default UI font after bootstrap.
pub fn default_font() -> Font {
    typo::UI
}

/// Window title that does not borrow application state.
///
/// ```
/// let t = icedtea::app::WindowTitle("demo".into());
/// assert_eq!(<icedtea::app::WindowTitle as icedtea::iced::application::TitleFn<()>>::title(&t, &()), "demo");
/// ```
#[derive(Debug, Clone)]
pub struct WindowTitle(pub String);

impl<S> iced::application::TitleFn<S> for WindowTitle {
    fn title(&self, _state: &S) -> String {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Locale;

    #[test]
    fn bootstrap_loads_fonts_and_window_kinds() {
        let mut boot = Boot::new("App", "dev.icedtea.app")
            .theme("nord")
            .density(DensityName::Compact);
        boot = boot.locale(Locale::new("ar"));
        let prep = bootstrap(&boot);
        assert_eq!(prep.theme_name, "nord");
        assert_eq!(prep.density.space, 4);
        assert_eq!(prep.locale.direction, crate::i18n::Direction::Rtl);
        assert_eq!(prep.direction(), crate::i18n::Direction::Rtl);
        assert_eq!(prep.catalog.t("direction"), "rtl");
        assert_eq!(prep.iced_settings.fonts.len(), 1);
        assert_eq!(default_font(), typo::UI);
        let ov = bootstrap(&Boot::new("p", "dev.x").overlay());
        assert!(!ov.window.decorations);
        let dlg = bootstrap(&Boot::new("d", "dev.x").dialog());
        assert!(dlg.window.decorations);
        let mut cat = ThemeCatalog::new();
        cat.register("brand", crate::theme::named("light").tokens, false);
        let prep = bootstrap_with_catalog(&Boot::new("a", "dev.x").theme("brand"), &cat);
        assert_eq!(prep.theme_name, "brand");
        let missing = bootstrap(&Boot::new("a", "dev.x").theme("nope"));
        assert_eq!(missing.theme_name, "dark");
        let t = WindowTitle("demo".into());
        assert_eq!(
            <WindowTitle as iced::application::TitleFn<()>>::title(&t, &()),
            "demo"
        );
    }
}
