//! List / table / tree models and virtualization.

/// Visible row window for a virtualized list or table.
///
/// ```
/// let v = icedtea::collection::visible_range(40.0, 200.0, 20.0, 100);
/// assert_eq!(v.start, 2);
/// assert!(v.end > v.start);
/// ```
pub fn visible_range(scroll: f32, viewport: f32, row_h: f32, n: usize) -> std::ops::Range<usize> {
    if n == 0 || row_h <= 0.0 || viewport <= 0.0 {
        return 0..0;
    }
    let start = ((scroll.max(0.0) / row_h).floor() as usize).min(n);
    let count = ((viewport / row_h).ceil() as usize).saturating_add(1);
    let end = (start + count).min(n);
    start..end
}

/// Top pad, visible indices, bottom pad so a scrollable can reach every row.
///
/// ```
/// let (top, vis, bot) = icedtea::collection::virtual_pads(100, 20.0, 40.0, 200.0);
/// assert_eq!(vis.start, 2);
/// assert!((top - 40.0).abs() < 0.01);
/// assert!(top + (vis.end - vis.start) as f32 * 20.0 + bot >= 2000.0 - 20.0);
/// ```
pub fn virtual_pads(
    len: usize,
    row_h: f32,
    scroll: f32,
    viewport: f32,
) -> (f32, std::ops::Range<usize>, f32) {
    let h = row_h.max(0.0);
    let vis = visible_range(scroll, viewport, h, len);
    let top = vis.start as f32 * h;
    let bot = (len.saturating_sub(vis.end) as f32) * h;
    (top, vis, bot)
}

/// Thumb offset and length on a rail. `min_handle` keeps the grab usable
/// when `content` is much taller than `viewport` (iced's own scroller
/// floors at 2px).
///
/// ```
/// let (y, h) = icedtea::collection::scroller_span(9000.0, 400.0, 0.0, 400.0, 24.0);
/// assert_eq!(h, 24.0);
/// assert_eq!(y, 0.0);
/// let (end, _) = icedtea::collection::scroller_span(9000.0, 400.0, 8600.0, 400.0, 24.0);
/// assert!((end - (400.0 - 24.0)).abs() < 0.01);
/// ```
pub fn scroller_span(
    content: f32,
    viewport: f32,
    scroll: f32,
    rail: f32,
    min_handle: f32,
) -> (f32, f32) {
    if rail <= 0.0 {
        return (0.0, 0.0);
    }
    if content <= viewport {
        return (0.0, rail);
    }
    let handle = (rail * (viewport / content)).max(min_handle).min(rail);
    let max_scroll = (content - viewport).max(1.0);
    let usable = (rail - handle).max(0.0);
    let t = (scroll.max(0.0) / max_scroll).clamp(0.0, 1.0);
    (usable * t, handle)
}

/// Scroll offset that puts the thumb at `thumb_y` on the rail.
pub fn scroll_from_rail(
    content: f32,
    viewport: f32,
    thumb_y: f32,
    rail: f32,
    min_handle: f32,
) -> f32 {
    let (_, handle) = scroller_span(content, viewport, 0.0, rail, min_handle);
    let max_scroll = (content - viewport).max(0.0);
    let usable = (rail - handle).max(1.0);
    (thumb_y.clamp(0.0, usable) / usable) * max_scroll
}

/// List model: length, identity, label.
pub trait ListModel {
    fn len(&self) -> usize;
    fn id(&self, index: usize) -> u64;
    fn label(&self, index: usize) -> String;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Simple owned list.
#[derive(Debug, Clone, Default)]
pub struct VecList {
    pub items: Vec<String>,
}

impl ListModel for VecList {
    fn len(&self) -> usize {
        self.items.len()
    }
    fn id(&self, index: usize) -> u64 {
        index as u64
    }
    fn label(&self, index: usize) -> String {
        self.items.get(index).cloned().unwrap_or_default()
    }
}

/// Selection for lists/tables/trees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    None,
    Single(usize),
    Multi(Vec<usize>),
}

impl Selection {
    pub fn contains(&self, i: usize) -> bool {
        match self {
            Self::None => false,
            Self::Single(s) => *s == i,
            Self::Multi(v) => v.contains(&i),
        }
    }

    pub fn select_single(&mut self, i: usize) {
        *self = Self::Single(i);
    }

    pub fn toggle_multi(&mut self, i: usize) {
        match self {
            Self::Multi(v) => {
                if let Some(p) = v.iter().position(|x| *x == i) {
                    v.remove(p);
                } else {
                    v.push(i);
                    v.sort_unstable();
                }
            }
            _ => *self = Self::Multi(vec![i]),
        }
    }

    pub fn primary(&self) -> Option<usize> {
        match self {
            Self::None => None,
            Self::Single(i) => Some(*i),
            Self::Multi(v) if v.is_empty() => None,
            Self::Multi(v) => Some(v[0]),
        }
    }
}

/// Table: headers + rows of cells.
#[derive(Debug, Clone, Default)]
pub struct TableModel {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub sort_col: Option<usize>,
    pub sort_asc: bool,
}

impl TableModel {
    pub fn sort(&mut self, col: usize) {
        if self.sort_col == Some(col) {
            self.sort_asc = !self.sort_asc;
        } else {
            self.sort_col = Some(col);
            self.sort_asc = true;
        }
        let asc = self.sort_asc;
        self.rows.sort_by(|a, b| {
            let av = a.get(col).map(String::as_str).unwrap_or("");
            let bv = b.get(col).map(String::as_str).unwrap_or("");
            if asc {
                av.cmp(bv)
            } else {
                bv.cmp(av)
            }
        });
    }

    pub fn cell(&self, row: usize, col: usize) -> &str {
        self.rows
            .get(row)
            .and_then(|r| r.get(col))
            .map(String::as_str)
            .unwrap_or("")
    }

    pub fn resize_column(widths: &mut [f32], col: usize, delta: f32, min: f32) {
        if let Some(w) = widths.get_mut(col) {
            *w = (*w + delta).max(min);
        }
    }
}

/// Tree node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    pub id: u64,
    pub label: String,
    pub expanded: bool,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn leaf(id: u64, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            expanded: false,
            children: Vec::new(),
        }
    }

    pub fn branch(id: u64, label: impl Into<String>, children: Vec<TreeNode>) -> Self {
        Self {
            id,
            label: label.into(),
            expanded: true,
            children,
        }
    }

    pub fn flatten(&self) -> Vec<(u32, u64, String, bool, bool)> {
        let mut out = Vec::new();
        flatten_into(self, 0, &mut out);
        out
    }
}

fn flatten_into(node: &TreeNode, depth: u32, out: &mut Vec<(u32, u64, String, bool, bool)>) {
    let has_children = !node.children.is_empty();
    out.push((
        depth,
        node.id,
        node.label.clone(),
        node.expanded,
        has_children,
    ));
    if node.expanded {
        for c in &node.children {
            flatten_into(c, depth + 1, out);
        }
    }
}

/// Toggle expand by id.
pub fn tree_toggle(node: &mut TreeNode, id: u64) -> bool {
    if node.id == id {
        node.expanded = !node.expanded;
        return true;
    }
    for c in &mut node.children {
        if tree_toggle(c, id) {
            return true;
        }
    }
    false
}

/// Pagination window.
///
/// ```
/// let r = icedtea::collection::page_range(100, 2, 10);
/// assert_eq!(r, 20..30);
/// ```
pub fn page_range(len: usize, page: usize, per_page: usize) -> std::ops::Range<usize> {
    let per = per_page.max(1);
    let start = page.saturating_mul(per).min(len);
    let end = start.saturating_add(per).min(len);
    start..end
}

pub fn page_count(len: usize, per_page: usize) -> usize {
    if len == 0 {
        return 0;
    }
    len.div_ceil(per_page.max(1))
}

/// Tab strip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tabs {
    pub titles: Vec<String>,
    pub active: usize,
    pub closable: bool,
}

impl Tabs {
    pub fn new(titles: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            titles: titles.into_iter().map(Into::into).collect(),
            active: 0,
            closable: false,
        }
    }

    pub fn select(&mut self, i: usize) {
        if i < self.titles.len() {
            self.active = i;
        }
    }

    pub fn close(&mut self, i: usize) -> Option<String> {
        if !self.closable || i >= self.titles.len() {
            return None;
        }
        let removed = self.titles.remove(i);
        if self.active >= self.titles.len() {
            self.active = self.titles.len().saturating_sub(1);
        } else if i < self.active {
            self.active -= 1;
        }
        Some(removed)
    }
}

/// Accordion: which section is open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accordion {
    pub open: Option<usize>,
}

impl Accordion {
    pub fn toggle(&mut self, i: usize) {
        self.open = if self.open == Some(i) { None } else { Some(i) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtualize_select_sort_tree_tabs() {
        assert_eq!(visible_range(0.0, 0.0, 20.0, 10), 0..0);
        assert_eq!(visible_range(0.0, 100.0, 0.0, 10), 0..0);
        let (top, vis, bot) = virtual_pads(100, 20.0, 40.0, 200.0);
        assert_eq!(vis.start, 2);
        assert!((top - 40.0).abs() < 0.01);
        assert!((top + (vis.end - vis.start) as f32 * 20.0 + bot - 2000.0).abs() < 0.01);
        let (t0, v0, b0) = virtual_pads(0, 20.0, 0.0, 100.0);
        assert_eq!((t0, v0, b0), (0.0, 0..0, 0.0));
        let v = visible_range(40.0, 200.0, 20.0, 100);
        assert_eq!(v.start, 2);
        assert!(v.end <= 100);
        let list = VecList {
            items: vec!["a".into(), "b".into()],
        };
        assert_eq!(list.len(), 2);
        assert_eq!(list.id(0), 0);
        assert_eq!(list.label(1), "b");
        assert!(!list.is_empty());
        assert!(VecList::default().is_empty());
        assert_eq!(list.label(9), "");
        let mut sel = Selection::None;
        assert!(!sel.contains(0));
        assert!(sel.primary().is_none());
        sel.select_single(2);
        assert!(sel.contains(2));
        assert_eq!(sel.primary(), Some(2));
        sel.toggle_multi(2);
        sel.toggle_multi(1);
        assert!(sel.contains(1) && sel.contains(2));
        sel.toggle_multi(2);
        assert!(!sel.contains(2));
        assert_eq!(sel.primary(), Some(1));
        let multi = Selection::Multi(vec![4, 5]);
        assert_eq!(multi.primary(), Some(4));
        assert!(Selection::Multi(vec![]).primary().is_none());
        let mut table = TableModel {
            headers: vec!["n".into()],
            rows: vec![vec!["b".into()], vec!["a".into()]],
            sort_col: None,
            sort_asc: true,
        };
        table.sort(0);
        assert_eq!(table.cell(0, 0), "a");
        table.sort(0);
        assert_eq!(table.cell(0, 0), "b");
        assert_eq!(table.cell(9, 9), "");
        let mut widths = [80.0, 80.0];
        TableModel::resize_column(&mut widths, 0, 10.0, 40.0);
        assert_eq!(widths[0], 90.0);
        TableModel::resize_column(&mut widths, 9, 1.0, 10.0);
        let mut tree = TreeNode::branch(
            1,
            "root",
            vec![TreeNode::leaf(2, "c"), TreeNode::leaf(3, "d")],
        );
        assert_eq!(tree.flatten().len(), 3);
        assert!(tree_toggle(&mut tree, 1));
        assert!(!tree.expanded);
        assert_eq!(tree.flatten().len(), 1);
        assert!(tree_toggle(&mut tree, 1));
        assert!(tree_toggle(&mut tree, 2));
        assert!(!tree_toggle(&mut tree, 99));
        assert_eq!(page_range(100, 2, 10), 20..30);
        assert_eq!(page_range(5, 9, 10), 5..5);
        assert_eq!(page_count(0, 10), 0);
        assert_eq!(page_count(11, 10), 2);
        let mut tabs = Tabs::new(["A", "B", "C"]);
        tabs.closable = true;
        tabs.select(9);
        tabs.select(2);
        assert_eq!(tabs.close(2).as_deref(), Some("C"));
        assert_eq!(tabs.active, 1);
        tabs.select(1);
        assert_eq!(tabs.close(0).as_deref(), Some("A"));
        assert_eq!(tabs.active, 0);
        let mut tabs = Tabs::new(["A", "B", "C", "D"]);
        tabs.closable = true;
        tabs.select(2);
        assert_eq!(tabs.close(0).as_deref(), Some("A"));
        assert_eq!(tabs.active, 1);
        tabs.closable = false;
        assert!(tabs.close(0).is_none());
        let mut acc = Accordion { open: None };
        acc.toggle(1);
        assert_eq!(acc.open, Some(1));
        acc.toggle(1);
        assert_eq!(acc.open, None);
    }

    #[test]
    fn scroller_keeps_a_usable_handle_on_tall_content() {
        let (y, h) = scroller_span(900.0 * 60.0, 400.0, 0.0, 400.0, 24.0);
        assert_eq!(h, 24.0);
        assert_eq!(y, 0.0);
        let max_scroll = 900.0 * 60.0 - 400.0;
        let (end, h2) = scroller_span(900.0 * 60.0, 400.0, max_scroll, 400.0, 24.0);
        assert_eq!(h2, 24.0);
        assert!((end - 376.0).abs() < 0.01);
        let mid = scroll_from_rail(900.0 * 60.0, 400.0, 188.0, 400.0, 24.0);
        assert!(mid > 0.0 && mid < max_scroll);
        let (y0, full) = scroller_span(100.0, 400.0, 0.0, 400.0, 24.0);
        assert_eq!((y0, full), (0.0, 400.0));
        assert_eq!(scroller_span(100.0, 50.0, 0.0, 0.0, 24.0), (0.0, 0.0));
        assert_eq!(scroll_from_rail(100.0, 400.0, 10.0, 400.0, 24.0), 0.0);
    }
}
