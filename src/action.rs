//! One action type for menus, toolbars, shortcuts, and the command palette.
//!
//! An action carries the application's message; `update` applies it.
//! The gallery Menu, Toolbar, and Command palette pages show the same
//! table in each chrome.
//!
//! ```
//! use icedtea::action::{Action, ActionTable};
//! use icedtea::shortcut::Shortcut;
//! let mut table = ActionTable::new();
//! table.insert(
//!     Action::new("file.save", "Save", "saved")
//!         .with_shortcut(Shortcut::parse("ctrl+s").unwrap()),
//! );
//! assert_eq!(table.invoke("file.save"), Some("saved"));
//! ```

use crate::icon::Icon;
use crate::shortcut::Shortcut;

/// Stable action id.
///
/// ```
/// let id = icedtea::action::ActionId::new("file.save");
/// assert_eq!(id.as_str(), "file.save");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionId(pub String);

impl ActionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A command that can appear in many chrome surfaces.
///
/// ```
/// use icedtea::action::Action;
/// let mut save = Action::new("file.save", "Save", "saved");
/// assert_eq!(save.invoke(), Some("saved"));
/// save.enabled = false;
/// assert_eq!(save.invoke(), None);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Action<M> {
    pub id: ActionId,
    pub title: String,
    pub icon: Option<Icon>,
    pub shortcut: Option<Shortcut>,
    pub tooltip: Option<String>,
    pub enabled: bool,
    pub checked: Option<bool>,
    pub message: M,
}

impl<M: Clone> Action<M> {
    pub fn new(id: impl Into<String>, title: impl Into<String>, message: M) -> Self {
        Self {
            id: ActionId::new(id),
            title: title.into(),
            icon: None,
            shortcut: None,
            tooltip: None,
            enabled: true,
            checked: None,
            message,
        }
    }

    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_shortcut(mut self, shortcut: Shortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn with_checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// Message when enabled.
    pub fn invoke(&self) -> Option<M> {
        self.enabled.then(|| self.message.clone())
    }

    pub fn search_blob(&self) -> String {
        let mut s = self.title.clone();
        s.push(' ');
        s.push_str(self.id.as_str());
        if let Some(tip) = &self.tooltip {
            s.push(' ');
            s.push_str(tip);
        }
        s
    }
}

/// Table of actions used by the palette, menus, and key dispatch.
#[derive(Debug, Clone, Default)]
pub struct ActionTable<M> {
    actions: Vec<Action<M>>,
}

impl<M: Clone> ActionTable<M> {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    pub fn insert(&mut self, action: Action<M>) {
        if let Some(existing) = self.actions.iter_mut().find(|a| a.id == action.id) {
            *existing = action;
        } else {
            self.actions.push(action);
        }
    }

    pub fn get(&self, id: &str) -> Option<&Action<M>> {
        self.actions.iter().find(|a| a.id.as_str() == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Action<M>> {
        self.actions.iter_mut().find(|a| a.id.as_str() == id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Action<M>> {
        self.actions.iter()
    }

    pub fn invoke(&self, id: &str) -> Option<M> {
        self.get(id).and_then(Action::invoke)
    }

    pub fn match_shortcut(&self, shortcut: &Shortcut) -> Option<&Action<M>> {
        self.actions.iter().find(|a| {
            a.shortcut
                .as_ref()
                .is_some_and(|s| s == shortcut && a.enabled)
        })
    }

    pub fn footer_hints(&self) -> Vec<String> {
        self.actions
            .iter()
            .filter(|a| a.enabled)
            .filter_map(|a| {
                a.shortcut
                    .as_ref()
                    .map(|s| format!("{s} {}", a.title.to_ascii_lowercase()))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shortcut::Shortcut;

    #[test]
    fn enable_invoke_and_table() {
        let mut table = ActionTable::new();
        let save = Action::new("file.save", "Save", 1u8)
            .with_icon(Icon::Check)
            .with_shortcut(Shortcut::parse("ctrl+s").unwrap())
            .with_tooltip("Write file")
            .with_checked(false);
        assert!(save.search_blob().contains("Save"));
        assert_eq!(save.invoke(), Some(1));
        table.insert(save.clone());
        table.insert(save);
        assert_eq!(table.iter().count(), 1);
        assert_eq!(table.invoke("file.save"), Some(1));
        table.get_mut("file.save").unwrap().enabled = false;
        assert_eq!(table.invoke("file.save"), None);
        assert!(table.get("missing").is_none());
        assert!(table.invoke("missing").is_none());
        let sc = Shortcut::parse("ctrl+s").unwrap();
        assert!(table.match_shortcut(&sc).is_none());
        table.get_mut("file.save").unwrap().enabled = true;
        assert!(table.match_shortcut(&sc).is_some());
        assert!(!table.footer_hints().is_empty());
        assert_eq!(ActionId::new("x").as_str(), "x");
    }
}
