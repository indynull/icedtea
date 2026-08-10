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

pub const ENTRIES: &[Entry] = &[
    Entry {
        id: "button",
        title: "Button",
        group: "input",
    },
    Entry {
        id: "split-button",
        title: "Split button",
        group: "input",
    },
    Entry {
        id: "toggle-button",
        title: "Toggle button",
        group: "input",
    },
    Entry {
        id: "checkbox",
        title: "Checkbox",
        group: "input",
    },
    Entry {
        id: "radio",
        title: "Radio",
        group: "input",
    },
    Entry {
        id: "switch",
        title: "Switch",
        group: "input",
    },
    Entry {
        id: "slider",
        title: "Slider",
        group: "input",
    },
    Entry {
        id: "progress",
        title: "Progress",
        group: "input",
    },
    Entry {
        id: "progress-ring",
        title: "Progress ring",
        group: "input",
    },
    Entry {
        id: "number",
        title: "Number",
        group: "input",
    },
    Entry {
        id: "text-input",
        title: "Text input",
        group: "input",
    },
    Entry {
        id: "password",
        title: "Password",
        group: "input",
    },
    Entry {
        id: "textarea",
        title: "Text area",
        group: "input",
    },
    Entry {
        id: "search",
        title: "Search",
        group: "input",
    },
    Entry {
        id: "select",
        title: "Select",
        group: "input",
    },
    Entry {
        id: "date",
        title: "Date",
        group: "input",
    },
    Entry {
        id: "time",
        title: "Time",
        group: "input",
    },
    Entry {
        id: "color",
        title: "Color",
        group: "input",
    },
    Entry {
        id: "label",
        title: "Label",
        group: "text",
    },
    Entry {
        id: "display",
        title: "Display reading",
        group: "text",
    },
    Entry {
        id: "markdown",
        title: "Markdown",
        group: "text",
    },
    Entry {
        id: "code",
        title: "Code",
        group: "text",
    },
    Entry {
        id: "icon",
        title: "Icon",
        group: "text",
    },
    Entry {
        id: "image",
        title: "Image",
        group: "text",
    },
    Entry {
        id: "tooltip",
        title: "Tooltip",
        group: "text",
    },
    Entry {
        id: "link",
        title: "Hyperlink",
        group: "text",
    },
    Entry {
        id: "list",
        title: "List",
        group: "collection",
    },
    Entry {
        id: "grid",
        title: "Item grid",
        group: "collection",
    },
    Entry {
        id: "table",
        title: "Data table",
        group: "collection",
    },
    Entry {
        id: "tree",
        title: "Tree",
        group: "collection",
    },
    Entry {
        id: "tabs",
        title: "Tabs",
        group: "collection",
    },
    Entry {
        id: "accordion",
        title: "Accordion",
        group: "collection",
    },
    Entry {
        id: "pagination",
        title: "Pagination",
        group: "collection",
    },
    Entry {
        id: "theme",
        title: "Theme",
        group: "chrome",
    },
    Entry {
        id: "card",
        title: "Card",
        group: "chrome",
    },
    Entry {
        id: "rule",
        title: "Rule",
        group: "chrome",
    },
    Entry {
        id: "chip",
        title: "Chip",
        group: "chrome",
    },
    Entry {
        id: "badge",
        title: "Badge",
        group: "chrome",
    },
    Entry {
        id: "wrap",
        title: "Wrap",
        group: "chrome",
    },
    Entry {
        id: "pad",
        title: "Pad",
        group: "chrome",
    },
    Entry {
        id: "command-bar",
        title: "Command bar",
        group: "chrome",
    },
    Entry {
        id: "context-menu",
        title: "Context menu",
        group: "chrome",
    },
    Entry {
        id: "scrollbar",
        title: "Scrollbar",
        group: "chrome",
    },
    Entry {
        id: "callout",
        title: "Callout",
        group: "chrome",
    },
    Entry {
        id: "banner",
        title: "Banner",
        group: "chrome",
    },
    Entry {
        id: "group-box",
        title: "Group box",
        group: "chrome",
    },
    Entry {
        id: "breadcrumb",
        title: "Breadcrumb",
        group: "chrome",
    },
    Entry {
        id: "menu",
        title: "Menu",
        group: "chrome",
    },
    Entry {
        id: "toolbar",
        title: "Toolbar",
        group: "chrome",
    },
    Entry {
        id: "status-bar",
        title: "Status bar",
        group: "chrome",
    },
    Entry {
        id: "toast",
        title: "Toast",
        group: "feedback",
    },
    Entry {
        id: "spinner",
        title: "Spinner",
        group: "feedback",
    },
    Entry {
        id: "skeleton",
        title: "Skeleton",
        group: "feedback",
    },
    Entry {
        id: "teaching-tip",
        title: "Teaching tip",
        group: "feedback",
    },
    Entry {
        id: "dialogs",
        title: "Dialogs",
        group: "dialogs",
    },
    Entry {
        id: "list-detail",
        title: "List/detail",
        group: "pattern",
    },
    Entry {
        id: "navigation",
        title: "Navigation view",
        group: "pattern",
    },
    Entry {
        id: "tab-view",
        title: "Tab view",
        group: "pattern",
    },
    Entry {
        id: "preferences",
        title: "Preferences",
        group: "pattern",
    },
    Entry {
        id: "about",
        title: "About",
        group: "pattern",
    },
    Entry {
        id: "status-page",
        title: "Status page",
        group: "pattern",
    },
    Entry {
        id: "palette",
        title: "Command palette",
        group: "pattern",
    },
    Entry {
        id: "main-window",
        title: "Main window",
        group: "pattern",
    },
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
        assert!(groups().contains(&"input"));
        assert!(ENTRIES.len() >= 40);
        assert_eq!(get("table").unwrap().group, "collection");
        assert_eq!(get("theme").unwrap().group, "chrome");
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
        ] {
            assert!(get(id).is_some(), "catalog missing {id}");
        }
        for name in [
            "install.md",
            "first-window.md",
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
            assert!(p.is_file(), "missing book page {p:?}");
        }
    }
}
