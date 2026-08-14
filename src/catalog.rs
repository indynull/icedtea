//! Public widget and pattern ids.
//!
//! Each id has one constructor. That `pub fn` takes [`crate::a11y::A11y`]
//! and tokens. Chrome rows take an [`crate::action::ActionTable`].
//! Rustdoc with a working example sits immediately above the function.
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
    // Button faces, toggle-icon, and slider sit first so idle QA shows
    // Outlined / Elevated, icon-plus-label, toggle-icon, and the vertical
    // slider above the fold.
    Entry { id: "button", title: "Button", group: "Controls", page: "controls" },
    Entry { id: "toggle-icon-button", title: "Toggle icon button", group: "Controls", page: "controls" },
    Entry { id: "slider", title: "Slider", group: "Controls", page: "controls" },
    Entry { id: "button-group", title: "Button group", group: "Controls", page: "controls" },
    Entry { id: "checkbox", title: "Checkbox", group: "Controls", page: "controls" },
    Entry { id: "radio", title: "Radio", group: "Controls", page: "controls" },
    Entry { id: "switch", title: "Switch", group: "Controls", page: "controls" },
    Entry { id: "range-slider", title: "Range slider", group: "Controls", page: "controls" },
    Entry { id: "segmented-button", title: "Segmented button", group: "Controls", page: "controls" },
    Entry { id: "icon-button", title: "Icon button", group: "Controls", page: "controls" },
    Entry { id: "checkbox-indeterminate", title: "Indeterminate checkbox", group: "Controls", page: "controls" },
    Entry { id: "split-button", title: "Split button", group: "Controls", page: "controls" },
    Entry { id: "toggle-button", title: "Toggle button", group: "Controls", page: "controls" },
    // Search view first so docked hits are above the fold on Fields.
    Entry { id: "search-view", title: "Search view", group: "Fields", page: "fields" },
    Entry { id: "text-input", title: "Text input", group: "Fields", page: "fields" },
    Entry { id: "field-support", title: "Field support", group: "Fields", page: "fields" },
    Entry { id: "password", title: "Password", group: "Fields", page: "fields" },
    Entry { id: "secret", title: "Secret field", group: "Fields", page: "fields" },
    Entry { id: "value-field", title: "Value field", group: "Fields", page: "fields" },
    Entry { id: "textarea", title: "Text area", group: "Fields", page: "fields" },
    Entry { id: "search", title: "Search", group: "Fields", page: "fields" },
    Entry { id: "suggest", title: "Suggest", group: "Fields", page: "fields" },
    Entry { id: "select", title: "Select", group: "Fields", page: "fields" },
    Entry { id: "number", title: "Number", group: "Fields", page: "fields" },
    Entry { id: "date", title: "Date", group: "Fields", page: "fields" },
    Entry { id: "time", title: "Time", group: "Fields", page: "fields" },
    Entry { id: "progress", title: "Progress", group: "Readout", page: "readout" },
    Entry { id: "progress-ring", title: "Progress ring", group: "Readout", page: "readout" },
    Entry { id: "spinner", title: "Spinner", group: "Readout", page: "readout" },
    Entry { id: "label", title: "Label", group: "Content", page: "type" },
    Entry { id: "icon", title: "Icon", group: "Content", page: "type" },
    Entry { id: "tooltip", title: "Tooltip", group: "Content", page: "type" },
    Entry { id: "rich-tooltip", title: "Rich tooltip", group: "Content", page: "type" },
    Entry { id: "link", title: "Hyperlink", group: "Content", page: "type" },
    Entry { id: "markdown", title: "Markdown", group: "Content", page: "markdown" },
    Entry { id: "code", title: "Code", group: "Content", page: "code" },
    Entry { id: "image", title: "Image", group: "Content", page: "image" },
    Entry { id: "selectable", title: "Selectable", group: "Content", page: "selectable" },
    // Virtual column first so open-face inject is above the fold.
    Entry { id: "virtual-column", title: "Virtual column", group: "Collections", page: "list" },
    Entry { id: "list", title: "List", group: "Collections", page: "list" },
    // Pages a large set; lives with list, not disclosure chrome.
    Entry { id: "pagination", title: "Pagination", group: "Collections", page: "list" },
    Entry { id: "log", title: "Log", group: "Collections", page: "log" },
    Entry { id: "grid", title: "Item grid", group: "Collections", page: "grid" },
    Entry { id: "table", title: "Data table", group: "Collections", page: "table" },
    Entry { id: "tree", title: "Tree", group: "Collections", page: "tree" },
    // Accordion and expander above the fold for inject proof.
    Entry { id: "accordion", title: "Accordion", group: "Collections", page: "sections" },
    Entry { id: "expander", title: "Expander", group: "Collections", page: "sections" },
    Entry { id: "tabs", title: "Tabs", group: "Collections", page: "sections" },
    Entry { id: "theme", title: "Theme", group: "Chrome", page: "theme" },
    Entry { id: "colors", title: "Colors", group: "Chrome", page: "colors" },
    Entry { id: "keys", title: "Keys", group: "Chrome", page: "keys" },
    Entry { id: "cheatsheet", title: "Cheatsheet", group: "Chrome", page: "keys" },
    // Filter chips first so selected/idle faces are above the fold on Marks.
    Entry { id: "filter-chips", title: "Filter chips", group: "Chrome", page: "marks" },
    Entry { id: "chip", title: "Chip", group: "Chrome", page: "marks" },
    Entry { id: "badge", title: "Badge", group: "Chrome", page: "marks" },
    Entry { id: "card", title: "Card", group: "Chrome", page: "marks" },
    Entry { id: "rule", title: "Rule", group: "Chrome", page: "marks" },
    Entry { id: "wrap", title: "Wrap", group: "Chrome", page: "marks" },
    Entry { id: "banner", title: "Banner", group: "Chrome", page: "marks" },
    Entry { id: "command-bar", title: "Command bar", group: "Chrome", page: "chrome-rows" },
    Entry { id: "context-menu", title: "Context menu", group: "Chrome", page: "chrome-rows" },
    Entry { id: "sectioned-menu", title: "Sectioned menu", group: "Chrome", page: "chrome-rows" },
    Entry { id: "cascade-menu", title: "Cascade menu", group: "Chrome", page: "chrome-rows" },
    Entry { id: "breadcrumb", title: "Breadcrumb", group: "Chrome", page: "chrome-rows" },
    Entry { id: "menu", title: "Menu", group: "Chrome", page: "chrome-rows" },
    Entry { id: "toolbar", title: "Toolbar", group: "Chrome", page: "chrome-rows" },
    Entry { id: "status-bar", title: "Status bar", group: "Chrome", page: "chrome-rows" },
    Entry { id: "busy", title: "Busy overlay", group: "Chrome", page: "feedback" },
    Entry { id: "toast", title: "Toast", group: "Chrome", page: "feedback" },
    Entry { id: "scrollbar", title: "Scrollbar", group: "Chrome", page: "feedback" },
    // Side sheet first so open-state inject is above the fold on Dialogs.
    Entry { id: "side-sheet", title: "Side sheet", group: "Patterns", page: "dialogs" },
    Entry { id: "dialogs", title: "Dialogs", group: "Patterns", page: "dialogs" },
    Entry { id: "list-detail", title: "List/detail", group: "Patterns", page: "list-detail" },
    Entry { id: "inspector", title: "Inspector", group: "Patterns", page: "inspector" },
    Entry { id: "drawer", title: "Drawer", group: "Patterns", page: "workspace" },
    Entry { id: "workspace", title: "Workspace", group: "Patterns", page: "workspace" },
    Entry { id: "tool-panel", title: "Tool panel", group: "Patterns", page: "workspace" },
    Entry { id: "nav-rail", title: "Navigation rail", group: "Patterns", page: "navigation" },
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
        "workspace" => "Workspace",
        id => get(id).map(|e| e.title).unwrap_or("Page"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn must(ok: bool, msg: impl std::fmt::Display) {
        if !ok {
            panic!("{msg}");
        }
    }

    #[test]
    #[should_panic(expected = "cover-must")]
    fn must_rejects_a_failed_check() {
        must(false, "cover-must");
    }

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
        for page in pages() {
            let title = page_title(page);
            assert!(!title.is_empty());
            assert_ne!(title, "Page", "{page}");
        }
        assert_eq!(page_title("no-such-page"), "Page");
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
            "colors",
            "keys",
            "button",
            "list",
        ] {
            assert!(get(id).is_some());
        }
        for id in crate::m3::mapping::deleted_ids() {
            assert!(get(id).is_none());
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
            "reference/controls.md",
            "reference/fields.md",
            "reference/readout.md",
            "reference/content.md",
            "reference/collections.md",
            "reference/chrome.md",
            "reference/patterns.md",
        ] {
            let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("book/src")
                .join(name);
            assert!(p.is_file());
        }
    }

    #[test]
    fn handbook_describes_every_catalog_id() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("book/src");
        let group_file = [
            ("Controls", "reference/controls.md"),
            ("Fields", "reference/fields.md"),
            ("Readout", "reference/readout.md"),
            ("Content", "reference/content.md"),
            ("Collections", "reference/collections.md"),
            ("Chrome", "reference/chrome.md"),
            ("Patterns", "reference/patterns.md"),
        ];
        let ctors = [
            ("button", "themed_button"),
            ("segmented-button", "segmented_button"),
            ("button-group", "button_group"),
            ("icon-button", "icon_button"),
            ("toggle-icon-button", "icon_button_toggle"),
            ("split-button", "split_button"),
            ("toggle-button", "toggle_button"),
            ("checkbox", "themed_checkbox"),
            ("checkbox-indeterminate", "checkbox_indeterminate"),
            ("radio", "themed_radio"),
            ("switch", "themed_switch"),
            ("slider", "themed_slider"),
            ("range-slider", "range_slider"),
            ("text-input", "themed_text_input"),
            ("field-support", "field_support"),
            ("password", "password_input"),
            ("secret", "secret_field"),
            ("value-field", "value_field"),
            ("textarea", "textarea"),
            ("search", "search_input"),
            ("search-view", "search_view"),
            ("suggest", "suggest_field"),
            ("select", "themed_pick_list"),
            ("number", "number_input"),
            ("date", "date_picker"),
            ("time", "time_picker"),
            ("progress", "progress"),
            ("progress-ring", "progress_ring"),
            ("spinner", "spinner"),
            ("label", "label"),
            ("icon", "icon_svg"),
            ("tooltip", "tooltip_wrap"),
            ("rich-tooltip", "tooltip_rich"),
            ("link", "hyperlink"),
            ("markdown", "markdown_view"),
            ("code", "highlighted_code"),
            ("image", "image_slot"),
            ("selectable", "selectable"),
            ("list", "list_view"),
            ("virtual-column", "virtual_column"),
            ("log", "log_view"),
            ("grid", "item_grid"),
            ("table", "data_table"),
            ("tree", "tree_view"),
            ("tabs", "tab_bar"),
            ("accordion", "accordion_view"),
            ("expander", "expander"),
            ("pagination", "pagination"),
            ("theme", "named"),
            ("colors", "mix"),
            ("keys", "handle"),
            ("cheatsheet", "cheatsheet"),
            ("card", "group_box"),
            ("rule", "rule_h"),
            ("chip", "chip"),
            ("filter-chips", "filter_chips"),
            ("badge", "badge"),
            ("wrap", "wrap"),
            ("banner", "banner"),
            ("command-bar", "command_bar"),
            ("context-menu", "context_menu"),
            ("sectioned-menu", "sectioned_menu"),
            ("cascade-menu", "cascade_menu"),
            ("breadcrumb", "breadcrumb"),
            ("menu", "menu_bar"),
            ("toolbar", "toolbar"),
            ("status-bar", "status_bar"),
            ("busy", "busy_overlay"),
            ("toast", "toast_view"),
            ("scrollbar", "themed_scroll"),
            ("dialogs", "dialog_sheet"),
            ("side-sheet", "side_sheet"),
            ("list-detail", "list_detail"),
            ("inspector", "inspector"),
            ("workspace", "workspace"),
            ("tool-panel", "tool_panel"),
            ("drawer", "drawer"),
            ("nav-rail", "nav_rail"),
            ("navigation", "navigation_view"),
            ("tab-view", "tab_view"),
            ("preferences", "preferences_page"),
            ("about", "about_page"),
            ("status-page", "status_page"),
            ("palette", "command_palette_view"),
            ("main-window", "main_window"),
        ];
        assert_eq!(ctors.len(), ENTRIES.len());
        for (group, rel) in group_file {
            let text = std::fs::read_to_string(root.join(rel)).unwrap();
            must(
                text.contains(&format!("# {group}")),
                format!("{rel} must be the {group} handbook page"),
            );
            for e in ENTRIES.iter().filter(|e| e.group == group) {
                let marker = format!("**`{}`**", e.id);
                let at = text
                    .find(&marker)
                    .unwrap_or_else(|| panic!("{rel} missing catalog id {}", e.id));
                let rest = &text[at..];
                let end = rest[marker.len()..]
                    .find("\n### ")
                    .map(|i| i + marker.len())
                    .unwrap_or(rest.len());
                let section = &rest[..end];
                let ctor = ctors
                    .iter()
                    .find(|(id, _)| *id == e.id)
                    .map(|(_, n)| *n)
                    .unwrap();
                assert!(section.contains(ctor), "{} section must name {ctor}", e.id);
                must(
                    section.contains("docs.rs/icedtea"),
                    format!("{} section must link rustdoc", e.id),
                );
                must(
                    section.contains("github.com/indynull/icedtea/blob/master"),
                    format!("{} section must link source", e.id),
                );
                let tea = section.contains("crates.io/crates/icedtea");
                let iced = section.contains("crates.io/crates/iced");
                must(
                    tea || iced,
                    format!("{} section must link the published crate", e.id),
                );
            }
        }
    }

    #[test]
    fn reader_path_omits_maintainer_process() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let files = [
            "README.md",
            "book/src/introduction.md",
            "book/src/first-window.md",
            "book/src/architecture.md",
            "book/src/actions.md",
            "book/src/layout.md",
            "book/src/theming.md",
            "book/src/navigation.md",
            "book/src/overlay-windows.md",
            "book/src/compact-tools.md",
            "book/src/widgets.md",
            "book/src/reference/controls.md",
            "book/src/reference/fields.md",
            "book/src/reference/readout.md",
            "book/src/reference/content.md",
            "book/src/reference/collections.md",
            "book/src/reference/chrome.md",
            "book/src/reference/patterns.md",
            "book/src/cookbook/save.md",
            "book/src/cookbook/list-detail.md",
            "book/src/cookbook/table.md",
            "book/src/cookbook/palette.md",
        ];
        for rel in files {
            let text = std::fs::read_to_string(root.join(rel)).unwrap();
            for needle in [
                "one catalog id",
                "one constructor",
                "llvm-cov",
                "fail-under",
                "identity token",
                "catalog id",
                "see also catalog",
            ] {
                must(
                    !text.to_ascii_lowercase().contains(needle),
                    format!("{rel} must not teach {needle}"),
                );
            }
            let stripped = text
                .to_ascii_lowercase()
                .replace("gallery.gif", "")
                .replace("assets/gallery.gif", "");
            must(
                !stripped.contains("gallery"),
                format!("{rel} must not send readers to the gallery demo"),
            );
        }
        let root_rs = include_str!("lib.rs");
        let tour = root_rs.split("#![cfg_attr").next().unwrap_or(root_rs);
        for needle in [
            "one catalog id",
            "one constructor",
            "llvm-cov",
            "fail-under",
            "gallery",
            "catalog id",
        ] {
            must(
                !tour.contains(needle),
                format!("crate-root tour must not teach {needle}"),
            );
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
            ("button", "themed_button", widget),
            ("segmented-button", "segmented_button", widget),
            ("button-group", "button_group", widget),
            ("icon-button", "icon_button", widget),
            ("toggle-icon-button", "icon_button_toggle", widget),
            ("split-button", "split_button", widget),
            ("toggle-button", "toggle_button", widget),
            ("checkbox", "themed_checkbox", widget),
            ("checkbox-indeterminate", "checkbox_indeterminate", widget),
            ("radio", "themed_radio", widget),
            ("switch", "themed_switch", widget),
            ("slider", "themed_slider", widget),
            ("range-slider", "range_slider", widget),
            ("text-input", "themed_text_input", widget),
            ("field-support", "field_support", widget),
            ("password", "password_input", widget),
            ("secret", "secret_field", widget),
            ("value-field", "value_field", widget),
            ("textarea", "textarea", widget),
            ("search", "search_input", widget),
            ("search-view", "search_view", widget),
            ("suggest", "suggest_field", widget),
            ("select", "themed_pick_list", widget),
            ("number", "number_input", widget),
            ("date", "date_picker", widget),
            ("time", "time_picker", widget),
            ("progress", "progress", widget),
            ("progress-ring", "progress_ring", widget),
            ("spinner", "spinner", widget),
            ("label", "label", widget),
            ("icon", "icon_svg", widget),
            ("tooltip", "tooltip_wrap", widget),
            ("rich-tooltip", "tooltip_rich", widget),
            ("link", "hyperlink", widget),
            ("markdown", "markdown_view", widget),
            ("code", "highlighted_code", widget),
            ("image", "image_slot", widget),
            ("selectable", "selectable", widget),
            ("list", "list_view", widget),
            ("virtual-column", "virtual_column", widget),
            ("log", "log_view", widget),
            ("grid", "item_grid", widget),
            ("table", "data_table", widget),
            ("tree", "tree_view", widget),
            ("tabs", "tab_bar", widget),
            ("accordion", "accordion_view", widget),
            ("expander", "expander", widget),
            ("pagination", "pagination", widget),
            ("theme", "named", theme),
            ("colors", "mix", theme),
            ("keys", "handle", key),
            ("cheatsheet", "cheatsheet", pattern),
            ("card", "group_box", widget),
            ("rule", "rule_h", widget),
            ("chip", "chip", widget),
            ("filter-chips", "filter_chips", widget),
            ("badge", "badge", widget),
            ("wrap", "wrap", layout),
            ("banner", "banner", widget),
            ("command-bar", "command_bar", pattern),
            ("context-menu", "context_menu", pattern),
            ("sectioned-menu", "sectioned_menu", pattern),
            ("cascade-menu", "cascade_menu", pattern),
            ("breadcrumb", "breadcrumb", widget),
            ("menu", "menu_bar", pattern),
            ("toolbar", "toolbar", pattern),
            ("status-bar", "status_bar", pattern),
            ("busy", "busy_overlay", widget),
            ("toast", "toast_view", widget),
            ("scrollbar", "themed_scroll", widget),
            ("dialogs", "dialog_sheet", pattern),
            ("side-sheet", "side_sheet", pattern),
            ("list-detail", "list_detail", pattern),
            ("inspector", "inspector", pattern),
            ("workspace", "workspace", pattern),
            ("tool-panel", "tool_panel", pattern),
            ("drawer", "drawer", pattern),
            ("nav-rail", "nav_rail", pattern),
            ("navigation", "navigation_view", pattern),
            ("tab-view", "tab_view", pattern),
            ("preferences", "preferences_page", pattern),
            ("about", "about_page", pattern),
            ("status-page", "status_page", pattern),
            ("palette", "command_palette_view", pattern),
            ("main-window", "main_window", pattern),
        ];
        assert_eq!(map.len(), ENTRIES.len());
        for e in ENTRIES {
            let hit = map.iter().find(|(id, _, _)| *id == e.id);
            let (_, name, src) = hit.expect(e.id);
            let at = find_pub_fn(src, name)
                .unwrap_or_else(|| panic!("{} missing constructor pub fn {}(", e.id, name));
            must(
                rustdoc_example_immediately_above(&src[..at]),
                format!(
                    "{} constructor pub fn {} needs a rustdoc example immediately above it",
                    e.id, name
                ),
            );
            let rustdoc = rustdoc_block_above(&src[..at]);
            must(
                !rustdoc.contains("catalog id") && !rustdoc.contains("Catalog `"),
                format!("{} rustdoc must not teach catalog id", e.id),
            );
            must(
                !rustdoc.contains("|_| ()") && !rustdoc.contains("|_, _| ()"),
                format!(
                    "{} rustdoc must name the message, not a dummy closure",
                    e.id
                ),
            );
        }
    }

    fn find_pub_fn(src: &str, name: &str) -> Option<usize> {
        let pat = format!("pub fn {name}");
        let mut start = 0;
        while let Some(rel) = src[start..].find(&pat) {
            let at = start + rel;
            let after = src[at + pat.len()..].chars().next();
            if matches!(after, Some('(') | Some('<')) {
                return Some(at);
            }
            start = at + pat.len();
        }
        None
    }

    fn rustdoc_block_above(before: &str) -> String {
        let mut docs = Vec::new();
        for line in before.lines().rev() {
            let t = line.trim_start();
            if t.is_empty() {
                if docs.is_empty() {
                    continue;
                }
                break;
            }
            if t.starts_with("///") || t.starts_with("#[") {
                docs.push(t);
                continue;
            }
            break;
        }
        docs.join("\n")
    }

    fn rustdoc_example_immediately_above(before: &str) -> bool {
        let mut docs = Vec::new();
        for line in before.lines().rev() {
            let t = line.trim_start();
            if t.is_empty() {
                if docs.is_empty() {
                    continue;
                }
                break;
            }
            if t.starts_with("///") || t.starts_with("#[") {
                docs.push(t);
                continue;
            }
            break;
        }
        docs.iter().any(|l| l.contains("```"))
    }

    #[test]
    fn constructor_name_is_not_a_prefix_of_a_neighbor() {
        assert_eq!(
            find_pub_fn("pub fn spinner_angles()\npub fn spinner()", "spinner"),
            Some("pub fn spinner_angles()\n".len())
        );
        assert_eq!(find_pub_fn("pub fn spinner_angles()", "spinner"), None);
        assert_eq!(find_pub_fn("pub fn progress<T>()", "progress"), Some(0));
        assert_eq!(find_pub_fn("pub fn foo", "foo"), None);
        assert_eq!(find_pub_fn("fn foo()", "foo"), None);
    }

    #[test]
    fn workspace_center_fills_first_leaf() {
        let pattern = include_str!("pattern.rs");
        let at = find_pub_fn(pattern, "workspace").expect("workspace constructor");
        let rustdoc = rustdoc_block_above(&pattern[..at]);
        must(
            rustdoc.contains("each leaf id"),
            "workspace rustdoc must say each leaf id gets a pane",
        );
        assert_eq!(rustdoc_block_above("\n\n"), "");
        assert!(rustdoc_block_above("/// hi\n/// there").contains("/// hi"));
        assert!(rustdoc_block_above("code\n\n/// doc").contains("/// doc"));
        assert!(rustdoc_block_above("fn other()\n/// doc").contains("/// doc"));
    }

    #[test]
    fn hello_is_a_tool() {
        let hello = include_str!("../examples/hello.rs");
        assert!(hello.contains("file.save"));
        assert!(hello.contains("ctrl+s"));
        assert!(hello.contains("pattern::toolbar") || hello.contains("main_window"));
        assert!(hello.contains("textarea") || hello.contains("themed_text_input"));
        assert!(!hello.contains("count.inc"));
        assert!(!hello.contains("n: i32"));
    }

    #[test]
    fn readme_points_at_hello() {
        let readme = include_str!("../README.md");
        assert!(readme.contains("examples/hello.rs"));
        assert!(readme.contains("icedtea::run!"));
        assert!(!readme.contains("struct Hello"));
    }

    #[test]
    fn crate_root_walks() {
        let root = include_str!("lib.rs");
        let tour = root.split("#![cfg_attr").next().unwrap_or(root);
        for heading in ["First compose", "Boot", "Keys", "Tokens", "Scope"] {
            assert!(tour.contains(heading));
        }
        assert!(tour.contains("file.save"));
        assert!(!tour.contains("catalog id"));
        assert!(!tour.contains("closed list"));
    }

    #[test]
    fn cookbook_exists() {
        let summary = include_str!("../book/src/SUMMARY.md");
        assert!(summary.contains("# Start"));
        assert!(summary.contains("# Compose"));
        assert!(summary.contains("# Cookbook"));
        assert!(summary.contains("# Reference"));
        for path in [
            "introduction.md",
            "install.md",
            "first-window.md",
            "architecture.md",
            "m3-foundations.md",
            "actions.md",
            "layout.md",
            "theming.md",
            "navigation.md",
            "overlay-windows.md",
            "compact-tools.md",
            "cookbook/save.md",
            "cookbook/list-detail.md",
            "cookbook/table.md",
            "cookbook/palette.md",
            "widgets.md",
            "reference/controls.md",
            "reference/fields.md",
            "reference/readout.md",
            "reference/content.md",
            "reference/collections.md",
            "reference/chrome.md",
            "reference/patterns.md",
        ] {
            assert!(summary.contains(path), "{path}");
        }
        let pages: [(&str, &[&str]); 4] = [
            (
                include_str!("../book/src/cookbook/save.md"),
                &["file.save", "toolbar", "key::handle", "textarea"],
            ),
            (
                include_str!("../book/src/cookbook/list-detail.md"),
                &["list_detail", "list_view", "layout::fixed"],
            ),
            (
                include_str!("../book/src/cookbook/table.md"),
                &["data_table", "TableModel", "ColumnLayout", "VisibleWindow"],
            ),
            (
                include_str!("../book/src/cookbook/palette.md"),
                &["CommandPalette", "command_palette_view"],
            ),
        ];
        for (text, needles) in pages {
            for n in needles.iter().copied() {
                assert!(text.contains(n));
            }
            assert!(!text.to_ascii_lowercase().contains("gallery"));
            assert!(!text.contains("Nop"));
        }
    }

    #[test]
    fn constructor_rustdoc_has_no_catalog_title() {
        let widget = include_str!("widget.rs");
        assert!(
            !rustdoc_block_above(&widget[..find_pub_fn(widget, "themed_button").unwrap()])
                .contains("catalog id")
        );
    }

    #[test]
    fn rustdoc_example_must_sit_on_the_constructor() {
        assert!(rustdoc_example_immediately_above(
            "/// ```\n/// x\n/// ```\n"
        ));
        assert!(rustdoc_example_immediately_above("/// ```\n#[must_use]\n"));
        assert!(rustdoc_example_immediately_above("/// ```\n\n"));
        assert!(!rustdoc_example_immediately_above("/// no fence\n"));
        assert!(!rustdoc_example_immediately_above(
            "/// ```\n/// neighbor\n/// ```\n\n/// this fn has no fence\n"
        ));
        assert!(!rustdoc_example_immediately_above("fn other() {}\n"));
    }

    #[test]
    fn collapsed_dual_paths_are_not_public() {
        let widget = include_str!("widget.rs");
        must(
            !widget.contains("pub fn image<"),
            "image_slot is the image constructor",
        );
        let key = include_str!("key.rs");
        must(
            !key.contains("pub fn listen_raw"),
            "listen is the keyboard subscription",
        );
        let recipes = include_str!("layout/recipes.rs");
        must(
            !recipes.contains("pub fn scroll_y<"),
            "themed_scroll is the scroll constructor",
        );
        must(
            !recipes.contains("pub fn sidebar_mode"),
            "Breakpoint::from_width picks the sidebar recipe",
        );
    }
}
