//! Catalog inventory: icedtea id → Material Design 3 component.

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
    MapRow {
        id: "button",
        m3: "Button",
        fate: Fate::Map,
    },
    MapRow {
        id: "checkbox",
        m3: "Checkbox",
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
        id: "text-input",
        m3: "Text field",
        fate: Fate::Map,
    },
    MapRow {
        id: "chip",
        m3: "Chip",
        fate: Fate::Map,
    },
    MapRow {
        id: "list",
        m3: "Lists",
        fate: Fate::Map,
    },
    MapRow {
        id: "tabs",
        m3: "Tabs",
        fate: Fate::Map,
    },
    MapRow {
        id: "card",
        m3: "Card",
        fate: Fate::Map,
    },
    MapRow {
        id: "dialogs",
        m3: "Dialogs",
        fate: Fate::Map,
    },
    MapRow {
        id: "menu",
        m3: "Menus",
        fate: Fate::Map,
    },
    MapRow {
        id: "navigation",
        m3: "Navigation",
        fate: Fate::Map,
    },
    MapRow {
        id: "progress",
        m3: "Progress indicator",
        fate: Fate::Map,
    },
    MapRow {
        id: "toast",
        m3: "Snackbar",
        fate: Fate::Map,
    },
    MapRow {
        id: "tooltip",
        m3: "Tooltip",
        fate: Fate::Map,
    },
    MapRow {
        id: "toolbar",
        m3: "App bars (desktop)",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "list-detail",
        m3: "Supporting pane",
        fate: Fate::Desktop,
    },
    MapRow {
        id: "table",
        m3: "Data table (desktop)",
        fate: Fate::Desktop,
    },
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inventory_maps_button_and_deletes_sparkline() {
        assert!(MAP.iter().any(|r| r.id == "button" && r.fate == Fate::Map));
        assert!(deleted_ids().any(|id| id == "sparkline"));
    }
}
