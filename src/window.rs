//! Window kinds and hide policy.

use iced::window::{self, Level, Position};
use iced::Size;

use crate::layout::{window_size_from_dock, DockSpec};

/// Kind of window icedtea can open.
///
/// ```
/// use icedtea::window::{HideEvent, HidePolicy, WindowKind, should_hide};
/// assert!(should_hide(HidePolicy::Escape, HideEvent::Escape));
/// let s = icedtea::window::settings(WindowKind::Overlay, "dev.icedtea.app");
/// assert!(!s.decorations);
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

pub fn should_hide(policy: HidePolicy, event: HideEvent) -> bool {
    match (policy, event) {
        (HidePolicy::Manual, _) => false,
        (HidePolicy::Escape, HideEvent::Escape) => true,
        (HidePolicy::FocusLoss, HideEvent::FocusLoss) => true,
        (HidePolicy::EscapeOrFocusLoss, _) => true,
        _ => false,
    }
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
            min_size: Some(Size::new(480.0, 320.0)),
            max_size: Some(Size::new(720.0, 480.0)),
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

fn apply_app_id(win: &mut window::Settings, application_id: &str, kind: WindowKind) {
    #[cfg(target_os = "linux")]
    {
        win.platform_specific.application_id = application_id.to_string();
        win.platform_specific.override_redirect = matches!(kind, WindowKind::Overlay);
    }
    let _ = (win, application_id, kind);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_and_hide_policy() {
        assert!(!should_hide(HidePolicy::Manual, HideEvent::Escape));
        assert!(!should_hide(HidePolicy::Manual, HideEvent::FocusLoss));
        assert!(should_hide(HidePolicy::Escape, HideEvent::Escape));
        assert!(!should_hide(HidePolicy::Escape, HideEvent::FocusLoss));
        assert!(should_hide(HidePolicy::FocusLoss, HideEvent::FocusLoss));
        assert!(!should_hide(HidePolicy::FocusLoss, HideEvent::Escape));
        assert!(should_hide(
            HidePolicy::EscapeOrFocusLoss,
            HideEvent::Escape
        ));
        assert!(should_hide(
            HidePolicy::EscapeOrFocusLoss,
            HideEvent::FocusLoss
        ));
        let app = settings(WindowKind::Application, "dev.icedtea.test");
        assert!(app.decorations && app.resizable);
        let dlg = settings(WindowKind::Dialog, "dev.icedtea.test");
        assert!(matches!(dlg.position, Position::Centered));
        let ov = settings(WindowKind::Overlay, "dev.icedtea.test");
        assert!(!ov.decorations);
        assert_eq!(ov.level, Level::AlwaysOnTop);
    }
}
