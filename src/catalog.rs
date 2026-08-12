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
    Entry { id: "value-field", title: "Value field", group: "Fields", page: "fields" },
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
    Entry { id: "spinner", title: "Spinner", group: "Readout", page: "readout" },
    Entry { id: "sparkline", title: "Sparkline", group: "Readout", page: "readout" },
    Entry { id: "display", title: "Display reading", group: "Readout", page: "readout" },
    Entry { id: "label", title: "Label", group: "Content", page: "type" },
    Entry { id: "rich-cell", title: "Rich cell", group: "Content", page: "type" },
    Entry { id: "icon", title: "Icon", group: "Content", page: "type" },
    Entry { id: "tooltip", title: "Tooltip", group: "Content", page: "type" },
    Entry { id: "link", title: "Hyperlink", group: "Content", page: "type" },
    Entry { id: "markdown", title: "Markdown", group: "Content", page: "markdown" },
    Entry { id: "code", title: "Code", group: "Content", page: "code" },
    Entry { id: "image", title: "Image", group: "Content", page: "image" },
    Entry { id: "selectable", title: "Selectable", group: "Content", page: "selectable" },
    Entry { id: "list", title: "List", group: "Collections", page: "list" },
    // Pages a large set; lives with list, not disclosure chrome.
    Entry { id: "pagination", title: "Pagination", group: "Collections", page: "list" },
    Entry { id: "log", title: "Log", group: "Collections", page: "log" },
    Entry { id: "grid", title: "Item grid", group: "Collections", page: "grid" },
    Entry { id: "table", title: "Data table", group: "Collections", page: "table" },
    Entry { id: "tree", title: "Tree", group: "Collections", page: "tree" },
    // Disclosure (not editor document strips).
    Entry { id: "tabs", title: "Tabs", group: "Collections", page: "sections" },
    Entry { id: "accordion", title: "Accordion", group: "Collections", page: "sections" },
    Entry { id: "expander", title: "Expander", group: "Collections", page: "sections" },
    // Editor document strip — own page; not mixed with accordion.
    Entry { id: "document-tabs", title: "Document tabs", group: "Collections", page: "document-tabs" },
    Entry { id: "theme", title: "Theme", group: "Chrome", page: "theme" },
    Entry { id: "colors", title: "Colors", group: "Chrome", page: "colors" },
    Entry { id: "keys", title: "Keys", group: "Chrome", page: "keys" },
    Entry { id: "cheatsheet", title: "Cheatsheet", group: "Chrome", page: "keys" },
    // Surfaces first, then inline marks, then layout helpers, then messaging.
    Entry { id: "card", title: "Card", group: "Chrome", page: "marks" },
    Entry { id: "group-box", title: "Group box", group: "Chrome", page: "marks" },
    Entry { id: "rule", title: "Rule", group: "Chrome", page: "marks" },
    Entry { id: "chip", title: "Chip", group: "Chrome", page: "marks" },
    Entry { id: "badge", title: "Badge", group: "Chrome", page: "marks" },
    Entry { id: "wrap", title: "Wrap", group: "Chrome", page: "marks" },
    Entry { id: "pad", title: "Pad", group: "Chrome", page: "marks" },
    Entry { id: "callout", title: "Callout", group: "Chrome", page: "marks" },
    Entry { id: "banner", title: "Banner", group: "Chrome", page: "marks" },
    Entry { id: "teaching-tip", title: "Teaching tip", group: "Chrome", page: "marks" },
    Entry { id: "skeleton", title: "Skeleton", group: "Chrome", page: "marks" },
    Entry { id: "menu", title: "Menu", group: "Chrome", page: "chrome-rows" },
    Entry { id: "toolbar", title: "Toolbar", group: "Chrome", page: "chrome-rows" },
    Entry { id: "command-bar", title: "Command bar", group: "Chrome", page: "chrome-rows" },
    Entry { id: "status-bar", title: "Status bar", group: "Chrome", page: "chrome-rows" },
    Entry { id: "breadcrumb", title: "Breadcrumb", group: "Chrome", page: "chrome-rows" },
    Entry { id: "context-menu", title: "Context menu", group: "Chrome", page: "chrome-rows" },
    Entry { id: "toast", title: "Toast", group: "Chrome", page: "feedback" },
    Entry { id: "jobs", title: "Jobs", group: "Chrome", page: "feedback" },
    Entry { id: "busy", title: "Busy overlay", group: "Chrome", page: "feedback" },
    Entry { id: "scrollbar", title: "Scrollbar", group: "Chrome", page: "feedback" },
    // Multi-pane and window structure.
    Entry { id: "main-window", title: "Main window", group: "Layouts", page: "main-window" },
    Entry { id: "navigation", title: "Navigation view", group: "Layouts", page: "navigation" },
    Entry { id: "tab-view", title: "Tab view", group: "Layouts", page: "tab-view" },
    Entry { id: "list-detail", title: "List/detail", group: "Layouts", page: "list-detail" },
    Entry { id: "inspector", title: "Inspector", group: "Layouts", page: "inspector" },
    Entry { id: "workspace", title: "Workspace", group: "Layouts", page: "workspace" },
    Entry { id: "drawer", title: "Drawer", group: "Layouts", page: "workspace" },
    Entry { id: "tool-panel", title: "Tool panel", group: "Layouts", page: "workspace" },
    // Float over content.
    Entry { id: "dialogs", title: "Dialogs", group: "Overlays", page: "dialogs" },
    Entry { id: "palette", title: "Command palette", group: "Overlays", page: "palette" },
    // Full-window views apps put on menu items (Help → About, …).
    Entry { id: "preferences", title: "Preferences", group: "Screens", page: "preferences" },
    Entry { id: "about", title: "About", group: "Screens", page: "about" },
    Entry { id: "status-page", title: "Status page", group: "Screens", page: "status-page" },
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
        "type" => "Text and icons",
        "list" => "List and pages",
        "sections" => "Tabs and disclosure",
        "document-tabs" => "Document tabs",
        "marks" => "Surfaces and marks",
        "chrome-rows" => "Window chrome",
        "feedback" => "Toasts and busy",
        "workspace" => "Workspace",
        id => get(id).map(|e| e.title).unwrap_or("Page"),
    }
}

/// Module path of the shipped constructor (`widget`, `pattern`, …).
pub fn constructor_module(id: &str) -> Option<&'static str> {
    CONSTRUCTORS
        .iter()
        .find(|(i, _, _)| *i == id)
        .map(|(_, m, _)| *m)
}

/// Rust function name of the shipped constructor (`themed_button`, …).
pub fn constructor_name(id: &str) -> Option<&'static str> {
    CONSTRUCTORS
        .iter()
        .find(|(i, _, _)| *i == id)
        .map(|(_, _, n)| *n)
}

/// `module::name` for gallery and handbook links.
pub fn constructor_path(id: &str) -> Option<String> {
    match (constructor_module(id), constructor_name(id)) {
        (Some(m), Some(n)) => Some(format!("{m}::{n}")),
        _ => None,
    }
}

/// One row per catalog id: (id, module, constructor name).
const CONSTRUCTORS: &[(&str, &str, &str)] = &[
    ("button", "widget", "themed_button"),
    ("split-button", "widget", "split_button"),
    ("toggle-button", "widget", "toggle_button"),
    ("checkbox", "widget", "themed_checkbox"),
    ("radio", "widget", "themed_radio"),
    ("switch", "widget", "themed_switch"),
    ("slider", "widget", "themed_slider"),
    ("text-input", "widget", "themed_text_input"),
    ("password", "widget", "password_input"),
    ("secret", "widget", "secret_field"),
    ("value-field", "widget", "value_field"),
    ("textarea", "widget", "textarea"),
    ("search", "widget", "search_input"),
    ("suggest", "widget", "suggest_field"),
    ("select", "widget", "themed_pick_list"),
    ("number", "widget", "number_input"),
    ("mask", "widget", "masked_input"),
    ("date", "widget", "date_picker"),
    ("time", "widget", "time_picker"),
    ("color", "widget", "color_swatch"),
    ("progress", "widget", "progress"),
    ("progress-ring", "widget", "progress_ring"),
    ("spinner", "widget", "spinner"),
    ("sparkline", "widget", "sparkline"),
    ("display", "widget", "display_reading"),
    ("label", "widget", "label"),
    ("rich-cell", "widget", "rich_cell"),
    ("icon", "widget", "icon_svg"),
    ("tooltip", "widget", "tooltip_wrap"),
    ("link", "widget", "hyperlink"),
    ("markdown", "widget", "markdown_view"),
    ("code", "widget", "highlighted_code"),
    ("image", "widget", "image_slot"),
    ("selectable", "widget", "selectable"),
    ("list", "widget", "list_view"),
    ("log", "widget", "log_view"),
    ("grid", "widget", "item_grid"),
    ("table", "widget", "data_table"),
    ("tree", "widget", "tree_view"),
    ("tabs", "widget", "tab_bar"),
    ("accordion", "widget", "accordion_view"),
    ("expander", "widget", "expander"),
    ("pagination", "widget", "pagination"),
    ("document-tabs", "pattern", "document_tabs"),
    ("theme", "theme", "named"),
    ("colors", "theme", "mix"),
    ("keys", "key", "handle"),
    ("cheatsheet", "pattern", "cheatsheet"),
    ("card", "widget", "group_box"),
    ("rule", "widget", "rule_h"),
    ("chip", "widget", "chip"),
    ("badge", "widget", "badge"),
    ("wrap", "layout", "wrap"),
    ("pad", "layout", "pad"),
    ("callout", "widget", "info_bar"),
    ("banner", "widget", "banner"),
    ("group-box", "widget", "group_box"),
    ("skeleton", "widget", "placeholder_skeleton"),
    ("teaching-tip", "widget", "teaching_tip"),
    ("command-bar", "pattern", "command_bar"),
    ("context-menu", "pattern", "context_menu"),
    ("breadcrumb", "widget", "breadcrumb"),
    ("menu", "pattern", "menu_bar"),
    ("toolbar", "pattern", "toolbar"),
    ("status-bar", "pattern", "status_bar"),
    ("busy", "widget", "busy_overlay"),
    ("toast", "widget", "toast_view"),
    ("jobs", "pattern", "job_strip"),
    ("scrollbar", "widget", "themed_scroll"),
    ("dialogs", "pattern", "dialog_sheet"),
    ("list-detail", "pattern", "list_detail"),
    ("inspector", "pattern", "inspector"),
    ("workspace", "pattern", "workspace"),
    ("tool-panel", "pattern", "tool_panel"),
    ("drawer", "pattern", "drawer"),
    ("navigation", "pattern", "navigation_view"),
    ("tab-view", "pattern", "tab_view"),
    ("preferences", "pattern", "preferences_page"),
    ("about", "pattern", "about_page"),
    ("status-page", "pattern", "status_page"),
    ("palette", "pattern", "command_palette_view"),
    ("main-window", "pattern", "main_window"),
];

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
                "Layouts",
                "Overlays",
                "Screens"
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
            assert!(!title.is_empty(), "{page}");
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
            "reference/controls.md",
            "reference/fields.md",
            "reference/readout.md",
            "reference/content.md",
            "reference/collections.md",
            "reference/chrome.md",
            "reference/layouts.md",
            "reference/overlays.md",
            "reference/screens.md",
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
            ("Layouts", "reference/layouts.md"),
            ("Overlays", "reference/overlays.md"),
            ("Screens", "reference/screens.md"),
        ];
        assert_eq!(CONSTRUCTORS.len(), ENTRIES.len());
        for e in ENTRIES {
            assert!(
                constructor_name(e.id).is_some(),
                "missing constructor for {}",
                e.id
            );
        }
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
                let ctor = constructor_name(e.id).unwrap();
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
            "book/src/reference/layouts.md",
            "book/src/reference/overlays.md",
            "book/src/reference/screens.md",
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
        let src = |m: &str| match m {
            "widget" => widget,
            "pattern" => pattern,
            "theme" => theme,
            "key" => key,
            "layout" => layout,
            _ => "",
        };
        assert_eq!(CONSTRUCTORS.len(), ENTRIES.len());
        for e in ENTRIES {
            let name = constructor_name(e.id).expect(e.id);
            let module = constructor_module(e.id).expect(e.id);
            let body = src(module);
            let at = find_pub_fn(body, name)
                .unwrap_or_else(|| panic!("{} missing constructor pub fn {}(", e.id, name));
            must(
                rustdoc_example_immediately_above(&body[..at]),
                format!(
                    "{} constructor pub fn {} needs a rustdoc example immediately above it",
                    e.id, name
                ),
            );
            let rustdoc = rustdoc_block_above(&body[..at]);
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
            assert!(tour.contains(heading), "tour missing {heading}");
        }
        assert!(tour.contains("file.save"));
        assert!(!tour.contains("catalog id"));
        assert!(!tour.contains("closed list"));
    }

    #[test]
    fn cookbook_exists() {
        let summary = include_str!("../book/src/SUMMARY.md");
        assert!(summary.contains("# Cookbook"));
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
                assert!(text.contains(n), "cookbook missing {n}");
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
