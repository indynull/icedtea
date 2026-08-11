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
    pub fn move_panel(&mut self, id: &str, dest: &str) -> bool {
        let Some(panel) = self.take(id) else {
            return false;
        };
        self.insert_beside(dest, panel)
    }

    fn take(&mut self, id: &str) -> Option<Panel> {
        match self {
            Self::Leaf(p) if p.id == id => None,
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
            Self::Split { first, second, .. } => first.take(id).or_else(|| second.take(id)),
        }
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
        match tabs {
            DockNode::Tabs { active, .. } => assert_eq!(active, 0),
            _ => panic!("tabs"),
        }
        let coding = Perspective::new("coding", DockNode::leaf("e", "E"));
        assert_eq!(coding.name, "coding");
        let mut leaf = DockNode::leaf("only", "Only");
        assert!(leaf.take("only").is_none());
        assert!(!leaf.insert_beside("nope", Panel::new("z", "Z")));
        assert!(leaf.insert_beside("only", Panel::new("z", "Z")));
        match &leaf {
            DockNode::Tabs { panes, .. } => assert_eq!(panes.len(), 2),
            _ => panic!("became tabs"),
        }
        let mut empty = DockNode::tabs(vec![], 3);
        empty.clamp_ratio();
        let mut split = DockNode::split(
            Axis::Vertical,
            1.5,
            DockNode::leaf("a", "A"),
            DockNode::leaf("b", "B"),
        );
        split.clamp_ratio();
        if let DockNode::Split { ratio, .. } = split {
            assert!(ratio <= 0.92);
        }
    }
}
