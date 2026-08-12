//! Named read-only text the user can select and copy.
//!
//! [`Selectables`] is UI state, not a document store. Bind a source
//! string; rebind is a no-op when the text is unchanged so a live
//! refresh keeps the highlight.

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

    /// Buffer for a bound id.
    pub fn get(&self, id: &str) -> &Content {
        self.items
            .iter()
            .find(|(k, _)| k == id)
            .map(|(_, c)| c)
            .unwrap_or_else(|| panic!("unbound selectable {id}"))
    }

    /// Mutable buffer for a bound id.
    pub fn get_mut(&mut self, id: &str) -> &mut Content {
        self.items
            .iter_mut()
            .find(|(k, _)| k == id)
            .map(|(_, c)| c)
            .unwrap_or_else(|| panic!("unbound selectable {id}"))
    }

    /// Apply a pointer or key action. Typing is dropped.
    pub fn perform(&mut self, id: &str, action: text_editor::Action) {
        self.get_mut(id).perform(select_only(action));
    }

    /// Selection if any, otherwise the full text.
    pub fn copy(&self, id: &str) -> String {
        let content = self.get(id);
        content.selection().unwrap_or_else(|| content.text())
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
        assert_eq!(fields.get("path").text(), "a/b");
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
        assert_eq!(fields.get("body").text(), "other");
        assert_eq!(fields.first_selection(), None);
        assert_eq!(fields.copy("body"), "other");
    }

    #[test]
    fn perform_drops_typing() {
        let mut fields = Selectables::new();
        fields.bind("body", "keep");
        fields.perform("body", Action::Edit(Edit::Insert('x')));
        assert_eq!(fields.get("body").text(), "keep");
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
    #[should_panic(expected = "unbound selectable missing")]
    fn get_requires_a_bound_id() {
        let fields = Selectables::new();
        let _ = fields.get("missing");
    }

    #[test]
    #[should_panic(expected = "unbound selectable missing")]
    fn perform_requires_a_bound_id() {
        let mut fields = Selectables::new();
        fields.perform("missing", Action::SelectAll);
    }
}
