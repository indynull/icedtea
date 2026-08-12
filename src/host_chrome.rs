//! Desktop chrome colors for [`crate::theme::OsChrome`].
//!
//! Each host fills what it can. Missing fields stay unset so the
//! named colorway remains the base.

use iced::Color;

use crate::theme::OsChrome;

/// One-shot read. Safe when the host has no colors (`OsChrome` empty).
/// On macOS prefer the main thread (same as mundy accent). Off-thread
/// or panic paths return [`OsChrome::empty`].
pub(crate) fn snapshot() -> OsChrome {
    std::panic::catch_unwind(|| {
        let mut chrome = platform::surfaces();
        chrome.primary = accent_from_mundy().or(chrome.primary);
        chrome
    })
    .unwrap_or_else(|_| OsChrome::empty())
}

/// Emits a full snapshot when accent or color-scheme changes.
pub(crate) fn listen() -> impl iced::futures::Stream<Item = OsChrome> {
    use iced::futures::StreamExt;
    let interest = mundy::Interest::AccentColor | mundy::Interest::ColorScheme;
    mundy::Preferences::stream(interest).map(|prefs| {
        std::panic::catch_unwind(|| {
            let mut chrome = platform::surfaces();
            if let Some(c) = prefs.accent_color.0 {
                chrome.primary = Some(srgba(c));
            }
            chrome
        })
        .unwrap_or_else(|_| OsChrome::empty())
    })
}

fn accent_from_mundy() -> Option<Color> {
    let prefs = mundy::Preferences::once_blocking(
        mundy::Interest::AccentColor,
        std::time::Duration::from_millis(80),
    )?;
    prefs.accent_color.0.map(srgba)
}

fn srgba(c: mundy::Srgba) -> Color {
    Color::from_rgba(c.red as f32, c.green as f32, c.blue as f32, c.alpha as f32)
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use objc2_app_kit::{NSColor, NSColorSpace};

    /// AppKit semantic colors (and control accent as primary fallback).
    pub(super) fn surfaces() -> OsChrome {
        OsChrome {
            primary: ns(NSColor::controlAccentColor()),
            canvas: ns(NSColor::windowBackgroundColor()),
            surface: ns(NSColor::textBackgroundColor()),
            panel: ns(NSColor::controlBackgroundColor()),
            text: ns(NSColor::labelColor()),
            muted: ns(NSColor::secondaryLabelColor()),
            border: ns(NSColor::separatorColor()),
        }
    }

    fn ns(color: objc2::rc::Retained<NSColor>) -> Option<Color> {
        let space = NSColorSpace::sRGBColorSpace();
        let c = color.colorUsingColorSpace(&space)?;
        let mut r = 0.0f64;
        let mut g = 0.0f64;
        let mut b = 0.0f64;
        let mut a = 0.0f64;
        unsafe {
            c.getRed_green_blue_alpha(
                std::ptr::from_mut(&mut r),
                std::ptr::from_mut(&mut g),
                std::ptr::from_mut(&mut b),
                std::ptr::from_mut(&mut a),
            );
        }
        Some(Color::from_rgba(r as f32, g as f32, b as f32, a as f32))
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;

    const COLOR_WINDOW: i32 = 5;
    const COLOR_WINDOWTEXT: i32 = 8;
    const COLOR_BTNFACE: i32 = 15;
    const COLOR_BTNSHADOW: i32 = 16;
    const COLOR_GRAYTEXT: i32 = 17;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetSysColor(index: i32) -> u32;
    }

    pub(super) fn surfaces() -> OsChrome {
        OsChrome {
            primary: None,
            canvas: sys(COLOR_WINDOW),
            surface: sys(COLOR_BTNFACE),
            panel: sys(COLOR_BTNFACE),
            text: sys(COLOR_WINDOWTEXT),
            muted: sys(COLOR_GRAYTEXT),
            border: sys(COLOR_BTNSHADOW),
        }
    }

    fn sys(index: i32) -> Option<Color> {
        let c = unsafe { GetSysColor(index) };
        Some(Color::from_rgb8(
            (c & 0xff) as u8,
            ((c >> 8) & 0xff) as u8,
            ((c >> 16) & 0xff) as u8,
        ))
    }
}

/// Portal (Wayland and X11): no standard surface keys. Primary comes
/// from the accent stream / one-shot in the parent module.
#[cfg(all(unix, not(target_os = "macos")))]
mod platform {
    use super::*;

    pub(super) fn surfaces() -> OsChrome {
        OsChrome::empty()
    }
}

#[cfg(not(any(unix, windows)))]
mod platform {
    use super::*;

    pub(super) fn surfaces() -> OsChrome {
        OsChrome::empty()
    }
}
