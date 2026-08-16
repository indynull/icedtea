//! M3 color roles (scheme).

use iced::Color;

/// Full Material Design 3 color scheme for one appearance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scheme {
    pub primary: Color,
    pub on_primary: Color,
    pub primary_container: Color,
    pub on_primary_container: Color,
    pub secondary: Color,
    pub on_secondary: Color,
    pub secondary_container: Color,
    pub on_secondary_container: Color,
    pub tertiary: Color,
    pub on_tertiary: Color,
    pub tertiary_container: Color,
    pub on_tertiary_container: Color,
    pub error: Color,
    pub on_error: Color,
    pub error_container: Color,
    pub on_error_container: Color,
    pub surface: Color,
    pub on_surface: Color,
    pub on_surface_variant: Color,
    pub surface_variant: Color,
    pub surface_container_lowest: Color,
    pub surface_container_low: Color,
    pub surface_container: Color,
    pub surface_container_high: Color,
    pub surface_container_highest: Color,
    pub outline: Color,
    pub outline_variant: Color,
    pub inverse_surface: Color,
    pub inverse_on_surface: Color,
    pub inverse_primary: Color,
    pub scrim: Color,
    pub shadow: Color,
    /// Desktop extension (not a core M3 role).
    pub success: Color,
    pub on_success: Color,
    /// Desktop extension (not a core M3 role).
    pub warning: Color,
    pub on_warning: Color,
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

/// Material 3 baseline light scheme.
pub fn scheme_light() -> Scheme {
    Scheme {
        primary: rgb(0x67, 0x50, 0xA4),
        on_primary: rgb(0xFF, 0xFF, 0xFF),
        primary_container: rgb(0xEA, 0xDD, 0xFF),
        on_primary_container: rgb(0x21, 0x00, 0x5D),
        secondary: rgb(0x62, 0x5B, 0x71),
        on_secondary: rgb(0xFF, 0xFF, 0xFF),
        secondary_container: rgb(0xE8, 0xDE, 0xF8),
        on_secondary_container: rgb(0x1D, 0x19, 0x2B),
        tertiary: rgb(0x7D, 0x52, 0x60),
        on_tertiary: rgb(0xFF, 0xFF, 0xFF),
        tertiary_container: rgb(0xFF, 0xD8, 0xE4),
        on_tertiary_container: rgb(0x31, 0x11, 0x1D),
        error: rgb(0xB3, 0x26, 0x1E),
        on_error: rgb(0xFF, 0xFF, 0xFF),
        error_container: rgb(0xF9, 0xDE, 0xDC),
        on_error_container: rgb(0x41, 0x0E, 0x0B),
        surface: rgb(0xFE, 0xF7, 0xFF),
        on_surface: rgb(0x1D, 0x1B, 0x20),
        on_surface_variant: rgb(0x49, 0x45, 0x4F),
        surface_variant: rgb(0xE7, 0xE0, 0xEC),
        surface_container_lowest: rgb(0xFF, 0xFF, 0xFF),
        surface_container_low: rgb(0xF7, 0xF2, 0xFA),
        surface_container: rgb(0xF3, 0xED, 0xF7),
        surface_container_high: rgb(0xEC, 0xE6, 0xF0),
        surface_container_highest: rgb(0xE6, 0xE0, 0xE9),
        outline: rgb(0x79, 0x74, 0x7E),
        outline_variant: rgb(0xCA, 0xC4, 0xD0),
        inverse_surface: rgb(0x32, 0x2F, 0x35),
        inverse_on_surface: rgb(0xF5, 0xEF, 0xF7),
        inverse_primary: rgb(0xD0, 0xBC, 0xFF),
        scrim: rgb(0x00, 0x00, 0x00),
        shadow: rgb(0x00, 0x00, 0x00),
        success: rgb(0x38, 0x6A, 0x20),
        on_success: rgb(0xFF, 0xFF, 0xFF),
        warning: rgb(0x7D, 0x57, 0x00),
        on_warning: rgb(0xFF, 0xFF, 0xFF),
    }
}

/// Material 3 baseline dark scheme.
pub fn scheme_dark() -> Scheme {
    Scheme {
        primary: rgb(0xD0, 0xBC, 0xFF),
        on_primary: rgb(0x38, 0x1E, 0x72),
        primary_container: rgb(0x4F, 0x37, 0x8B),
        on_primary_container: rgb(0xEA, 0xDD, 0xFF),
        secondary: rgb(0xCC, 0xC2, 0xDC),
        on_secondary: rgb(0x33, 0x2D, 0x41),
        secondary_container: rgb(0x4A, 0x44, 0x58),
        on_secondary_container: rgb(0xE8, 0xDE, 0xF8),
        tertiary: rgb(0xEF, 0xB8, 0xC8),
        on_tertiary: rgb(0x49, 0x25, 0x32),
        tertiary_container: rgb(0x63, 0x3B, 0x48),
        on_tertiary_container: rgb(0xFF, 0xD8, 0xE4),
        error: rgb(0xF2, 0xB8, 0xB5),
        on_error: rgb(0x60, 0x14, 0x10),
        error_container: rgb(0x8C, 0x1D, 0x18),
        on_error_container: rgb(0xF9, 0xDE, 0xDC),
        surface: rgb(0x14, 0x12, 0x18),
        on_surface: rgb(0xE6, 0xE0, 0xE9),
        on_surface_variant: rgb(0xCA, 0xC4, 0xD0),
        surface_variant: rgb(0x49, 0x45, 0x4F),
        surface_container_lowest: rgb(0x0F, 0x0D, 0x13),
        surface_container_low: rgb(0x1D, 0x1B, 0x20),
        surface_container: rgb(0x21, 0x1F, 0x26),
        surface_container_high: rgb(0x2B, 0x29, 0x30),
        surface_container_highest: rgb(0x36, 0x34, 0x3B),
        outline: rgb(0x93, 0x8F, 0x99),
        outline_variant: rgb(0x49, 0x45, 0x4F),
        inverse_surface: rgb(0xE6, 0xE0, 0xE9),
        inverse_on_surface: rgb(0x32, 0x2F, 0x35),
        inverse_primary: rgb(0x67, 0x50, 0xA4),
        scrim: rgb(0x00, 0x00, 0x00),
        shadow: rgb(0x00, 0x00, 0x00),
        success: rgb(0xA6, 0xD2, 0x89),
        on_success: rgb(0x14, 0x38, 0x00),
        warning: rgb(0xFF, 0xB9, 0x50),
        on_warning: rgb(0x42, 0x2C, 0x00),
    }
}

pub fn mix(fg: Color, bg: Color, amount: f32) -> Color {
    let t = amount.clamp(0.0, 1.0);
    Color::from_rgb(
        fg.r * t + bg.r * (1.0 - t),
        fg.g * t + bg.g * (1.0 - t),
        fg.b * t + bg.b * (1.0 - t),
    )
}

pub fn state_hover(scheme: Scheme) -> Color {
    layer_on(scheme.surface, scheme.on_surface, 0.08)
}

pub fn state_pressed(scheme: Scheme) -> Color {
    layer_on(scheme.surface, scheme.on_surface, 0.12)
}

pub fn state_selected(scheme: Scheme) -> Color {
    layer_on(scheme.surface, scheme.primary, 0.12)
}

/// Composite a state layer of `on` over `base` at the given opacity (0..=1).
pub fn layer_on(base: Color, on: Color, opacity: f32) -> Color {
    mix(on, base, opacity.clamp(0.0, 1.0))
}

/// M3 state layer over a container color, using content ink for the layer.
pub fn face(
    base: Color,
    on: Color,
    surface: Color,
    state: super::state::ControlState,
) -> (Color, Color) {
    use super::state::ControlState;
    match state {
        // Filled pairs use on-primary / on-error (dark ink on a bright
        // fill). Layering that ink on the canvas is a near-black brick
        // on dark. Disabled uses muted on-surface: light on dark canvas,
        // dark on light.
        ControlState::Disabled => {
            let ink = if relative_luma(surface) < 0.5 {
                Color::WHITE
            } else {
                Color::BLACK
            };
            // 0.12 / 0.38 is the M3 mute, but 0.38 on 0.12 is muddy on
            // dark surface-container. Stronger fill and ink keep the
            // label readable on the brick, not only on the canvas.
            (layer_on(surface, ink, 0.22), layer_on(surface, ink, 0.68))
        }
        ControlState::Hovered => (layer_on(base, on, 0.08), on),
        ControlState::Focused => (layer_on(base, on, 0.10), on),
        ControlState::Pressed => (layer_on(base, on, 0.12), on),
        ControlState::Selected => (layer_on(base, on, 0.12), on),
        ControlState::Error => (base, on),
        ControlState::Enabled => (base, on),
    }
}

pub fn relative_luma(c: Color) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

fn srgb_lin(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn wcag_luma(c: Color) -> f32 {
    0.2126 * srgb_lin(c.r) + 0.7152 * srgb_lin(c.g) + 0.0722 * srgb_lin(c.b)
}

/// WCAG 2 contrast ratio of two sRGB colors.
pub fn contrast_ratio(a: Color, b: Color) -> f32 {
    let (l1, l2) = (wcag_luma(a), wcag_luma(b));
    let (hi, lo) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

/// Mix `ink` toward black or white until it holds 4.5:1 on `canvas`.
pub fn ink_on(ink: Color, canvas: Color) -> Color {
    if contrast_ratio(ink, canvas) >= 4.5 {
        return ink;
    }
    let toward = if relative_luma(canvas) < 0.45 {
        Color::WHITE
    } else {
        Color::BLACK
    };
    let mut lo = 0.0f32;
    let mut hi = 1.0f32;
    let mut best = toward;
    for _ in 0..12 {
        let mid = (lo + hi) * 0.5;
        let candidate = mix(toward, ink, mid);
        if contrast_ratio(candidate, canvas) >= 4.5 {
            best = candidate;
            hi = mid;
        } else {
            lo = mid;
        }
    }
    best
}

fn mul_a(c: Color, t: f32) -> Color {
    Color { a: c.a * t, ..c }
}

impl Scheme {
    pub fn is_dark(self) -> bool {
        relative_luma(self.surface) < 0.5
    }

    /// Multiply every role's alpha by `amount` (0..=1).
    pub fn fade(self, amount: f32) -> Self {
        let t = amount.clamp(0.0, 1.0);
        Self {
            primary: mul_a(self.primary, t),
            on_primary: mul_a(self.on_primary, t),
            primary_container: mul_a(self.primary_container, t),
            on_primary_container: mul_a(self.on_primary_container, t),
            secondary: mul_a(self.secondary, t),
            on_secondary: mul_a(self.on_secondary, t),
            secondary_container: mul_a(self.secondary_container, t),
            on_secondary_container: mul_a(self.on_secondary_container, t),
            tertiary: mul_a(self.tertiary, t),
            on_tertiary: mul_a(self.on_tertiary, t),
            tertiary_container: mul_a(self.tertiary_container, t),
            on_tertiary_container: mul_a(self.on_tertiary_container, t),
            error: mul_a(self.error, t),
            on_error: mul_a(self.on_error, t),
            error_container: mul_a(self.error_container, t),
            on_error_container: mul_a(self.on_error_container, t),
            surface: mul_a(self.surface, t),
            on_surface: mul_a(self.on_surface, t),
            on_surface_variant: mul_a(self.on_surface_variant, t),
            surface_variant: mul_a(self.surface_variant, t),
            surface_container_lowest: mul_a(self.surface_container_lowest, t),
            surface_container_low: mul_a(self.surface_container_low, t),
            surface_container: mul_a(self.surface_container, t),
            surface_container_high: mul_a(self.surface_container_high, t),
            surface_container_highest: mul_a(self.surface_container_highest, t),
            outline: mul_a(self.outline, t),
            outline_variant: mul_a(self.outline_variant, t),
            inverse_surface: mul_a(self.inverse_surface, t),
            inverse_on_surface: mul_a(self.inverse_on_surface, t),
            inverse_primary: mul_a(self.inverse_primary, t),
            scrim: mul_a(self.scrim, t),
            shadow: mul_a(self.shadow, t),
            success: mul_a(self.success, t),
            on_success: mul_a(self.on_success, t),
            warning: mul_a(self.warning, t),
            on_warning: mul_a(self.on_warning, t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_and_dark_baseline_roles_differ() {
        let l = scheme_light();
        let d = scheme_dark();
        assert!(relative_luma(l.surface) > relative_luma(d.surface));
        assert_ne!(l.primary, d.primary);
        assert!(!l.is_dark());
        assert!(d.is_dark());
    }

    #[test]
    fn fade_scales_every_role_alpha() {
        let s = scheme_light();
        let mid = s.fade(0.5);
        assert!((mid.surface.a - 0.5).abs() < 1e-5);
        assert!((mid.on_surface.a - 0.5).abs() < 1e-5);
        assert!((mid.primary.a - 0.5).abs() < 1e-5);
        assert!((mid.primary_container.a - 0.5).abs() < 1e-5);
        assert_eq!(s.fade(1.0).surface.a, s.surface.a);
        assert_eq!(s.fade(0.0).error.a, 0.0);
    }

    #[test]
    fn state_layers_darken_or_shift_base() {
        use crate::m3::ControlState;
        let s = scheme_light();
        let hover = state_hover(s);
        let pressed = state_pressed(s);
        assert_ne!(hover, s.surface);
        assert_ne!(pressed, s.surface);
        let (bg, fg) = face(s.primary, s.on_primary, s.surface, ControlState::Enabled);
        assert_eq!(bg, s.primary);
        assert_eq!(fg, s.on_primary);
        let (dbg, dfg) = face(s.primary, s.on_primary, s.surface, ControlState::Disabled);
        assert_ne!(dbg, s.primary);
        assert_ne!(dfg, s.on_primary);
        let _ = face(s.primary, s.on_primary, s.surface, ControlState::Focused);
        let _ = face(s.primary, s.on_primary, s.surface, ControlState::Selected);
        let _ = face(s.primary, s.on_primary, s.surface, ControlState::Error);
        let layered = layer_on(s.primary, s.on_primary, 0.08);
        assert_ne!(layered, s.primary);
    }

    #[test]
    fn disabled_filled_ink_contrasts_with_canvas() {
        use crate::m3::ControlState;
        for name in ["dark", "light"] {
            let s = crate::theme::named(name).tokens.scheme();
            let (bg, fg) = face(s.primary, s.on_primary, s.surface, ControlState::Disabled);
            let canvas = relative_luma(s.surface);
            assert!(
                (relative_luma(fg) - canvas).abs() > 0.15,
                "{name}: disabled ink {fg:?} vanishes on {canvas}"
            );
            assert_ne!(bg, s.surface, "{name}: disabled fill equals canvas");
            let danger = face(s.error, s.on_error, s.surface, ControlState::Disabled);
            assert!(
                (relative_luma(danger.1) - canvas).abs() > 0.15,
                "{name}: disabled danger ink vanishes"
            );
        }
    }

    #[test]
    fn mix_and_state_layers_differ_from_surface() {
        let s = scheme_light();
        let m = mix(Color::WHITE, Color::BLACK, 0.5);
        assert!((m.r - 0.5).abs() < 0.01);
        let h = state_hover(s);
        assert_ne!(h, s.surface);
        assert_ne!(state_selected(s), s.surface);
    }

    #[test]
    fn ink_on_lifts_a_weak_role_off_its_wash() {
        let cream = Color::from_rgb8(0xFB, 0xF1, 0xC7);
        let olive = Color::from_rgb8(0x98, 0x97, 0x1A);
        assert!(contrast_ratio(olive, cream) < 4.5);
        assert!(contrast_ratio(ink_on(olive, cream), cream) >= 4.5);
        assert_eq!(ink_on(Color::BLACK, Color::WHITE), Color::BLACK);
    }
}
