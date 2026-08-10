//! Keyboard shortcuts on actions.
//!
//! `ctrl` in a spec is the host Save chord: Command on macOS, Control
//! on Linux and Windows. Menus print that host form. Write `ctrl+s`
//! once; do not branch on the target.

use iced::keyboard::{key::Named, Key, Modifiers};

/// Command on macOS, Control on Linux and Windows.
pub fn primary() -> Modifiers {
    if cfg!(target_os = "macos") {
        Modifiers::LOGO
    } else {
        Modifiers::CTRL
    }
}

const FUNCTION_KEYS: [Named; 24] = [
    Named::F1,
    Named::F2,
    Named::F3,
    Named::F4,
    Named::F5,
    Named::F6,
    Named::F7,
    Named::F8,
    Named::F9,
    Named::F10,
    Named::F11,
    Named::F12,
    Named::F13,
    Named::F14,
    Named::F15,
    Named::F16,
    Named::F17,
    Named::F18,
    Named::F19,
    Named::F20,
    Named::F21,
    Named::F22,
    Named::F23,
    Named::F24,
];

pub(crate) fn function_named(n: u8) -> Option<Named> {
    FUNCTION_KEYS.get(usize::from(n).checked_sub(1)?).copied()
}

pub(crate) fn function_number(named: Named) -> Option<u8> {
    FUNCTION_KEYS
        .iter()
        .position(|&k| k == named)
        .map(|i| (i + 1) as u8)
}

/// A key plus modifiers.
///
/// ```
/// use icedtea::shortcut::{primary, Shortcut};
/// let s = Shortcut::parse("ctrl+s").unwrap();
/// assert!(s.matches(primary(), &icedtea::iced::keyboard::Key::Character("s".into())));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortcut {
    pub modifiers: Modifiers,
    pub key: Key,
}

impl Shortcut {
    pub fn new(modifiers: Modifiers, key: Key) -> Self {
        Self { modifiers, key }
    }

    /// Parse `ctrl+shift+p`, `cmd+k`, `esc`, `f1`.
    ///
    /// `ctrl` / `control` store [`primary`]. `cmd` / `super` / `meta` /
    /// `win` are the logo key on every host.
    pub fn parse(spec: &str) -> Option<Self> {
        let spec = spec.trim().to_ascii_lowercase();
        if spec.is_empty() {
            return None;
        }
        let mut modifiers = Modifiers::empty();
        let mut key_part = spec.as_str();
        let parts: Vec<&str> = spec.split('+').collect();
        for (i, part) in parts.iter().enumerate() {
            if i + 1 == parts.len() {
                key_part = part;
                break;
            }
            match *part {
                "ctrl" | "control" => modifiers |= primary(),
                "shift" => modifiers |= Modifiers::SHIFT,
                "alt" | "option" => modifiers |= Modifiers::ALT,
                "cmd" | "super" | "meta" | "win" => modifiers |= Modifiers::LOGO,
                _ => return None,
            }
        }
        let key = parse_key(key_part)?;
        Some(Self { modifiers, key })
    }

    pub fn matches(&self, modifiers: Modifiers, key: &Key) -> bool {
        self.modifiers == modifiers && &self.key == key
    }
}

fn parse_key(part: &str) -> Option<Key> {
    if let Some(named) = parse_named(part) {
        return Some(Key::Named(named));
    }
    if part.chars().count() == 1 {
        let c = part.chars().next()?;
        return Some(Key::Character(c.to_string().into()));
    }
    None
}

fn parse_named(part: &str) -> Option<Named> {
    match part {
        "esc" | "escape" => Some(Named::Escape),
        "enter" | "return" => Some(Named::Enter),
        "tab" => Some(Named::Tab),
        "space" => Some(Named::Space),
        "backspace" => Some(Named::Backspace),
        "delete" | "del" => Some(Named::Delete),
        "up" => Some(Named::ArrowUp),
        "down" => Some(Named::ArrowDown),
        "left" => Some(Named::ArrowLeft),
        "right" => Some(Named::ArrowRight),
        other => other
            .strip_prefix('f')
            .and_then(|n| n.parse().ok())
            .and_then(function_named),
    }
}

impl std::fmt::Display for Shortcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.modifiers.control() {
            write!(f, "ctrl+")?;
        }
        if self.modifiers.logo() {
            write!(f, "cmd+")?;
        }
        if self.modifiers.alt() {
            write!(f, "alt+")?;
        }
        if self.modifiers.shift() {
            write!(f, "shift+")?;
        }
        match &self.key {
            Key::Named(Named::Escape) => write!(f, "esc"),
            Key::Named(Named::Enter) => write!(f, "enter"),
            Key::Named(Named::Tab) => write!(f, "tab"),
            Key::Named(Named::Space) => write!(f, "space"),
            Key::Named(Named::Backspace) => write!(f, "backspace"),
            Key::Named(Named::Delete) => write!(f, "delete"),
            Key::Named(Named::ArrowUp) => write!(f, "up"),
            Key::Named(Named::ArrowDown) => write!(f, "down"),
            Key::Named(Named::ArrowLeft) => write!(f, "left"),
            Key::Named(Named::ArrowRight) => write!(f, "right"),
            Key::Named(n) => match function_number(*n) {
                Some(i) => write!(f, "f{i}"),
                None => write!(f, "{n:?}"),
            },
            Key::Character(c) => write!(f, "{c}"),
            other => write!(f, "{other:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_match_common_chords() {
        let s = Shortcut::parse("ctrl+s").unwrap();
        assert!(s.matches(primary(), &Key::Character("s".into())));
        assert!(!s.matches(Modifiers::SHIFT, &Key::Character("s".into())));
        let host = if cfg!(target_os = "macos") {
            "cmd+s"
        } else {
            "ctrl+s"
        };
        assert_eq!(s.to_string(), host);
        assert_eq!(
            Shortcut::parse("cmd+shift+p").unwrap().to_string(),
            "cmd+shift+p"
        );
        assert_eq!(Shortcut::parse("esc").unwrap().to_string(), "esc");
        assert_eq!(
            Shortcut::parse("alt+enter").unwrap().to_string(),
            "alt+enter"
        );
        assert!(Shortcut::parse("").is_none());
        assert!(Shortcut::parse("ctrl+").is_none());
        assert!(Shortcut::parse("foo+s").is_none());
        for spec in [
            "tab",
            "space",
            "backspace",
            "delete",
            "up",
            "down",
            "left",
            "right",
            "enter",
        ] {
            let parsed = Shortcut::parse(spec).unwrap();
            assert_eq!(parsed.to_string(), spec);
        }
        for n in 1u8..=24 {
            let spec = format!("f{n}");
            let parsed = Shortcut::parse(&spec).unwrap();
            assert_eq!(parsed.to_string(), spec);
            assert_eq!(
                function_named(n),
                Some(match parsed.key {
                    Key::Named(named) => named,
                    _ => panic!("function key"),
                })
            );
            assert_eq!(function_number(function_named(n).unwrap()), Some(n));
        }
        assert!(Shortcut::parse("f0").is_none());
        assert!(Shortcut::parse("f25").is_none());
        assert!(function_named(0).is_none());
        assert!(function_number(Named::Escape).is_none());
        assert!(Shortcut::parse("control+x").is_some());
        assert!(Shortcut::parse("super+k").is_some());
        assert!(Shortcut::parse("option+a").is_some());
        let _ = Shortcut::new(Modifiers::CTRL, Key::Named(Named::Escape)).to_string();
        let weird = Shortcut::new(Modifiers::empty(), Key::Unidentified);
        assert!(weird.to_string().contains("Unidentified"));
    }
}
