//! Named read-only text the user can select and copy.
//!
//! [`Selectables`] is UI state, not a document store. Bind a source
//! string; rebind is a no-op when the text is unchanged so a live
//! refresh keeps the highlight. App-facing select-and-copy contract:
//! [`crate::select`].

use iced::widget::text_editor::{self, Content};

use crate::widget::select_only;

/// Map of id to selectable buffer.
///
/// ```
/// use icedtea::field::Selectables;
/// use icedtea::iced::widget::text_editor::Action;
/// let mut fields = Selectables::new();
/// fields.bind("path", "sessions/a/transcript.jsonl");
/// fields.bind("body", "hello");
/// fields.perform("body", Action::SelectAll);
/// assert_eq!(fields.copy("path"), "sessions/a/transcript.jsonl");
/// assert_eq!(fields.first_selection().as_deref(), Some("hello"));
/// assert!(fields.contains("path"));
/// assert!(fields.get("missing").is_none());
/// fields.perform("missing", Action::SelectAll);
/// fields.bind("body", "hello");
/// assert_eq!(fields.first_selection().as_deref(), Some("hello"));
/// ```
#[derive(Debug, Default)]
pub struct Selectables {
    items: Vec<(String, Content)>,
}

impl Selectables {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace `id`. Same text leaves the buffer (and
    /// selection) alone.
    pub fn bind(&mut self, id: impl Into<String>, text: impl AsRef<str>) {
        let id = id.into();
        let text = text.as_ref();
        if let Some((_, content)) = self.items.iter_mut().find(|(k, _)| *k == id) {
            if content.text() == text {
                return;
            }
            *content = Content::with_text(text);
            return;
        }
        self.items.push((id, Content::with_text(text)));
    }

    /// Bind `id` if missing or the text changed; return the buffer.
    ///
    /// For expand cards: call when a row opens so load is not O(n) on
    /// every id up front.
    pub fn ensure(&mut self, id: impl Into<String>, text: impl AsRef<str>) -> &Content {
        let id = id.into();
        self.bind(id.clone(), text);
        self.get(&id).expect("ensure just bound id")
    }

    /// Drop ids for which `keep` returns false (closed cards, recycled rows).
    pub fn retain(&mut self, mut keep: impl FnMut(&str) -> bool) {
        self.items.retain(|(k, _)| keep(k.as_str()));
    }

    /// Remove one id. Returns whether it was bound.
    pub fn unbind(&mut self, id: &str) -> bool {
        if let Some(i) = self.items.iter().position(|(k, _)| k == id) {
            self.items.remove(i);
            true
        } else {
            false
        }
    }

    /// Whether `id` is bound.
    pub fn contains(&self, id: &str) -> bool {
        self.items.iter().any(|(k, _)| k == id)
    }

    /// Buffer for a bound id.
    pub fn get(&self, id: &str) -> Option<&Content> {
        self.items.iter().find(|(k, _)| k == id).map(|(_, c)| c)
    }

    /// Mutable buffer for a bound id.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Content> {
        self.items.iter_mut().find(|(k, _)| k == id).map(|(_, c)| c)
    }

    /// Apply a pointer or key action. Typing is dropped. Unbound is a
    /// no-op.
    pub fn perform(&mut self, id: &str, action: text_editor::Action) {
        if let Some(content) = self.get_mut(id) {
            content.perform(select_only(action));
        }
    }

    /// Selection if any, otherwise the full text. Unbound is empty.
    pub fn copy(&self, id: &str) -> String {
        self.get(id)
            .map(|content| content.selection().unwrap_or_else(|| content.text()))
            .unwrap_or_default()
    }

    /// First non-empty selection in bind order.
    pub fn first_selection(&self) -> Option<String> {
        self.items.iter().find_map(|(_, c)| c.selection())
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::text_editor::{Action, Edit};

    #[test]
    fn bind_get_and_copy_full_text() {
        let mut fields = Selectables::new();
        assert!(fields.is_empty());
        fields.bind("path", "a/b");
        assert_eq!(fields.len(), 1);
        assert!(fields.contains("path"));
        assert_eq!(fields.get("path").map(|c| c.text()), Some("a/b".into()));
        assert_eq!(fields.copy("path"), "a/b");
        assert_eq!(fields.first_selection(), None);
    }

    #[test]
    fn bind_same_text_keeps_selection() {
        let mut fields = Selectables::new();
        fields.bind("body", "hello");
        fields.perform("body", Action::SelectAll);
        assert_eq!(fields.first_selection().as_deref(), Some("hello"));
        fields.bind("body", "hello");
        assert_eq!(fields.first_selection().as_deref(), Some("hello"));
        assert_eq!(fields.copy("body"), "hello");
    }

    #[test]
    fn bind_new_text_replaces_the_buffer() {
        let mut fields = Selectables::new();
        fields.bind("body", "hello");
        fields.perform("body", Action::SelectAll);
        fields.bind("body", "other");
        assert_eq!(fields.get("body").map(|c| c.text()), Some("other".into()));
        assert_eq!(fields.first_selection(), None);
        assert_eq!(fields.copy("body"), "other");
    }

    #[test]
    fn perform_drops_typing() {
        let mut fields = Selectables::new();
        fields.bind("body", "keep");
        fields.perform("body", Action::Edit(Edit::Insert('x')));
        assert_eq!(fields.get("body").map(|c| c.text()), Some("keep".into()));
    }

    #[test]
    fn a_left_click_clears_the_range() {
        let mut fields = Selectables::new();
        fields.bind("body", "alpha beta");
        fields.perform("body", Action::SelectAll);
        assert_eq!(fields.first_selection().as_deref(), Some("alpha beta"));
        fields.perform("body", Action::Click(iced::Point::new(4.0, 0.0)));
        assert_eq!(fields.first_selection(), None);
        assert_eq!(fields.copy("body"), "alpha beta");
    }

    #[test]
    fn first_selection_walks_bind_order() {
        let mut fields = Selectables::new();
        fields.bind("path", "p");
        fields.bind("body", "hello");
        fields.perform("body", Action::SelectAll);
        assert_eq!(fields.first_selection().as_deref(), Some("hello"));
        fields.perform("path", Action::SelectAll);
        assert_eq!(fields.first_selection().as_deref(), Some("p"));
        let _ = fields.get_mut("body");
    }

    #[test]
    fn unbound_get_is_none_and_perform_is_noop() {
        let mut fields = Selectables::new();
        assert!(!fields.contains("missing"));
        assert!(fields.get("missing").is_none());
        assert!(fields.get_mut("missing").is_none());
        fields.perform("missing", Action::SelectAll);
        assert_eq!(fields.copy("missing"), "");
        assert!(fields.is_empty());
    }

    #[test]
    fn ensure_binds_lazily_and_retain_drops_closed() {
        let mut fields = Selectables::new();
        assert_eq!(fields.ensure("card.3", "body three").text(), "body three");
        assert!(fields.contains("card.3"));
        fields.ensure("card.1", "one");
        fields.ensure("card.9", "nine");
        fields.retain(|id| id == "card.3" || id == "card.1");
        assert!(fields.contains("card.3"));
        assert!(fields.contains("card.1"));
        assert!(!fields.contains("card.9"));
        assert!(fields.unbind("card.1"));
        assert!(!fields.unbind("card.1"));
        assert_eq!(fields.len(), 1);
    }
}
