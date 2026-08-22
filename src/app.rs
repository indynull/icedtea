//! Bootstrap: theme, locale, window settings for iced's application builder.

use iced::{Font, Pixels, Settings, Size};

use crate::density::{Density, DensityName};
use crate::i18n::{Catalog, Locale};
use crate::m3::{ElevationPolicy, ShapePolicy};
use crate::theme::{self, ThemeCatalog, Tokens};
use crate::typo;
use crate::window::{self, DisplayBounds, WindowKind};

/// How an icedtea application boots.
///
/// ```
/// let boot = icedtea::app::Boot::new("demo", "dev.icedtea.demo");
/// let prep = icedtea::app::bootstrap(&boot);
/// assert_eq!(prep.tokens.canvas, icedtea::theme::named("dark").tokens.canvas);
/// assert!(prep.iced_settings.fonts.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct Boot {
    pub title: String,
    pub application_id: String,
    pub window: WindowKind,
    pub theme_name: String,
    pub locale: Locale,
    pub density: DensityName,
    pub font_scale: f32,
    pub shape: ShapePolicy,
    pub elevation: ElevationPolicy,
    pub size: Option<Size>,
    pub min_size: Option<Size>,
    pub max_size: Option<Size>,
    pub pointer: Option<(f32, f32)>,
    pub displays: Vec<DisplayBounds>,
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
            font_scale: 1.0,
            shape: ShapePolicy::Desktop,
            elevation: ElevationPolicy::Desktop,
            size: None,
            min_size: None,
            max_size: None,
            pointer: None,
            displays: Vec::new(),
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

    /// Type-scale multiplier (1.0 is the Material body size).
    pub fn font_scale(mut self, scale: f32) -> Self {
        self.font_scale = scale;
        self
    }

    /// Corner policy applied to every constructor.
    pub fn shape(mut self, shape: ShapePolicy) -> Self {
        self.shape = shape;
        self
    }

    /// Shadow policy applied to every constructor.
    pub fn elevation(mut self, elevation: ElevationPolicy) -> Self {
        self.elevation = elevation;
        self
    }

    /// Inner window size in pixels.
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Some(Size::new(width, height));
        self
    }

    /// Minimum inner window size in pixels.
    pub fn min_size(mut self, width: f32, height: f32) -> Self {
        self.min_size = Some(Size::new(width, height));
        self
    }

    /// Maximum inner window size in pixels.
    pub fn max_size(mut self, width: f32, height: f32) -> Self {
        self.max_size = Some(Size::new(width, height));
        self
    }

    /// Pointer used to pick the overlay's display.
    pub fn pointer(mut self, x: f32, y: f32) -> Self {
        self.pointer = Some((x, y));
        self
    }

    /// Display rectangles in the same space as [`Self::pointer`].
    pub fn displays(mut self, displays: impl Into<Vec<DisplayBounds>>) -> Self {
        self.displays = displays.into();
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

/// Load theme and window settings. Does not open a window.
pub fn bootstrap(boot: &Boot) -> Prepared {
    bootstrap_with_catalog(boot, &ThemeCatalog::new())
}

pub fn bootstrap_with_catalog(boot: &Boot, themes: &ThemeCatalog) -> Prepared {
    let named = themes
        .get(&boot.theme_name)
        .or_else(|| themes.get("dark"))
        .expect("dark theme");
    let tokens = named
        .tokens
        .with_density(Density::named(boot.density))
        .with_font_scale(boot.font_scale)
        .with_shape(boot.shape)
        .with_elevation(boot.elevation)
        .with_direction(boot.locale.direction)
        .with_clock_digits(crate::i18n::ClockDigits::for_lang(&boot.locale.lang));
    let iced_theme = theme::iced_theme(&named.name, tokens);
    let iced_settings = Settings {
        id: Some(boot.application_id.clone()),
        fonts: vec![],
        default_font: typo::UI,
        default_text_size: Pixels(tokens.body()),
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
        window: {
            let mut win = window::settings(boot.window, &boot.application_id);
            if let Some(size) = boot.size {
                win.size = size;
            }
            if let Some(min) = boot.min_size {
                win.min_size = Some(min);
            }
            if let Some(max) = boot.max_size {
                win.max_size = Some(max);
            }
            if boot.window == WindowKind::Overlay {
                if let Some(pointer) = boot.pointer {
                    win.position = iced::window::Position::Specific(window::place(
                        pointer,
                        win.size,
                        &boot.displays,
                    ));
                }
            }
            win
        },
        iced_theme,
    }
}

impl Prepared {
    /// Text direction from [`Boot::locale`], used by chrome recipes.
    pub fn direction(&self) -> crate::i18n::Direction {
        self.locale.direction
    }

    /// Open a window with these settings. Used from [`crate::daemon!`].
    ///
    /// The process stays up when the window closes. Map the [`iced::Task`]
    /// to store the id, then [`iced::window::close`] to hide.
    ///
    /// ```
    /// let prep = icedtea::bootstrap(&icedtea::Boot::new("hud", "dev.hud").overlay());
    /// let (id, _open) = prep.open();
    /// let (again, _) = prep.open();
    /// assert_ne!(id, again);
    /// ```
    pub fn open(&self) -> (iced::window::Id, iced::Task<iced::window::Id>) {
        iced::window::open(self.window.clone())
    }

    /// Open a decorated desktop window from the same settings.
    ///
    /// [`crate::window::retarget`] first. Pop-out from an overlay.
    ///
    /// ```
    /// let prep = icedtea::bootstrap(&icedtea::Boot::new("hud", "dev.hud").overlay());
    /// let (id, _open) = prep.open_desktop();
    /// let _ = id;
    /// ```
    pub fn open_desktop(&self) -> (iced::window::Id, iced::Task<iced::window::Id>) {
        let mut win = self.window.clone();
        let app_id = self.iced_settings.id.as_deref().unwrap_or("");
        crate::window::retarget(&mut win, app_id);
        iced::window::open(win)
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

    fn specific_xy(pos: iced::window::Position) -> Option<(f32, f32)> {
        match pos {
            iced::window::Position::Specific(p) => Some((p.x, p.y)),
            iced::window::Position::Centered
            | iced::window::Position::Default
            | iced::window::Position::SpecificWith(_) => None,
        }
    }

    #[test]
    fn bootstrap_loads_theme_and_window_kinds() {
        let mut boot = Boot::new("App", "dev.icedtea.app")
            .theme("nord")
            .density(DensityName::Compact)
            .font_scale(1.25)
            .shape(ShapePolicy::Material)
            .elevation(ElevationPolicy::Flat);
        boot = boot.locale(Locale::new("ar"));
        let prep = bootstrap(&boot);
        assert_eq!(prep.theme_name, "nord");
        assert_eq!(prep.density.space, 4);
        assert_eq!(prep.tokens.density.space, 4);
        assert_eq!(prep.tokens.body(), 18.0);
        assert_eq!(prep.tokens.shape, ShapePolicy::Material);
        assert_eq!(prep.tokens.elevation, ElevationPolicy::Flat);
        assert_eq!(prep.locale.direction, crate::i18n::Direction::Rtl);
        assert_eq!(prep.direction(), crate::i18n::Direction::Rtl);
        assert_eq!(prep.catalog.t("direction"), "rtl");
        assert!(prep.iced_settings.fonts.is_empty());
        assert_eq!(default_font(), typo::UI);
        let ov = bootstrap(&Boot::new("p", "dev.x").overlay());
        assert!(!ov.window.decorations);
        assert!(ov.window.max_size.is_none());
        let placed = bootstrap(
            &Boot::new("p", "dev.x")
                .overlay()
                .size(900.0, 700.0)
                .pointer(100.0, 80.0)
                .displays(vec![DisplayBounds {
                    x: 0.0,
                    y: 0.0,
                    width: 1920.0,
                    height: 1080.0,
                }]),
        );
        assert_eq!(placed.window.size.width, 900.0);
        assert_eq!(placed.window.size.height, 700.0);
        assert!(placed.window.max_size.is_none());
        assert_eq!(specific_xy(placed.window.position), Some((100.0, 80.0)));
        assert_eq!(specific_xy(iced::window::Position::Centered), None);
        assert_eq!(specific_xy(iced::window::Position::Default), None);
        let place: fn(iced::Size, iced::Size) -> iced::Point = |_, _| iced::Point::ORIGIN;
        assert_eq!(
            place(iced::Size::ZERO, iced::Size::ZERO),
            iced::Point::ORIGIN
        );
        assert_eq!(
            specific_xy(iced::window::Position::SpecificWith(place)),
            None
        );
        let dlg = bootstrap(&Boot::new("d", "dev.x").dialog());
        assert!(dlg.window.decorations);
        let mut cat = ThemeCatalog::new();
        cat.register("brand", crate::theme::named("light").tokens, false);
        let prep = bootstrap_with_catalog(&Boot::new("a", "dev.x").theme("brand"), &cat);
        assert_eq!(prep.theme_name, "brand");
        let missing = bootstrap(&Boot::new("a", "dev.x").theme("nope"));
        assert_eq!(missing.theme_name, "dark");
        let sized = bootstrap(
            &Boot::new("tool", "dev.tool")
                .size(380.0, 640.0)
                .min_size(360.0, 560.0)
                .max_size(420.0, 720.0),
        );
        assert_eq!(sized.window.size.width, 380.0);
        assert_eq!(sized.window.size.height, 640.0);
        assert_eq!(sized.window.min_size.unwrap().width, 360.0);
        assert_eq!(sized.window.min_size.unwrap().height, 560.0);
        assert_eq!(sized.window.max_size.unwrap().width, 420.0);
        let t = WindowTitle("demo".into());
        assert_eq!(
            <WindowTitle as iced::application::TitleFn<()>>::title(&t, &()),
            "demo"
        );
    }
}
