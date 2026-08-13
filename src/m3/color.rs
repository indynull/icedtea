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
        ControlState::Disabled => (layer_on(surface, on, 0.12), layer_on(surface, on, 0.38)),
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

impl Scheme {
    pub fn is_dark(self) -> bool {
        relative_luma(self.surface) < 0.5
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
        let layered = layer_on(s.primary, s.on_primary, 0.08);
        assert_ne!(layered, s.primary);
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
}
