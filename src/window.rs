//! Window kinds, overlay placement, and hide policy.
//!
//! `Boot::overlay` plus `place` / `place_centered` position a palette.
//! The gallery Command palette page walks hide policy and retarget.

use iced::window::{self, Level, Position};
use iced::{Point, Size};

use crate::layout::{window_size_from_dock, DockSpec};

/// Kind of window icedtea can open.
///
/// ```
/// use icedtea::window::{HideEvent, HidePolicy, WindowKind, should_hide};
/// assert!(should_hide(HidePolicy::Escape, HideEvent::Escape, false));
/// assert!(should_hide(HidePolicy::Escape, HideEvent::Escape, true));
/// assert!(!should_hide(HidePolicy::FocusLoss, HideEvent::FocusLoss, true));
/// let s = icedtea::window::settings(WindowKind::Overlay, "dev.icedtea.app");
/// assert!(!s.decorations);
/// assert!(s.max_size.is_none());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowKind {
    Application,
    Dialog,
    Overlay,
}

/// When an overlay hides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HidePolicy {
    Escape,
    FocusLoss,
    EscapeOrFocusLoss,
    Manual,
}

/// Event that may hide an overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HideEvent {
    Escape,
    FocusLoss,
}

/// Whether the overlay should hide.
///
/// `in_card` (search field or result list) suppresses only
/// [`HideEvent::FocusLoss`]. Escape still hides. iced 0.14 `text_input`
/// captures Escape; the application must forward that captured key
/// into this function.
pub fn should_hide(policy: HidePolicy, event: HideEvent, in_card: bool) -> bool {
    if in_card && event == HideEvent::FocusLoss {
        return false;
    }
    match (policy, event) {
        (HidePolicy::Manual, _) => false,
        (HidePolicy::Escape, HideEvent::Escape) => true,
        (HidePolicy::FocusLoss, HideEvent::FocusLoss) => true,
        (HidePolicy::EscapeOrFocusLoss, _) => true,
        _ => false,
    }
}

/// One display rectangle in the same coordinate space as the pointer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisplayBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl DisplayBounds {
    pub fn contains(self, pointer: (f32, f32)) -> bool {
        pointer.0 >= self.x
            && pointer.0 < self.x + self.width
            && pointer.1 >= self.y
            && pointer.1 < self.y + self.height
    }
}

/// Place an overlay on the display under `pointer`, clamped so the
/// window stays on that display.
///
/// ```
/// use icedtea::window::{place, DisplayBounds};
/// let d = DisplayBounds { x: 0.0, y: 0.0, width: 1920.0, height: 1080.0 };
/// let p = place((100.0, 80.0), iced::Size::new(400.0, 300.0), &[d]);
/// assert_eq!(p.x, 100.0);
/// assert_eq!(p.y, 80.0);
/// ```
pub fn place(pointer: (f32, f32), size: Size, displays: &[DisplayBounds]) -> Point {
    let Some(d) = display_at(pointer, displays) else {
        return Point::new(pointer.0, pointer.1);
    };
    clamp_on_display(pointer.0, pointer.1, size, d)
}

/// Center `size` on the display under `pointer` (else the first display).
///
/// ```
/// use icedtea::window::{place_centered, DisplayBounds};
/// let left = DisplayBounds { x: 0.0, y: 0.0, width: 1920.0, height: 1080.0 };
/// let right = DisplayBounds { x: 1920.0, y: 0.0, width: 1280.0, height: 800.0 };
/// let p = place_centered((2000.0, 10.0), iced::Size::new(400.0, 300.0), &[left, right]);
/// assert_eq!(p.x, 2360.0);
/// assert_eq!(p.y, 250.0);
/// ```
pub fn place_centered(pointer: (f32, f32), size: Size, displays: &[DisplayBounds]) -> Point {
    let Some(d) = display_at(pointer, displays) else {
        return Point::new(pointer.0 - size.width / 2.0, pointer.1 - size.height / 2.0);
    };
    clamp_on_display(
        d.x + (d.width - size.width) / 2.0,
        d.y + (d.height - size.height) / 2.0,
        size,
        d,
    )
}

fn display_at(pointer: (f32, f32), displays: &[DisplayBounds]) -> Option<DisplayBounds> {
    displays
        .iter()
        .copied()
        .find(|d| d.contains(pointer))
        .or_else(|| displays.first().copied())
}

fn clamp_on_display(x: f32, y: f32, size: Size, d: DisplayBounds) -> Point {
    let max_x = (d.x + d.width - size.width).max(d.x);
    let max_y = (d.y + d.height - size.height).max(d.y);
    Point::new(x.clamp(d.x, max_x), y.clamp(d.y, max_y))
}

/// iced window settings for a kind.
pub fn settings(kind: WindowKind, application_id: &str) -> window::Settings {
    let (def, min) = window_size_from_dock(DockSpec::default());
    let mut win = match kind {
        WindowKind::Application => window::Settings {
            size: def,
            min_size: Some(min),
            resizable: true,
            decorations: true,
            level: Level::Normal,
            position: Position::Default,
            exit_on_close_request: true,
            ..window::Settings::default()
        },
        WindowKind::Dialog => window::Settings {
            size: Size::new(480.0, 320.0),
            min_size: Some(Size::new(360.0, 200.0)),
            resizable: true,
            decorations: true,
            level: Level::Normal,
            position: Position::Centered,
            exit_on_close_request: true,
            ..window::Settings::default()
        },
        WindowKind::Overlay => window::Settings {
            size: Size::new(720.0, 480.0),
            min_size: Some(Size::new(200.0, 160.0)),
            max_size: None,
            resizable: false,
            decorations: false,
            level: Level::AlwaysOnTop,
            position: Position::Centered,
            exit_on_close_request: false,
            ..window::Settings::default()
        },
    };
    apply_app_id(&mut win, application_id, kind);
    win
}

/// Turn overlay settings into a decorated application window.
///
/// Keeps size and position. The application opens or replaces the
/// window; iced cannot change Linux `override_redirect` or Windows
/// `skip_taskbar` on a live window.
///
/// ```
/// use icedtea::window::{settings, retarget, WindowKind};
/// let mut s = settings(WindowKind::Overlay, "dev.icedtea.app");
/// retarget(&mut s, "dev.icedtea.app");
/// assert!(s.decorations && s.resizable);
/// assert_eq!(s.level, iced::window::Level::Normal);
/// ```
pub fn retarget(win: &mut window::Settings, application_id: &str) {
    win.resizable = true;
    win.decorations = true;
    win.level = Level::Normal;
    win.exit_on_close_request = true;
    apply_app_id(win, application_id, WindowKind::Application);
}

fn apply_app_id(win: &mut window::Settings, application_id: &str, kind: WindowKind) {
    #[cfg(target_os = "linux")]
    {
        win.platform_specific.application_id = application_id.to_string();
        win.platform_specific.override_redirect = matches!(kind, WindowKind::Overlay);
    }
    #[cfg(target_os = "windows")]
    {
        win.platform_specific.skip_taskbar = matches!(kind, WindowKind::Overlay);
    }
    let _ = (win, application_id, kind);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_and_hide_policy() {
        assert!(!should_hide(HidePolicy::Manual, HideEvent::Escape, false));
        assert!(!should_hide(
            HidePolicy::Manual,
            HideEvent::FocusLoss,
            false
        ));
        assert!(should_hide(HidePolicy::Escape, HideEvent::Escape, false));
        assert!(!should_hide(
            HidePolicy::Escape,
            HideEvent::FocusLoss,
            false
        ));
        assert!(should_hide(
            HidePolicy::FocusLoss,
            HideEvent::FocusLoss,
            false
        ));
        assert!(!should_hide(
            HidePolicy::FocusLoss,
            HideEvent::Escape,
            false
        ));
        assert!(should_hide(
            HidePolicy::EscapeOrFocusLoss,
            HideEvent::Escape,
            false
        ));
        assert!(should_hide(
            HidePolicy::EscapeOrFocusLoss,
            HideEvent::FocusLoss,
            false
        ));
        let app = settings(WindowKind::Application, "dev.icedtea.test");
        assert!(app.decorations && app.resizable);
        let dlg = settings(WindowKind::Dialog, "dev.icedtea.test");
        assert!(matches!(dlg.position, Position::Centered));
        let ov = settings(WindowKind::Overlay, "dev.icedtea.test");
        assert!(!ov.decorations);
        assert_eq!(ov.level, Level::AlwaysOnTop);
        assert!(ov.max_size.is_none());
        assert_eq!(ov.size, Size::new(720.0, 480.0));
        let d = DisplayBounds {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        let other = DisplayBounds {
            x: 1920.0,
            y: 0.0,
            width: 1280.0,
            height: 800.0,
        };
        let p = place((100.0, 80.0), Size::new(400.0, 300.0), &[d, other]);
        assert_eq!(p, Point::new(100.0, 80.0));
        let edge = place((1900.0, 1000.0), Size::new(400.0, 300.0), &[d]);
        assert!((edge.x - 1520.0).abs() < 0.01);
        assert!((edge.y - 780.0).abs() < 0.01);
        let second = place((2000.0, 10.0), Size::new(200.0, 100.0), &[d, other]);
        assert!(second.x >= 1920.0);
        let raw = place((8.0, 9.0), Size::new(10.0, 10.0), &[]);
        assert_eq!(raw, Point::new(8.0, 9.0));
        let fallback = place((-10.0, -10.0), Size::new(100.0, 80.0), &[other]);
        assert_eq!(fallback.x, other.x);
        assert_eq!(fallback.y, other.y);
        assert!(d.contains((10.0, 10.0)));
        assert!(!d.contains((2000.0, 10.0)));
    }

    #[test]
    fn place_centered_uses_pointer_display() {
        let left = DisplayBounds {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        let right = DisplayBounds {
            x: 1920.0,
            y: 0.0,
            width: 1280.0,
            height: 800.0,
        };
        let size = Size::new(400.0, 300.0);
        let mid = place_centered((2000.0, 10.0), size, &[left, right]);
        assert_eq!(mid, Point::new(2360.0, 250.0));
        let first = place_centered((100.0, 80.0), size, &[left, right]);
        assert_eq!(first, Point::new(760.0, 390.0));
        let miss = place_centered((-10.0, -10.0), size, &[left, right]);
        assert_eq!(miss, Point::new(760.0, 390.0));
        let empty = place_centered((8.0, 9.0), Size::new(10.0, 10.0), &[]);
        assert_eq!(empty, Point::new(3.0, 4.0));
        let huge = place_centered((100.0, 80.0), Size::new(3000.0, 2000.0), &[left]);
        assert_eq!(huge, Point::new(0.0, 0.0));
    }

    #[test]
    fn in_card_suppresses_focus_loss_only() {
        assert!(should_hide(HidePolicy::Escape, HideEvent::Escape, true));
        assert!(should_hide(
            HidePolicy::EscapeOrFocusLoss,
            HideEvent::Escape,
            true
        ));
        assert!(!should_hide(HidePolicy::Escape, HideEvent::FocusLoss, true));
        assert!(!should_hide(
            HidePolicy::EscapeOrFocusLoss,
            HideEvent::FocusLoss,
            true
        ));
        assert!(!should_hide(
            HidePolicy::FocusLoss,
            HideEvent::FocusLoss,
            true
        ));
        assert!(!should_hide(HidePolicy::Manual, HideEvent::Escape, true));
    }

    #[test]
    fn retarget_makes_decorated_application() {
        let mut s = settings(WindowKind::Overlay, "dev.icedtea.test");
        let size = s.size;
        assert!(!s.decorations);
        assert!(!s.resizable);
        assert_eq!(s.level, Level::AlwaysOnTop);
        assert!(!s.exit_on_close_request);
        #[cfg(target_os = "linux")]
        assert!(s.platform_specific.override_redirect);
        #[cfg(target_os = "windows")]
        assert!(s.platform_specific.skip_taskbar);
        retarget(&mut s, "dev.icedtea.test");
        assert!(s.decorations && s.resizable);
        assert_eq!(s.level, Level::Normal);
        assert!(s.exit_on_close_request);
        assert_eq!(s.size, size);
        #[cfg(target_os = "linux")]
        {
            assert!(!s.platform_specific.override_redirect);
            assert_eq!(s.platform_specific.application_id, "dev.icedtea.test");
        }
        #[cfg(target_os = "windows")]
        assert!(!s.platform_specific.skip_taskbar);
    }
}
