//! Catalog inventory: icedtea id → Material Design 3 component.
//!
//! Every public [`crate::catalog::ENTRIES`] id is listed with fate
//! [`Fate::Map`] (M3 component family) or [`Fate::Desktop`] (desktop
//! chrome expressed in M3 tokens). [`Fate::Delete`] rows are removed
//! from the catalog; do not re-export them.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fate {
    Map,
    Desktop,
    Delete,
}

#[derive(Debug, Clone, Copy)]
pub struct MapRow {
    pub id: &'static str,
    pub m3: &'static str,
    pub fate: Fate,
}

pub const MAP: &[MapRow] = &[
    // Controls
    MapRow {
        id: "button",
        m3: "Button",
        fate: Fate::Map,
    },
    MapRow {
        id: "segmented-button",
        m3: "Segmented button",
        fate: Fate::Map,
    },
    MapRow {
        id: "icon-button",
        m3: "Icon button",
        fate: Fate::Map,
    },
    MapRow {
        id: "split-button",
        m3: "Button (split)",
        fate: Fate::Map,
    },
    MapRow {
        id: "toggle-button",
        m3: "Icon button / toggle",
        fate: Fate::Map,
    },
    MapRow {
        id: "checkbox",
        m3: "Checkbox",
        fate: Fate::Map,
    },
    MapRow {
        id: "checkbox-indeterminate",
        m3: "Checkbox (indeterminate)",
        fate: Fate::Map,
    },
    MapRow {
        id: "radio",
        m3: "Radio button",
        fate: Fate::Map,
    },
    MapRow {
        id: "switch",
        m3: "Switch",
        fate: Fate::Map,
    },
    MapRow {
        id: "slider",
        m3: "Slider",
        fate: Fate::Map,
    },
    MapRow {
        id: "range-slider",
        m3: "Slider (range)",
        fate: Fate::Map,
    },
    // Fields
    MapRow {
        id: "text-input",
        m3: "Text field",
        fate: Fate::Map,
    },
    MapRow {
        id: "field-support",
        m3: "Text field (supporting / error)",
        fate: Fate::Map,
    },
    MapRow {
        id: "password",
        m3: "Text field",
        fate: Fate::Map,
    },
    MapRow {
        id: "secret",
        m3: "Text field",
        fate: Fate::Map,
    },
    MapRow {
        id: "value-field",
        m3: "Text field (labeled)",
        fate: Fate::Map,
    },
    MapRow {
        id: "textarea",
        m3: "Text field (multi-line)",
        fate: Fate::Map,
    },
    MapRow {
        id: "search",
        m3: "Search",
        fate: Fate::Map,
    },
    MapRow {
        id: "suggest",
        m3: "Menus (suggest)",
        fate: Fate::Map,
    },
    MapRow {
        id: "select",
        m3: "Menus",
        fate: Fate::Map,
    },
    MapRow {
        id: "number",
        m3: "Text field",
        fate: Fate::Map,
    },
    MapRow {
        id: "date",
        m3: "Date pickers (desktop)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "time",
        m3: "Time pickers (desktop)",
        fate: Fate::Desktop,
    },
    // Readout
    MapRow {
        id: "progress",
        m3: "Progress indicator",
        fate: Fate::Map,
    },
    MapRow {
        id: "progress-ring",
        m3: "Progress indicator",
        fate: Fate::Map,
    },
    MapRow {
        id: "spinner",
        m3: "Progress indicator",
        fate: Fate::Map,
    },
    // Content
    MapRow {
        id: "label",
        m3: "Typography",
        fate: Fate::Map,
    },
    MapRow {
        id: "icon",
        m3: "Icons",
        fate: Fate::Map,
    },
    MapRow {
        id: "tooltip",
        m3: "Tooltip",
        fate: Fate::Map,
    },
    MapRow {
        id: "link",
        m3: "Button (text)",
        fate: Fate::Map,
    },
    MapRow {
        id: "markdown",
        m3: "Typography (rich)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "code",
        m3: "Typography (mono)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "image",
        m3: "Image",
        fate: Fate::Map,
    },
    MapRow {
        id: "selectable",
        m3: "Typography (select)",
        fate: Fate::Desktop,
    },
    // Collections
    MapRow {
        id: "list",
        m3: "Lists",
        fate: Fate::Map,
    },
    MapRow {
        id: "virtual-column",
        m3: "Lists (variable height)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "pagination",
        m3: "Lists (paging)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "log",
        m3: "Lists (log)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "grid",
        m3: "Lists (grid)",
        fate: Fate::Map,
    },
    MapRow {
        id: "table",
        m3: "Data table (desktop)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "tree",
        m3: "Lists (tree)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "accordion",
        m3: "Lists (expand)",
        fate: Fate::Map,
    },
    MapRow {
        id: "expander",
        m3: "Lists (expand)",
        fate: Fate::Map,
    },
    MapRow {
        id: "tabs",
        m3: "Tabs",
        fate: Fate::Map,
    },
    // Chrome / marks
    MapRow {
        id: "theme",
        m3: "Color system",
        fate: Fate::Map,
    },
    MapRow {
        id: "colors",
        m3: "Color system",
        fate: Fate::Map,
    },
    MapRow {
        id: "keys",
        m3: "App bars (shortcuts)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "cheatsheet",
        m3: "App bars (shortcuts)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "card",
        m3: "Card",
        fate: Fate::Map,
    },
    MapRow {
        id: "rule",
        m3: "Divider",
        fate: Fate::Map,
    },
    MapRow {
        id: "chip",
        m3: "Chip",
        fate: Fate::Map,
    },
    MapRow {
        id: "filter-chips",
        m3: "Chip (filter set)",
        fate: Fate::Map,
    },
    MapRow {
        id: "badge",
        m3: "Badge",
        fate: Fate::Map,
    },
    MapRow {
        id: "wrap",
        m3: "Layout",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "banner",
        m3: "Banner",
        fate: Fate::Map,
    },
    MapRow {
        id: "command-bar",
        m3: "App bars (desktop)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "context-menu",
        m3: "Menus",
        fate: Fate::Map,
    },
    MapRow {
        id: "sectioned-menu",
        m3: "Menus (sections)",
        fate: Fate::Map,
    },
    MapRow {
        id: "cascade-menu",
        m3: "Menus (cascade)",
        fate: Fate::Map,
    },
    MapRow {
        id: "breadcrumb",
        m3: "Navigation (breadcrumb)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "menu",
        m3: "Menus",
        fate: Fate::Map,
    },
    MapRow {
        id: "toolbar",
        m3: "App bars (desktop)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "status-bar",
        m3: "App bars (desktop)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "busy",
        m3: "Progress indicator",
        fate: Fate::Map,
    },
    MapRow {
        id: "toast",
        m3: "Snackbar",
        fate: Fate::Map,
    },
    MapRow {
        id: "scrollbar",
        m3: "Lists (scroll)",
        fate: Fate::Map,
    },
    // Patterns
    MapRow {
        id: "dialogs",
        m3: "Dialogs",
        fate: Fate::Map,
    },
    MapRow {
        id: "side-sheet",
        m3: "Side sheets",
        fate: Fate::Map,
    },
    MapRow {
        id: "list-detail",
        m3: "Supporting pane",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "inspector",
        m3: "Supporting pane",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "drawer",
        m3: "Navigation drawer",
        fate: Fate::Map,
    },
    MapRow {
        id: "workspace",
        m3: "Layout (dock)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "tool-panel",
        m3: "Supporting pane",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "navigation",
        m3: "Navigation",
        fate: Fate::Map,
    },
    MapRow {
        id: "tab-view",
        m3: "Tabs",
        fate: Fate::Map,
    },
    MapRow {
        id: "preferences",
        m3: "Lists (settings)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "about",
        m3: "Dialogs",
        fate: Fate::Map,
    },
    MapRow {
        id: "status-page",
        m3: "Empty states",
        fate: Fate::Map,
    },
    MapRow {
        id: "palette",
        m3: "Menus (command)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "main-window",
        m3: "App bars (desktop)",
        fate: Fate::Desktop,
    },
    // Removed from public catalog (no dual path)
    MapRow {
        id: "sparkline",
        m3: "—",
        fate: Fate::Delete,
    },
    MapRow {
        id: "display",
        m3: "—",
        fate: Fate::Delete,
    },
    MapRow {
        id: "pad",
        m3: "—",
        fate: Fate::Delete,
    },
    MapRow {
        id: "jobs",
        m3: "—",
        fate: Fate::Delete,
    },
    MapRow {
        id: "document-tabs",
        m3: "—",
        fate: Fate::Delete,
    },
    MapRow {
        id: "teaching-tip",
        m3: "—",
        fate: Fate::Delete,
    },
    MapRow {
        id: "skeleton",
        m3: "—",
        fate: Fate::Delete,
    },
    MapRow {
        id: "callout",
        m3: "—",
        fate: Fate::Delete,
    },
    MapRow {
        id: "group-box",
        m3: "—",
        fate: Fate::Delete,
    },
    MapRow {
        id: "mask",
        m3: "—",
        fate: Fate::Delete,
    },
    MapRow {
        id: "color",
        m3: "—",
        fate: Fate::Delete,
    },
    MapRow {
        id: "rich-cell",
        m3: "—",
        fate: Fate::Delete,
    },
];

pub fn deleted_ids() -> impl Iterator<Item = &'static str> {
    MAP.iter().filter(|r| r.fate == Fate::Delete).map(|r| r.id)
}

pub fn live_ids() -> impl Iterator<Item = &'static str> {
    MAP.iter().filter(|r| r.fate != Fate::Delete).map(|r| r.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_maps_button_and_deletes_sparkline() {
        assert!(MAP.iter().any(|r| r.id == "button" && r.fate == Fate::Map));
        assert!(deleted_ids().any(|id| id == "sparkline"));
    }

    #[test]
    fn deleted_ids_are_not_catalogued() {
        for id in deleted_ids() {
            assert!(crate::catalog::get(id).is_none());
        }
    }

    #[test]
    fn map_rows_have_unique_ids() {
        let mut seen = std::collections::HashSet::new();
        for row in MAP {
            assert!(seen.insert(row.id));
            assert!(!row.m3.is_empty());
        }
    }

    #[test]
    fn every_catalog_entry_is_mapped_live() {
        let live: std::collections::HashSet<_> = live_ids().collect();
        for e in crate::catalog::ENTRIES {
            assert!(live.contains(e.id));
        }
        for id in live_ids() {
            assert!(crate::catalog::get(id).is_some());
        }
    }
}
