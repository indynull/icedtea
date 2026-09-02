//! Semantic color tokens, mixing rules, and the named community catalog.
//!
//! `named` picks a colorway. `mix` builds washes. `light` and `dark` are
//! a neutral desktop pair. With `follow_os` (persist default),
//! [`apply_os_chrome`] layers optional desktop colors ([`OsChrome`])
//! onto that pair; a named colorway is a choice on top of that.
//!
//! ```
//! let dark = icedtea::theme::named("dark");
//! assert_eq!(dark.name, "dark");
//! assert_eq!(
//!     dark.tokens.selection,
//!     dark.tokens.scheme().secondary_container
//! );
//! let pure = icedtea::theme::apply_os_chrome(
//!     dark.tokens,
//!     false,
//!     icedtea::theme::OsChrome::empty(),
//! );
//! assert_eq!(pure, dark.tokens);
//! ```

use std::collections::BTreeMap;
use std::sync::OnceLock;

use iced::Color;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Semantic colors used by every styled widget.
///
/// Fields map to Material Design 3 roles (see [`crate::m3::Scheme`]):
/// - `canvas` → surface
/// - `surface` → surface_container
/// - `panel` → surface_container_high
/// - `text` → on_surface
/// - `muted` → on_surface_variant
/// - `primary` → primary
/// - `accent` → secondary
/// - `danger` → error
/// - `border` → outline
/// - `selection` → secondary_container
/// - `selection_text` → on_secondary_container
///
/// Prefer reading via [`Tokens::scheme`] for the full role set.
/// `light` / `dark` are the desktop pair, not the M3 baseline palettes.
///
/// ```
/// let dark = icedtea::theme::named("dark");
/// assert_eq!(dark.name, "dark");
/// assert!(dark.tokens.canvas.r < 0.2);
/// assert_eq!(
///     dark.tokens.primary,
///     icedtea::iced::Color::from_rgb8(0x6B, 0x9E, 0xFF)
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tokens {
    pub canvas: Color,
    pub surface: Color,
    pub panel: Color,
    pub text: Color,
    pub muted: Color,
    pub primary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub border: Color,
    pub selection: Color,
    pub selection_text: Color,
    /// Compact / default / comfortable pad and control height.
    pub density: crate::m3::Density,
    /// When true, motion durations are 0 ms and progress snaps.
    pub reduced_motion: bool,
    /// Multiplier on [`crate::typo`] steps (1.0 is the M3 scale).
    pub font_scale: f32,
    /// Corner policy constructors read through [`Self::radius`].
    pub shape: crate::m3::ShapePolicy,
    /// Shadow policy constructors read through [`Self::shadow`].
    pub elevation: crate::m3::ElevationPolicy,
    /// Start side for chrome and control rows.
    pub direction: crate::i18n::Direction,
    /// Clock face digits. Hebrew stays Western on a right-to-left window.
    pub clock_digits: crate::i18n::ClockDigits,
    /// Exact M3 scheme. [`Self::scheme`] returns this without mixing.
    full: crate::m3::Scheme,
}

impl From<crate::m3::Scheme> for Tokens {
    fn from(s: crate::m3::Scheme) -> Self {
        Self {
            canvas: s.surface,
            surface: s.surface_container,
            panel: s.surface_container_high,
            text: s.on_surface,
            muted: s.on_surface_variant,
            primary: s.primary,
            accent: s.secondary,
            success: s.success,
            warning: s.warning,
            danger: s.error,
            border: s.outline,
            selection: s.secondary_container,
            selection_text: s.on_secondary_container,
            density: crate::m3::Density::default(),
            reduced_motion: false,
            font_scale: 1.0,
            shape: crate::m3::ShapePolicy::Desktop,
            elevation: crate::m3::ElevationPolicy::Desktop,
            direction: crate::i18n::Direction::Ltr,
            clock_digits: crate::i18n::ClockDigits::Western,
            full: s,
        }
    }
}

impl Tokens {
    /// Full M3 role set for this token face.
    ///
    /// Round-trips exactly when tokens were built from [`crate::m3::Scheme`].
    /// Named colorways (including the `light` / `dark` desktop pair) keep
    /// container roles derived from the short fields.
    pub fn scheme(self) -> crate::m3::Scheme {
        self.full
    }

    /// Build tokens from the short color aliases.
    ///
    /// Selection wash is primary mixed onto canvas. Density, shape,
    /// elevation, and direction stay at the desktop defaults; chain
    /// [`Self::with_density`] and the other setters.
    ///
    /// ```
    /// use icedtea::iced::Color;
    /// let tok = icedtea::theme::Tokens::from_aliases(
    ///     Color::from_rgb8(0x1e, 0x1e, 0x2e),
    ///     Color::from_rgb8(0x31, 0x32, 0x44),
    ///     Color::from_rgb8(0x18, 0x18, 0x25),
    ///     Color::from_rgb8(0xcd, 0xd6, 0xf4),
    ///     Color::from_rgb8(0xa6, 0xad, 0xc8),
    ///     Color::from_rgb8(0x89, 0xb4, 0xfa),
    ///     Color::from_rgb8(0xf5, 0xc2, 0xe7),
    ///     Color::from_rgb8(0xa6, 0xe3, 0xa1),
    ///     Color::from_rgb8(0xf9, 0xe2, 0xaf),
    ///     Color::from_rgb8(0xf3, 0x8b, 0xa8),
    ///     Color::from_rgb8(0x45, 0x47, 0x5a),
    /// );
    /// assert_eq!(tok.scheme().primary, tok.primary);
    /// assert_eq!(tok.scheme().surface, tok.canvas);
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn from_aliases(
        canvas: Color,
        surface: Color,
        panel: Color,
        text: Color,
        muted: Color,
        primary: Color,
        accent: Color,
        success: Color,
        warning: Color,
        danger: Color,
        border: Color,
    ) -> Self {
        let selection = mix(primary, canvas, 0.28);
        Self {
            canvas,
            surface,
            panel,
            text,
            muted,
            primary,
            accent,
            success,
            warning,
            danger,
            border,
            selection,
            selection_text: text,
            density: crate::m3::Density::default(),
            reduced_motion: false,
            font_scale: 1.0,
            shape: crate::m3::ShapePolicy::Desktop,
            elevation: crate::m3::ElevationPolicy::Desktop,
            direction: crate::i18n::Direction::Ltr,
            clock_digits: crate::i18n::ClockDigits::Western,
            full: crate::m3::scheme_dark(),
        }
        .sync_full_from_aliases()
    }

    /// Same tokens with a different density (pad and control height).
    pub fn with_density(mut self, density: crate::m3::Density) -> Self {
        self.density = density;
        self
    }

    /// Same tokens with reduced motion on or off.
    pub fn with_reduced_motion(mut self, on: bool) -> Self {
        self.reduced_motion = on;
        self
    }

    /// Same tokens with a type-scale multiplier (clamped 0.75..=1.5).
    ///
    /// ```
    /// let tok = icedtea::theme::named("dark").tokens.with_font_scale(1.25);
    /// assert_eq!(tok.body(), 18.0);
    /// ```
    pub fn with_font_scale(mut self, scale: f32) -> Self {
        self.font_scale = scale.clamp(0.75, 1.5);
        self
    }

    /// Same tokens with a corner policy.
    ///
    /// ```
    /// use icedtea::m3::{Component, Shape, ShapePolicy};
    /// let tok = icedtea::theme::named("dark")
    ///     .tokens
    ///     .with_shape(ShapePolicy::Material);
    /// assert_eq!(
    ///     Component::Card.shape_for(tok.shape),
    ///     Shape::Medium
    /// );
    /// ```
    pub fn with_shape(mut self, shape: crate::m3::ShapePolicy) -> Self {
        self.shape = shape;
        self
    }

    /// Same tokens with a shadow policy.
    ///
    /// ```
    /// use icedtea::m3::{Elevation, ElevationPolicy};
    /// let tok = icedtea::theme::named("dark")
    ///     .tokens
    ///     .with_elevation(ElevationPolicy::Flat);
    /// assert_eq!(tok.shadow(Elevation::Level2).blur_radius, 0.0);
    /// ```
    pub fn with_elevation(mut self, elevation: crate::m3::ElevationPolicy) -> Self {
        self.elevation = elevation;
        self
    }

    /// Same tokens with a text direction.
    pub fn with_direction(mut self, direction: crate::i18n::Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Same tokens with a clock digit set.
    pub fn with_clock_digits(mut self, clock_digits: crate::i18n::ClockDigits) -> Self {
        self.clock_digits = clock_digits;
        self
    }

    /// Pixel size for a type role after [`Self::font_scale`].
    pub fn type_px(self, role: crate::typo::TypeRole) -> f32 {
        (role.size() as f32 * self.font_scale).round()
    }

    /// Body type size (M3 Body Medium × scale).
    pub fn body(self) -> f32 {
        self.type_px(crate::typo::TypeRole::Body)
    }

    /// Meta / label type size.
    pub fn meta(self) -> f32 {
        self.type_px(crate::typo::TypeRole::Meta)
    }

    /// Section title type size.
    pub fn title(self) -> f32 {
        self.type_px(crate::typo::TypeRole::Title)
    }

    /// Page title type size.
    pub fn page(self) -> f32 {
        self.type_px(crate::typo::TypeRole::Page)
    }

    /// Code / monospace type size.
    pub fn code(self) -> f32 {
        self.type_px(crate::typo::TypeRole::Code)
    }

    /// Display type size.
    pub fn display(self) -> f32 {
        self.type_px(crate::typo::TypeRole::Display)
    }

    /// Corner radius for a control family under [`Self::shape`].
    pub fn radius(self, component: crate::m3::Component) -> iced::border::Radius {
        component.radius_for(self.shape)
    }

    /// Drop shadow for a requested level under [`Self::elevation`].
    ///
    /// Flat policy zeros the shadow. Tonal surfaces stay as requested.
    pub fn shadow(self, level: crate::m3::Elevation) -> iced::Shadow {
        match self.elevation {
            crate::m3::ElevationPolicy::Desktop => level.shadow(),
            crate::m3::ElevationPolicy::Flat => iced::Shadow::default(),
        }
    }

    /// Multiply every scheme role's alpha by `amount` (0..=1).
    ///
    /// Overlay chrome builds its child with `tok.fade(progress)` so
    /// fills, ink, and icons fade with the slide. `Slide::None` uses
    /// the same fade with no translate.
    pub fn fade(self, amount: f32) -> Self {
        let mut out = Self::from(self.full.fade(amount));
        out.density = self.density;
        out.reduced_motion = self.reduced_motion;
        out.font_scale = self.font_scale;
        out.shape = self.shape;
        out.elevation = self.elevation;
        out.direction = self.direction;
        out
    }

    /// Rebuild `full` after short fields change (OS chrome, catalog).
    fn sync_full_from_aliases(mut self) -> Self {
        let base = if relative_luma(self.canvas) < 0.45 {
            crate::m3::scheme_dark()
        } else {
            crate::m3::scheme_light()
        };
        let mut s = base;
        s.primary = self.primary;
        s.secondary = self.accent;
        s.error = self.danger;
        s.success = self.success;
        s.warning = self.warning;
        // Community colorways only list short aliases. Recompute "on" roles
        // for solid fills so switch thumbs / button labels are not stuck on
        // the M3 baseline (e.g. dark purple on_primary next to gruvbox primary).
        s.on_primary = ink_on_fill(self.primary, self.text, self.canvas);
        s.on_secondary = ink_on_fill(self.accent, self.text, self.canvas);
        s.on_error = ink_on_fill(self.danger, self.text, self.canvas);
        s.on_success = ink_on_fill(self.success, self.text, self.canvas);
        s.on_warning = ink_on_fill(self.warning, self.text, self.canvas);
        s.surface = self.canvas;
        s.surface_container = self.surface;
        s.surface_container_high = self.panel;
        s.surface_container_low = mix(self.surface, self.canvas, 0.5);
        s.surface_container_lowest = self.canvas;
        s.surface_container_highest = mix(self.text, self.panel, 0.08);
        s.surface_variant = mix(self.text, self.canvas, 0.12);
        s.on_surface = self.text;
        s.on_surface_variant = self.muted;
        s.outline = self.border;
        s.outline_variant = mix(self.border, self.canvas, 0.5);
        s.secondary_container = self.selection;
        s.on_secondary_container = self.selection_text;
        s.primary_container = mix(self.primary, self.canvas, 0.25);
        s.on_primary_container = self.text;
        s.error_container = mix(self.danger, self.canvas, 0.20);
        s.on_error_container = self.text;
        s.inverse_surface = self.text;
        s.inverse_on_surface = self.canvas;
        s.inverse_primary = self.primary;
        self.full = s;
        self
    }
}

/// Ink that contrasts with a solid fill using the colorway's text/canvas pair.
fn ink_on_fill(fill: Color, light: Color, dark: Color) -> Color {
    let on_light = (relative_luma(light) - relative_luma(fill)).abs();
    let on_dark = (relative_luma(dark) - relative_luma(fill)).abs();
    if on_light >= on_dark {
        light
    } else {
        dark
    }
}

/// A named theme: catalog key plus tokens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NamedTheme {
    pub name: &'static str,
    pub tokens: Tokens,
    pub dark: bool,
}

/// Blend `fg` over `bg` by `amount` (0 = bg, 1 = fg). Result is opaque.
///
/// Hover, pressed, and selection washes use this.
///
///
/// ```
/// use iced::Color;
/// let mixed = icedtea::theme::mix(Color::WHITE, Color::BLACK, 0.5);
/// assert!((mixed.r - 0.5).abs() < 0.01);
/// assert_eq!(mixed.a, 1.0);
/// ```
pub fn mix(fg: Color, bg: Color, amount: f32) -> Color {
    let t = amount.clamp(0.0, 1.0);
    Color::from_rgb(
        fg.r * t + bg.r * (1.0 - t),
        fg.g * t + bg.g * (1.0 - t),
        fg.b * t + bg.b * (1.0 - t),
    )
}

/// Hover wash over canvas.
pub fn hover_fill(tok: Tokens) -> Color {
    crate::m3::color::state_hover(tok.scheme())
}

/// Pressed wash over surface.
pub fn pressed_fill(tok: Tokens) -> Color {
    crate::m3::color::state_pressed(tok.scheme())
}

/// Chip / assist fill (M3 secondary container).
pub fn chip_fill(tok: Tokens) -> Color {
    tok.scheme().secondary_container
}

/// Primary wash used for selected rows (M3 secondary container).
pub fn selection_fill(tok: Tokens) -> Color {
    tok.scheme().secondary_container
}

/// Mix `color` toward white.
pub fn lighten(color: Color, amount: f32) -> Color {
    mix(Color::WHITE, color, amount)
}

/// Mix `color` toward black.
pub fn darken(color: Color, amount: f32) -> Color {
    mix(Color::BLACK, color, amount)
}

/// Token ink that stays readable on `bg`.
pub fn text_on(tok: Tokens, bg: Color) -> Color {
    let on_text = (relative_luma(tok.text) - relative_luma(bg)).abs();
    let on_canvas = (relative_luma(tok.canvas) - relative_luma(bg)).abs();
    if on_text >= on_canvas {
        tok.text
    } else {
        tok.canvas
    }
}

/// Washes and chrome colors derived from [`Tokens`] via [`mix`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Faces {
    pub hover: Color,
    pub pressed: Color,
    pub chip: Color,
    pub selection: Color,
    pub text_on_canvas: Color,
    pub text_on_surface: Color,
    pub text_on_panel: Color,
    pub text_on_primary: Color,
    pub scrollbar: Color,
    pub input_cursor: Color,
    pub input_selection: Color,
    pub link: Color,
    pub focus: Color,
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

#[allow(clippy::too_many_arguments)]
fn tokens(
    canvas: Color,
    surface: Color,
    panel: Color,
    text: Color,
    muted: Color,
    primary: Color,
    accent: Color,
    success: Color,
    warning: Color,
    danger: Color,
    border: Color,
) -> Tokens {
    Tokens {
        canvas,
        surface,
        panel,
        text,
        muted,
        primary,
        accent,
        success,
        warning,
        danger,
        border,
        selection: mix(primary, canvas, 0.28),
        selection_text: text,
        density: crate::m3::Density::default(),
        reduced_motion: false,
        font_scale: 1.0,
        shape: crate::m3::ShapePolicy::Desktop,
        elevation: crate::m3::ElevationPolicy::Desktop,
        direction: crate::i18n::Direction::Ltr,
        clock_digits: crate::i18n::ClockDigits::Western,
        full: crate::m3::scheme_dark(), // replaced by sync
    }
    .sync_full_from_aliases()
}

const CATALOG_JSON: &str = include_str!("../assets/themes/catalog.json");

fn intern(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

fn parse_hex(s: &str) -> Option<Color> {
    let t = s.trim().trim_start_matches('#');
    if t.len() < 6 || !t.as_bytes()[..6].iter().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let r = u8::from_str_radix(&t[0..2], 16).ok()?;
    let g = u8::from_str_radix(&t[2..4], 16).ok()?;
    let b = u8::from_str_radix(&t[4..6], 16).ok()?;
    Some(Color::from_rgb8(r, g, b))
}

fn hex_of(obj: &Value, key: &str, fallback: Color) -> Color {
    obj.get(key)
        .and_then(Value::as_str)
        .and_then(parse_hex)
        .unwrap_or(fallback)
}

fn high_contrast() -> NamedTheme {
    NamedTheme {
        name: "high-contrast",
        dark: true,
        tokens: Tokens {
            canvas: rgb(0x00, 0x00, 0x00),
            surface: rgb(0x00, 0x00, 0x00),
            panel: rgb(0x1A, 0x1A, 0x1A),
            text: rgb(0xFF, 0xFF, 0xFF),
            muted: rgb(0xE0, 0xE0, 0xE0),
            primary: rgb(0xFF, 0xFF, 0x00),
            accent: rgb(0x00, 0xFF, 0xFF),
            success: rgb(0x00, 0xFF, 0x00),
            warning: rgb(0xFF, 0xC0, 0x00),
            danger: rgb(0xFF, 0x40, 0x40),
            border: rgb(0xFF, 0xFF, 0xFF),
            selection: rgb(0x00, 0x00, 0xAA),
            selection_text: rgb(0xFF, 0xFF, 0xFF),
            density: crate::m3::Density::default(),
            reduced_motion: false,
            font_scale: 1.0,
            shape: crate::m3::ShapePolicy::Desktop,
            elevation: crate::m3::ElevationPolicy::Desktop,
            direction: crate::i18n::Direction::Ltr,
            clock_digits: crate::i18n::ClockDigits::Western,
            full: crate::m3::scheme_dark(),
        }
        .sync_full_from_aliases(),
    }
}

fn alias(name: &str) -> &str {
    match name {
        "gruvbox-dark" => "gruvbox",
        "one-dark" | "onedark" => "atom-one-dark",
        "one-light" => "atom-one-light",
        _ => name,
    }
}

fn catalog_from_json(raw: &str) -> BTreeMap<&'static str, NamedTheme> {
    let mut map = BTreeMap::new();
    if let Ok(Value::Object(root)) = serde_json::from_str::<Value>(raw) {
        for (name, rec) in root {
            let Some(obj) = rec.as_object() else {
                continue;
            };
            let rec = Value::Object(obj.clone());
            let canvas = hex_of(&rec, "canvas", rgb(0x20, 0x20, 0x20));
            let text = auto_ink(canvas, 0.87);
            let muted = auto_ink(canvas, 0.60);
            let primary = hex_of(&rec, "primary", rgb(0x6B, 0x9E, 0xFF));
            let panel = hex_of(&rec, "panel", mix(text, canvas, 0.10));
            let surface = hex_of(&rec, "surface", panel);
            let key = intern(&name);
            map.insert(
                key,
                NamedTheme {
                    name: key,
                    dark: rec.get("dark").and_then(Value::as_bool).unwrap_or(true),
                    tokens: tokens(
                        canvas,
                        surface,
                        panel,
                        text,
                        muted,
                        primary,
                        hex_of(&rec, "accent", rgb(0x5E, 0xEA, 0xD4)),
                        hex_of(&rec, "success", rgb(0x4A, 0xDE, 0x80)),
                        hex_of(&rec, "warning", rgb(0xFB, 0xBF, 0x24)),
                        hex_of(&rec, "danger", rgb(0xF8, 0x71, 0x71)),
                        hex_of(&rec, "border", mix(primary, canvas, 0.35)),
                    ),
                },
            );
        }
    }
    map.insert("high-contrast", high_contrast());
    map
}

fn load_catalog() -> BTreeMap<&'static str, NamedTheme> {
    catalog_from_json(CATALOG_JSON)
}

fn catalog() -> &'static BTreeMap<&'static str, NamedTheme> {
    static CATALOG: OnceLock<BTreeMap<&'static str, NamedTheme>> = OnceLock::new();
    CATALOG.get_or_init(load_catalog)
}

fn builtin(name: &str) -> Option<NamedTheme> {
    let key = alias(name.trim());
    if let Some(t) = catalog().get(key) {
        return Some(*t);
    }
    catalog().get(key.to_ascii_lowercase().as_str()).copied()
}

/// Built-in catalog keys, `dark` / `light` / `high-contrast` first.
pub fn builtin_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = catalog().keys().copied().collect();
    names.sort_unstable();
    let mut ordered = Vec::with_capacity(names.len());
    for first in ["dark", "light", "high-contrast"] {
        if names.contains(&first) {
            ordered.push(first);
        }
    }
    for n in names {
        if !ordered.contains(&n) {
            ordered.push(n);
        }
    }
    ordered
}

/// Rec. 709 luma in 0..1.
pub fn relative_luma(c: Color) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

/// Textual `Color.brightness` (Rec. 601). Auto ink picks black or white on this.
fn rec601_brightness(c: Color) -> f32 {
    (299.0 * c.r + 587.0 * c.g + 114.0 * c.b) / 1000.0
}

/// Body or mute ink on *canvas*: white or black mixed toward the paper.
///
/// Textual `$text` is `auto 87%` and `$text-muted` is `auto 60%`. Catalog
/// `foreground` is often a mid-gray (Solarized base0) that fails on the
/// same paper.
///
/// ```
/// let paper = iced::Color::from_rgb8(0x07, 0x36, 0x42);
/// let body = icedtea::theme::auto_ink(paper, 0.87);
/// assert!(body.r > 0.8);
/// ```
pub fn auto_ink(canvas: Color, amount: f32) -> Color {
    let ink = if rec601_brightness(canvas) < 0.5 {
        Color::WHITE
    } else {
        Color::BLACK
    };
    mix(ink, canvas, amount)
}

impl Tokens {
    /// True when the canvas is a dark background.
    pub fn canvas_is_dark(self) -> bool {
        relative_luma(self.canvas) < 0.45
    }

    /// Derived washes and chrome colors from [`Self::scheme`].
    ///
    /// ```
    /// let tok = icedtea::theme::named("dark").tokens;
    /// let faces = tok.faces();
    /// let s = tok.scheme();
    /// assert_eq!(faces.link, s.primary);
    /// assert_eq!(faces.hover, icedtea::theme::hover_fill(tok));
    /// ```
    pub fn faces(self) -> Faces {
        let s = self.scheme();
        Faces {
            hover: hover_fill(self),
            pressed: pressed_fill(self),
            chip: chip_fill(self),
            selection: selection_fill(self),
            text_on_canvas: text_on(self, self.canvas),
            text_on_surface: text_on(self, self.surface),
            text_on_panel: text_on(self, self.panel),
            text_on_primary: text_on(self, self.primary),
            scrollbar: s.outline,
            input_cursor: s.primary,
            input_selection: s.secondary_container,
            link: s.primary,
            focus: s.primary,
        }
    }
}

/// iced highlighter face that fits this UI colorway.
pub fn code_highlight(name: &str) -> iced::highlighter::Theme {
    use iced::highlighter::Theme as H;
    let t = named(name);
    if !t.dark {
        return H::InspiredGitHub;
    }
    let n = t.name;
    if n.contains("solarized") {
        H::SolarizedDark
    } else if n.contains("mocha")
        || n.contains("frappe")
        || n.contains("macchiato")
        || n.contains("catppuccin")
        || n.contains("rose-pine")
        || n.contains("palenight")
        || n.contains("material")
        || n.contains("night-owl")
    {
        H::Base16Mocha
    } else if n.contains("nord")
        || n.contains("tokyo")
        || n.contains("nightfox")
        || n.contains("kanagawa")
        || n.contains("ayu")
        || n.contains("github")
        || n.contains("oxocarbon")
    {
        H::Base16Ocean
    } else if n.contains("gruvbox")
        || n.contains("monokai")
        || n.contains("dracula")
        || n.contains("everforest")
        || n.contains("cobalt")
    {
        H::Base16Eighties
    } else {
        H::SolarizedDark
    }
}

/// Look up a built-in colorway by name. Unknown names resolve to `dark`.
/// `light` and `dark` are the desktop pair; other names are a choice.
///
///
/// ```
/// let t = icedtea::theme::named("nope");
/// assert_eq!(t.name, "dark");
/// assert!(icedtea::theme::named("solarized-light").tokens.canvas_is_dark() == false);
/// ```
pub fn named(name: &str) -> NamedTheme {
    builtin(name).unwrap_or_else(|| builtin("dark").expect("dark exists"))
}

/// Application-owned catalog: builtins plus registered extras.
///
/// ```
/// let mut cat = icedtea::theme::ThemeCatalog::new();
/// assert!(cat.get("dark").is_some());
/// let custom = icedtea::theme::named("dark").tokens;
/// cat.register("app-brand", custom, true);
/// assert!(cat.get("app-brand").is_some());
/// assert_eq!(cat.resolve("app-brand").primary, custom.primary);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ThemeCatalog {
    extra: BTreeMap<String, (Tokens, bool)>,
}

impl ThemeCatalog {
    /// Built-in themes only.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register or replace an application theme.
    pub fn register(&mut self, name: impl Into<String>, tokens: Tokens, dark: bool) {
        self.extra.insert(name.into(), (tokens, dark));
    }

    /// Tokens for `name`, or dark if missing.
    pub fn resolve(&self, name: &str) -> Tokens {
        self.get(name)
            .map(|t| t.tokens)
            .unwrap_or_else(|| named("dark").tokens)
    }

    /// Full named theme if present (builtin or registered).
    pub fn get(&self, name: &str) -> Option<ResolvedTheme> {
        let key = name.trim();
        if let Some((tokens, dark)) = self.extra.get(key) {
            return Some(ResolvedTheme {
                name: key.to_string(),
                tokens: *tokens,
                dark: *dark,
            });
        }
        builtin(key).map(|n| ResolvedTheme {
            name: n.name.to_string(),
            tokens: n.tokens,
            dark: n.dark,
        })
    }

    /// Catalog keys: builtins then registered names, sorted extras after.
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = builtin_names().into_iter().map(str::to_string).collect();
        for k in self.extra.keys() {
            if !names.iter().any(|n| n == k) {
                names.push(k.clone());
            }
        }
        names
    }
}

/// Owned theme from the catalog (registered names are not `&'static`).
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedTheme {
    pub name: String,
    pub tokens: Tokens,
    pub dark: bool,
}

/// iced [`Theme`](iced::Theme) built from tokens.
pub fn iced_theme(name: &str, tokens: Tokens) -> iced::Theme {
    iced::Theme::custom(
        name.to_string(),
        iced::theme::Palette {
            background: tokens.canvas,
            text: tokens.text,
            primary: tokens.primary,
            success: tokens.success,
            warning: tokens.warning,
            danger: tokens.danger,
        },
    )
}

/// Persistable theme id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeId(pub String);

impl ThemeId {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

/// Light or dark appearance. Injected in tests; live apps take
/// [`iced::theme::Mode`] from iced / mundy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Appearance {
    Light,
    Dark,
}

impl Appearance {
    /// Map iced / mundy mode. `None` is no preference (GNOME Light is
    /// often `default` on the portal), so it follows the light member.
    pub fn from_mode(mode: iced::theme::Mode) -> Self {
        match mode {
            iced::theme::Mode::Light | iced::theme::Mode::None => Self::Light,
            iced::theme::Mode::Dark => Self::Dark,
        }
    }
}

/// Explicit light/dark pair in the catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Family {
    pub id: &'static str,
    pub light: &'static str,
    pub dark: &'static str,
}

/// Built-in pairs. Names that are not a member stay on themselves.
pub const FAMILIES: &[Family] = &[
    Family {
        id: "default",
        light: "light",
        dark: "dark",
    },
    Family {
        id: "atom-one",
        light: "atom-one-light",
        dark: "atom-one-dark",
    },
    Family {
        id: "ayu",
        light: "ayu-light",
        dark: "ayu-dark",
    },
    Family {
        id: "catppuccin",
        light: "catppuccin-latte",
        dark: "catppuccin-mocha",
    },
    Family {
        id: "everforest",
        light: "everforest-light",
        dark: "everforest-dark",
    },
    Family {
        id: "github",
        light: "github-light",
        dark: "github-dark",
    },
    Family {
        id: "gruvbox",
        light: "gruvbox-light",
        dark: "gruvbox",
    },
    Family {
        id: "kanagawa",
        light: "kanagawa-lotus",
        dark: "kanagawa-wave",
    },
    Family {
        id: "modus",
        light: "modus-operandi",
        dark: "modus-vivendi",
    },
    Family {
        id: "rose-pine",
        light: "rose-pine-dawn",
        dark: "rose-pine",
    },
    Family {
        id: "solarized",
        light: "solarized-light",
        dark: "solarized-dark",
    },
    Family {
        id: "tokyo-night",
        light: "tokyo-night-day",
        dark: "tokyo-night",
    },
];

pub fn family(id: &str) -> Option<&'static Family> {
    FAMILIES.iter().find(|f| f.id == id)
}

pub fn family_of_name(name: &str) -> Option<&'static Family> {
    let key = alias(name.trim());
    FAMILIES
        .iter()
        .find(|f| f.id == key || f.light == key || f.dark == key)
}

/// Light or dark member of `family_id`.
pub fn follow(family_id: &str, appearance: Appearance) -> Option<&'static str> {
    let f = family(family_id)?;
    Some(match appearance {
        Appearance::Light => f.light,
        Appearance::Dark => f.dark,
    })
}

/// Concrete catalog name for a preference.
///
/// Follow-OS uses the family (or the pair of `name`). High-contrast and
/// unpaired names stay on themselves. `Boot.theme` stays a concrete name.
pub fn resolve_pref(
    name: &str,
    family_id: Option<&str>,
    follow_os: bool,
    appearance: Appearance,
) -> String {
    if !follow_os {
        return named(name).name.to_string();
    }
    if alias(name.trim()) == "high-contrast" {
        return "high-contrast".into();
    }
    let fam = family_id.and_then(family).or_else(|| family_of_name(name));
    match fam {
        Some(f) => match appearance {
            Appearance::Light => f.light.to_string(),
            Appearance::Dark => f.dark.to_string(),
        },
        None => named(name).name.to_string(),
    }
}

/// Optional desktop chrome colors layered onto a named colorway.
///
/// Persist defaults `follow_os` on. When true, [`apply_os_chrome`]
/// overwrites only fields that are `Some`. Hosts fill what they can.
/// Leave every field `None` (or call [`OsChrome::empty`]) and set
/// `follow_os` to `false` to keep a chosen colorway as authored.
///
/// | Field | Typical host source |
/// | --- | --- |
/// | `primary` | Accent (portal, Windows accent, macOS control accent) |
/// | `canvas` | Window / content background (macOS, Windows) |
/// | `surface` | Text / content fill (macOS, Windows) |
/// | `panel` | Control / chrome strip fill (macOS, Windows) |
/// | `text` | Primary label (macOS, Windows) |
/// | `muted` | Secondary label / gray text (macOS, Windows) |
/// | `border` | Separator / shadow edge (macOS, Windows) |
///
/// Linux (Wayland and X11 via the settings portal) currently provides
/// `primary` when the desktop publishes an accent. Other fields stay
/// unset so the colorway keeps surfaces and ink. Success, warning, and
/// danger always stay on the colorway.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct OsChrome {
    /// System accent → [`Tokens::primary`].
    pub primary: Option<Color>,
    /// Window / content background → [`Tokens::canvas`].
    pub canvas: Option<Color>,
    /// Raised content fill → [`Tokens::surface`].
    pub surface: Option<Color>,
    /// Chrome strip / control fill → [`Tokens::panel`].
    pub panel: Option<Color>,
    /// Primary label → [`Tokens::text`].
    pub text: Option<Color>,
    /// Secondary label → [`Tokens::muted`].
    pub muted: Option<Color>,
    /// Separator → [`Tokens::border`].
    pub border: Option<Color>,
}

impl OsChrome {
    /// No host colors — applying this with `follow_os` is a no-op.
    pub const fn empty() -> Self {
        Self {
            primary: None,
            canvas: None,
            surface: None,
            panel: None,
            text: None,
            muted: None,
            border: None,
        }
    }

    /// True when at least one field would change tokens under follow-OS.
    pub fn any(self) -> bool {
        self.primary.is_some()
            || self.canvas.is_some()
            || self.surface.is_some()
            || self.panel.is_some()
            || self.text.is_some()
            || self.muted.is_some()
            || self.border.is_some()
    }
}

/// One-shot desktop chrome. Prefer [`listen_os_chrome`] in a running
/// app. On macOS call this on the main thread only (same as system
/// accent reads).
///
/// ```
/// let chrome = icedtea::theme::os_chrome();
/// let _ = chrome.primary;
/// ```
pub fn os_chrome() -> OsChrome {
    crate::host_chrome::snapshot()
}

/// Emits a full [`OsChrome`] when the desktop accent or color-scheme
/// changes. First item is the current snapshot.
pub fn listen_os_chrome() -> iced::Subscription<OsChrome> {
    iced::Subscription::run(crate::host_chrome::listen)
}

/// Layer host chrome onto a colorway.
///
/// - `follow_os == false`: returns `tokens` unchanged (opt out).
/// - `follow_os == true`: each `Some` field in `chrome` replaces the
///   matching token; then selection is rebuilt from primary + canvas.
///
/// Decorated windows keep the native title bar. High-contrast colorways
/// are not special-cased here — turn `follow_os` off if the catalog face
/// must stay absolute.
///
/// ```
/// use iced::Color;
/// use icedtea::theme::{self, OsChrome};
/// let tok = theme::named("dark").tokens;
/// let chrome = OsChrome {
///     primary: Some(Color::from_rgb8(0, 122, 255)),
///     canvas: Some(Color::from_rgb8(30, 30, 30)),
///     ..OsChrome::empty()
/// };
/// let out = theme::apply_os_chrome(tok, true, chrome);
/// assert_eq!(out.primary, chrome.primary.unwrap());
/// assert_eq!(out.canvas, chrome.canvas.unwrap());
/// assert_eq!(out.text, tok.text);
/// let off = theme::apply_os_chrome(tok, false, chrome);
/// assert_eq!(off, tok);
/// ```
pub fn apply_os_chrome(tokens: Tokens, follow_os: bool, chrome: OsChrome) -> Tokens {
    if !follow_os {
        return tokens;
    }
    let mut tokens = tokens;
    let mut dirty = false;
    if let Some(c) = chrome.primary {
        tokens.primary = c;
        dirty = true;
    }
    if let Some(c) = chrome.canvas {
        tokens.canvas = c;
        dirty = true;
    }
    if let Some(c) = chrome.surface {
        tokens.surface = c;
        dirty = true;
    }
    if let Some(c) = chrome.panel {
        tokens.panel = c;
        dirty = true;
    }
    if let Some(c) = chrome.text {
        tokens.text = c;
        tokens.selection_text = c;
        dirty = true;
    }
    if let Some(c) = chrome.muted {
        tokens.muted = c;
        dirty = true;
    }
    if let Some(c) = chrome.border {
        tokens.border = c;
        dirty = true;
    }
    if dirty {
        // Rebuild selection wash from primary on canvas (M3 secondary container tone).
        tokens.selection = mix(tokens.primary, tokens.canvas, 0.28);
        if chrome.text.is_some() {
            tokens.selection_text = tokens.text;
        }
        tokens = tokens.sync_full_from_aliases();
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheme_roundtrip_preserves_baseline_roles() {
        for baseline in [crate::m3::scheme_light(), crate::m3::scheme_dark()] {
            let tok: Tokens = baseline.into();
            assert_eq!(tok.scheme(), baseline);
            assert_eq!(tok.primary, baseline.primary);
            assert_eq!(tok.canvas, baseline.surface);
        }
        let dark = named("dark").tokens;
        assert_eq!(dark.primary, rgb(0x6B, 0x9E, 0xFF));
        assert_eq!(dark.canvas, rgb(0x20, 0x20, 0x20));
        assert_eq!(dark.scheme().secondary_container, dark.selection);
        let light = named("light").tokens;
        assert_eq!(light.primary, rgb(0x25, 0x63, 0xEB));
        assert_eq!(light.canvas, rgb(0xF3, 0xF3, 0xF3));
        assert_ne!(dark.primary, crate::m3::scheme_dark().primary);
        assert_ne!(light.primary, crate::m3::scheme_light().primary);
    }

    #[test]
    fn from_aliases_scheme_matches_the_passed_colors() {
        let canvas = rgb(0x1e, 0x1e, 0x2e);
        let primary = rgb(0x89, 0xb4, 0xfa);
        let tok = Tokens::from_aliases(
            canvas,
            rgb(0x31, 0x32, 0x44),
            rgb(0x18, 0x18, 0x25),
            rgb(0xcd, 0xd6, 0xf4),
            rgb(0xa6, 0xad, 0xc8),
            primary,
            rgb(0xf5, 0xc2, 0xe7),
            rgb(0xa6, 0xe3, 0xa1),
            rgb(0xf9, 0xe2, 0xaf),
            rgb(0xf3, 0x8b, 0xa8),
            rgb(0x45, 0x47, 0x5a),
        );
        assert_eq!(tok.canvas, canvas);
        assert_eq!(tok.primary, primary);
        assert_eq!(tok.scheme().surface, canvas);
        assert_eq!(tok.scheme().primary, primary);
        assert_ne!(tok.scheme().primary, named("dark").tokens.primary);
        assert_ne!(tok.scheme().primary, crate::m3::scheme_dark().primary);
        let _ = crate::style::button_style(tok, crate::variant::Variant::Primary);
    }

    #[test]
    fn community_on_primary_is_not_m3_baseline_purple() {
        // Gruvbox (and other catalog colorways) set primary from JSON but used
        // to leave on_primary as scheme_dark's purple (#381E72). Switch thumbs
        // and filled-control ink read that residual as an out-of-palette blue.
        let g = named("gruvbox").tokens.scheme();
        let m3_dark = crate::m3::scheme_dark();
        assert_eq!(g.primary, named("gruvbox").tokens.primary);
        assert_ne!(g.on_primary, m3_dark.on_primary);
        assert_eq!(
            g.on_primary,
            ink_on_fill(
                g.primary,
                named("gruvbox").tokens.text,
                named("gruvbox").tokens.canvas
            )
        );
        let nord = named("nord").tokens.scheme();
        assert_ne!(nord.on_primary, m3_dark.on_primary);
        // Solid fills use contrasting pair ink.
        assert_ne!(g.on_primary, g.primary);
    }

    #[test]
    fn mix_is_opaque_between_endpoints() {
        let a = Color::from_rgb8(255, 0, 0);
        let b = Color::from_rgb8(0, 0, 0);
        let m = mix(a, b, 0.5);
        assert!((m.r - 0.5).abs() < 0.01);
        assert_eq!(m.a, 1.0);
        assert_eq!(mix(a, b, 0.0), b);
        assert_eq!(mix(a, b, 1.0).r, a.r);
        assert_eq!(mix(a, b, 2.0).r, a.r);
        assert_eq!(mix(a, b, -1.0), b);
    }

    #[test]
    fn washes_are_between_text_and_canvas() {
        let t = named("dark").tokens;
        let h = hover_fill(t);
        assert!(h.r > t.canvas.r);
        assert!(pressed_fill(t).r >= h.r);
        assert!(chip_fill(t).a == 1.0);
        let sel = selection_fill(t);
        assert_eq!(sel, t.selection);
    }

    #[test]
    fn fade_scales_surface_and_ink() {
        let tok = named("dark")
            .tokens
            .with_reduced_motion(true)
            .with_density(crate::m3::Density::named(crate::m3::DensityName::Compact))
            .with_font_scale(1.25)
            .with_shape(crate::m3::ShapePolicy::Material)
            .with_elevation(crate::m3::ElevationPolicy::Flat)
            .with_direction(crate::i18n::Direction::Rtl);
        let mid = tok.fade(0.5);
        assert!(mid.reduced_motion);
        assert_eq!(
            mid.density,
            crate::m3::Density::named(crate::m3::DensityName::Compact)
        );
        assert!((mid.font_scale - 1.25).abs() < f32::EPSILON);
        assert_eq!(mid.shape, crate::m3::ShapePolicy::Material);
        assert_eq!(mid.elevation, crate::m3::ElevationPolicy::Flat);
        assert_eq!(mid.direction, crate::i18n::Direction::Rtl);
        assert_eq!(mid.body(), 18.0);
        assert_eq!(mid.shadow(crate::m3::Elevation::Level3).blur_radius, 0.0);
        assert_eq!(named("dark").tokens.with_font_scale(0.1).font_scale, 0.75);
        assert_eq!(named("dark").tokens.with_font_scale(9.0).font_scale, 1.5);
        let scaled = named("dark").tokens.with_font_scale(1.25);
        assert_eq!(scaled.meta(), 15.0);
        assert_eq!(scaled.title(), 20.0);
        assert_eq!(scaled.page(), 28.0);
        assert_eq!(scaled.code(), 15.0);
        assert_eq!(scaled.display(), 45.0);
        assert_eq!(scaled.radius(crate::m3::Component::Card).top_left, 0.0);
        assert!(
            named("dark")
                .tokens
                .shadow(crate::m3::Elevation::Level2)
                .blur_radius
                > 0.0
        );
        assert_eq!(
            scaled
                .with_shape(crate::m3::ShapePolicy::Soft)
                .radius(crate::m3::Component::Button)
                .top_left,
            12.0
        );
        assert!((mid.scheme().surface.a - 0.5).abs() < 1e-5);
        assert!((mid.text.a - 0.5).abs() < 1e-5);
        assert!((mid.primary.a - 0.5).abs() < 1e-5);
        assert_eq!(tok.fade(1.0).text.a, tok.text.a);
    }

    #[test]
    fn builtin_names_resolve() {
        let names = builtin_names();
        assert_eq!(names.len(), 40);
        assert_eq!(names[0], "dark");
        assert_eq!(names[1], "light");
        assert_eq!(names[2], "high-contrast");
        for name in &names {
            let t = named(name);
            assert_eq!(t.name, *name);
        }
        assert_eq!(named("").name, "dark");
        assert_eq!(named("Dark").name, "dark");
        assert_eq!(named("LIGHT").name, "light");
        assert_eq!(named("  nord  ").name, "nord");
        assert_eq!(named("dark").tokens.canvas, rgb(0x20, 0x20, 0x20));
        assert_eq!(named("light").tokens.canvas, rgb(0xF3, 0xF3, 0xF3));
        assert!(named("light").tokens.canvas.r > named("dark").tokens.canvas.r);
        assert_eq!(named("high-contrast").tokens.border, rgb(0xFF, 0xFF, 0xFF));
        assert_ne!(named("gruvbox").tokens.canvas, named("nord").tokens.canvas);
        assert_eq!(named("gruvbox-dark").name, "gruvbox");
        assert!(!named("solarized-light").dark);
        assert!(named("solarized-dark").dark);
        assert!(named("catppuccin-mocha").dark);
        assert!(!named("catppuccin-latte").tokens.canvas_is_dark());
        assert_eq!(named("one-dark").name, "atom-one-dark");
        assert_eq!(named("one-light").name, "atom-one-light");
        assert!(named("everforest-dark").dark);
        assert!(!named("kanagawa-lotus").dark);
    }

    #[test]
    fn code_highlight_follows_colorway() {
        use iced::highlighter::Theme as H;
        assert_eq!(code_highlight("solarized-dark"), H::SolarizedDark);
        assert_eq!(code_highlight("solarized-light"), H::InspiredGitHub);
        assert_eq!(code_highlight("light"), H::InspiredGitHub);
        assert_eq!(code_highlight("catppuccin-mocha"), H::Base16Mocha);
        assert_eq!(code_highlight("nord"), H::Base16Ocean);
        assert_eq!(code_highlight("gruvbox"), H::Base16Eighties);
        assert_eq!(code_highlight("dracula"), H::Base16Eighties);
    }

    #[test]
    fn catalog_register_and_resolve() {
        let mut cat = ThemeCatalog::new();
        assert_eq!(cat.names().len(), builtin_names().len());
        assert!(cat.get("dark").is_some());
        assert!(cat.get("missing").is_none());
        assert_eq!(cat.resolve("missing"), named("dark").tokens);
        let brand = named("light").tokens;
        cat.register("brand", brand, false);
        assert_eq!(cat.resolve("brand"), brand);
        assert!(!cat.get("brand").unwrap().dark);
        assert!(cat.names().contains(&"brand".to_string()));
        cat.register("dark", brand, false);
        assert_eq!(cat.resolve("dark"), brand);
    }

    #[test]
    fn iced_theme_uses_token_canvas() {
        let t = named("dark").tokens;
        let iced = iced_theme("dark", t);
        assert_eq!(iced.palette().background, t.canvas);
        let id = ThemeId::new("dark");
        assert_eq!(id.0, "dark");
    }

    #[test]
    fn families_follow_os_and_leave_unpaired() {
        assert_eq!(family("github").unwrap().light, "github-light");
        assert_eq!(family_of_name("github-dark").unwrap().id, "github");
        assert_eq!(family_of_name("gruvbox-dark").unwrap().id, "gruvbox");
        assert!(family_of_name("nord").is_none());
        assert!(family_of_name("high-contrast").is_none());
        assert_eq!(follow("default", Appearance::Light), Some("light"));
        assert_eq!(follow("default", Appearance::Dark), Some("dark"));
        assert!(follow("missing", Appearance::Dark).is_none());
        assert_eq!(
            resolve_pref("github-dark", None, true, Appearance::Light),
            "github-light"
        );
        assert_eq!(resolve_pref("nord", None, true, Appearance::Light), "nord");
        assert_eq!(
            resolve_pref("high-contrast", Some("default"), true, Appearance::Light),
            "high-contrast"
        );
        assert_eq!(
            resolve_pref("nord", Some("github"), true, Appearance::Light),
            "github-light"
        );
        assert_eq!(resolve_pref("dark", None, false, Appearance::Light), "dark");
        assert_eq!(
            resolve_pref("dark", Some("default"), true, Appearance::Light),
            "light"
        );
        assert_eq!(
            resolve_pref(
                "dark",
                Some("default"),
                true,
                Appearance::from_mode(iced::theme::Mode::None)
            ),
            "light"
        );
        assert_eq!(
            resolve_pref("default", Some("default"), true, Appearance::Dark),
            "dark"
        );
        assert_eq!(
            Appearance::from_mode(iced::theme::Mode::Light),
            Appearance::Light
        );
        assert_eq!(
            Appearance::from_mode(iced::theme::Mode::Dark),
            Appearance::Dark
        );
        assert_eq!(
            Appearance::from_mode(iced::theme::Mode::None),
            Appearance::Light
        );
        assert_eq!(FAMILIES[0].id, "default");
    }

    #[test]
    fn light_selection_keeps_dark_ink() {
        let t = named("light").tokens;
        assert!(relative_luma(t.selection_text) < 0.5);
        assert!(relative_luma(t.selection) > 0.7);
        let sun = named("solarized-light").tokens;
        assert!(relative_luma(sun.text) < 0.5);
        assert!((relative_luma(sun.selection) - relative_luma(sun.text)).abs() > 0.05);
    }

    #[test]
    fn os_chrome_fills_only_set_fields_when_follow_os() {
        let tok = named("dark").tokens;
        let accent = Color::from_rgb8(0, 122, 255);
        let canvas = Color::from_rgb8(40, 40, 40);
        let surface = Color::from_rgb8(50, 50, 50);
        let panel = Color::from_rgb8(60, 60, 60);
        let text = Color::from_rgb8(240, 240, 240);
        let muted = Color::from_rgb8(160, 160, 160);
        let border = Color::from_rgb8(90, 90, 90);
        let chrome = OsChrome {
            primary: Some(accent),
            canvas: Some(canvas),
            surface: Some(surface),
            panel: Some(panel),
            text: Some(text),
            muted: Some(muted),
            border: Some(border),
        };
        assert!(chrome.any());
        assert!(!OsChrome::empty().any());
        let on = apply_os_chrome(tok, true, chrome);
        assert_eq!(on.primary, accent);
        assert_eq!(on.canvas, canvas);
        assert_eq!(on.surface, surface);
        assert_eq!(on.panel, panel);
        assert_eq!(on.text, text);
        assert_eq!(on.selection_text, text);
        assert_eq!(on.muted, muted);
        assert_eq!(on.border, border);
        assert_ne!(on.selection, tok.selection);
        // Partial chrome leaves unset fields on the colorway.
        let partial = OsChrome {
            primary: Some(accent),
            canvas: Some(canvas),
            text: Some(text),
            ..OsChrome::empty()
        };
        let part = apply_os_chrome(tok, true, partial);
        assert_eq!(part.muted, tok.muted);
        assert_eq!(part.panel, tok.panel);
        assert_eq!(apply_os_chrome(tok, false, chrome), tok);
        assert_eq!(
            apply_os_chrome(tok, true, OsChrome::empty()).primary,
            tok.primary
        );
        let _ = listen_os_chrome();
        // Snapshot is safe off the main thread (returns empty fields if host panics).
        let _ = apply_os_chrome(tok, true, os_chrome());
    }

    #[test]
    fn catalog_body_and_mute_follow_auto_ink() {
        for name in [
            "solarized-dark",
            "solarized-light",
            "nord",
            "gruvbox",
            "tokyo-night",
            "dark",
            "light",
        ] {
            let t = named(name).tokens;
            assert_eq!(t.text, auto_ink(t.canvas, 0.87), "{name} text");
            assert_eq!(t.muted, auto_ink(t.canvas, 0.60), "{name} muted");
            assert_ne!(t.text, t.muted, "{name}");
        }
        let solar = named("solarized-dark").tokens;
        let raw_fg = Color::from_rgb8(0x83, 0x94, 0x96);
        assert_ne!(solar.text, raw_fg);
        assert!(relative_luma(solar.text) > relative_luma(raw_fg));
    }

    #[test]
    fn catalog_json_skips_bad_records_and_hex() {
        assert!(parse_hex("12").is_none());
        assert!(parse_hex("zzzzzz").is_none());
        assert_eq!(parse_hex("#4ebf71"), Some(rgb(0x4E, 0xBF, 0x71)));
        let skip = catalog_from_json(r#"{"x": 1}"#);
        assert!(!skip.contains_key("x"));
        assert!(skip.contains_key("high-contrast"));
        let empty = catalog_from_json("not-json");
        assert_eq!(empty.len(), 1);
        assert!(empty.contains_key("high-contrast"));
        let fallback = hex_of(
            &serde_json::json!({"canvas": "nope"}),
            "canvas",
            rgb(0x01, 0x02, 0x03),
        );
        assert_eq!(fallback, rgb(0x01, 0x02, 0x03));
    }

    #[test]
    fn faces_derive_from_tokens() {
        let tok = named("dark").tokens;
        let s = tok.scheme();
        let faces = tok.faces();
        assert_eq!(faces.link, s.primary);
        assert_eq!(faces.hover, hover_fill(tok));
        assert_eq!(faces.pressed, pressed_fill(tok));
        assert_eq!(faces.chip, chip_fill(tok));
        assert_eq!(faces.selection, selection_fill(tok));
        assert_eq!(faces.input_selection, s.secondary_container);
        assert_eq!(faces.focus, s.primary);
        assert_eq!(faces.scrollbar, s.outline);
        assert_eq!(text_on(tok, tok.canvas), tok.text);
        let red = Color::from_rgb8(200, 40, 40);
        assert!(relative_luma(lighten(red, 0.4)) > relative_luma(red));
        assert!(relative_luma(darken(red, 0.4)) < relative_luma(red));
        let light = named("light").tokens;
        assert_eq!(text_on(light, light.primary), light.canvas);
    }
}
