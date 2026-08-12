//! Semantic color tokens, mixing rules, and the named community catalog.
//!
//! `named` picks a colorway. `mix` builds washes.
//!
//! ```
//! let dark = icedtea::theme::named("dark");
//! assert_eq!(dark.name, "dark");
//! let mixed = icedtea::theme::mix(dark.tokens.primary, dark.tokens.canvas, 0.28);
//! assert_eq!(mixed, dark.tokens.selection);
//! ```

use std::collections::BTreeMap;
use std::sync::OnceLock;

use iced::Color;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Semantic colors used by every styled widget.
///
/// ```
/// let dark = icedtea::theme::named("dark");
/// assert_eq!(dark.name, "dark");
/// assert!(dark.tokens.canvas.r < 0.2);
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
    mix(tok.text, tok.canvas, 0.08)
}

/// Pressed wash over canvas.
pub fn pressed_fill(tok: Tokens) -> Color {
    mix(tok.text, tok.canvas, 0.14)
}

/// Chip / quiet fill.
pub fn chip_fill(tok: Tokens) -> Color {
    mix(tok.text, tok.canvas, 0.10)
}

/// Primary wash used for selected rows.
pub fn selection_fill(tok: Tokens) -> Color {
    mix(tok.primary, tok.canvas, 0.28)
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
    }
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
        },
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
            let text = hex_of(&rec, "text", rgb(0xF2, 0xF2, 0xF2));
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
                        hex_of(&rec, "muted", mix(text, canvas, 0.55)),
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
    catalog().get(key).copied()
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

impl Tokens {
    /// True when the canvas is a dark background.
    pub fn canvas_is_dark(self) -> bool {
        relative_luma(self.canvas) < 0.45
    }

    /// Derived washes and chrome colors. Named colorways stay the input.
    ///
    /// ```
    /// let tok = icedtea::theme::named("dark").tokens;
    /// let faces = tok.faces();
    /// assert_eq!(faces.link, tok.accent);
    /// assert_eq!(faces.hover, icedtea::theme::hover_fill(tok));
    /// ```
    pub fn faces(self) -> Faces {
        Faces {
            hover: hover_fill(self),
            pressed: pressed_fill(self),
            chip: chip_fill(self),
            selection: selection_fill(self),
            text_on_canvas: text_on(self, self.canvas),
            text_on_surface: text_on(self, self.surface),
            text_on_panel: text_on(self, self.panel),
            text_on_primary: text_on(self, self.primary),
            scrollbar: mix(self.text, self.canvas, 0.35),
            input_cursor: self.primary,
            input_selection: self.selection,
            link: self.accent,
            focus: self.primary,
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

/// When `follow_os` is on, fill [`Tokens::primary`] from the desktop
/// accent. Canvas and text stay. Decorated windows keep the native
/// title bar.
///
/// ```
/// use iced::Color;
/// let tok = icedtea::theme::named("dark").tokens;
/// let accent = Color::from_rgb8(0, 122, 255);
/// let out = icedtea::theme::apply_os_accent(tok, true, Some(accent));
/// assert_eq!(out.primary, accent);
/// assert_eq!(out.canvas, tok.canvas);
/// assert_eq!(out.text, tok.text);
/// ```
pub fn apply_os_accent(tokens: Tokens, follow_os: bool, os_accent: Option<Color>) -> Tokens {
    match (follow_os, os_accent) {
        (true, Some(accent)) => {
            let mut tokens = tokens;
            tokens.primary = accent;
            tokens.selection = selection_fill(tokens);
            tokens
        }
        _ => tokens,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(sel, mix(t.primary, t.canvas, 0.28));
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
        assert_eq!(t.selection_text, t.text);
        assert!(t.selection.r < 1.0);
        let sun = named("solarized-light").tokens;
        assert_eq!(sun.selection_text, sun.text);
        assert!((relative_luma(sun.selection) - relative_luma(sun.text)).abs() > 0.15);
    }

    #[test]
    fn os_accent_fills_primary_when_follow_os() {
        let tok = named("dark").tokens;
        let accent = Color::from_rgb8(0, 122, 255);
        let on = apply_os_accent(tok, true, Some(accent));
        assert_eq!(on.primary, accent);
        assert_eq!(on.canvas, tok.canvas);
        assert_eq!(on.text, tok.text);
        assert_ne!(on.selection, tok.selection);
        assert_eq!(
            apply_os_accent(tok, false, Some(accent)).primary,
            tok.primary
        );
        assert_eq!(apply_os_accent(tok, true, None).primary, tok.primary);
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
        let faces = tok.faces();
        assert_eq!(faces.link, tok.accent);
        assert_eq!(faces.hover, hover_fill(tok));
        assert_eq!(faces.pressed, pressed_fill(tok));
        assert_eq!(faces.chip, chip_fill(tok));
        assert_eq!(faces.selection, selection_fill(tok));
        assert_eq!(faces.input_selection, tok.selection);
        assert_eq!(faces.focus, tok.primary);
        assert_eq!(text_on(tok, tok.canvas), tok.text);
        let red = Color::from_rgb8(200, 40, 40);
        assert!(relative_luma(lighten(red, 0.4)) > relative_luma(red));
        assert!(relative_luma(darken(red, 0.4)) < relative_luma(red));
        let light = named("light").tokens;
        assert_eq!(text_on(light, light.primary), light.canvas);
    }
}
