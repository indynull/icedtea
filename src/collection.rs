//! List / table / tree models and virtualization.

/// Extra rows mounted above and below the strict viewport.
pub const OVERSCAN: usize = 4;

/// Visible row window for a virtualized list or table.
///
/// `start..end` are mounted indices (overscan and cover already applied).
/// `scroll` is the live pixel offset. The rail is a view of that number.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisibleWindow {
    pub start: usize,
    pub end: usize,
    pub scroll: f32,
    pub viewport: f32,
}

impl VisibleWindow {
    pub fn new(viewport: f32) -> Self {
        Self {
            start: 0,
            end: 0,
            scroll: 0.0,
            viewport,
        }
    }

    pub fn mounted(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn range(self) -> std::ops::Range<usize> {
        self.start..self.end
    }
}

impl Default for VisibleWindow {
    fn default() -> Self {
        Self::new(240.0)
    }
}

/// Strict viewport indices (no overscan, no cover).
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

/// Mounted window: viewport plus overscan, plus an optional cover index
/// so a selected row stays mounted when it leaves the viewport.
///
/// ```
/// let w = icedtea::collection::visible_window(40.0, 200.0, 20.0, 100, 2, Some(0));
/// assert_eq!(w.start, 0);
/// assert!(w.end > 10);
/// assert_eq!(w.scroll, 40.0);
/// ```
pub fn visible_window(
    scroll: f32,
    viewport: f32,
    row_h: f32,
    n: usize,
    overscan: usize,
    cover: Option<usize>,
) -> VisibleWindow {
    let scroll = scroll.max(0.0);
    let vis = visible_range(scroll, viewport, row_h, n);
    if n == 0 {
        return VisibleWindow {
            start: 0,
            end: 0,
            scroll,
            viewport,
        };
    }
    let mut start = vis.start.saturating_sub(overscan);
    let mut end = vis.end.saturating_add(overscan).min(n);
    if let Some(c) = cover {
        if c < n {
            start = start.min(c);
            end = end.max(c.saturating_add(1)).min(n);
        }
    }
    VisibleWindow {
        start,
        end,
        scroll,
        viewport,
    }
}

/// `Some(next)` when the mounted index range or viewport size changed.
///
/// ```
/// use icedtea::collection::{range_if_changed, VisibleWindow};
/// let a = VisibleWindow { start: 0, end: 10, scroll: 0.0, viewport: 200.0 };
/// let pixel = VisibleWindow { start: 0, end: 10, scroll: 4.0, viewport: 200.0 };
/// assert!(range_if_changed(a, pixel).is_none());
/// let next = VisibleWindow { start: 1, end: 11, scroll: 20.0, viewport: 200.0 };
/// assert_eq!(range_if_changed(a, next).unwrap().start, 1);
/// ```
pub fn range_if_changed(prev: VisibleWindow, next: VisibleWindow) -> Option<VisibleWindow> {
    if prev.start != next.start
        || prev.end != next.end
        || (prev.viewport - next.viewport).abs() > f32::EPSILON
    {
        Some(next)
    } else {
        None
    }
}

/// Next window after a rail, wheel, or pane resize. One pixel `scroll`.
pub fn window_after_scroll(
    prev: VisibleWindow,
    scroll: f32,
    viewport: f32,
    row_h: f32,
    len: usize,
    overscan: usize,
    cover: Option<usize>,
) -> VisibleWindow {
    let h = row_h.max(0.0);
    let viewport = if viewport > 0.0 {
        viewport
    } else {
        prev.viewport
    };
    let max_scroll = (len as f32 * h - viewport).max(0.0);
    visible_window(
        scroll.clamp(0.0, max_scroll),
        viewport,
        h,
        len,
        overscan,
        cover,
    )
}

/// Top pad, mounted window, bottom pad so a scrollable can reach every row.
///
/// ```
/// let (top, win, bot) = icedtea::collection::virtual_pads(100, 20.0, 40.0, 200.0, 0, None);
/// assert_eq!(win.start, 2);
/// assert!((top - 40.0).abs() < 0.01);
/// assert!(top + win.mounted() as f32 * 20.0 + bot >= 2000.0 - 20.0);
/// ```
pub fn virtual_pads(
    len: usize,
    row_h: f32,
    scroll: f32,
    viewport: f32,
    overscan: usize,
    cover: Option<usize>,
) -> (f32, VisibleWindow, f32) {
    let h = row_h.max(0.0);
    let win = visible_window(scroll, viewport, h, len, overscan, cover);
    let top = win.start as f32 * h;
    let bot = (len.saturating_sub(win.end) as f32) * h;
    (top, win, bot)
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

/// List model: length, identity, borrowed title and optional meta.
pub trait ListModel {
    fn len(&self) -> usize;
    fn id(&self, index: usize) -> u64;
    fn title(&self, index: usize) -> &str;
    fn meta(&self, index: usize) -> Option<&str> {
        let _ = index;
        None
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn is_separator(&self, index: usize) -> bool {
        let _ = index;
        false
    }
}

/// One owned list row.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListRow {
    pub title: String,
    pub meta: Option<String>,
    pub separator: bool,
}

impl ListRow {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            meta: None,
            separator: false,
        }
    }

    pub fn with_meta(mut self, meta: impl Into<String>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    pub fn separator() -> Self {
        Self {
            title: String::new(),
            meta: None,
            separator: true,
        }
    }
}

/// Simple owned list.
#[derive(Debug, Clone, Default)]
pub struct VecList {
    pub items: Vec<ListRow>,
}

impl VecList {
    pub fn titles(items: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            items: items.into_iter().map(ListRow::new).collect(),
        }
    }
}

impl ListModel for VecList {
    fn len(&self) -> usize {
        self.items.len()
    }
    fn id(&self, index: usize) -> u64 {
        index as u64
    }
    fn title(&self, index: usize) -> &str {
        self.items
            .get(index)
            .map(|r| r.title.as_str())
            .unwrap_or("")
    }
    fn meta(&self, index: usize) -> Option<&str> {
        self.items.get(index).and_then(|r| r.meta.as_deref())
    }

    fn is_separator(&self, index: usize) -> bool {
        self.items.get(index).is_some_and(|r| r.separator)
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

    /// Inclusive range from `from` through `to`.
    pub fn select_range(&mut self, from: usize, to: usize) {
        let (a, b) = if from <= to { (from, to) } else { (to, from) };
        *self = Self::Multi((a..=b).collect());
    }

    /// Grow a multi selection from the primary index to `to`.
    pub fn extend_to(&mut self, to: usize) {
        let from = self.primary().unwrap_or(to);
        self.select_range(from, to);
    }

    /// Move the primary index by `delta`, clamped to `0..len`.
    pub fn move_primary(&mut self, delta: i32, len: usize) -> Option<usize> {
        if len == 0 {
            *self = Self::None;
            return None;
        }
        let cur = self.primary().unwrap_or(0) as i32;
        let next = (cur + delta).clamp(0, len as i32 - 1) as usize;
        self.select_single(next);
        Some(next)
    }
}

/// Prefix sums of per-row heights. `out[0] == 0`, `out[n] == total`.
///
/// ```
/// let off = icedtea::collection::row_offsets(&[20.0, 40.0, 20.0]);
/// assert_eq!(off, vec![0.0, 20.0, 60.0, 80.0]);
/// ```
pub fn row_offsets(heights: &[f32]) -> Vec<f32> {
    let mut out = Vec::with_capacity(heights.len() + 1);
    out.push(0.0);
    let mut acc = 0.0;
    for h in heights {
        acc += h.max(0.0);
        out.push(acc);
    }
    out
}

/// Strict viewport for variable-height rows.
///
/// ```
/// let h = [20.0, 40.0, 20.0, 80.0, 20.0];
/// let r = icedtea::collection::visible_range_var(20.0, 60.0, &h);
/// assert_eq!(r, 1..3);
/// ```
pub fn visible_range_var(scroll: f32, viewport: f32, heights: &[f32]) -> std::ops::Range<usize> {
    let n = heights.len();
    if n == 0 || viewport <= 0.0 {
        return 0..0;
    }
    let off = row_offsets(heights);
    let scroll = scroll.max(0.0);
    let end_y = scroll + viewport;
    let start = off
        .iter()
        .position(|&y| y > scroll)
        .map(|i| i.saturating_sub(1))
        .unwrap_or(n)
        .min(n);
    let end = off
        .iter()
        .position(|&y| y >= end_y)
        .unwrap_or(n)
        .min(n)
        .max(start);
    start..end
}

/// Mounted window for variable-height rows.
pub fn visible_window_var(
    scroll: f32,
    viewport: f32,
    heights: &[f32],
    overscan: usize,
    cover: Option<usize>,
) -> VisibleWindow {
    let n = heights.len();
    let scroll = scroll.max(0.0);
    let vis = visible_range_var(scroll, viewport, heights);
    if n == 0 {
        return VisibleWindow {
            start: 0,
            end: 0,
            scroll,
            viewport,
        };
    }
    let mut start = vis.start.saturating_sub(overscan);
    let mut end = vis.end.saturating_add(overscan).min(n);
    if let Some(c) = cover {
        if c < n {
            start = start.min(c);
            end = end.max(c.saturating_add(1)).min(n);
        }
    }
    VisibleWindow {
        start,
        end,
        scroll,
        viewport,
    }
}

/// Pads for a variable-height virtual list.
pub fn virtual_pads_var(
    heights: &[f32],
    scroll: f32,
    viewport: f32,
    overscan: usize,
    cover: Option<usize>,
) -> (f32, VisibleWindow, f32) {
    let off = row_offsets(heights);
    let win = visible_window_var(scroll, viewport, heights, overscan, cover);
    let top = off.get(win.start).copied().unwrap_or(0.0);
    let total = off.last().copied().unwrap_or(0.0);
    let mounted_end = off.get(win.end).copied().unwrap_or(total);
    (top, win, (total - mounted_end).max(0.0))
}

/// True when `index` is in the mounted window (lazy / recycle).
pub fn row_is_mounted(window: VisibleWindow, index: usize) -> bool {
    index >= window.start && index < window.end
}

/// Move `index` from `from` into `to` at `dest`. Returns the new dest index.
pub fn transfer_index(
    from: &mut Vec<usize>,
    to: &mut Vec<usize>,
    index: usize,
    dest: usize,
) -> usize {
    if let Some(p) = from.iter().position(|&i| i == index) {
        from.remove(p);
    }
    let dest = dest.min(to.len());
    to.insert(dest, index);
    dest
}

/// Column order and widths for a virtual table.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnLayout {
    pub widths: Vec<f32>,
    pub order: Vec<usize>,
}

impl ColumnLayout {
    pub fn new(widths: impl Into<Vec<f32>>) -> Self {
        let widths = widths.into();
        let order: Vec<usize> = (0..widths.len()).collect();
        Self { widths, order }
    }

    pub fn resize(&mut self, col: usize, delta: f32, min: f32) {
        TableModel::resize_column(&mut self.widths, col, delta, min);
    }

    pub fn reorder(&mut self, from: usize, to: usize) {
        if from >= self.order.len() || to >= self.order.len() || from == to {
            return;
        }
        let id = self.order.remove(from);
        self.order.insert(to, id);
    }

    /// Columns in `order`.
    pub fn display(&self) -> Vec<usize> {
        self.order.clone()
    }

    pub fn width(&self, col: usize) -> f32 {
        self.widths.get(col).copied().unwrap_or(96.0)
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

/// App-owned table storage. [`TableModel`] is one impl.
pub trait TableSource {
    fn row_count(&self) -> usize;
    fn column_count(&self) -> usize;
    fn header(&self, col: usize) -> &str;
    fn cell(&self, row: usize, col: usize) -> &str;
}

impl TableSource for TableModel {
    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn column_count(&self) -> usize {
        self.headers.len()
    }

    fn header(&self, col: usize) -> &str {
        self.headers.get(col).map(String::as_str).unwrap_or("")
    }

    fn cell(&self, row: usize, col: usize) -> &str {
        TableModel::cell(self, row, col)
    }
}

/// Tree node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    pub id: u64,
    pub label: String,
    pub expanded: bool,
    pub children: Vec<TreeNode>,
    pub dir: bool,
}

impl TreeNode {
    pub fn leaf(id: u64, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            expanded: false,
            children: Vec::new(),
            dir: false,
        }
    }

    pub fn branch(id: u64, label: impl Into<String>, children: Vec<TreeNode>) -> Self {
        Self {
            id,
            label: label.into(),
            expanded: true,
            children,
            dir: true,
        }
    }

    /// Empty folder. The application fills `children` when expanded.
    pub fn folder(id: u64, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            expanded: false,
            children: Vec::new(),
            dir: true,
        }
    }

    pub fn flatten(&self) -> Vec<(u32, u64, String, bool, bool)> {
        let mut out = Vec::new();
        flatten_into(self, 0, &mut out);
        out
    }
}

fn flatten_into(node: &TreeNode, depth: u32, out: &mut Vec<(u32, u64, String, bool, bool)>) {
    let has_children = node.dir || !node.children.is_empty();
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

/// Tab strip. [`Tabs::new`] starts with `closable: false` (pinned).
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

/// Document tabs with dirty flags. The application owns the documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentTabs {
    pub tabs: Tabs,
    pub dirty: Vec<bool>,
}

impl DocumentTabs {
    pub fn new(titles: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let tabs = Tabs::new(titles);
        let dirty = vec![false; tabs.titles.len()];
        Self { tabs, dirty }
    }

    pub fn mark_dirty(&mut self, i: usize, dirty: bool) {
        if let Some(d) = self.dirty.get_mut(i) {
            *d = dirty;
        }
    }

    /// `Some(true)` when the tab is dirty and the app should confirm close.
    pub fn close_needs_confirm(&self, i: usize) -> bool {
        self.tabs.closable && self.dirty.get(i).copied().unwrap_or(false)
    }

    pub fn title(&self, i: usize) -> String {
        let name = self.tabs.titles.get(i).cloned().unwrap_or_default();
        if self.dirty.get(i).copied().unwrap_or(false) {
            format!("• {name}")
        } else {
            name
        }
    }
}

/// Background job for the status strip.
#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    pub id: u64,
    pub title: String,
    pub progress: Option<f32>,
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

    struct TitlesOnly;

    impl ListModel for TitlesOnly {
        fn len(&self) -> usize {
            1
        }
        fn id(&self, _index: usize) -> u64 {
            0
        }
        fn title(&self, _index: usize) -> &str {
            "row"
        }
    }

    #[test]
    fn list_model_default_row_is_not_a_separator() {
        assert_eq!(TitlesOnly.len(), 1);
        assert_eq!(TitlesOnly.id(0), 0);
        assert_eq!(TitlesOnly.title(0), "row");
        assert!(!TitlesOnly.is_separator(0));
        assert!(TitlesOnly.meta(0).is_none());
        assert!(!TitlesOnly.is_empty());
    }

    #[test]
    fn cover_keeps_selected_row_mounted_above_the_viewport() {
        let row_h = 20.0;
        let viewport = 200.0;
        let n = 100;
        let scroll = row_h;
        let vis = visible_range(scroll, viewport, row_h, n);
        assert_eq!(vis.start, 1);
        let (top, win, bot) = virtual_pads(n, row_h, scroll, viewport, 4, Some(0));
        assert_eq!(win.start, 0);
        assert!(win.range().contains(&0));
        assert_eq!(top, win.start as f32 * row_h);
        assert!(vis.start > win.start);
        assert!((top + win.mounted() as f32 * row_h + bot - n as f32 * row_h).abs() < 0.01);
        let pixel = VisibleWindow {
            start: win.start,
            end: win.end,
            scroll: scroll + 4.0,
            viewport,
        };
        assert!(range_if_changed(win, pixel).is_none());
        let pixel_moved = window_after_scroll(win, scroll + 4.0, viewport, row_h, n, 4, Some(0));
        assert_eq!(pixel_moved.start, win.start);
        assert_eq!(pixel_moved.end, win.end);
        assert!((pixel_moved.scroll - (scroll + 4.0)).abs() < 0.01);
        let content = n as f32 * row_h;
        let (thumb0, _) = scroller_span(content, viewport, win.scroll, viewport, 24.0);
        let (thumb1, _) = scroller_span(content, viewport, pixel_moved.scroll, viewport, 24.0);
        assert!(thumb1 > thumb0);
        let jumped = window_after_scroll(win, viewport, viewport, row_h, n, 4, Some(0));
        assert_eq!(jumped.start, 0);
        assert_ne!(jumped.end, win.end);
        let next_row = visible_window(viewport, viewport, row_h, n, 4, Some(0));
        assert_eq!(next_row.start, 0);
        assert!(range_if_changed(win, next_row).is_some());
    }

    #[test]
    fn virtualize_select_sort_tree_tabs() {
        assert_eq!(visible_range(0.0, 0.0, 20.0, 10), 0..0);
        assert_eq!(visible_range(0.0, 100.0, 0.0, 10), 0..0);
        let (top, vis, bot) = virtual_pads(100, 20.0, 40.0, 200.0, 0, None);
        assert_eq!(vis.start, 2);
        assert!((top - 40.0).abs() < 0.01);
        assert!((top + vis.mounted() as f32 * 20.0 + bot - 2000.0).abs() < 0.01);
        let (t0, v0, b0) = virtual_pads(0, 20.0, 0.0, 100.0, 0, None);
        assert_eq!(t0, 0.0);
        assert_eq!(v0.range(), 0..0);
        assert_eq!(b0, 0.0);
        let v = visible_range(40.0, 200.0, 20.0, 100);
        assert_eq!(v.start, 2);
        assert!(v.end <= 100);
        let over = visible_window(40.0, 200.0, 20.0, 100, 2, None);
        assert_eq!(over.start, 0);
        assert!(over.end >= 13);
        let covered = visible_window(400.0, 200.0, 20.0, 100, 0, Some(0));
        assert_eq!(covered.start, 0);
        assert!(covered.end > 20);
        let same = VisibleWindow {
            start: 2,
            end: 14,
            scroll: 40.0,
            viewport: 200.0,
        };
        let pixel = VisibleWindow {
            start: 2,
            end: 14,
            scroll: 48.0,
            viewport: 200.0,
        };
        assert!(range_if_changed(same, pixel).is_none());
        assert!(range_if_changed(
            same,
            VisibleWindow {
                start: 3,
                end: 15,
                scroll: 60.0,
                viewport: 200.0,
            }
        )
        .is_some());
        assert!(range_if_changed(
            same,
            VisibleWindow {
                start: 2,
                end: 14,
                scroll: 40.0,
                viewport: 240.0,
            }
        )
        .is_some());
        assert_eq!(VisibleWindow::default().viewport, 240.0);
        assert_eq!(VisibleWindow::new(100.0).range(), 0..0);
        struct Titles(&'static [&'static str]);
        impl ListModel for Titles {
            fn len(&self) -> usize {
                self.0.len()
            }
            fn id(&self, index: usize) -> u64 {
                index as u64
            }
            fn title(&self, index: usize) -> &str {
                self.0.get(index).copied().unwrap_or("")
            }
        }
        let titles = Titles(&["a"]);
        assert_eq!(titles.len(), 1);
        assert_eq!(titles.id(0), 0);
        assert_eq!(titles.title(0), "a");
        assert_eq!(titles.title(9), "");
        assert!(titles.meta(0).is_none());
        assert_eq!(VecList::titles(["a", "b"]).title(0), "a");
        let list = VecList {
            items: vec![ListRow::new("a"), ListRow::new("b").with_meta("meta")],
        };
        assert_eq!(list.len(), 2);
        assert_eq!(list.id(0), 0);
        assert_eq!(list.title(1), "b");
        assert_eq!(list.meta(1), Some("meta"));
        assert!(list.meta(0).is_none());
        assert!(!list.is_separator(0));
        let mut opts = VecList::titles(["All"]);
        opts.items.push(ListRow::separator());
        opts.items.push(ListRow::new("A"));
        assert!(opts.is_separator(1));
        assert!(!opts.is_separator(2));
        assert!(!list.is_empty());
        assert!(VecList::default().is_empty());
        assert_eq!(list.title(9), "");
        assert!(list.meta(9).is_none());
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
        assert_eq!(TableSource::row_count(&table), 2);
        assert_eq!(TableSource::column_count(&table), 1);
        assert_eq!(TableSource::header(&table, 0), "n");
        assert_eq!(TableSource::cell(&table, 0, 0), "b");
        assert_eq!(TableSource::header(&table, 9), "");
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
        let folder = TreeNode::folder(9, "empty");
        assert!(folder.dir);
        assert!(folder.children.is_empty());
        assert!(!TreeNode::leaf(8, "file").dir);
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
    fn window_after_scroll_keeps_pixel_offset() {
        let row_h = 20.0;
        let viewport = 200.0;
        let n = 100;
        let prev = visible_window(40.0, viewport, row_h, n, 4, None);
        let next = window_after_scroll(prev, 44.0, viewport, row_h, n, 4, None);
        assert_eq!(next.start, prev.start);
        assert_eq!(next.end, prev.end);
        assert!((next.scroll - 44.0).abs() < 0.01);
        let content = n as f32 * row_h;
        let (y0, h0) = scroller_span(content, viewport, prev.scroll, viewport, 24.0);
        let (y1, h1) = scroller_span(content, viewport, next.scroll, viewport, 24.0);
        assert_eq!(h0, h1);
        assert!(y1 > y0);
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

    #[test]
    fn one_offset_drives_pads_and_separator_geometry() {
        let row_h = 20.0;
        let viewport = 200.0;
        let n = 100;
        let prev = VisibleWindow::new(viewport);
        let next = window_after_scroll(prev, 44.0, viewport, row_h, n, 4, None);
        assert!((next.scroll - 44.0).abs() < 0.01);
        assert!((next.viewport - viewport).abs() < 0.01);
        let (top, win, bot) = virtual_pads(n, row_h, next.scroll, next.viewport, 4, None);
        assert!((win.scroll - next.scroll).abs() < 0.01);
        assert!((top - win.start as f32 * row_h).abs() < 0.01);
        assert_eq!(top + win.mounted() as f32 * row_h + bot, n as f32 * row_h);
        let vis = visible_range(44.0, viewport, row_h, n);
        assert_eq!(vis.start, 2);
        assert!(vis.end - vis.start <= (viewport / row_h).ceil() as usize + 2);
        let sep = 1usize;
        assert!((sep as f32 * row_h - 20.0).abs() < 0.01);
        let zero_vp = window_after_scroll(next, 44.0, 0.0, row_h, n, 4, None);
        assert!((zero_vp.viewport - viewport).abs() < 0.01);
        let clamped = window_after_scroll(prev, 9_000.0, viewport, row_h, n, 4, None);
        let max_scroll = n as f32 * row_h - viewport;
        assert!((clamped.scroll - max_scroll).abs() < 0.01);
    }

    #[test]
    fn variable_height_range_and_selection_range() {
        let h = [20.0, 40.0, 20.0, 80.0, 20.0];
        assert_eq!(row_offsets(&h), vec![0.0, 20.0, 60.0, 80.0, 160.0, 180.0]);
        assert_eq!(visible_range_var(20.0, 60.0, &h), 1..3);
        assert_eq!(visible_range_var(0.0, 0.0, &h), 0..0);
        assert_eq!(visible_range_var(0.0, 40.0, &[]), 0..0);
        assert_eq!(visible_range_var(200.0, 20.0, &[10.0]), 1..1);
        let past = visible_window_var(0.0, 10.0, &[], 2, Some(0));
        assert_eq!(past.end, 0);
        let (top, win, bot) = virtual_pads_var(&h, 20.0, 60.0, 0, None);
        assert!((top - 20.0).abs() < 0.01);
        assert_eq!(win.start, 1);
        assert!(bot > 0.0);
        let covered = visible_window_var(80.0, 40.0, &h, 0, Some(0));
        assert_eq!(covered.start, 0);
        assert!(row_is_mounted(win, 2));
        assert!(!row_is_mounted(win, 0));
        let mut sel = Selection::None;
        sel.select_range(1, 3);
        assert!(sel.contains(1) && sel.contains(3));
        sel.extend_to(4);
        assert!(sel.contains(4));
        assert_eq!(sel.move_primary(1, 5), Some(2));
        let mut empty = Selection::None;
        assert_eq!(empty.move_primary(0, 0), None);
        let mut from = vec![1, 2, 3];
        let mut to = vec![9];
        assert_eq!(transfer_index(&mut from, &mut to, 2, 0), 0);
        assert_eq!(from, vec![1, 3]);
        assert_eq!(to, vec![2, 9]);
        let mut cols = ColumnLayout::new(vec![80.0, 120.0, 60.0]);
        cols.reorder(2, 1);
        assert_eq!(cols.display()[0], 0);
        assert_eq!(cols.display()[1], 2);
        cols.resize(0, 10.0, 40.0);
        assert_eq!(cols.width(0), 90.0);
        cols.reorder(9, 0);
        assert_eq!(cols.width(9), 96.0);
        let mut docs = DocumentTabs::new(["a.rs", "b.rs"]);
        docs.tabs.closable = true;
        docs.mark_dirty(0, true);
        assert!(docs.close_needs_confirm(0));
        assert!(!docs.close_needs_confirm(1));
        assert!(docs.title(0).starts_with('•'));
        assert_eq!(docs.title(1), "b.rs");
        let job = Job {
            id: 1,
            title: "Index".into(),
            progress: Some(0.4),
        };
        assert_eq!(job.title, "Index");
    }
}
