//! Keyboard-first command palette over an action table.

use crate::action::{Action, ActionTable};
use crate::fuzzy;
use crate::theme::Tokens;
use iced::Element;

/// How each palette row is painted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaletteFace {
    #[default]
    Default,
    Compact,
    Detail,
}

/// How hit rows are grouped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaletteGroup {
    None,
    #[default]
    Section,
    Prefix,
}

/// Empty-list copy, or omit the hits region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmptyHits<'a> {
    #[default]
    Default,
    Omit,
    Copy(&'a str),
}

impl<'a> EmptyHits<'a> {
    pub const IDLE: &'static str = "Favorites and recent appear here.";
    pub const MISS: &'static str = "No matching commands";

    /// `None` means omit the hits region (Spotlight idle, or empty Copy).
    pub fn text(self, miss: bool) -> Option<&'a str> {
        match self {
            Self::Default => Some(if miss { Self::MISS } else { Self::IDLE }),
            Self::Omit => None,
            Self::Copy("") => None,
            Self::Copy(s) => Some(s),
        }
    }
}

/// Nested page currently open in the palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PalettePage {
    pub action: String,
    pub title: String,
}

/// Optional leading or trailing row chrome (clones labels; `'static` paint).
pub type PaletteSlot<M> = fn(&Action<M>, Tokens) -> Element<'static, M>;

/// Constructor knobs for [`crate::pattern::command_palette_view`].
#[derive(Clone)]
pub struct PaletteOpts<'a, M> {
    pub face: PaletteFace,
    pub group: PaletteGroup,
    pub highlight: bool,
    pub page: Option<&'a PalettePage>,
    pub empty_idle: EmptyHits<'a>,
    pub empty_miss: EmptyHits<'a>,
    pub width: f32,
    pub max_height: f32,
    pub scroll_after: usize,
    pub scroll_height: f32,
    pub favorite_count: usize,
    pub favorites_label: &'a str,
    pub recent_label: &'a str,
    pub leading: Option<PaletteSlot<M>>,
    pub trailing: Option<PaletteSlot<M>>,
}

impl<'a, M> PaletteOpts<'a, M> {
    pub const DEFAULT_WIDTH: f32 = 560.0;
    pub const DEFAULT_MAX_HEIGHT: f32 = 420.0;
    pub const DEFAULT_SCROLL_AFTER: usize = 8;
    pub const DEFAULT_SCROLL_HEIGHT: f32 = 280.0;

    pub fn new() -> Self {
        Self {
            face: PaletteFace::Default,
            group: PaletteGroup::Section,
            highlight: true,
            page: None,
            empty_idle: EmptyHits::Default,
            empty_miss: EmptyHits::Default,
            width: Self::DEFAULT_WIDTH,
            max_height: Self::DEFAULT_MAX_HEIGHT,
            scroll_after: Self::DEFAULT_SCROLL_AFTER,
            scroll_height: Self::DEFAULT_SCROLL_HEIGHT,
            favorite_count: 0,
            favorites_label: "Favorites",
            recent_label: "Recent",
            leading: None,
            trailing: None,
        }
    }
}

impl<M> Default for PaletteOpts<'static, M> {
    fn default() -> Self {
        Self::new()
    }
}

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
    pages: Vec<PalettePage>,
}

/// Parameter the palette asks for after a command (go to line, rename).
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
            pages: Vec::new(),
        }
    }

    pub fn page(&self) -> Option<&PalettePage> {
        self.pages.last()
    }

    pub fn push_page(&mut self, action: impl Into<String>, title: impl Into<String>) {
        self.pages.push(PalettePage {
            action: action.into(),
            title: title.into(),
        });
        self.query.clear();
        self.selected = 0;
        self.hits.clear();
    }

    pub fn pop_page(&mut self) -> Option<PalettePage> {
        let page = self.pages.pop()?;
        self.query.clear();
        self.selected = 0;
        self.hits.clear();
        Some(page)
    }

    /// Favorites that are in the current idle hit list.
    pub fn favorite_hit_count(&self) -> usize {
        if !self.query.trim().is_empty() || self.page().is_some() {
            return 0;
        }
        self.favorites
            .iter()
            .filter(|id| self.hits.iter().any(|h| h == *id))
            .count()
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
        self.pages.clear();
        self.prompt = None;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn set_query<M: Clone>(&mut self, table: &ActionTable<M>, query: impl Into<String>) {
        self.query = query.into();
        self.refresh(table);
    }

    pub fn set_query_with<M, F>(
        &mut self,
        table: &ActionTable<M>,
        query: impl Into<String>,
        rank: F,
    ) where
        M: Clone,
        F: Fn(&str, &Action<M>) -> Option<u32>,
    {
        self.query = query.into();
        self.refresh_with(table, Some(rank));
    }

    pub fn refresh<M: Clone>(&mut self, table: &ActionTable<M>) {
        self.refresh_with(table, None::<fn(&str, &Action<M>) -> Option<u32>>);
    }

    pub fn refresh_with<M, F>(&mut self, table: &ActionTable<M>, rank: Option<F>)
    where
        M: Clone,
        F: Fn(&str, &Action<M>) -> Option<u32>,
    {
        let scoped: Vec<&Action<M>> = if let Some(page) = self.pages.last() {
            table
                .get(&page.action)
                .map(|parent| {
                    parent
                        .children
                        .iter()
                        .filter_map(|id| table.get(id))
                        .filter(|a| a.enabled)
                        .collect()
                })
                .unwrap_or_default()
        } else {
            table.iter().filter(|a| a.enabled).collect()
        };

        if self.query.trim().is_empty() {
            if self.pages.last().is_some() {
                self.hits = scoped.iter().map(|a| a.id.0.clone()).collect();
            } else {
                let mut hits = Vec::new();
                for id in self.favorites.iter().chain(self.recent.iter()) {
                    if table.get(id).is_some_and(|a| a.enabled) && !hits.iter().any(|h| h == id) {
                        hits.push(id.clone());
                    }
                }
                self.hits = hits;
            }
        } else {
            let q = self.query.as_str();
            let mut scored: Vec<(u32, usize, String)> = scoped
                .iter()
                .enumerate()
                .filter_map(|(i, a)| {
                    let score = match rank.as_ref() {
                        Some(f) => f(q, a)?,
                        None => fuzzy::score(q, &a.search_blob())?,
                    };
                    Some((score, i, a.id.0.clone()))
                })
                .collect();
            scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
            self.hits = scored.into_iter().map(|(_, _, id)| id).collect();
        }
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

    /// Open a child page when the row has children; otherwise invoke.
    pub fn activate<M: Clone>(&mut self, table: &ActionTable<M>, index: usize) -> Option<M> {
        let id = self.hits.get(index)?.clone();
        let action = table.get(&id)?;
        if !action.children.is_empty() {
            self.push_page(id, action.title.clone());
            self.refresh(table);
            return None;
        }
        action.invoke()
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
        assert!(pal.results(&table).is_empty());
        pal.set_query(&table, "nope");
        assert!(pal.results(&table).is_empty());
        assert_eq!(pal.invoke_selected(&table), None);
        pal.move_sel(3);
        assert_eq!(pal.selected(), 0);
        pal.set_query(&table, "file");
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
        pal.set_query(&table, "");
        let empty = pal.results(&table);
        assert_eq!(empty.len(), 2);
        assert_eq!(empty[0].id.as_str(), "file.save");
        pal.ask("go.line", "Line");
        pal.prompt.as_mut().unwrap().value = "12".into();
        let p = pal.answer().unwrap();
        assert_eq!(p.action, "go.line");
        assert_eq!(p.value, "12");
        assert!(pal.answer().is_none());
    }

    #[test]
    fn spotlight_query_keeps_icon_and_name() {
        use crate::icon::Icon;
        let mut table = ActionTable::new();
        table.insert(Action::new("app.notes", "Notes", 1u8).with_icon(Icon::Document));
        table.insert(Action::new("app.mail", "Mail", 2u8).with_icon(Icon::MailCompose));
        let mut pal = CommandPalette::new();
        pal.set_query(&table, "no");
        let hit = pal.results(&table)[0];
        assert_eq!(hit.title, "Notes");
        assert_eq!(hit.icon, Some(Icon::Document));
        assert_eq!(pal.invoke_selected(&table), Some(1));
        pal.move_sel(1);
        assert_eq!(pal.selected(), 0);
    }

    #[test]
    fn command_list_filters_and_invokes() {
        let mut table = ActionTable::new();
        table.insert(Action::new("file.save", "Save", 10u8));
        table.insert(Action::new("file.quit", "Quit", 11u8));
        let mut pal = CommandPalette::new();
        pal.set_query(&table, "qu");
        assert_eq!(pal.results(&table)[0].title, "Quit");
        assert!(pal.results(&table)[0].icon.is_none());
        assert_eq!(pal.invoke_selected(&table), Some(11));
    }

    #[test]
    fn mixed_media_rows_filter_and_invoke() {
        use crate::icon::Icon;
        let mut table = ActionTable::new();
        table.insert(
            Action::new("media.photo", "Harbor", 21u8)
                .with_icon(Icon::FileImage)
                .with_tooltip("photos/harbor.jpg"),
        );
        table.insert(
            Action::new("media.clip", "Demo reel", 22u8)
                .with_icon(Icon::FileVideo)
                .with_tooltip("videos/reel.mp4"),
        );
        table.insert(
            Action::new("media.track", "Theme", 23u8)
                .with_icon(Icon::FileAudio)
                .with_tooltip("audio/theme.ogg"),
        );
        let mut pal = CommandPalette::new();
        pal.set_query(&table, "reel");
        let hit = pal.results(&table)[0];
        assert_eq!(hit.title, "Demo reel");
        assert_eq!(hit.icon, Some(Icon::FileVideo));
        assert_eq!(hit.tooltip.as_deref(), Some("videos/reel.mp4"));
        assert_eq!(pal.invoke_selected(&table), Some(22));
    }

    #[test]
    fn search_blob_is_title_id_and_tooltip() {
        let a = Action::new("file.save", "Save", 1u8)
            .with_tooltip("Write file")
            .with_context("editor");
        let blob = a.search_blob();
        assert!(blob.contains("Save"));
        assert!(blob.contains("file.save"));
        assert!(blob.contains("Write file"));
        assert!(!blob.contains("editor"));
    }

    #[test]
    fn refresh_ranks_enabled_actions_without_using_context() {
        let mut table = ActionTable::new();
        table.insert(Action::new("file.save", "Save", 1u8).with_context("editor"));
        table.insert(Action::new("view.zoom", "Zoom", 2u8).with_context("canvas"));
        let mut pal = CommandPalette::new();
        pal.set_query(&table, "z");
        let hits = pal.results(&table);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id.as_str(), "view.zoom");
        pal.set_query(&table, "save");
        assert_eq!(pal.results(&table)[0].id.as_str(), "file.save");
    }

    #[test]
    fn empty_query_is_a_flat_favorites_then_recent_list() {
        let mut table = ActionTable::new();
        table.insert(Action::new("file.save", "Save", 1u8));
        table.insert(Action::new("file.quit", "Quit", 2u8));
        table.insert(Action::new("view.zoom", "Zoom", 3u8));
        let mut pal = CommandPalette::new();
        pal.pin_favorite("view.zoom");
        pal.remember("file.quit");
        pal.remember("file.save");
        pal.set_query(&table, "");
        let ids: Vec<_> = pal.results(&table).iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids, ["view.zoom", "file.save", "file.quit"]);
        assert_eq!(pal.favorite_hit_count(), 1);
    }

    #[test]
    fn keyword_query_ranks_aliased_action() {
        let mut table = ActionTable::new();
        table.insert(Action::new("file.save", "Save", 1u8).with_keywords(["write"]));
        table.insert(Action::new("file.quit", "Quit", 2u8));
        let mut pal = CommandPalette::new();
        pal.set_query(&table, "write");
        assert_eq!(pal.results(&table)[0].id.as_str(), "file.save");
    }

    #[test]
    fn custom_rank_overrides_fuzzy_order() {
        let mut table = ActionTable::new();
        table.insert(Action::new("file.save", "Save", 1u8));
        table.insert(Action::new("file.quit", "Quit", 2u8));
        let mut pal = CommandPalette::new();
        pal.set_query(&table, "s");
        assert_eq!(pal.results(&table)[0].id.as_str(), "file.save");
        pal.set_query_with(&table, "s", |_, a| {
            if a.id.as_str() == "file.quit" {
                Some(100)
            } else {
                Some(1)
            }
        });
        assert_eq!(pal.results(&table)[0].id.as_str(), "file.quit");
        pal.set_query(&table, "s");
        assert_eq!(pal.results(&table)[0].id.as_str(), "file.save");
    }

    #[test]
    fn activate_opens_child_page_and_keeps_query_path() {
        let mut table = ActionTable::new();
        table.insert(
            Action::new("theme", "Theme", 0u8).with_children(["theme.light", "theme.dark"]),
        );
        table.insert(Action::new("theme.light", "Light", 1u8));
        table.insert(Action::new("theme.dark", "Dark", 2u8));
        let mut pal = CommandPalette::new();
        pal.set_query(&table, "theme");
        assert!(pal.activate(&table, 0).is_none());
        assert_eq!(pal.page().unwrap().title, "Theme");
        let kids: Vec<_> = pal.results(&table).iter().map(|a| a.id.as_str()).collect();
        assert_eq!(kids, ["theme.light", "theme.dark"]);
        pal.set_query(&table, "da");
        assert_eq!(pal.activate(&table, 0), Some(2));
        pal.pop_page();
        assert!(pal.page().is_none());
        assert!(pal.pop_page().is_none());
        pal.set_query(&table, "theme");
        assert_eq!(pal.favorite_hit_count(), 0);
        pal.push_page("theme", "Theme");
        pal.refresh(&table);
        assert_eq!(pal.favorite_hit_count(), 0);
    }

    #[test]
    fn empty_hits_and_opts_defaults() {
        use crate::palette::{EmptyHits, PaletteOpts};
        assert_eq!(EmptyHits::Default.text(false), Some(EmptyHits::IDLE));
        assert_eq!(EmptyHits::Default.text(true), Some(EmptyHits::MISS));
        assert_eq!(EmptyHits::Omit.text(false), None);
        assert_eq!(EmptyHits::Copy("").text(false), None);
        assert_eq!(EmptyHits::Copy("None found").text(true), Some("None found"));
        let opts = PaletteOpts::<()>::default();
        assert_eq!(opts.width, PaletteOpts::<()>::DEFAULT_WIDTH);
        assert_eq!(opts.scroll_after, PaletteOpts::<()>::DEFAULT_SCROLL_AFTER);
    }
}
