//! Keyboard-first command palette over an action table.

use crate::action::{Action, ActionTable};
use crate::fuzzy;

/// Palette query + filtered results.
///
/// ```
/// use icedtea::action::{Action, ActionTable};
/// use icedtea::palette::CommandPalette;
/// let mut table = ActionTable::new();
/// table.insert(Action::new("file.save", "Save", 1u8));
/// table.insert(Action::new("file.quit", "Quit", 2u8));
/// let mut pal = CommandPalette::new();
/// pal.set_query(&table, "sa");
/// assert_eq!(pal.results(&table)[0].id.as_str(), "file.save");
/// assert_eq!(pal.invoke_selected(&table), Some(1));
/// ```
#[derive(Debug, Clone)]
pub struct CommandPalette {
    pub open: bool,
    query: String,
    selected: usize,
    hits: Vec<String>,
    pub recent: Vec<String>,
    pub favorites: Vec<String>,
    pub prompt: Option<Prompt>,
}

/// Parameter prompt opened from the palette (Go to line, rename).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    pub action: String,
    pub label: String,
    pub value: String,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        Self {
            open: false,
            query: String::new(),
            selected: 0,
            hits: Vec::new(),
            recent: Vec::new(),
            favorites: Vec::new(),
            prompt: None,
        }
    }

    pub fn pin_favorite(&mut self, id: impl Into<String>) {
        let id = id.into();
        if !self.favorites.iter().any(|f| f == &id) {
            self.favorites.push(id);
        }
    }

    pub fn remember(&mut self, id: impl Into<String>) {
        let id = id.into();
        self.recent.retain(|r| r != &id);
        self.recent.insert(0, id);
        self.recent.truncate(12);
    }

    pub fn ask(&mut self, action: impl Into<String>, label: impl Into<String>) {
        self.prompt = Some(Prompt {
            action: action.into(),
            label: label.into(),
            value: String::new(),
        });
    }

    pub fn answer(&mut self) -> Option<Prompt> {
        self.prompt.take()
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
        self.hits.clear();
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn set_query<M: Clone>(&mut self, table: &ActionTable<M>, query: impl Into<String>) {
        self.query = query.into();
        self.refresh(table);
    }

    pub fn refresh<M: Clone>(&mut self, table: &ActionTable<M>) {
        let blobs: Vec<(String, String)> = table
            .iter()
            .filter(|a| a.enabled)
            .map(|a| (a.id.0.clone(), a.search_blob()))
            .collect();
        let ranked = fuzzy::rank(&self.query, blobs.iter().map(|(_, b)| b.as_str()));
        self.hits = ranked
            .into_iter()
            .filter_map(|blob| {
                blobs
                    .iter()
                    .find(|(_, b)| b == blob)
                    .map(|(id, _)| id.clone())
            })
            .collect();
        if self.selected >= self.hits.len() {
            self.selected = 0;
        }
    }

    pub fn move_sel(&mut self, delta: i32) {
        if self.hits.is_empty() {
            self.selected = 0;
            return;
        }
        let len = self.hits.len() as i32;
        let next = (self.selected as i32 + delta).rem_euclid(len);
        self.selected = next as usize;
    }

    /// Arrow, page, home, and end move the highlight.
    pub fn apply_press(&mut self, press: &crate::key::Press, page: usize) {
        use crate::key::Press;
        if matches!(
            press,
            Press::ArrowUp
                | Press::ArrowDown
                | Press::ArrowLeft
                | Press::ArrowRight
                | Press::PageUp
                | Press::PageDown
                | Press::Home
                | Press::End
        ) {
            self.selected = press
                .clone()
                .step_index(self.selected, self.hits.len(), page);
        }
    }

    pub fn results<'a, M: Clone>(&self, table: &'a ActionTable<M>) -> Vec<&'a Action<M>> {
        self.hits.iter().filter_map(|id| table.get(id)).collect()
    }

    pub fn invoke_selected<M: Clone>(&self, table: &ActionTable<M>) -> Option<M> {
        let id = self.hits.get(self.selected)?;
        table.invoke(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;

    #[test]
    fn search_select_and_invoke() {
        let mut table = ActionTable::new();
        table.insert(Action::new("file.save", "Save", 1u8));
        table.insert(Action::new("file.quit", "Quit", 2u8));
        let mut pal = CommandPalette::default();
        pal.open();
        assert!(pal.open);
        pal.set_query(&table, "sa");
        assert_eq!(pal.query(), "sa");
        assert_eq!(pal.results(&table)[0].id.as_str(), "file.save");
        assert_eq!(pal.invoke_selected(&table), Some(1));
        pal.move_sel(1);
        pal.move_sel(-1);
        pal.set_query(&table, "");
        assert_eq!(pal.results(&table).len(), 2);
        pal.set_query(&table, "nope");
        assert!(pal.results(&table).is_empty());
        assert_eq!(pal.invoke_selected(&table), None);
        pal.move_sel(3);
        assert_eq!(pal.selected(), 0);
        pal.set_query(&table, "");
        pal.apply_press(&crate::key::Press::End, 5);
        assert_eq!(pal.selected(), 1);
        pal.apply_press(&crate::key::Press::Home, 5);
        assert_eq!(pal.selected(), 0);
        pal.apply_press(&crate::key::Press::ArrowDown, 5);
        assert_eq!(pal.selected(), 1);
        pal.apply_press(&crate::key::Press::Enter, 5);
        assert_eq!(pal.selected(), 1);
        pal.close();
        assert!(!pal.open);
        pal.remember("file.save");
        pal.remember("file.quit");
        pal.remember("file.save");
        assert_eq!(pal.recent[0], "file.save");
        pal.pin_favorite("file.save");
        pal.pin_favorite("file.save");
        assert_eq!(pal.favorites.len(), 1);
        pal.ask("go.line", "Line");
        pal.prompt.as_mut().unwrap().value = "12".into();
        let p = pal.answer().unwrap();
        assert_eq!(p.action, "go.line");
        assert_eq!(p.value, "12");
        assert!(pal.answer().is_none());
    }
}
