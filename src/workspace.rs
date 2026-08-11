//! Nested dock tree: splits, tab groups, persist, perspectives.
//!
//! Applications own panel content. [`crate::pattern::workspace`] draws the
//! chrome; this module is the layout tree.

use serde::{Deserialize, Serialize};

use crate::layout::{Axis, Breakpoint};

/// One leaf panel in a workspace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Panel {
    pub id: String,
    pub title: String,
    pub min: f32,
    pub max: f32,
}

impl Panel {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            min: 80.0,
            max: 10_000.0,
        }
    }

    pub fn with_limits(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max.max(min);
        self
    }
}

/// Nested dock: a leaf, a split, or a tab group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DockNode {
    Leaf(Panel),
    Split {
        axis: Axis,
        ratio: f32,
        first: Box<DockNode>,
        second: Box<DockNode>,
    },
    Tabs {
        active: usize,
        panes: Vec<Panel>,
    },
}

impl DockNode {
    pub fn leaf(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self::Leaf(Panel::new(id, title))
    }

    pub fn split(axis: Axis, ratio: f32, first: DockNode, second: DockNode) -> Self {
        Self::Split {
            axis,
            ratio: ratio.clamp(0.08, 0.92),
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    pub fn tabs(panes: Vec<Panel>, active: usize) -> Self {
        let active = active.min(panes.len().saturating_sub(1));
        Self::Tabs { active, panes }
    }

    pub fn clamp_ratio(&mut self) {
        match self {
            Self::Split {
                ratio,
                first,
                second,
                ..
            } => {
                *ratio = ratio.clamp(0.08, 0.92);
                first.clamp_ratio();
                second.clamp_ratio();
            }
            Self::Tabs { active, panes } => {
                if panes.is_empty() {
                    *active = 0;
                } else {
                    *active = (*active).min(panes.len() - 1);
                }
            }
            Self::Leaf(_) => {}
        }
    }

    /// Find a leaf by id.
    pub fn find(&self, id: &str) -> Option<&Panel> {
        match self {
            Self::Leaf(p) if p.id == id => Some(p),
            Self::Leaf(_) => None,
            Self::Split { first, second, .. } => first.find(id).or_else(|| second.find(id)),
            Self::Tabs { panes, .. } => panes.iter().find(|p| p.id == id),
        }
    }

    /// Move `id` into the tab group that already contains `dest`, or beside it.
    ///
    /// Failed dest leaves the tree unchanged.
    pub fn move_panel(&mut self, id: &str, dest: &str) -> bool {
        let before = self.clone();
        let Some(panel) = self.take(id) else {
            return false;
        };
        if self.insert_beside(dest, panel) {
            true
        } else {
            *self = before;
            false
        }
    }

    fn take(&mut self, id: &str) -> Option<Panel> {
        match self {
            Self::Leaf(_) => None,
            Self::Tabs { panes, active } => {
                if let Some(i) = panes.iter().position(|p| p.id == id) {
                    let p = panes.remove(i);
                    if *active >= panes.len() {
                        *active = panes.len().saturating_sub(1);
                    }
                    return Some(p);
                }
                None
            }
            Self::Split { first, second, .. } => {
                if let DockNode::Leaf(p) = first.as_ref() {
                    if p.id == id {
                        let panel = p.clone();
                        *self = *second.clone();
                        return Some(panel);
                    }
                }
                if let Some(panel) = first.take(id) {
                    if is_vacant(first) {
                        *self = *second.clone();
                    }
                    return Some(panel);
                }
                if let DockNode::Leaf(p) = second.as_ref() {
                    if p.id == id {
                        let panel = p.clone();
                        *self = *first.clone();
                        return Some(panel);
                    }
                }
                if let Some(panel) = second.take(id) {
                    if is_vacant(second) {
                        *self = *first.clone();
                    }
                    return Some(panel);
                }
                None
            }
        }
    }

    /// Axis of the `index`th split in depth-first order.
    pub fn split_axis(&self, index: usize) -> Option<Axis> {
        split_axis_at(self, &mut 0, index)
    }

    /// Ratio of the `index`th split in depth-first order.
    pub fn split_ratio(&self, index: usize) -> Option<f32> {
        split_ratio_get(self, &mut 0, index)
    }

    /// Write a clamped ratio onto the `index`th split.
    pub fn set_split_ratio(&mut self, index: usize, ratio: f32) -> bool {
        if let Some(slot) = split_ratio_at(self, &mut 0, index) {
            *slot = ratio.clamp(0.08, 0.92);
            true
        } else {
            false
        }
    }

    /// Select tab `i` in the group that contains `member`.
    pub fn select_tab(&mut self, member: &str, i: usize) -> bool {
        match self {
            Self::Tabs { panes, active } => {
                if panes.iter().any(|p| p.id == member) && i < panes.len() {
                    *active = i;
                    true
                } else {
                    false
                }
            }
            Self::Split { first, second, .. } => {
                first.select_tab(member, i) || second.select_tab(member, i)
            }
            Self::Leaf(_) => false,
        }
    }

    /// Select tab `i` in the `group`th tab node (depth-first).
    pub fn select_tab_group(&mut self, group: usize, i: usize) -> bool {
        select_tab_at(self, &mut 0, group, i)
    }

    fn insert_beside(&mut self, dest: &str, panel: Panel) -> bool {
        match self {
            Self::Leaf(p) if p.id == dest => {
                let keep = p.clone();
                *self = Self::tabs(vec![keep, panel], 1);
                true
            }
            Self::Tabs { panes, .. } => {
                if panes.iter().any(|p| p.id == dest) {
                    panes.push(panel);
                    true
                } else {
                    false
                }
            }
            Self::Split { first, second, .. } => {
                first.insert_beside(dest, panel.clone()) || second.insert_beside(dest, panel)
            }
            Self::Leaf(_) => false,
        }
    }

    /// Flatten for a view: (id, title, active-in-tabs).
    pub fn slots(&self) -> Vec<(String, String, bool)> {
        let mut out = Vec::new();
        collect_slots(self, &mut out);
        out
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        let mut n: Self = serde_json::from_str(s)?;
        n.clamp_ratio();
        Ok(n)
    }
}

fn is_vacant(node: &DockNode) -> bool {
    matches!(node, DockNode::Tabs { panes, .. } if panes.is_empty())
}

fn select_tab_at(node: &mut DockNode, i: &mut usize, want: usize, tab: usize) -> bool {
    match node {
        DockNode::Tabs { panes, active } => {
            let hit = *i == want;
            *i += 1;
            if hit && tab < panes.len() {
                *active = tab;
                true
            } else {
                false
            }
        }
        DockNode::Split { first, second, .. } => {
            select_tab_at(first, i, want, tab) || select_tab_at(second, i, want, tab)
        }
        DockNode::Leaf(_) => false,
    }
}

fn split_ratio_get(node: &DockNode, i: &mut usize, want: usize) -> Option<f32> {
    match node {
        DockNode::Split {
            ratio,
            first,
            second,
            ..
        } => {
            if *i == want {
                return Some(*ratio);
            }
            *i += 1;
            split_ratio_get(first, i, want).or_else(|| split_ratio_get(second, i, want))
        }
        _ => None,
    }
}

fn split_axis_at(node: &DockNode, i: &mut usize, want: usize) -> Option<Axis> {
    match node {
        DockNode::Split {
            axis,
            first,
            second,
            ..
        } => {
            if *i == want {
                return Some(*axis);
            }
            *i += 1;
            split_axis_at(first, i, want).or_else(|| split_axis_at(second, i, want))
        }
        _ => None,
    }
}

fn split_ratio_at<'a>(node: &'a mut DockNode, i: &mut usize, want: usize) -> Option<&'a mut f32> {
    match node {
        DockNode::Split {
            ratio,
            first,
            second,
            ..
        } => {
            if *i == want {
                return Some(ratio);
            }
            *i += 1;
            split_ratio_at(first, i, want).or_else(|| split_ratio_at(second, i, want))
        }
        _ => None,
    }
}

fn collect_slots(node: &DockNode, out: &mut Vec<(String, String, bool)>) {
    match node {
        DockNode::Leaf(p) => out.push((p.id.clone(), p.title.clone(), true)),
        DockNode::Split { first, second, .. } => {
            collect_slots(first, out);
            collect_slots(second, out);
        }
        DockNode::Tabs { active, panes } => {
            for (i, p) in panes.iter().enumerate() {
                out.push((p.id.clone(), p.title.clone(), i == *active));
            }
        }
    }
}

/// Named layout snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Perspective {
    pub name: String,
    pub root: DockNode,
}

impl Perspective {
    pub fn new(name: impl Into<String>, root: DockNode) -> Self {
        Self {
            name: name.into(),
            root,
        }
    }
}

/// Compact width collapses side docks into a drawer.
pub fn drawer_open(width: f32) -> bool {
    !Breakpoint::from_width(width).sidebar_beside()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_save_restore_and_move() {
        let root = DockNode::split(
            Axis::Horizontal,
            0.25,
            DockNode::leaf("explorer", "Explorer"),
            DockNode::split(
                Axis::Vertical,
                0.7,
                DockNode::tabs(
                    vec![Panel::new("edit", "Edit"), Panel::new("term", "Terminal")],
                    0,
                ),
                DockNode::leaf("panel", "Panel"),
            ),
        );
        let json = root.to_json().unwrap();
        let back = DockNode::from_json(&json).unwrap();
        assert_eq!(back.find("explorer").unwrap().title, "Explorer");
        assert_eq!(back.slots().len(), 4);
        let mut moved = back;
        assert!(moved.move_panel("term", "panel"));
        assert!(moved.find("term").is_some());
        assert!(!moved.move_panel("missing", "edit"));
        let mut p = Panel::new("x", "X").with_limits(40.0, 200.0);
        p = p.with_limits(120.0, 80.0);
        assert_eq!(p.min, 120.0);
        assert_eq!(p.max, 120.0);
        assert!(drawer_open(400.0));
        assert!(!drawer_open(1200.0));
        assert!(DockNode::from_json("not").is_err());
        let mut tabs = DockNode::tabs(vec![Panel::new("a", "A")], 9);
        tabs.clamp_ratio();
        assert!(matches!(tabs, DockNode::Tabs { active: 0, .. }));
        let coding = Perspective::new("coding", DockNode::leaf("e", "E"));
        assert_eq!(coding.name, "coding");
        let mut pair = DockNode::split(
            Axis::Horizontal,
            0.4,
            DockNode::leaf("a", "A"),
            DockNode::leaf("b", "B"),
        );
        assert!(pair.move_panel("a", "b"));
        assert!(pair.find("a").is_some());
        assert!(pair.find("b").is_some());
        assert!(matches!(
            &pair,
            DockNode::Tabs { panes, .. }
                if panes.len() == 2
                    && panes.iter().any(|p| p.id == "a")
                    && panes.iter().any(|p| p.id == "b")
        ));
        let mut root_only = DockNode::leaf("only", "Only");
        assert!(!root_only.move_panel("only", "only"));
        assert!(root_only.find("only").is_some());
        assert!(!root_only.insert_beside("nope", Panel::new("z", "Z")));
        assert!(root_only.insert_beside("only", Panel::new("z", "Z")));
        assert!(matches!(&root_only, DockNode::Tabs { panes, .. } if panes.len() == 2));
        assert!(pair.select_tab("a", 1));
        assert_eq!(pair.split_axis(0), None);
        let mut ratios = DockNode::split(
            Axis::Vertical,
            0.3,
            DockNode::leaf("l", "L"),
            DockNode::leaf("r", "R"),
        );
        assert_eq!(ratios.split_axis(0), Some(Axis::Vertical));
        assert!(ratios.set_split_ratio(0, 0.55));
        assert!(!ratios.set_split_ratio(3, 0.2));
        let mut empty = DockNode::tabs(vec![], 3);
        empty.clamp_ratio();
        let mut split = DockNode::split(
            Axis::Vertical,
            1.5,
            DockNode::leaf("a", "A"),
            DockNode::leaf("b", "B"),
        );
        split.clamp_ratio();
        assert!(matches!(split, DockNode::Split { ratio, .. } if ratio <= 0.92));
    }

    #[test]
    fn move_panel_failed_dest_leaves_tree() {
        let mut tabs = DockNode::tabs(vec![Panel::new("a", "A"), Panel::new("b", "B")], 0);
        let before = tabs.clone();
        assert!(!tabs.move_panel("a", "missing"));
        assert_eq!(tabs, before);
        assert!(tabs.find("a").is_some());
        assert!(!tabs.move_panel("a", "a"));
        assert_eq!(tabs, before);
    }

    #[test]
    fn move_panel_last_tab_collapses_split() {
        let mut root = DockNode::split(
            Axis::Horizontal,
            0.5,
            DockNode::tabs(vec![Panel::new("a", "A")], 0),
            DockNode::leaf("b", "B"),
        );
        assert!(root.move_panel("a", "b"));
        assert!(root.find("a").is_some());
        assert!(root.find("b").is_some());
        assert!(matches!(
            &root,
            DockNode::Tabs { panes, .. }
                if panes.len() == 2
                    && panes.iter().any(|p| p.id == "a")
                    && panes.iter().any(|p| p.id == "b")
        ));
    }

    #[test]
    fn move_panel_takes_second_leaf_and_nested_tab() {
        let mut pair = DockNode::split(
            Axis::Horizontal,
            0.4,
            DockNode::leaf("a", "A"),
            DockNode::leaf("b", "B"),
        );
        assert!(!pair.move_panel("missing", "a"));
        assert!(pair.find("a").is_some());
        assert!(pair.move_panel("b", "a"));
        assert!(matches!(&pair, DockNode::Tabs { panes, .. } if panes.len() == 2));
        let mut nested = DockNode::split(
            Axis::Horizontal,
            0.5,
            DockNode::leaf("keep", "Keep"),
            DockNode::tabs(vec![Panel::new("gone", "Gone")], 0),
        );
        assert!(nested.move_panel("gone", "keep"));
        assert!(matches!(&nested, DockNode::Tabs { panes, .. } if panes.len() == 2));
        let mut keep_tabs = DockNode::split(
            Axis::Horizontal,
            0.5,
            DockNode::tabs(vec![Panel::new("a", "A"), Panel::new("x", "X")], 1),
            DockNode::leaf("b", "B"),
        );
        assert!(keep_tabs.move_panel("a", "b"));
        assert!(keep_tabs.find("x").is_some());
        assert!(keep_tabs.find("a").is_some());
        assert!(matches!(
            &keep_tabs,
            DockNode::Split { first, .. }
                if matches!(
                    first.as_ref(),
                    DockNode::Tabs { panes, active } if panes.len() == 1 && *active == 0
                )
        ));
    }

    #[test]
    fn split_ratio_and_select_tab_walk_nested_nodes() {
        let mut tree = DockNode::split(
            Axis::Horizontal,
            0.3,
            DockNode::leaf("a", "A"),
            DockNode::split(
                Axis::Vertical,
                0.4,
                DockNode::tabs(vec![Panel::new("b", "B"), Panel::new("c", "C")], 0),
                DockNode::leaf("d", "D"),
            ),
        );
        assert_eq!(tree.split_ratio(0), Some(0.3));
        assert_eq!(tree.split_ratio(1), Some(0.4));
        assert_eq!(tree.split_axis(1), Some(Axis::Vertical));
        assert!(tree.set_split_ratio(1, 0.6));
        assert_eq!(tree.split_ratio(1).map(|r| (r * 100.0).round()), Some(60.0));
        assert!(tree.select_tab("b", 1));
        assert!(!tree.select_tab("b", 9));
        assert!(!tree.select_tab("missing", 0));
        assert!(!DockNode::leaf("z", "Z").select_tab("z", 0));
        assert!(!tree.select_tab_group(4, 0));
        let mut tabs = DockNode::tabs(vec![Panel::new("a", "A"), Panel::new("b", "B")], 0);
        assert!(!tabs.insert_beside("nope", Panel::new("z", "Z")));
        assert!(tabs.insert_beside("a", Panel::new("z", "Z")));
        assert_eq!(tree.split_ratio(9), None);
        assert_eq!(tree.split_axis(9), None);
    }

    #[test]
    fn select_tab_group_out_of_range_skips_other_groups() {
        let mut root = DockNode::split(
            Axis::Horizontal,
            0.5,
            DockNode::tabs(vec![Panel::new("a", "A")], 0),
            DockNode::tabs(vec![Panel::new("b", "B"), Panel::new("c", "C")], 0),
        );
        assert!(!root.select_tab_group(0, 1));
        assert!(matches!(
            &root,
            DockNode::Split { second, .. }
                if matches!(
                    second.as_ref(),
                    DockNode::Tabs { active, panes }
                        if *active == 0 && panes[0].id == "b"
                )
        ));
        assert!(root.select_tab_group(1, 1));
        assert!(matches!(
            &root,
            DockNode::Split { second, .. }
                if matches!(second.as_ref(), DockNode::Tabs { active: 1, .. })
        ));
    }
}
