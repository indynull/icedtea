//! Public widget and pattern ids. The gallery must page every entry.

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
}

/// Closed list, like iced's `Theme::ALL`. Named fields so id / title /
/// group cannot be swapped. One row per public surface.
#[rustfmt::skip]
pub const ENTRIES: &[Entry] = &[
    Entry { id: "button", title: "Button", group: "Controls" },
    Entry { id: "split-button", title: "Split button", group: "Controls" },
    Entry { id: "toggle-button", title: "Toggle button", group: "Controls" },
    Entry { id: "checkbox", title: "Checkbox", group: "Controls" },
    Entry { id: "radio", title: "Radio", group: "Controls" },
    Entry { id: "switch", title: "Switch", group: "Controls" },
    Entry { id: "slider", title: "Slider", group: "Controls" },
    Entry { id: "text-input", title: "Text input", group: "Fields" },
    Entry { id: "password", title: "Password", group: "Fields" },
    Entry { id: "secret", title: "Secret field", group: "Fields" },
    Entry { id: "textarea", title: "Text area", group: "Fields" },
    Entry { id: "search", title: "Search", group: "Fields" },
    Entry { id: "suggest", title: "Suggest", group: "Fields" },
    Entry { id: "select", title: "Select", group: "Fields" },
    Entry { id: "number", title: "Number", group: "Fields" },
    Entry { id: "mask", title: "Masked field", group: "Fields" },
    Entry { id: "date", title: "Date", group: "Fields" },
    Entry { id: "time", title: "Time", group: "Fields" },
    Entry { id: "color", title: "Color", group: "Fields" },
    Entry { id: "progress", title: "Progress", group: "Readout" },
    Entry { id: "progress-ring", title: "Progress ring", group: "Readout" },
    Entry { id: "sparkline", title: "Sparkline", group: "Readout" },
    Entry { id: "spinner", title: "Spinner", group: "Readout" },
    Entry { id: "display", title: "Display reading", group: "Readout" },
    Entry { id: "label", title: "Label", group: "Content" },
    Entry { id: "rich-cell", title: "Rich cell", group: "Content" },
    Entry { id: "markdown", title: "Markdown", group: "Content" },
    Entry { id: "code", title: "Code", group: "Content" },
    Entry { id: "icon", title: "Icon", group: "Content" },
    Entry { id: "image", title: "Image", group: "Content" },
    Entry { id: "tooltip", title: "Tooltip", group: "Content" },
    Entry { id: "link", title: "Hyperlink", group: "Content" },
    Entry { id: "list", title: "List", group: "Collections" },
    Entry { id: "log", title: "Log", group: "Collections" },
    Entry { id: "grid", title: "Item grid", group: "Collections" },
    Entry { id: "table", title: "Data table", group: "Collections" },
    Entry { id: "tree", title: "Tree", group: "Collections" },
    Entry { id: "tabs", title: "Tabs", group: "Collections" },
    Entry { id: "accordion", title: "Accordion", group: "Collections" },
    Entry { id: "pagination", title: "Pagination", group: "Collections" },
    Entry { id: "theme", title: "Theme", group: "Chrome" },
    Entry { id: "colors", title: "Colors", group: "Chrome" },
    Entry { id: "keys", title: "Keys", group: "Chrome" },
    Entry { id: "card", title: "Card", group: "Chrome" },
    Entry { id: "rule", title: "Rule", group: "Chrome" },
    Entry { id: "chip", title: "Chip", group: "Chrome" },
    Entry { id: "badge", title: "Badge", group: "Chrome" },
    Entry { id: "wrap", title: "Wrap", group: "Chrome" },
    Entry { id: "pad", title: "Pad", group: "Chrome" },
    Entry { id: "command-bar", title: "Command bar", group: "Chrome" },
    Entry { id: "context-menu", title: "Context menu", group: "Chrome" },
    Entry { id: "scrollbar", title: "Scrollbar", group: "Chrome" },
    Entry { id: "callout", title: "Callout", group: "Chrome" },
    Entry { id: "banner", title: "Banner", group: "Chrome" },
    Entry { id: "group-box", title: "Group box", group: "Chrome" },
    Entry { id: "breadcrumb", title: "Breadcrumb", group: "Chrome" },
    Entry { id: "menu", title: "Menu", group: "Chrome" },
    Entry { id: "toolbar", title: "Toolbar", group: "Chrome" },
    Entry { id: "status-bar", title: "Status bar", group: "Chrome" },
    Entry { id: "toast", title: "Toast", group: "Chrome" },
    Entry { id: "busy", title: "Busy overlay", group: "Chrome" },
    Entry { id: "skeleton", title: "Skeleton", group: "Chrome" },
    Entry { id: "teaching-tip", title: "Teaching tip", group: "Chrome" },
    Entry { id: "dialogs", title: "Dialogs", group: "Patterns" },
    Entry { id: "list-detail", title: "List/detail", group: "Patterns" },
    Entry { id: "navigation", title: "Navigation view", group: "Patterns" },
    Entry { id: "tab-view", title: "Tab view", group: "Patterns" },
    Entry { id: "preferences", title: "Preferences", group: "Patterns" },
    Entry { id: "about", title: "About", group: "Patterns" },
    Entry { id: "status-page", title: "Status page", group: "Patterns" },
    Entry { id: "palette", title: "Command palette", group: "Patterns" },
    Entry { id: "main-window", title: "Main window", group: "Patterns" },
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
}
