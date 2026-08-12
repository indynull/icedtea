//! iced style closures painted from [`crate::theme::Tokens`].

use iced::border::Border;
use iced::overlay::menu as overlay_menu;
use iced::widget::{
    button, checkbox, container, pick_list, progress_bar, radio, rule, scrollable, slider,
    text_input, toggler,
};
use iced::{Background, Color, Shadow};

use crate::chrome::{Corner, Elevation};
use crate::theme::{chip_fill, hover_fill, mix, pressed_fill, Tokens};
use crate::variant::Variant;

fn radius(c: Corner) -> iced::border::Radius {
    c.radius()
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

pub fn card(tok: Tokens, focus: bool) -> container::Style {
    container::Style {
        background: Some(Background::Color(tok.surface)),
        text_color: Some(tok.text),
        border: Border {
            color: if focus { tok.primary } else { tok.border },
            width: 1.0,
            radius: radius(Corner::Tight),
        },
        shadow: Elevation::Flat.shadow(),
        snap: false,
    }
}

pub fn raised_card(tok: Tokens) -> container::Style {
    container::Style {
        background: Some(Background::Color(tok.surface)),
        text_color: Some(tok.text),
        border: Border {
            color: tok.border,
            width: 1.0,
            radius: radius(Corner::Soft),
        },
        shadow: Elevation::Raised.shadow(),
        snap: false,
    }
}

pub fn shell(tok: Tokens) -> container::Style {
    container::Style {
        background: Some(Background::Color(tok.canvas)),
        text_color: Some(tok.text),
        border: Border {
            color: tok.primary,
            width: 1.0,
            radius: radius(Corner::None),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn panel(tok: Tokens) -> container::Style {
    fill(tok.panel, tok.text)
}

pub fn footer(tok: Tokens) -> container::Style {
    fill(tok.panel, tok.muted)
}

pub fn hairline(tok: Tokens) -> container::Style {
    container::Style {
        background: Some(Background::Color(tok.border)),
        snap: false,
        ..container::Style::default()
    }
}

pub fn dim_backdrop(_tok: Tokens) -> container::Style {
    // Black wash so the sheet sits above the scene on light and dark colorways.
    // Canvas-tinted alpha was nearly invisible on dark themes.
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.55))),
        snap: false,
        ..container::Style::default()
    }
}

pub fn list_row(tok: Tokens, selected: bool) -> container::Style {
    let bg = if selected {
        tok.selection
    } else {
        Color::TRANSPARENT
    };
    let fg = if selected {
        tok.selection_text
    } else {
        tok.text
    };
    fill(bg, fg)
}

pub fn callout(tok: Tokens, kind: crate::toast::ToastKind) -> container::Style {
    let accent = match kind {
        crate::toast::ToastKind::Info => tok.primary,
        crate::toast::ToastKind::Success => tok.success,
        crate::toast::ToastKind::Warning => tok.warning,
        crate::toast::ToastKind::Danger => tok.danger,
    };
    container::Style {
        background: Some(Background::Color(mix(accent, tok.canvas, 0.16))),
        text_color: Some(tok.text),
        border: Border {
            color: accent,
            width: 1.0,
            radius: radius(Corner::Tight),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn skeleton(tok: Tokens) -> container::Style {
    fill(mix(tok.text, tok.canvas, 0.08), tok.muted)
}

fn button_colors(tok: Tokens, variant: Variant, status: button::Status) -> (Color, Color) {
    let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
    let disabled = matches!(status, button::Status::Disabled);
    let (bg, fg) = match variant {
        Variant::Primary => (
            mix(tok.primary, tok.canvas, if hover { 0.90 } else { 0.75 }),
            tok.text,
        ),
        Variant::Danger => (
            mix(tok.danger, tok.canvas, if hover { 0.55 } else { 0.35 }),
            tok.danger,
        ),
        Variant::Success => (
            mix(tok.success, tok.canvas, if hover { 0.55 } else { 0.35 }),
            tok.success,
        ),
        Variant::Warning => (
            mix(tok.warning, tok.canvas, if hover { 0.55 } else { 0.35 }),
            tok.warning,
        ),
        Variant::Quiet => (
            if hover {
                pressed_fill(tok)
            } else {
                hover_fill(tok)
            },
            tok.text,
        ),
        Variant::Ghost => (
            if hover {
                hover_fill(tok)
            } else {
                Color::TRANSPARENT
            },
            tok.text,
        ),
        Variant::Chip => (chip_fill(tok), tok.muted),
    };
    if disabled {
        (mix(bg, tok.canvas, 0.5), tok.muted)
    } else {
        (bg, fg)
    }
}

pub fn button_style(
    tok: Tokens,
    variant: Variant,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let (bg, fg) = button_colors(tok, variant, status);
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: fg,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: radius(if variant == Variant::Chip {
                    Corner::Soft
                } else {
                    Corner::Tight
                }),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

pub fn tab_style(
    tok: Tokens,
    active: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let bg = if active {
            Some(Background::Color(mix(tok.primary, tok.canvas, 0.28)))
        } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
            Some(Background::Color(hover_fill(tok)))
        } else {
            None
        };
        button::Style {
            background: bg,
            text_color: if active { tok.text } else { tok.muted },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: radius(Corner::None),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

pub fn search_style(tok: Tokens) -> impl Fn(&iced::Theme, text_input::Status) -> text_input::Style {
    move |_theme, status| {
        let focused = matches!(status, text_input::Status::Focused { .. });
        text_input::Style {
            background: Background::Color(tok.panel),
            border: Border {
                color: if focused { tok.primary } else { tok.border },
                width: 1.0,
                radius: radius(Corner::Tight),
            },
            icon: tok.muted,
            placeholder: tok.muted,
            value: tok.text,
            selection: tok.selection,
        }
    }
}

pub fn picker_style(tok: Tokens) -> impl Fn(&iced::Theme, pick_list::Status) -> pick_list::Style {
    move |_theme, status| {
        let hot = !matches!(status, pick_list::Status::Active);
        pick_list::Style {
            text_color: tok.text,
            placeholder_color: tok.muted,
            handle_color: tok.muted,
            background: Background::Color(tok.panel),
            border: Border {
                color: if hot { tok.primary } else { tok.border },
                width: 1.0,
                radius: radius(Corner::Tight),
            },
        }
    }
}

/// Overlay list under a menu title: surface, hover selection, light shadow.
pub fn overlay_menu_style(tok: Tokens) -> impl Fn(&iced::Theme) -> overlay_menu::Style {
    move |_theme| overlay_menu::Style {
        background: Background::Color(tok.surface),
        border: Border {
            width: 1.0,
            radius: radius(Corner::Tight),
            color: tok.border,
        },
        text_color: tok.text,
        selected_text_color: tok.selection_text,
        selected_background: Background::Color(tok.selection),
        shadow: Elevation::Raised.shadow(),
    }
}

pub fn checkbox_style(tok: Tokens) -> impl Fn(&iced::Theme, checkbox::Status) -> checkbox::Style {
    move |_theme, status| {
        let (checked, disabled) = match status {
            checkbox::Status::Active { is_checked } => (is_checked, false),
            checkbox::Status::Hovered { is_checked } => (is_checked, false),
            checkbox::Status::Disabled { is_checked } => (is_checked, true),
        };
        checkbox::Style {
            background: Background::Color(if checked { tok.primary } else { tok.panel }),
            icon_color: tok.canvas,
            border: Border {
                color: if disabled { tok.muted } else { tok.primary },
                width: 1.0,
                radius: radius(Corner::Tight),
            },
            text_color: Some(if disabled { tok.muted } else { tok.text }),
        }
    }
}

pub fn radio_style(tok: Tokens) -> impl Fn(&iced::Theme, radio::Status) -> radio::Style {
    move |_theme, status| {
        let selected = match status {
            radio::Status::Active { is_selected } | radio::Status::Hovered { is_selected } => {
                is_selected
            }
        };
        radio::Style {
            background: Background::Color(tok.panel),
            dot_color: if selected {
                tok.primary
            } else {
                Color::TRANSPARENT
            },
            border_width: 1.0,
            border_color: tok.primary,
            text_color: Some(tok.text),
        }
    }
}

pub fn switch_style(tok: Tokens) -> impl Fn(&iced::Theme, toggler::Status) -> toggler::Style {
    move |_theme, status| {
        let on = match status {
            toggler::Status::Active { is_toggled }
            | toggler::Status::Hovered { is_toggled }
            | toggler::Status::Disabled { is_toggled } => is_toggled,
        };
        toggler::Style {
            background: Background::Color(if on { tok.primary } else { tok.panel }),
            background_border_width: 1.0,
            background_border_color: tok.border,
            foreground: Background::Color(tok.surface),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
            text_color: Some(tok.text),
            border_radius: Some(radius(Corner::Soft)),
            padding_ratio: 0.2,
        }
    }
}

pub fn slider_style(tok: Tokens) -> impl Fn(&iced::Theme, slider::Status) -> slider::Style {
    move |_theme, _status| slider::Style {
        rail: slider::Rail {
            backgrounds: (Background::Color(tok.primary), Background::Color(tok.panel)),
            width: 4.0,
            border: Border {
                color: tok.border,
                width: 0.0,
                radius: radius(Corner::Soft),
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 7.0 },
            background: Background::Color(tok.primary),
            border_width: 0.0,
            border_color: tok.border,
        },
    }
}

pub fn progress_style(tok: Tokens) -> impl Fn(&iced::Theme) -> progress_bar::Style {
    move |_theme| progress_bar::Style {
        background: Background::Color(tok.panel),
        bar: Background::Color(tok.primary),
        border: Border {
            color: tok.border,
            width: 0.0,
            radius: radius(Corner::Soft),
        },
    }
}

pub fn rule_style(tok: Tokens) -> impl Fn(&iced::Theme) -> rule::Style {
    move |_theme| rule::Style {
        color: tok.border,
        radius: radius(Corner::None),
        fill_mode: rule::FillMode::Full,
        snap: false,
    }
}

pub fn scroll_style(tok: Tokens) -> impl Fn(&iced::Theme, scrollable::Status) -> scrollable::Style {
    move |_theme, _status| {
        let rail = scrollable::Rail {
            background: Some(Background::Color(tok.panel)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: radius(Corner::Soft),
            },
            scroller: scrollable::Scroller {
                background: Background::Color(mix(tok.text, tok.canvas, 0.35)),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: radius(Corner::Soft),
                },
            },
        };
        scrollable::Style {
            container: fill(Color::TRANSPARENT, tok.text),
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
            auto_scroll: scrollable::AutoScroll {
                background: Background::Color(tok.panel),
                border: Border {
                    color: tok.border,
                    width: 1.0,
                    radius: radius(Corner::Soft),
                },
                shadow: Shadow::default(),
                icon: tok.text,
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
    fn styles_cover_states_and_variants() {
        let tok = named("dark").tokens;
        let theme = crate::theme::iced_theme("dark", tok);
        let _ = fill(tok.canvas, tok.text);
        let _ = card(tok, true);
        let _ = card(tok, false);
        let _ = raised_card(tok);
        let _ = shell(tok);
        let _ = panel(tok);
        let _ = footer(tok);
        let _ = hairline(tok);
        let _ = dim_backdrop(tok);
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
        let s = search_style(tok);
        let _ = s(&theme, text_input::Status::Active);
        let _ = s(&theme, text_input::Status::Hovered);
        let _ = s(&theme, text_input::Status::Focused { is_hovered: false });
        let _ = s(&theme, text_input::Status::Disabled);
        let p = picker_style(tok);
        let _ = p(&theme, pick_list::Status::Active);
        let _ = p(&theme, pick_list::Status::Hovered);
        let _ = p(&theme, pick_list::Status::Opened { is_hovered: true });
        let _ = overlay_menu_style(tok)(&theme);
        let c = checkbox_style(tok);
        let _ = c(&theme, checkbox::Status::Active { is_checked: true });
        let _ = c(&theme, checkbox::Status::Hovered { is_checked: false });
        let _ = c(&theme, checkbox::Status::Disabled { is_checked: true });
        let r = radio_style(tok);
        let _ = r(&theme, radio::Status::Active { is_selected: true });
        let _ = r(&theme, radio::Status::Hovered { is_selected: false });
        let sw = switch_style(tok);
        let _ = sw(&theme, toggler::Status::Active { is_toggled: true });
        let _ = sw(&theme, toggler::Status::Hovered { is_toggled: false });
        let _ = sw(&theme, toggler::Status::Disabled { is_toggled: true });
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
