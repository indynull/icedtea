//! Keyboard shortcuts on actions.

use iced::keyboard::{key::Named, Key, Modifiers};

/// A key plus modifiers.
///
/// ```
/// use icedtea::shortcut::Shortcut;
/// let s = Shortcut::parse("ctrl+s").unwrap();
/// assert_eq!(s.to_string(), "ctrl+s");
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
                "ctrl" | "control" => modifiers |= Modifiers::CTRL,
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
    match part {
        "esc" | "escape" => Some(Key::Named(Named::Escape)),
        "enter" | "return" => Some(Key::Named(Named::Enter)),
        "tab" => Some(Key::Named(Named::Tab)),
        "space" => Some(Key::Named(Named::Space)),
        "backspace" => Some(Key::Named(Named::Backspace)),
        "delete" | "del" => Some(Key::Named(Named::Delete)),
        "up" => Some(Key::Named(Named::ArrowUp)),
        "down" => Some(Key::Named(Named::ArrowDown)),
        "left" => Some(Key::Named(Named::ArrowLeft)),
        "right" => Some(Key::Named(Named::ArrowRight)),
        "f1" => Some(Key::Named(Named::F1)),
        "f2" => Some(Key::Named(Named::F2)),
        "f3" => Some(Key::Named(Named::F3)),
        "f4" => Some(Key::Named(Named::F4)),
        "f5" => Some(Key::Named(Named::F5)),
        "f10" => Some(Key::Named(Named::F10)),
        "f11" => Some(Key::Named(Named::F11)),
        "f12" => Some(Key::Named(Named::F12)),
        other if other.chars().count() == 1 => {
            let c = other.chars().next()?;
            Some(Key::Character(c.to_string().into()))
        }
        _ => None,
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
            Key::Named(Named::F1) => write!(f, "f1"),
            Key::Named(Named::F2) => write!(f, "f2"),
            Key::Named(Named::F3) => write!(f, "f3"),
            Key::Named(Named::F4) => write!(f, "f4"),
            Key::Named(Named::F5) => write!(f, "f5"),
            Key::Named(Named::F10) => write!(f, "f10"),
            Key::Named(Named::F11) => write!(f, "f11"),
            Key::Named(Named::F12) => write!(f, "f12"),
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
        assert!(s.matches(Modifiers::CTRL, &Key::Character("s".into())));
        assert!(!s.matches(Modifiers::SHIFT, &Key::Character("s".into())));
        assert_eq!(s.to_string(), "ctrl+s");
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
            "f1",
            "f2",
            "f3",
            "f4",
            "f5",
            "f10",
            "f11",
            "f12",
        ] {
            let parsed = Shortcut::parse(spec).unwrap();
            assert_eq!(parsed.to_string(), spec);
        }
        assert!(Shortcut::parse("control+x").is_some());
        assert!(Shortcut::parse("super+k").is_some());
        assert!(Shortcut::parse("option+a").is_some());
        let _ = Shortcut::new(Modifiers::CTRL, Key::Named(Named::Escape)).to_string();
        let weird = Shortcut::new(Modifiers::empty(), Key::Unidentified);
        assert!(weird.to_string().contains("Unidentified"));
    }
}
