//! Public widget and pattern ids. The gallery must page every entry.
//!
//! Each id has one constructor. Widgets take [`crate::a11y::A11y`] and
//! tokens. Chrome rows take an [`crate::action::ActionTable`].
//!
//! ```
//! assert!(icedtea::catalog::get("button").is_some());
//! assert_eq!(icedtea::catalog::get("button").unwrap().page, "controls");
//! ```

/// Every exported control or pattern. Gallery pages this list.
///
/// ```
/// assert!(icedtea::catalog::ENTRIES.iter().any(|e| e.id == "button"));
/// assert!(icedtea::catalog::ENTRIES.iter().any(|e| e.id == "about"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    pub id: &'static str,
    pub title: &'static str,
    pub group: &'static str,
    /// Gallery page that hosts this widget. Several ids can share a page.
    pub page: &'static str,
}

/// Closed list, like iced's `Theme::ALL`. Named fields so id / title /
/// group cannot be swapped. One row per public surface.
#[rustfmt::skip]
pub const ENTRIES: &[Entry] = &[
    Entry { id: "button", title: "Button", group: "Controls", page: "controls" },
    Entry { id: "split-button", title: "Split button", group: "Controls", page: "controls" },
    Entry { id: "toggle-button", title: "Toggle button", group: "Controls", page: "controls" },
    Entry { id: "checkbox", title: "Checkbox", group: "Controls", page: "controls" },
    Entry { id: "radio", title: "Radio", group: "Controls", page: "controls" },
    Entry { id: "switch", title: "Switch", group: "Controls", page: "controls" },
    Entry { id: "slider", title: "Slider", group: "Controls", page: "controls" },
    Entry { id: "text-input", title: "Text input", group: "Fields", page: "fields" },
    Entry { id: "password", title: "Password", group: "Fields", page: "fields" },
    Entry { id: "secret", title: "Secret field", group: "Fields", page: "fields" },
    Entry { id: "textarea", title: "Text area", group: "Fields", page: "fields" },
    Entry { id: "search", title: "Search", group: "Fields", page: "fields" },
    Entry { id: "suggest", title: "Suggest", group: "Fields", page: "fields" },
    Entry { id: "select", title: "Select", group: "Fields", page: "fields" },
    Entry { id: "number", title: "Number", group: "Fields", page: "fields" },
    Entry { id: "mask", title: "Masked field", group: "Fields", page: "fields" },
    Entry { id: "date", title: "Date", group: "Fields", page: "fields" },
    Entry { id: "time", title: "Time", group: "Fields", page: "fields" },
    Entry { id: "color", title: "Color", group: "Fields", page: "fields" },
    Entry { id: "progress", title: "Progress", group: "Readout", page: "readout" },
    Entry { id: "progress-ring", title: "Progress ring", group: "Readout", page: "readout" },
    Entry { id: "sparkline", title: "Sparkline", group: "Readout", page: "readout" },
    Entry { id: "spinner", title: "Spinner", group: "Readout", page: "readout" },
    Entry { id: "display", title: "Display reading", group: "Readout", page: "readout" },
    Entry { id: "label", title: "Label", group: "Content", page: "type" },
    Entry { id: "rich-cell", title: "Rich cell", group: "Content", page: "type" },
    Entry { id: "icon", title: "Icon", group: "Content", page: "type" },
    Entry { id: "tooltip", title: "Tooltip", group: "Content", page: "type" },
    Entry { id: "link", title: "Hyperlink", group: "Content", page: "type" },
    Entry { id: "markdown", title: "Markdown", group: "Content", page: "markdown" },
    Entry { id: "code", title: "Code", group: "Content", page: "code" },
    Entry { id: "image", title: "Image", group: "Content", page: "image" },
    Entry { id: "list", title: "List", group: "Collections", page: "list" },
    Entry { id: "log", title: "Log", group: "Collections", page: "log" },
    Entry { id: "grid", title: "Item grid", group: "Collections", page: "grid" },
    Entry { id: "table", title: "Data table", group: "Collections", page: "table" },
    Entry { id: "tree", title: "Tree", group: "Collections", page: "tree" },
    Entry { id: "tabs", title: "Tabs", group: "Collections", page: "sections" },
    Entry { id: "accordion", title: "Accordion", group: "Collections", page: "sections" },
    Entry { id: "pagination", title: "Pagination", group: "Collections", page: "sections" },
    Entry { id: "theme", title: "Theme", group: "Chrome", page: "theme" },
    Entry { id: "colors", title: "Colors", group: "Chrome", page: "colors" },
    Entry { id: "keys", title: "Keys", group: "Chrome", page: "keys" },
    Entry { id: "card", title: "Card", group: "Chrome", page: "marks" },
    Entry { id: "rule", title: "Rule", group: "Chrome", page: "marks" },
    Entry { id: "chip", title: "Chip", group: "Chrome", page: "marks" },
    Entry { id: "badge", title: "Badge", group: "Chrome", page: "marks" },
    Entry { id: "wrap", title: "Wrap", group: "Chrome", page: "marks" },
    Entry { id: "pad", title: "Pad", group: "Chrome", page: "marks" },
    Entry { id: "callout", title: "Callout", group: "Chrome", page: "marks" },
    Entry { id: "banner", title: "Banner", group: "Chrome", page: "marks" },
    Entry { id: "group-box", title: "Group box", group: "Chrome", page: "marks" },
    Entry { id: "skeleton", title: "Skeleton", group: "Chrome", page: "marks" },
    Entry { id: "teaching-tip", title: "Teaching tip", group: "Chrome", page: "marks" },
    Entry { id: "command-bar", title: "Command bar", group: "Chrome", page: "chrome-rows" },
    Entry { id: "context-menu", title: "Context menu", group: "Chrome", page: "chrome-rows" },
    Entry { id: "breadcrumb", title: "Breadcrumb", group: "Chrome", page: "chrome-rows" },
    Entry { id: "menu", title: "Menu", group: "Chrome", page: "chrome-rows" },
    Entry { id: "toolbar", title: "Toolbar", group: "Chrome", page: "chrome-rows" },
    Entry { id: "status-bar", title: "Status bar", group: "Chrome", page: "chrome-rows" },
    Entry { id: "scrollbar", title: "Scrollbar", group: "Chrome", page: "feedback" },
    Entry { id: "toast", title: "Toast", group: "Chrome", page: "feedback" },
    Entry { id: "busy", title: "Busy overlay", group: "Chrome", page: "feedback" },
    Entry { id: "dialogs", title: "Dialogs", group: "Patterns", page: "dialogs" },
    Entry { id: "list-detail", title: "List/detail", group: "Patterns", page: "list-detail" },
    Entry { id: "navigation", title: "Navigation view", group: "Patterns", page: "navigation" },
    Entry { id: "tab-view", title: "Tab view", group: "Patterns", page: "tab-view" },
    Entry { id: "preferences", title: "Preferences", group: "Patterns", page: "preferences" },
    Entry { id: "about", title: "About", group: "Patterns", page: "about" },
    Entry { id: "status-page", title: "Status page", group: "Patterns", page: "status-page" },
    Entry { id: "palette", title: "Command palette", group: "Patterns", page: "palette" },
    Entry { id: "main-window", title: "Main window", group: "Patterns", page: "main-window" },
];

pub fn get(id: &str) -> Option<&'static Entry> {
    ENTRIES.iter().find(|e| e.id == id)
}

pub fn groups() -> Vec<&'static str> {
    let mut g = Vec::new();
    for e in ENTRIES {
        if !g.contains(&e.group) {
            g.push(e.group);
        }
    }
    g
}

/// Gallery page ids in catalog order. Several widgets can share a page.
pub fn pages() -> Vec<&'static str> {
    let mut p = Vec::new();
    for e in ENTRIES {
        if !p.contains(&e.page) {
            p.push(e.page);
        }
    }
    p
}

pub fn page_entries(page: &str) -> impl Iterator<Item = &'static Entry> + '_ {
    ENTRIES.iter().filter(move |e| e.page == page)
}

pub fn page_title(page: &str) -> &'static str {
    match page {
        "controls" => "Controls",
        "fields" => "Fields",
        "readout" => "Readout",
        "type" => "Type",
        "sections" => "Tabs and sections",
        "marks" => "Marks",
        "chrome-rows" => "Chrome rows",
        "feedback" => "Feedback",
        id => get(id).map(|e| e.title).unwrap_or("Page"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_covers_public_surfaces() {
        assert!(get("button").is_some());
        assert!(get("about").is_some());
        assert!(get("missing").is_none());
        assert_eq!(
            groups(),
            [
                "Controls",
                "Fields",
                "Readout",
                "Content",
                "Collections",
                "Chrome",
                "Patterns"
            ]
        );
        assert!(ENTRIES.len() >= 40);
        assert_eq!(get("table").unwrap().group, "Collections");
        assert_eq!(get("theme").unwrap().group, "Chrome");
        assert_eq!(get("time").unwrap().group, "Fields");
        assert_eq!(get("button").unwrap().page, "controls");
        assert_eq!(get("checkbox").unwrap().page, "controls");
        assert_eq!(page_title("controls"), "Controls");
        assert!(pages().contains(&"controls"));
        assert!(page_entries("controls").count() > 1);
        assert!(pages().len() < ENTRIES.len());
        for id in [
            "image",
            "progress-ring",
            "wrap",
            "badge",
            "command-bar",
            "context-menu",
            "scrollbar",
            "display",
            "pad",
            "rich-cell",
            "colors",
            "keys",
        ] {
            assert!(get(id).is_some());
        }
        for name in [
            "install.md",
            "introduction.md",
            "first-window.md",
            "widgets.md",
            "actions.md",
            "layout.md",
            "theming.md",
            "architecture.md",
            "navigation.md",
            "overlay-windows.md",
            "compact-tools.md",
        ] {
            let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("book/src")
                .join(name);
            assert!(p.is_file());
        }
    }

    #[test]
    fn every_catalog_id_has_one_shipped_constructor() {
        let widget = include_str!("widget.rs");
        let pattern = include_str!("pattern.rs");
        let theme = include_str!("theme.rs");
        let key = include_str!("key.rs");
        let layout = include_str!("layout/recipes.rs");
        let map = [
            ("button", "pub fn themed_button", widget),
            ("split-button", "pub fn split_button", widget),
            ("toggle-button", "pub fn toggle_button", widget),
            ("checkbox", "pub fn themed_checkbox", widget),
            ("radio", "pub fn themed_radio", widget),
            ("switch", "pub fn themed_switch", widget),
            ("slider", "pub fn themed_slider", widget),
            ("text-input", "pub fn themed_text_input", widget),
            ("password", "pub fn password_input", widget),
            ("secret", "pub fn secret_field", widget),
            ("textarea", "pub fn textarea", widget),
            ("search", "pub fn search_input", widget),
            ("suggest", "pub fn suggest_field", widget),
            ("select", "pub fn themed_pick_list", widget),
            ("number", "pub fn number_input", widget),
            ("mask", "pub fn masked_input", widget),
            ("date", "pub fn date_picker", widget),
            ("time", "pub fn time_picker", widget),
            ("color", "pub fn color_swatch", widget),
            ("progress", "pub fn progress<", widget),
            ("progress-ring", "pub fn progress_ring", widget),
            ("sparkline", "pub fn sparkline", widget),
            ("spinner", "pub fn spinner", widget),
            ("display", "pub fn display_reading", widget),
            ("label", "pub fn label", widget),
            ("rich-cell", "pub fn rich_cell", widget),
            ("icon", "pub fn icon_svg", widget),
            ("tooltip", "pub fn tooltip_wrap", widget),
            ("link", "pub fn hyperlink", widget),
            ("markdown", "pub fn markdown_view", widget),
            ("code", "pub fn highlighted_code", widget),
            ("image", "pub fn image_slot", widget),
            ("list", "pub fn list_view", widget),
            ("log", "pub fn log_view", widget),
            ("grid", "pub fn item_grid", widget),
            ("table", "pub fn data_table", widget),
            ("tree", "pub fn tree_view", widget),
            ("tabs", "pub fn tab_bar", widget),
            ("accordion", "pub fn accordion_view", widget),
            ("pagination", "pub fn pagination", widget),
            ("theme", "pub fn named", theme),
            ("colors", "pub fn mix", theme),
            ("keys", "pub fn handle", key),
            ("card", "pub fn group_box", widget),
            ("rule", "pub fn rule_h", widget),
            ("chip", "pub fn chip", widget),
            ("badge", "pub fn badge", widget),
            ("wrap", "pub fn wrap", layout),
            ("pad", "pub fn pad", layout),
            ("callout", "pub fn info_bar", widget),
            ("banner", "pub fn banner", widget),
            ("group-box", "pub fn group_box", widget),
            ("skeleton", "pub fn placeholder_skeleton", widget),
            ("teaching-tip", "pub fn teaching_tip", widget),
            ("command-bar", "pub fn command_bar", pattern),
            ("context-menu", "pub fn context_menu", pattern),
            ("breadcrumb", "pub fn breadcrumb", widget),
            ("menu", "pub fn menu_bar", pattern),
            ("toolbar", "pub fn toolbar", pattern),
            ("status-bar", "pub fn status_bar", pattern),
            ("scrollbar", "pub fn themed_scroll", widget),
            ("toast", "pub fn toast_view", widget),
            ("busy", "pub fn busy_overlay", widget),
            ("dialogs", "pub fn dialog_sheet", pattern),
            ("list-detail", "pub fn list_detail", pattern),
            ("navigation", "pub fn navigation_view", pattern),
            ("tab-view", "pub fn tab_view", pattern),
            ("preferences", "pub fn preferences_page", pattern),
            ("about", "pub fn about_page", pattern),
            ("status-page", "pub fn status_page", pattern),
            ("palette", "pub fn command_palette_view", pattern),
            ("main-window", "pub fn main_window", pattern),
        ];
        assert_eq!(map.len(), ENTRIES.len());
        for e in ENTRIES {
            let hit = map.iter().find(|(id, _, _)| *id == e.id);
            let (_, needle, src) = hit.expect(e.id);
            assert!(
                src.contains(needle),
                "{} missing constructor {}",
                e.id,
                needle
            );
            let at = src.find(needle).unwrap();
            let window = &src[at.saturating_sub(700)..at];
            assert!(
                window.contains("```"),
                "{} constructor {} needs a rustdoc example",
                e.id,
                needle
            );
        }
    }

    #[test]
    fn collapsed_dual_paths_are_not_public() {
        let widget = include_str!("widget.rs");
        assert!(
            !widget.contains("pub fn image<"),
            "image_slot is the image constructor"
        );
        let key = include_str!("key.rs");
        assert!(
            !key.contains("pub fn listen_raw"),
            "listen is the keyboard subscription"
        );
        let recipes = include_str!("layout/recipes.rs");
        assert!(
            !recipes.contains("pub fn scroll_y<"),
            "themed_scroll is the scroll constructor"
        );
        assert!(
            !recipes.contains("pub fn sidebar_mode"),
            "Breakpoint::from_width picks the sidebar recipe"
        );
    }
}
