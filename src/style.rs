//! iced style closures painted from M3 roles and control states.

use iced::border::Border;
use iced::overlay::menu as overlay_menu;
use iced::widget::{
    button, checkbox, container, pick_list, progress_bar, radio, rule, scrollable, slider,
    text_input, toggler,
};
use iced::{Background, Color, Shadow};

use crate::chrome::Elevation;
use crate::m3::color::{face, layer_on};
use crate::m3::shape::Component;
use crate::m3::ControlState;
use crate::theme::{hover_fill, Tokens};
use crate::variant::Variant;

fn component_radius(tok: Tokens, c: Component) -> iced::border::Radius {
    tok.radius(c)
}

fn button_status(status: button::Status) -> ControlState {
    match status {
        button::Status::Active => ControlState::Enabled,
        button::Status::Hovered => ControlState::Hovered,
        button::Status::Pressed => ControlState::Pressed,
        button::Status::Disabled => ControlState::Disabled,
    }
}

/// Solid fill container.
pub fn fill(bg: Color, fg: Color) -> container::Style {
    container::Style {
        background: Some(Background::Color(bg)),
        text_color: Some(fg),
        snap: false,
        ..container::Style::default()
    }
}

/// M3 filled / elevated card (surface container; desktop Component radius).
pub fn card(tok: Tokens, focus: bool) -> container::Style {
    let s = tok.scheme();
    container::Style {
        background: Some(Background::Color(s.surface_container_low)),
        text_color: Some(s.on_surface),
        border: Border {
            color: if focus { s.primary } else { s.outline_variant },
            width: if focus { 2.0 } else { 1.0 },
            radius: component_radius(tok, Component::Card),
        },
        shadow: tok.shadow(Elevation::Level1),
        snap: false,
    }
}

/// Outline-only card (transparent fill).
pub fn outlined_card(tok: Tokens) -> container::Style {
    let s = tok.scheme();
    container::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: Some(s.on_surface),
        border: Border {
            color: s.outline,
            width: 1.0,
            radius: component_radius(tok, Component::Card),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

/// M3 elevated card (level 2 surface + shadow).
pub fn raised_card(tok: Tokens) -> container::Style {
    let s = tok.scheme();
    container::Style {
        background: Some(Background::Color(Elevation::Level2.surface(s))),
        text_color: Some(s.on_surface),
        border: Border {
            color: s.outline_variant,
            width: 0.0,
            radius: component_radius(tok, Component::Card),
        },
        shadow: tok.shadow(Elevation::Level2),
        snap: false,
    }
}

pub fn shell(tok: Tokens) -> container::Style {
    let s = tok.scheme();
    container::Style {
        background: Some(Background::Color(s.surface)),
        text_color: Some(s.on_surface),
        border: Border {
            color: s.outline,
            width: 1.0,
            radius: component_radius(tok, Component::Shell),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn panel(tok: Tokens) -> container::Style {
    let s = tok.scheme();
    fill(s.surface_container_high, s.on_surface)
}

pub fn footer(tok: Tokens) -> container::Style {
    let s = tok.scheme();
    fill(s.surface_container_high, s.on_surface_variant)
}

pub fn hairline(tok: Tokens) -> container::Style {
    let s = tok.scheme();
    container::Style {
        background: Some(Background::Color(s.outline_variant)),
        snap: false,
        ..container::Style::default()
    }
}

pub fn dim_backdrop(tok: Tokens) -> container::Style {
    dim_backdrop_at(tok, 1.0)
}

/// Multiply a container face alpha by `progress` (0..=1).
pub fn fade_face(mut face: container::Style, progress: f32) -> container::Style {
    let t = progress.clamp(0.0, 1.0);
    if let Some(Background::Color(c)) = &mut face.background {
        c.a *= t;
    }
    if let Some(c) = &mut face.text_color {
        c.a *= t;
    }
    face.border.color.a *= t;
    face.shadow.color.a *= t;
    face
}

/// Scrim whose opacity follows overlay progress (0 = clear, 1 = rest).
pub fn dim_backdrop_at(tok: Tokens, progress: f32) -> container::Style {
    let t = progress.clamp(0.0, 1.0);
    let scrim = tok.scheme().scrim;
    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            scrim.r,
            scrim.g,
            scrim.b,
            0.64 * t,
        ))),
        snap: false,
        ..container::Style::default()
    }
}

/// M3 list row: selected uses secondary container; idle is transparent.
pub fn list_row(tok: Tokens, selected: bool) -> container::Style {
    let s = tok.scheme();
    if selected {
        fill(s.secondary_container, s.on_secondary_container)
    } else {
        fill(Color::TRANSPARENT, s.on_surface)
    }
}

/// Hovered list row (state layer over surface).
pub fn list_row_hover(tok: Tokens, selected: bool) -> container::Style {
    let s = tok.scheme();
    if selected {
        let bg = layer_on(s.secondary_container, s.on_secondary_container, 0.08);
        fill(bg, s.on_secondary_container)
    } else {
        fill(hover_fill(tok), s.on_surface)
    }
}

/// Data-table cell faces (M3 list/data-table selection, not zebra-as-selection).
///
/// - **Selected row:** secondary container across the whole row (one wash).
/// - **Focused cell:** 2dp primary outline on that wash (or primary container
///   when the row is not selected).
/// - **Zebra:** quiet `surface_container_high` — never secondary container.
pub fn table_cell(tok: Tokens, selected: bool, focused: bool, zebra: bool) -> container::Style {
    let s = tok.scheme();
    let mut st = if selected {
        fill(s.secondary_container, s.on_secondary_container)
    } else if zebra {
        fill(s.surface_container_high, s.on_surface)
    } else {
        fill(Color::TRANSPARENT, s.on_surface)
    };
    if focused {
        if selected {
            st.border = Border {
                color: s.primary,
                width: 2.0,
                radius: 0.0.into(),
            };
        } else {
            st = fill(s.primary_container, s.on_primary_container);
            st.border = Border {
                color: s.primary,
                width: 2.0,
                radius: 0.0.into(),
            };
        }
    }
    st
}

/// Page banner: callout wash, banner family corners (flush under Material).
pub fn banner(tok: Tokens) -> container::Style {
    let mut st = callout(tok, crate::toast::ToastKind::Info);
    st.border.radius = component_radius(tok, Component::Banner);
    st
}

/// Hover tip (M3 tooltip): raised card fill, tooltip family corners.
pub fn tooltip(tok: Tokens) -> container::Style {
    let mut st = raised_card(tok);
    st.border.radius = component_radius(tok, Component::Tooltip);
    st
}

pub fn callout(tok: Tokens, kind: crate::toast::ToastKind) -> container::Style {
    let s = tok.scheme();
    let (bg, fg, border) = match kind {
        crate::toast::ToastKind::Info => (s.primary_container, s.on_primary_container, s.primary),
        crate::toast::ToastKind::Success => (
            layer_on(s.surface, s.success, 0.16),
            s.on_surface,
            s.success,
        ),
        crate::toast::ToastKind::Warning => (
            layer_on(s.surface, s.warning, 0.16),
            s.on_surface,
            s.warning,
        ),
        crate::toast::ToastKind::Danger => (s.error_container, s.on_error_container, s.error),
    };
    container::Style {
        background: Some(Background::Color(bg)),
        text_color: Some(fg),
        border: Border {
            color: border,
            width: 1.0,
            radius: component_radius(tok, Component::Toast),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn skeleton(tok: Tokens) -> container::Style {
    let s = tok.scheme();
    fill(s.surface_container_highest, s.on_surface_variant)
}

/// Unfilled disabled ink. Same 68% mute as filled `face`, so Ghost /
/// Outlined labels stay readable on dark canvas.
fn disabled_ink(surface: Color) -> Color {
    let ink = if crate::m3::color::relative_luma(surface) < 0.5 {
        Color::WHITE
    } else {
        Color::BLACK
    };
    layer_on(surface, ink, 0.68)
}

fn button_border(tok: Tokens, comp: Component) -> Border {
    Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: component_radius(tok, comp),
    }
}

/// Map icedtea [`Variant`] onto M3 button color styles.
///
/// - Primary → filled
/// - Quiet → filled tonal
/// - Ghost → text
/// - Chip → tonal (assist chip container)
/// - Danger → filled error
/// - Success / Warning → filled tonal with desktop roles
fn button_face(
    tok: Tokens,
    variant: Variant,
    state: ControlState,
) -> (Color, Color, Border, Shadow) {
    let s = tok.scheme();
    let surface = s.surface;
    let pill = button_border(tok, Component::Button);
    match variant {
        Variant::Primary => {
            let (bg, fg) = face(s.primary, s.on_primary, surface, state);
            (bg, fg, pill, Shadow::default())
        }
        Variant::Quiet => {
            let (bg, fg) = face(
                s.secondary_container,
                s.on_secondary_container,
                surface,
                state,
            );
            (bg, fg, pill, Shadow::default())
        }
        Variant::Ghost => {
            let (bg, fg) = match state {
                ControlState::Disabled => (Color::TRANSPARENT, disabled_ink(surface)),
                ControlState::Hovered | ControlState::Focused => {
                    (layer_on(surface, s.primary, 0.08), s.primary)
                }
                ControlState::Pressed => (layer_on(surface, s.primary, 0.12), s.primary),
                _ => (Color::TRANSPARENT, s.primary),
            };
            (bg, fg, pill, Shadow::default())
        }
        Variant::Chip => {
            let (bg, fg) = face(
                s.secondary_container,
                s.on_secondary_container,
                surface,
                state,
            );
            (
                bg,
                fg,
                button_border(tok, Component::Chip),
                Shadow::default(),
            )
        }
        Variant::Danger => {
            let (bg, fg) = face(s.error, s.on_error, surface, state);
            (bg, fg, pill, Shadow::default())
        }
        Variant::Success => {
            let (bg, fg) = face(s.success, s.on_success, surface, state);
            (bg, fg, pill, Shadow::default())
        }
        Variant::Warning => {
            let (bg, fg) = face(s.warning, s.on_warning, surface, state);
            (bg, fg, pill, Shadow::default())
        }
        Variant::Outlined => {
            let (bg, fg) = match state {
                ControlState::Disabled => (Color::TRANSPARENT, disabled_ink(surface)),
                ControlState::Hovered | ControlState::Focused => {
                    (layer_on(surface, s.primary, 0.08), s.primary)
                }
                ControlState::Pressed => (layer_on(surface, s.primary, 0.12), s.primary),
                _ => (Color::TRANSPARENT, s.primary),
            };
            (
                bg,
                fg,
                Border {
                    color: if matches!(state, ControlState::Disabled) {
                        s.outline.scale_alpha(0.38)
                    } else {
                        s.outline
                    },
                    width: 1.0,
                    radius: pill.radius,
                },
                Shadow::default(),
            )
        }
        Variant::Elevated => {
            let (bg, fg) = face(s.surface_container_high, s.primary, surface, state);
            let level = crate::m3::Elevation::Level1;
            let level = if matches!(
                state,
                ControlState::Hovered | ControlState::Pressed | ControlState::Focused
            ) {
                level.raise()
            } else {
                level
            };
            (bg, fg, pill, tok.shadow(level))
        }
    }
}

pub fn button_style(
    tok: Tokens,
    variant: Variant,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let state = button_status(status);
        let (bg, fg, border, shadow) = button_face(tok, variant, state);
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border,
            shadow,
            snap: false,
        }
    }
}

/// M3 secondary tab *label* only — flush; active indicator is
/// [`crate::widget::tab_bar`]'s underbar, not a full iced border.
pub fn tab_style(
    tok: Tokens,
    active: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let s = tok.scheme();
        let state = button_status(status);
        let bg = match (active, state) {
            (true, _) => Some(Background::Color(Color::TRANSPARENT)),
            (false, ControlState::Hovered | ControlState::Pressed) => {
                Some(Background::Color(hover_fill(tok)))
            }
            _ => Some(Background::Color(Color::TRANSPARENT)),
        };
        button::Style {
            background: bg,
            text_color: if active {
                s.primary
            } else {
                s.on_surface_variant
            },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: component_radius(tok, Component::Tab),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

/// Selected command-palette row (wash, not a filled button).
pub fn palette_hit(
    tok: Tokens,
    selected: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let s = tok.scheme();
        let state = button_status(status);
        let bg = if selected {
            s.secondary_container
        } else {
            match state {
                ControlState::Hovered | ControlState::Pressed => hover_fill(tok),
                _ => Color::TRANSPARENT,
            }
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if selected {
                s.on_secondary_container
            } else {
                s.on_surface
            },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: component_radius(tok, Component::Menu),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

/// List, menu, and disclosure row. Menu family; not a Button stadium.
pub fn menu_item_style(tok: Tokens) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (bg, fg, mut border, shadow) = button_face(tok, Variant::Ghost, button_status(status));
        border.radius = component_radius(tok, Component::Menu);
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border,
            shadow,
            snap: false,
        }
    }
}

/// Exclusive in-pane segment. Flush corners; selected wash, not a stadium.
pub fn segment_style(
    tok: Tokens,
    selected: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let variant = if selected {
            Variant::Primary
        } else {
            Variant::Quiet
        };
        let (bg, fg, mut border, shadow) = button_face(tok, variant, button_status(status));
        border.radius = component_radius(tok, Component::Segment);
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border,
            shadow,
            snap: false,
        }
    }
}

/// Related action in a button group. Flush cell; the group outline
/// carries the Button family radius.
pub fn joined_button_style(
    tok: Tokens,
    variant: Variant,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (bg, fg, mut border, shadow) = button_face(tok, variant, button_status(status));
        border.radius = component_radius(tok, Component::Segment);
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border,
            shadow,
            snap: false,
        }
    }
}

/// Active tab underbar (3dp primary). Separate from the label button.
pub fn tab_indicator(tok: Tokens) -> container::Style {
    let s = tok.scheme();
    container::Style {
        background: Some(Background::Color(s.primary)),
        snap: false,
        ..container::Style::default()
    }
}

/// M3 dialog / sheet container: elevated surface container, large corners.
pub fn dialog_sheet_face(tok: Tokens) -> container::Style {
    let s = tok.scheme();
    container::Style {
        background: Some(Background::Color(Elevation::Level3.surface(s))),
        text_color: Some(s.on_surface),
        border: Border {
            color: s.outline_variant,
            width: 0.0,
            radius: component_radius(tok, Component::Dialog),
        },
        shadow: tok.shadow(Elevation::Level3),
        snap: false,
    }
}

/// M3 top app bar / menu strip: surface container.
pub fn app_bar(tok: Tokens) -> container::Style {
    let s = tok.scheme();
    container::Style {
        background: Some(Background::Color(s.surface_container)),
        text_color: Some(s.on_surface),
        border: Border {
            color: s.outline_variant,
            width: 0.0,
            radius: component_radius(tok, Component::AppBar),
        },
        shadow: tok.shadow(Elevation::Level0),
        snap: false,
    }
}

/// Navigation rail / drawer surface.
pub fn nav_rail(tok: Tokens, selected: bool) -> container::Style {
    let s = tok.scheme();
    if selected {
        fill(s.secondary_container, s.on_secondary_container)
    } else {
        fill(Color::TRANSPARENT, s.on_surface_variant)
    }
}

/// M3 filled or outlined text field.
pub fn field_style(
    tok: Tokens,
    outlined: bool,
) -> impl Fn(&iced::Theme, text_input::Status) -> text_input::Style {
    field_style_paint(tok, outlined, false)
}

/// Same as [`field_style`]. `hide_value` clears the iced value ink so a
/// field can paint [`crate::widget::FieldRun`]s over the typed text.
pub fn field_style_paint(
    tok: Tokens,
    outlined: bool,
    hide_value: bool,
) -> impl Fn(&iced::Theme, text_input::Status) -> text_input::Style {
    move |_theme, status| {
        let s = tok.scheme();
        let (border_color, width) = match status {
            text_input::Status::Focused { .. } => (s.primary, 2.0),
            text_input::Status::Hovered => (s.on_surface, 1.0),
            text_input::Status::Disabled => (s.on_surface.scale_alpha(0.12), 1.0),
            text_input::Status::Active => (s.outline, 1.0),
        };
        let bg = if outlined {
            Color::TRANSPARENT
        } else {
            match status {
                text_input::Status::Disabled => layer_on(s.surface, s.on_surface, 0.04),
                _ => s.surface_container_highest,
            }
        };
        text_input::Style {
            background: Background::Color(bg),
            border: Border {
                color: border_color,
                width,
                radius: component_radius(tok, Component::Field),
            },
            icon: s.on_surface_variant,
            placeholder: s.on_surface_variant,
            value: if hide_value {
                Color::TRANSPARENT
            } else {
                match status {
                    text_input::Status::Disabled => layer_on(s.surface, s.on_surface, 0.38),
                    _ => s.on_surface,
                }
            },
            selection: s.secondary_container,
        }
    }
}

/// M3 filled text field (surface container highest, outline, primary focus).
pub fn search_style(tok: Tokens) -> impl Fn(&iced::Theme, text_input::Status) -> text_input::Style {
    search_style_paint(tok, false)
}

/// Same as [`search_style`]. `hide_value` clears the iced value ink so a
/// field can paint [`crate::widget::FieldRun`]s over the typed text.
pub fn search_style_paint(
    tok: Tokens,
    hide_value: bool,
) -> impl Fn(&iced::Theme, text_input::Status) -> text_input::Style {
    move |_theme, status| {
        let s = tok.scheme();
        let (border_color, width) = match status {
            text_input::Status::Focused { .. } => (s.primary, 2.0),
            text_input::Status::Hovered => (s.on_surface, 1.0),
            text_input::Status::Disabled => (s.on_surface.scale_alpha(0.12), 1.0),
            text_input::Status::Active => (s.outline, 1.0),
        };
        let bg = match status {
            text_input::Status::Disabled => layer_on(s.surface, s.on_surface, 0.04),
            _ => s.surface_container_highest,
        };
        text_input::Style {
            background: Background::Color(bg),
            border: Border {
                color: border_color,
                width,
                radius: component_radius(tok, Component::Search),
            },
            icon: s.on_surface_variant,
            placeholder: s.on_surface_variant,
            value: if hide_value {
                Color::TRANSPARENT
            } else {
                match status {
                    text_input::Status::Disabled => layer_on(s.surface, s.on_surface, 0.38),
                    _ => s.on_surface,
                }
            },
            selection: s.secondary_container,
        }
    }
}

pub fn picker_style(tok: Tokens) -> impl Fn(&iced::Theme, pick_list::Status) -> pick_list::Style {
    move |_theme, status| {
        let s = tok.scheme();
        let (border_color, width) = match status {
            pick_list::Status::Hovered => (s.on_surface, 1.0),
            pick_list::Status::Active | pick_list::Status::Opened { .. } => (s.outline, 1.0),
        };
        pick_list::Style {
            text_color: s.on_surface,
            placeholder_color: s.on_surface_variant,
            handle_color: s.on_surface_variant,
            background: Background::Color(s.surface_container_highest),
            border: Border {
                color: border_color,
                width,
                radius: component_radius(tok, Component::Field),
            },
        }
    }
}

/// Overlay menu: surface container, raised elevation.
pub fn overlay_menu_style(tok: Tokens) -> impl Fn(&iced::Theme) -> overlay_menu::Style {
    move |_theme| {
        let s = tok.scheme();
        overlay_menu::Style {
            background: Background::Color(s.surface_container),
            border: Border {
                width: 1.0,
                radius: component_radius(tok, Component::Menu),
                color: s.outline_variant,
            },
            text_color: s.on_surface,
            selected_text_color: s.on_secondary_container,
            selected_background: Background::Color(s.secondary_container),
            shadow: tok.shadow(Elevation::Level2),
        }
    }
}

pub fn checkbox_style(tok: Tokens) -> impl Fn(&iced::Theme, checkbox::Status) -> checkbox::Style {
    move |_theme, status| {
        let s = tok.scheme();
        let (checked, disabled, hovered) = match status {
            checkbox::Status::Active { is_checked } => (is_checked, false, false),
            checkbox::Status::Hovered { is_checked } => (is_checked, false, true),
            checkbox::Status::Disabled { is_checked } => (is_checked, true, false),
        };
        let (bg, border_c, icon) = if disabled {
            (
                layer_on(s.surface, s.on_surface, 0.12),
                layer_on(s.surface, s.on_surface, 0.38),
                s.surface,
            )
        } else if checked {
            let base = s.primary;
            let bg = if hovered {
                layer_on(base, s.on_primary, 0.08)
            } else {
                base
            };
            (bg, s.primary, s.on_primary)
        } else {
            let bg = if hovered {
                layer_on(s.surface, s.on_surface, 0.08)
            } else {
                s.surface_container_highest
            };
            (bg, s.outline, s.on_primary)
        };
        checkbox::Style {
            background: Background::Color(bg),
            icon_color: icon,
            border: Border {
                color: border_c,
                width: 2.0,
                radius: component_radius(tok, Component::Checkbox),
            },
            text_color: Some(if disabled {
                layer_on(s.surface, s.on_surface, 0.38)
            } else {
                s.on_surface
            }),
        }
    }
}

pub fn radio_style(tok: Tokens) -> impl Fn(&iced::Theme, radio::Status) -> radio::Style {
    move |_theme, status| {
        let s = tok.scheme();
        let (selected, hovered) = match status {
            radio::Status::Active { is_selected } => (is_selected, false),
            radio::Status::Hovered { is_selected } => (is_selected, true),
        };
        let bg = if hovered {
            layer_on(s.surface, s.primary, 0.08)
        } else {
            s.surface_container_highest
        };
        radio::Style {
            background: Background::Color(bg),
            dot_color: if selected {
                s.primary
            } else {
                Color::TRANSPARENT
            },
            border_width: 2.0,
            border_color: if selected { s.primary } else { s.outline },
            text_color: Some(s.on_surface),
        }
    }
}

pub fn switch_style(tok: Tokens) -> impl Fn(&iced::Theme, toggler::Status) -> toggler::Style {
    move |_theme, status| {
        let s = tok.scheme();
        let (on, disabled, hovered) = match status {
            toggler::Status::Active { is_toggled } => (is_toggled, false, false),
            toggler::Status::Hovered { is_toggled } => (is_toggled, false, true),
            toggler::Status::Disabled { is_toggled } => (is_toggled, true, false),
        };
        let track = if disabled {
            layer_on(s.surface, s.on_surface, 0.12)
        } else if on {
            let base = s.primary;
            if hovered {
                layer_on(base, s.on_primary, 0.08)
            } else {
                base
            }
        } else {
            let base = s.surface_container_highest;
            if hovered {
                layer_on(base, s.on_surface, 0.08)
            } else {
                base
            }
        };
        let thumb = if disabled {
            layer_on(s.surface, s.on_surface, 0.38)
        } else if on {
            s.on_primary
        } else {
            s.outline
        };
        toggler::Style {
            background: Background::Color(track),
            background_border_width: if on { 0.0 } else { 2.0 },
            background_border_color: if on { Color::TRANSPARENT } else { s.outline },
            foreground: Background::Color(thumb),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
            text_color: Some(s.on_surface),
            border_radius: Some(component_radius(tok, Component::Track)),
            padding_ratio: 0.2,
        }
    }
}

pub fn slider_style(tok: Tokens) -> impl Fn(&iced::Theme, slider::Status) -> slider::Style {
    slider_style_rail(tok, false)
}

/// Iced paints the first rail color from physical left to the handle.
/// Horizontal RTL sliders invert the value and swap these so fill
/// comes from start.
pub(crate) fn slider_style_rail(
    tok: Tokens,
    rtl_horizontal: bool,
) -> impl Fn(&iced::Theme, slider::Status) -> slider::Style {
    move |_theme, status| {
        let s = tok.scheme();
        let handle_r = match status {
            slider::Status::Active => 10.0,
            slider::Status::Hovered | slider::Status::Dragged => 12.0,
        };
        let (head, tail) = if rtl_horizontal {
            (s.surface_container_highest, s.primary)
        } else {
            (s.primary, s.surface_container_highest)
        };
        slider::Style {
            rail: slider::Rail {
                backgrounds: (Background::Color(head), Background::Color(tail)),
                width: 4.0,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: component_radius(tok, Component::Track),
                },
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: handle_r },
                background: Background::Color(s.primary),
                border_width: 0.0,
                border_color: s.outline,
            },
        }
    }
}

pub fn progress_style(tok: Tokens) -> impl Fn(&iced::Theme) -> progress_bar::Style {
    move |_theme| {
        let s = tok.scheme();
        progress_bar::Style {
            background: Background::Color(s.surface_container_highest),
            bar: Background::Color(s.primary),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: component_radius(tok, Component::Track),
            },
        }
    }
}

pub fn rule_style(tok: Tokens) -> impl Fn(&iced::Theme) -> rule::Style {
    move |_theme| {
        let s = tok.scheme();
        rule::Style {
            color: s.outline_variant,
            radius: component_radius(tok, Component::Shell),
            fill_mode: rule::FillMode::Full,
            snap: false,
        }
    }
}

pub fn scroll_style(tok: Tokens) -> impl Fn(&iced::Theme, scrollable::Status) -> scrollable::Style {
    move |_theme, _status| {
        let s = tok.scheme();
        let rail = scrollable::Rail {
            background: Some(Background::Color(s.surface_container_low)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: component_radius(tok, Component::Track),
            },
            scroller: scrollable::Scroller {
                background: Background::Color(s.outline),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: component_radius(tok, Component::Track),
                },
            },
        };
        scrollable::Style {
            container: fill(Color::TRANSPARENT, s.on_surface),
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
            auto_scroll: scrollable::AutoScroll {
                background: Background::Color(s.surface_container),
                border: Border {
                    color: s.outline,
                    width: 1.0,
                    radius: component_radius(tok, Component::Field),
                },
                shadow: Shadow::default(),
                icon: s.on_surface,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::named;
    use crate::toast::ToastKind;

    #[test]
    fn filled_button_uses_m3_primary_roles() {
        let tok = named("light").tokens;
        let s = tok.scheme();
        let theme = crate::theme::iced_theme("light", tok);
        let f = button_style(tok, Variant::Primary);
        let active = f(&theme, button::Status::Active);
        assert_eq!(active.background, Some(Background::Color(s.primary)));
        assert_eq!(active.text_color, s.on_primary);
        let disabled = f(&theme, button::Status::Disabled);
        assert_ne!(disabled.background, Some(Background::Color(s.primary)));
        let tonal = button_style(tok, Variant::Quiet)(&theme, button::Status::Active);
        assert_eq!(
            tonal.background,
            Some(Background::Color(s.secondary_container))
        );
        assert_eq!(tonal.text_color, s.on_secondary_container);
        let ghost = button_style(tok, Variant::Ghost)(&theme, button::Status::Active);
        assert_eq!(
            ghost.background,
            Some(Background::Color(Color::TRANSPARENT))
        );
        assert_eq!(ghost.text_color, s.primary);
    }

    #[test]
    fn disabled_filled_button_ink_reads_on_dark() {
        use crate::m3::color::relative_luma;
        let tok = named("dark").tokens;
        let theme = crate::theme::iced_theme("dark", tok);
        let canvas = relative_luma(tok.scheme().surface);
        for v in [
            Variant::Primary,
            Variant::Danger,
            Variant::Success,
            Variant::Warning,
        ] {
            let st = button_style(tok, v)(&theme, button::Status::Disabled);
            let ink = relative_luma(st.text_color);
            for bg in [st.background, None] {
                if let Some(Background::Color(c)) = bg {
                    let fill = relative_luma(c);
                    let dark_msg = format!("{v:?}: disabled ink vanishes on dark canvas");
                    assert!((ink - canvas).abs() > 0.15, "{dark_msg}");
                    let fill_msg = format!("{v:?}: disabled ink vanishes on its fill");
                    assert!((ink - fill).abs() > 0.15, "{fill_msg}");
                }
            }
        }
        for v in [Variant::Ghost, Variant::Outlined] {
            let st = button_style(tok, v)(&theme, button::Status::Disabled);
            let ink = relative_luma(st.text_color);
            let ghost_msg = format!("{v:?}: disabled ink vanishes on dark canvas");
            assert!((ink - canvas).abs() > 0.15, "{ghost_msg}");
        }
        let light = named("light").tokens;
        let light_theme = crate::theme::iced_theme("light", light);
        let light_canvas = relative_luma(light.scheme().surface);
        assert!(light_canvas >= 0.5);
        for v in [Variant::Ghost, Variant::Outlined] {
            let st = button_style(light, v)(&light_theme, button::Status::Disabled);
            let ink = relative_luma(st.text_color);
            let light_msg = format!("{v:?}: disabled ink vanishes on light canvas");
            assert!((ink - light_canvas).abs() > 0.15, "{light_msg}");
        }
    }

    #[test]
    fn elevated_button_raises_one_level_on_hover() {
        let tok = named("dark").tokens;
        let theme = crate::theme::iced_theme("dark", tok);
        let st = button_style(tok, Variant::Elevated);
        let rest = st(&theme, button::Status::Active);
        let hover = st(&theme, button::Status::Hovered);
        assert_eq!(
            rest.shadow.blur_radius,
            tok.shadow(crate::m3::Elevation::Level1).blur_radius
        );
        assert_eq!(
            hover.shadow.blur_radius,
            tok.shadow(crate::m3::Elevation::Level1.raise()).blur_radius
        );
        assert!(hover.shadow.blur_radius > rest.shadow.blur_radius);
        assert!(rest.shadow.blur_radius > 0.0);
    }

    #[test]
    fn list_row_selected_is_secondary_container() {
        let tok = named("dark").tokens;
        let s = tok.scheme();
        let row = list_row(tok, true);
        assert_eq!(
            row.background,
            Some(Background::Color(s.secondary_container))
        );
        assert_eq!(row.text_color, Some(s.on_secondary_container));
        let idle = list_row(tok, false);
        assert_eq!(idle.background, Some(Background::Color(Color::TRANSPARENT)));
        let _ = list_row_hover(tok, true);
        let _ = list_row_hover(tok, false);
    }

    #[test]
    fn table_cell_faces_separate_selection_from_zebra() {
        let tok = named("dark").tokens;
        let s = tok.scheme();
        // Selected row is one secondary wash; focus adds primary outline.
        let focused_sel = table_cell(tok, true, true, true);
        assert_eq!(
            focused_sel.background,
            Some(Background::Color(s.secondary_container))
        );
        assert_eq!(focused_sel.border.color, s.primary);
        assert!(focused_sel.border.width >= 2.0);
        let selected = table_cell(tok, true, false, true);
        assert_eq!(
            selected.background,
            Some(Background::Color(s.secondary_container))
        );
        assert_eq!(selected.text_color, Some(s.on_secondary_container));
        assert_eq!(selected.border.width, 0.0);
        let zebra = table_cell(tok, false, false, true);
        assert_eq!(
            zebra.background,
            Some(Background::Color(s.surface_container_high))
        );
        assert_ne!(zebra.background, selected.background);
        let idle = table_cell(tok, false, false, false);
        assert_eq!(idle.background, Some(Background::Color(Color::TRANSPARENT)));
        let focus_only = table_cell(tok, false, true, false);
        assert_eq!(
            focus_only.background,
            Some(Background::Color(s.primary_container))
        );
    }

    #[test]
    fn text_field_focus_uses_primary_outline() {
        let tok = named("light").tokens;
        let s = tok.scheme();
        let theme = crate::theme::iced_theme("light", tok);
        let st = search_style(tok);
        let focused = st(&theme, text_input::Status::Focused { is_hovered: false });
        assert_eq!(focused.border.color, s.primary);
        assert!(focused.border.width >= 2.0);
        let active = st(&theme, text_input::Status::Active);
        assert_eq!(active.border.color, s.outline);
    }

    #[test]
    fn dialog_tabs_and_app_bar_use_m3_surfaces() {
        let tok = named("dark").tokens;
        let s = tok.scheme();
        let theme = crate::theme::iced_theme("dark", tok);
        let sheet = dialog_sheet_face(tok);
        assert_eq!(
            sheet.background,
            Some(Background::Color(Elevation::Level3.surface(s)))
        );
        assert!(sheet.shadow.blur_radius > 0.0);
        let bar = app_bar(tok);
        assert_eq!(bar.background, Some(Background::Color(s.surface_container)));
        let rail = nav_rail(tok, true);
        assert_eq!(
            rail.background,
            Some(Background::Color(s.secondary_container))
        );
        let rail_idle = nav_rail(tok, false);
        assert_eq!(
            rail_idle.background,
            Some(Background::Color(Color::TRANSPARENT))
        );
        let active = tab_style(tok, true)(&theme, button::Status::Active);
        assert_eq!(active.text_color, s.primary);
        // Label is flush; underbar is tab_indicator, not a full border.
        assert_eq!(active.border.width, 0.0);
        assert_eq!(active.border.color, Color::TRANSPARENT);
        let bar = tab_indicator(tok);
        assert_eq!(bar.background, Some(Background::Color(s.primary)));
        let idle = tab_style(tok, false)(&theme, button::Status::Hovered);
        assert_eq!(idle.text_color, s.on_surface_variant);
        assert!(idle.background.is_some());
        let idle_active = tab_style(tok, false)(&theme, button::Status::Active);
        assert_eq!(idle_active.border.width, 0.0);
        let pressed = tab_style(tok, false)(&theme, button::Status::Pressed);
        assert!(pressed.background.is_some());
        let sk = skeleton(tok);
        assert!(sk.background.is_some());
        // Desktop flat: every component is M3 shape None (0 dp).
        assert_eq!(
            crate::m3::shape::Component::Button.shape(),
            crate::m3::shape::Shape::None
        );
        assert_eq!(
            crate::m3::shape::Component::Field.shape(),
            crate::m3::shape::Shape::None
        );
        assert_eq!(
            card(tok, false).border.radius,
            component_radius(tok, crate::m3::shape::Component::Card)
        );
        assert_eq!(card(tok, false).border.radius, 0.0.into());
        let material = tok.with_shape(crate::m3::ShapePolicy::Material);
        assert_eq!(
            card(material, false).border.radius,
            component_radius(material, crate::m3::shape::Component::Card)
        );
        assert!(card(material, false).border.radius.top_left > 0.0);
        let pill = tok.with_shape(crate::m3::ShapePolicy::Pill);
        assert_eq!(
            card(pill, false).border.radius.top_left,
            crate::m3::Shape::Medium.dp()
        );
        assert!(card(pill, false).border.radius.top_left < 20.0);
        assert_eq!(
            crate::m3::shape::Component::Button
                .shape_for(crate::m3::ShapePolicy::Pill)
                .dp(),
            crate::m3::Shape::Full.dp()
        );
        let flat = tok.with_elevation(crate::m3::ElevationPolicy::Flat);
        assert_eq!(raised_card(flat).shadow.blur_radius, 0.0);
        assert_eq!(dialog_sheet_face(flat).shadow.blur_radius, 0.0);
    }

    #[test]
    fn toast_and_tooltip_corners_follow_shape_policy() {
        let tok = named("dark").tokens;
        assert_eq!(callout(tok, ToastKind::Info).border.radius.top_left, 0.0);
        assert_eq!(tooltip(tok).border.radius.top_left, 0.0);
        let material = tok.with_shape(crate::m3::ShapePolicy::Material);
        assert_eq!(
            callout(material, ToastKind::Success).border.radius.top_left,
            crate::m3::Shape::ExtraSmall.dp()
        );
        assert_eq!(
            tooltip(material).border.radius.top_left,
            crate::m3::Shape::ExtraSmall.dp()
        );
        let pill = tok.with_shape(crate::m3::ShapePolicy::Pill);
        assert_eq!(
            callout(pill, ToastKind::Warning).border.radius.top_left,
            crate::m3::Shape::Medium.dp()
        );
        assert_eq!(
            tooltip(pill).border.radius.top_left,
            crate::m3::Shape::Medium.dp()
        );
        assert_ne!(
            crate::m3::shape::Component::Toast.shape_for(crate::m3::ShapePolicy::Pill),
            crate::m3::shape::Component::Button.shape_for(crate::m3::ShapePolicy::Pill)
        );
    }

    #[test]
    fn tab_and_segment_stay_flush_under_every_shape_policy() {
        let tok = named("dark").tokens;
        for policy in [
            crate::m3::ShapePolicy::Desktop,
            crate::m3::ShapePolicy::Tight,
            crate::m3::ShapePolicy::Soft,
            crate::m3::ShapePolicy::Pill,
            crate::m3::ShapePolicy::Material,
        ] {
            let t = tok.with_shape(policy);
            let th = crate::theme::iced_theme("dark", t);
            let tab = tab_style(t, true)(&th, button::Status::Active);
            assert_eq!(tab.border.radius.top_left, 0.0);
            let seg = segment_style(t, true)(&th, button::Status::Active);
            assert_eq!(seg.border.radius.top_left, 0.0);
            let joined = joined_button_style(t, Variant::Quiet)(&th, button::Status::Active);
            assert_eq!(joined.border.radius.top_left, 0.0);
            let hit = palette_hit(t, true)(&th, button::Status::Active);
            if policy == crate::m3::ShapePolicy::Pill {
                assert_eq!(
                    hit.border.radius.top_left,
                    crate::m3::Shape::ExtraSmall.dp()
                );
                assert_ne!(
                    hit.border.radius.top_left,
                    crate::m3::shape::Component::Button.shape_for(policy).dp()
                );
            }
            if policy == crate::m3::ShapePolicy::Desktop {
                assert_eq!(hit.border.radius.top_left, 0.0);
            }
        }
        let pill = tok.with_shape(crate::m3::ShapePolicy::Pill);
        let ptheme = crate::theme::iced_theme("dark", pill);
        let btn = button_style(pill, Variant::Primary)(&ptheme, button::Status::Active);
        assert_eq!(btn.border.radius.top_left, crate::m3::Shape::Full.dp());
        let tab = tab_style(pill, false)(&ptheme, button::Status::Active);
        assert_eq!(tab.border.radius.top_left, 0.0);
        let menu = menu_item_style(pill)(&ptheme, button::Status::Hovered);
        assert_eq!(
            menu.border.radius.top_left,
            crate::m3::Shape::ExtraSmall.dp()
        );
        assert_ne!(menu.border.radius.top_left, crate::m3::Shape::Full.dp());
        let box_st = checkbox_style(pill)(&ptheme, checkbox::Status::Active { is_checked: false });
        assert_eq!(
            box_st.border.radius.top_left,
            crate::m3::Shape::ExtraSmall.dp()
        );
        assert_ne!(
            box_st.border.radius.top_left,
            crate::m3::shape::Component::Field
                .shape_for(crate::m3::ShapePolicy::Pill)
                .dp()
        );
    }

    #[test]
    fn banner_corners_stay_flush_under_material_and_pill() {
        let tok = named("dark").tokens;
        assert_eq!(banner(tok).border.radius.top_left, 0.0);
        let material = tok.with_shape(crate::m3::ShapePolicy::Material);
        assert_eq!(banner(material).border.radius.top_left, 0.0);
        let pill = tok.with_shape(crate::m3::ShapePolicy::Pill);
        assert_eq!(banner(pill).border.radius.top_left, 0.0);
        assert_ne!(
            crate::m3::shape::Component::Banner.shape_for(crate::m3::ShapePolicy::Pill),
            crate::m3::shape::Component::Card.shape_for(crate::m3::ShapePolicy::Pill)
        );
    }

    #[test]
    fn search_corners_follow_shape_policy() {
        let tok = named("dark").tokens;
        let theme = crate::theme::iced_theme("dark", tok);
        let desktop = search_style(tok)(&theme, text_input::Status::Active);
        assert_eq!(desktop.border.radius.top_left, 0.0);
        let material = tok.with_shape(crate::m3::ShapePolicy::Material);
        let material_st = search_style(material)(
            &crate::theme::iced_theme("dark", material),
            text_input::Status::Active,
        );
        assert_eq!(
            material_st.border.radius.top_left,
            crate::m3::Shape::ExtraLarge.dp()
        );
        let pill = tok.with_shape(crate::m3::ShapePolicy::Pill);
        let pill_st = search_style(pill)(
            &crate::theme::iced_theme("dark", pill),
            text_input::Status::Active,
        );
        assert_eq!(pill_st.border.radius.top_left, crate::m3::Shape::Full.dp());
        let field = field_style(tok.with_shape(crate::m3::ShapePolicy::Material), false)(
            &crate::theme::iced_theme("dark", tok),
            text_input::Status::Active,
        );
        assert_eq!(
            field.border.radius.top_left,
            crate::m3::Shape::ExtraSmall.dp()
        );
    }

    #[test]
    fn track_corners_follow_shape_policy() {
        let tok = named("dark").tokens;
        let theme = crate::theme::iced_theme("dark", tok);
        let sw = switch_style(tok)(&theme, toggler::Status::Active { is_toggled: true });
        assert_eq!(sw.border_radius.map(|r| r.top_left), Some(0.0));
        let sl = slider_style(tok)(&theme, slider::Status::Active);
        assert_eq!(sl.rail.border.radius.top_left, 0.0);
        let s = tok.scheme();
        let ltr = slider_style_rail(tok, false)(&theme, slider::Status::Active);
        let rtl = slider_style_rail(tok, true)(&theme, slider::Status::Active);
        assert_eq!(ltr.rail.backgrounds.0, Background::Color(s.primary));
        assert_eq!(
            rtl.rail.backgrounds.0,
            Background::Color(s.surface_container_highest)
        );
        assert_eq!(progress_style(tok)(&theme).border.radius.top_left, 0.0);
        let material = tok.with_shape(crate::m3::ShapePolicy::Material);
        let mtheme = crate::theme::iced_theme("dark", material);
        let full = crate::m3::Shape::Full.dp();
        assert_eq!(
            switch_style(material)(&mtheme, toggler::Status::Active { is_toggled: false })
                .border_radius
                .map(|r| r.top_left),
            Some(full)
        );
        assert_eq!(
            slider_style(material)(&mtheme, slider::Status::Hovered)
                .rail
                .border
                .radius
                .top_left,
            full
        );
        assert_eq!(
            progress_style(material)(&mtheme).border.radius.top_left,
            full
        );
        let pill = tok.with_shape(crate::m3::ShapePolicy::Pill);
        assert_eq!(
            crate::m3::shape::Component::Track.shape_for(crate::m3::ShapePolicy::Pill),
            crate::m3::Shape::Full
        );
        assert_eq!(
            progress_style(pill)(&crate::theme::iced_theme("dark", pill))
                .border
                .radius
                .top_left,
            full
        );
    }

    #[test]
    fn switch_on_thumb_uses_scheme_on_primary() {
        let tok = named("gruvbox").tokens;
        let s = tok.scheme();
        let theme = crate::theme::iced_theme("gruvbox", tok);
        let st = switch_style(tok)(&theme, toggler::Status::Active { is_toggled: true });
        assert_eq!(st.background, Background::Color(s.primary));
        assert_eq!(st.foreground, Background::Color(s.on_primary));
        // Residual M3 dark on_primary would be this purple-blue.
        assert_ne!(s.on_primary, crate::m3::scheme_dark().on_primary);
    }

    #[test]
    fn dim_backdrop_at_rest_reads_on_dark() {
        let tok = named("dark").tokens;
        for bg in [dim_backdrop(tok).background, None] {
            if let Some(Background::Color(c)) = bg {
                assert!(c.a >= 0.55, "rest dim must read on dark surface-container");
            }
        }
    }

    #[test]
    fn styles_cover_states_and_variants() {
        let tok = named("dark").tokens;
        let theme = crate::theme::iced_theme("dark", tok);
        let _ = fill(tok.canvas, tok.text);
        let _ = card(tok, true);
        let _ = card(tok, false);
        let _ = raised_card(tok);
        let _ = outlined_card(tok);
        let _ = shell(tok);
        let _ = panel(tok);
        let _ = footer(tok);
        let _ = hairline(tok);
        let dim = dim_backdrop(tok);
        for bg in [dim.background, None] {
            if let Some(Background::Color(c)) = bg {
                assert!(c.a >= 0.55, "rest dim must read on dark surface-container");
            }
        }
        let _ = dialog_sheet_face(tok);
        let faded = fade_face(dialog_sheet_face(tok), 0.5);
        let blank = fade_face(container::Style::default(), 0.5);
        let alphas: Vec<f32> = [faded, blank]
            .into_iter()
            .filter_map(|face| match face.background {
                Some(Background::Color(c)) => Some(c.a),
                _ => None,
            })
            .collect();
        assert_eq!(alphas.len(), 1);
        assert!((alphas[0] - 0.5).abs() < 1e-5);
        let _ = app_bar(tok);
        let _ = nav_rail(tok, true);
        let _ = nav_rail(tok, false);
        let _ = list_row(tok, true);
        let _ = list_row(tok, false);
        for k in [
            ToastKind::Info,
            ToastKind::Success,
            ToastKind::Warning,
            ToastKind::Danger,
        ] {
            let _ = callout(tok, k);
        }
        let _ = tooltip(tok);
        let _ = banner(tok);
        let _ = skeleton(tok);
        for v in Variant::ALL {
            let f = button_style(tok, v);
            for st in [
                button::Status::Active,
                button::Status::Hovered,
                button::Status::Pressed,
                button::Status::Disabled,
            ] {
                let _ = f(&theme, st);
            }
        }
        let tab = tab_style(tok, true);
        let tab2 = tab_style(tok, false);
        let _ = tab(&theme, button::Status::Active);
        let _ = tab2(&theme, button::Status::Hovered);
        let _ = tab2(&theme, button::Status::Active);
        let outlined = field_style(tok, true);
        let _ = outlined(&theme, text_input::Status::Active);
        let _ = outlined(&theme, text_input::Status::Hovered);
        let _ = outlined(&theme, text_input::Status::Focused { is_hovered: true });
        let _ = outlined(&theme, text_input::Status::Disabled);
        let filled = field_style(tok, false);
        let _ = filled(&theme, text_input::Status::Disabled);
        let _ = filled(&theme, text_input::Status::Active);
        let s = search_style(tok);
        let _ = s(&theme, text_input::Status::Active);
        let _ = s(&theme, text_input::Status::Hovered);
        let _ = s(&theme, text_input::Status::Focused { is_hovered: false });
        let _ = s(&theme, text_input::Status::Disabled);
        let p = picker_style(tok);
        let _ = p(&theme, pick_list::Status::Active);
        let _ = p(&theme, pick_list::Status::Hovered);
        let open = p(&theme, pick_list::Status::Opened { is_hovered: true });
        assert_eq!(open.border.width, 1.0);
        assert_ne!(open.border.color, tok.scheme().primary);
        let _ = overlay_menu_style(tok)(&theme);
        let pill = tok.with_shape(crate::m3::ShapePolicy::Pill);
        let soft = tok.with_shape(crate::m3::ShapePolicy::Soft);
        assert_eq!(
            overlay_menu_style(pill)(&theme).border.radius.top_left,
            crate::m3::Shape::ExtraSmall.dp()
        );
        assert_eq!(
            overlay_menu_style(soft)(&theme).border.radius.top_left,
            crate::m3::Shape::ExtraSmall.dp()
        );
        assert!(
            overlay_menu_style(pill)(&theme).border.radius.top_left
                < picker_style(pill)(&theme, pick_list::Status::Active)
                    .border
                    .radius
                    .top_left
        );
        let c = checkbox_style(tok);
        let _ = c(&theme, checkbox::Status::Active { is_checked: true });
        let _ = c(&theme, checkbox::Status::Active { is_checked: false });
        let _ = c(&theme, checkbox::Status::Hovered { is_checked: true });
        let _ = c(&theme, checkbox::Status::Hovered { is_checked: false });
        let _ = c(&theme, checkbox::Status::Disabled { is_checked: true });
        let r = radio_style(tok);
        let _ = r(&theme, radio::Status::Active { is_selected: true });
        let _ = r(&theme, radio::Status::Hovered { is_selected: false });
        let sw = switch_style(tok);
        let _ = sw(&theme, toggler::Status::Active { is_toggled: true });
        let _ = sw(&theme, toggler::Status::Active { is_toggled: false });
        let _ = sw(&theme, toggler::Status::Hovered { is_toggled: true });
        let _ = sw(&theme, toggler::Status::Hovered { is_toggled: false });
        let _ = sw(&theme, toggler::Status::Disabled { is_toggled: true });
        let _ = sw(&theme, toggler::Status::Disabled { is_toggled: false });
        let sl = slider_style(tok);
        let _ = sl(&theme, slider::Status::Active);
        let _ = sl(&theme, slider::Status::Hovered);
        let _ = sl(&theme, slider::Status::Dragged);
        let _ = progress_style(tok)(&theme);
        let _ = rule_style(tok)(&theme);
        let sc = scroll_style(tok);
        let _ = sc(
            &theme,
            scrollable::Status::Active {
                is_horizontal_scrollbar_disabled: false,
                is_vertical_scrollbar_disabled: false,
            },
        );
        let _ = sc(
            &theme,
            scrollable::Status::Hovered {
                is_horizontal_scrollbar_hovered: true,
                is_vertical_scrollbar_hovered: false,
                is_horizontal_scrollbar_disabled: false,
                is_vertical_scrollbar_disabled: false,
            },
        );
        let _ = sc(
            &theme,
            scrollable::Status::Dragged {
                is_horizontal_scrollbar_dragged: true,
                is_vertical_scrollbar_dragged: false,
                is_horizontal_scrollbar_disabled: false,
                is_vertical_scrollbar_disabled: true,
            },
        );
    }
}
