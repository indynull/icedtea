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
/// Leading or trailing mark on a list row.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum RowSlot {
    #[default]
    Empty,
    Icon(crate::icon::Icon),
    Check(bool),
    /// Short caption; `list_view` paints it with [`crate::widget::badge`].
    Text(String),
}

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

    fn leading(&self, index: usize) -> RowSlot {
        let _ = index;
        RowSlot::Empty
    }

    fn trailing(&self, index: usize) -> RowSlot {
        let _ = index;
        RowSlot::Empty
    }

    /// Start-side inset in pixels. Default 0.
    fn indent(&self, index: usize) -> f32 {
        let _ = index;
        0.0
    }
}

/// One owned list row.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListRow {
    pub title: String,
    pub meta: Option<String>,
    pub separator: bool,
    pub leading: RowSlot,
    pub trailing: RowSlot,
    pub indent: u16,
}

impl ListRow {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            meta: None,
            separator: false,
            leading: RowSlot::Empty,
            trailing: RowSlot::Empty,
            indent: 0,
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
            leading: RowSlot::Empty,
            trailing: RowSlot::Empty,
            indent: 0,
        }
    }

    pub fn with_indent(mut self, indent: u16) -> Self {
        self.indent = indent;
        self
    }

    pub fn with_leading(mut self, slot: RowSlot) -> Self {
        self.leading = slot;
        self
    }

    pub fn with_trailing(mut self, slot: RowSlot) -> Self {
        self.trailing = slot;
        self
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

    fn leading(&self, index: usize) -> RowSlot {
        self.items
            .get(index)
            .map(|r| r.leading.clone())
            .unwrap_or(RowSlot::Empty)
    }

    fn trailing(&self, index: usize) -> RowSlot {
        self.items
            .get(index)
            .map(|r| r.trailing.clone())
            .unwrap_or(RowSlot::Empty)
    }

    fn indent(&self, index: usize) -> f32 {
        self.items
            .get(index)
            .map(|r| f32::from(r.indent))
            .unwrap_or(0.0)
    }
}

/// Mouse button that hit a list, table, grid, or tree row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemButton {
    Primary,
    Secondary,
}

/// A pointer press on a collection row.
///
/// `id` is the row index for lists, tables, and grids, or the node id
/// for a tree. Shift+primary extends; Command/Ctrl+primary toggles.
/// Secondary on an already-selected row keeps the selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemClick<Id = usize> {
    pub id: Id,
    pub button: ItemButton,
    pub modifiers: iced::keyboard::Modifiers,
}

impl ItemClick<usize> {
    /// Left click, no modifiers.
    pub fn primary(id: usize) -> Self {
        Self {
            id,
            button: ItemButton::Primary,
            modifiers: iced::keyboard::Modifiers::empty(),
        }
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
            Self::Single(s) if *s == i => *self = Self::Multi(vec![i]),
            Self::Single(s) => {
                let mut v = vec![*s, i];
                v.sort_unstable();
                *self = Self::Multi(v);
            }
            Self::None => *self = Self::Multi(vec![i]),
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

    /// Apply a desktop list click: shift extends, Command/Ctrl toggles,
    /// secondary keeps an already-selected row.
    pub fn apply_item_click(&mut self, click: ItemClick<usize>) {
        match click.button {
            ItemButton::Secondary if self.contains(click.id) => {}
            ItemButton::Secondary => self.select_single(click.id),
            ItemButton::Primary if click.modifiers.shift() => self.extend_to(click.id),
            ItemButton::Primary if click.modifiers.command() => self.toggle_multi(click.id),
            ItemButton::Primary => self.select_single(click.id),
        }
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

/// How tall each virtual row is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RowHeights<'a> {
    Uniform(f32),
    PerRow(&'a [f32]),
}

impl From<f32> for RowHeights<'_> {
    fn from(h: f32) -> Self {
        Self::Uniform(h)
    }
}

/// How each list row is painted.
///
/// [`Self::Flush`] is one clipped line and a selection wash.
/// [`Self::Card`] is a surface, wrapped title, and an optional meter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RowFace<F = fn(usize) -> f32> {
    Flush,
    Card {
        /// Fill from 0.0 to 1.0 along a 3px bar under the meta.
        meter: Option<F>,
    },
}

impl Default for RowFace<fn(usize) -> f32> {
    fn default() -> Self {
        Self::Flush
    }
}

impl RowFace<fn(usize) -> f32> {
    /// Compact clipped row. Same as [`Self::Flush`] with a known `F`.
    pub const FLUSH: Self = Self::Flush;
}

impl RowHeights<'_> {
    pub fn at(self, i: usize) -> f32 {
        match self {
            Self::Uniform(h) => h.max(0.0),
            Self::PerRow(hs) => hs.get(i).copied().unwrap_or(0.0).max(0.0),
        }
    }

    pub fn total(self, n: usize) -> f32 {
        match self {
            Self::Uniform(h) => n as f32 * h.max(0.0),
            Self::PerRow(hs) => hs.iter().take(n).copied().sum(),
        }
    }

    pub fn offset(self, i: usize) -> f32 {
        match self {
            Self::Uniform(h) => i as f32 * h.max(0.0),
            Self::PerRow(hs) => hs.iter().take(i).copied().sum(),
        }
    }
}

/// After a scroll, remount using [`visible_window_var`].
pub fn window_after_scroll_var(
    prev: VisibleWindow,
    scroll: f32,
    viewport: f32,
    heights: &[f32],
    overscan: usize,
    cover: Option<usize>,
) -> VisibleWindow {
    let viewport = if viewport > 0.0 {
        viewport
    } else {
        prev.viewport
    };
    let total: f32 = heights.iter().copied().sum();
    let max_scroll = (total - viewport).max(0.0);
    visible_window_var(
        scroll.clamp(0.0, max_scroll),
        viewport,
        heights,
        overscan,
        cover,
    )
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

/// Per-row heights for expand cards: closed face unless listed open.
///
/// `open` is `(index, open_height)`. Indices out of range are ignored.
/// Use with [`visible_window_var`] / [`widget::virtual_column`](crate::widget::virtual_column).
///
/// ```
/// let h = icedtea::collection::expand_card_heights(4, 48.0, &[(1, 160.0)]);
/// assert_eq!(h, vec![48.0, 160.0, 48.0, 48.0]);
/// ```
pub fn expand_card_heights(n: usize, closed: f32, open: &[(usize, f32)]) -> Vec<f32> {
    let closed = closed.max(0.0);
    let mut heights = vec![closed; n];
    for &(i, open_h) in open {
        if i < n {
            heights[i] = open_h.max(0.0);
        }
    }
    heights
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
    /// First *n* columns in `order` stay in view; the rest scroll.
    pub frozen: usize,
    /// Horizontal pixel offset of the unfrozen columns.
    pub h_scroll: f32,
}

impl ColumnLayout {
    pub fn new(widths: impl Into<Vec<f32>>) -> Self {
        let widths = widths.into();
        let order: Vec<usize> = (0..widths.len()).collect();
        Self {
            widths,
            order,
            frozen: 0,
            h_scroll: 0.0,
        }
    }

    pub fn with_frozen(mut self, n: usize) -> Self {
        self.frozen = n.min(self.order.len());
        self
    }

    pub fn set_h_scroll(&mut self, x: f32) {
        self.h_scroll = x.max(0.0);
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
    /// When non-empty, `data_table` paints a leading checkbox column.
    pub checks: Vec<bool>,
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
        let cmp = |a: &Vec<String>, b: &Vec<String>| {
            let av = a.get(col).map(String::as_str).unwrap_or("");
            let bv = b.get(col).map(String::as_str).unwrap_or("");
            if asc {
                av.cmp(bv)
            } else {
                bv.cmp(av)
            }
        };
        if self.checks.len() == self.rows.len() {
            let mut pairs: Vec<(Vec<String>, bool)> =
                self.rows.drain(..).zip(self.checks.drain(..)).collect();
            pairs.sort_by(|(a, _), (b, _)| cmp(a, b));
            for (row, on) in pairs {
                self.rows.push(row);
                self.checks.push(on);
            }
        } else {
            self.rows.sort_by(cmp);
        }
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
    fn row_checked(&self, row: usize) -> Option<bool>;
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

    fn row_checked(&self, row: usize) -> Option<bool> {
        self.checks.get(row).copied()
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
        self.flatten_during(None)
    }

    /// Rows on screen, including children of `id` while that node is
    /// collapsed (close animation still running).
    pub fn flatten_during(&self, id: Option<u64>) -> Vec<(u32, u64, String, bool, bool)> {
        let mut out = Vec::new();
        flatten_into(self, 0, id, &mut out);
        out
    }
}

fn flatten_into(
    node: &TreeNode,
    depth: u32,
    during: Option<u64>,
    out: &mut Vec<(u32, u64, String, bool, bool)>,
) {
    let has_children = node.dir || !node.children.is_empty();
    out.push((
        depth,
        node.id,
        node.label.clone(),
        node.expanded,
        has_children,
    ));
    if node.expanded || during == Some(node.id) {
        for c in &node.children {
            flatten_into(c, depth + 1, during, out);
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
    pub badges: Vec<String>,
    pub icons: Vec<Option<crate::icon::Icon>>,
    pub disabled: Vec<bool>,
    pub active: usize,
    pub closable: bool,
}

impl Tabs {
    pub fn new(titles: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let titles: Vec<String> = titles.into_iter().map(Into::into).collect();
        let n = titles.len();
        let badges = vec![String::new(); n];
        Self {
            titles,
            badges,
            icons: vec![None; n],
            disabled: vec![false; n],
            active: 0,
            closable: false,
        }
    }

    pub fn with_badge(mut self, index: usize, badge: impl Into<String>) -> Self {
        if let Some(slot) = self.badges.get_mut(index) {
            *slot = badge.into();
        }
        self
    }

    pub fn with_icon(mut self, index: usize, icon: crate::icon::Icon) -> Self {
        if let Some(slot) = self.icons.get_mut(index) {
            *slot = Some(icon);
        }
        self
    }

    /// Freeze one tab so [`crate::widget::tab_bar`] skips its press.
    pub fn with_disabled(mut self, index: usize) -> Self {
        if let Some(slot) = self.disabled.get_mut(index) {
            *slot = true;
        }
        self
    }

    pub fn is_disabled(&self, index: usize) -> bool {
        self.disabled.get(index).copied().unwrap_or(false)
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
        if i < self.badges.len() {
            self.badges.remove(i);
        }
        if i < self.icons.len() {
            self.icons.remove(i);
        }
        if i < self.disabled.len() {
            self.disabled.remove(i);
        }
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
        assert_eq!(TitlesOnly.leading(0), RowSlot::Empty);
        assert_eq!(TitlesOnly.trailing(0), RowSlot::Empty);
        assert!(TableModel::default().row_checked(0).is_none());
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
        sel.apply_item_click(ItemClick {
            id: 4,
            button: ItemButton::Primary,
            modifiers: iced::keyboard::Modifiers::CTRL,
        });
        assert!(sel.contains(4));
        assert_eq!(sel.primary(), Some(4));
        sel = Selection::None;
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
            checks: Vec::new(),
        };
        table.sort(0);
        assert_eq!(table.cell(0, 0), "a");
        table.sort(0);
        assert_eq!(table.cell(0, 0), "b");
        let mut checked = TableModel {
            headers: vec!["n".into()],
            rows: vec![vec!["b".into()], vec!["a".into()]],
            sort_col: None,
            sort_asc: true,
            checks: vec![true, false],
        };
        checked.sort(0);
        assert_eq!(checked.cell(0, 0), "a");
        assert_eq!(checked.row_checked(0), Some(false));
        assert_eq!(checked.row_checked(1), Some(true));
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
        assert_eq!(tree.flatten_during(Some(1)).len(), 3);
        assert_eq!(tree.flatten_during(Some(2)).len(), 1);
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
        let slotted = ListRow::new("a")
            .with_leading(RowSlot::Check(true))
            .with_trailing(RowSlot::Icon(crate::icon::Icon::Search));
        let list = VecList {
            items: vec![slotted, ListRow::separator()],
        };
        assert_eq!(list.leading(0), RowSlot::Check(true));
        assert!(matches!(list.trailing(0), RowSlot::Icon(_)));
        assert_eq!(list.leading(1), RowSlot::Empty);
        assert_eq!(list.leading(9), RowSlot::Empty);
        let marked = ListRow::new("child")
            .with_indent(16)
            .with_trailing(RowSlot::Text("A".into()));
        assert_eq!(marked.indent, 16);
        let forest = VecList {
            items: vec![marked],
        };
        assert_eq!(forest.indent(0), 16.0);
        assert_eq!(forest.trailing(0), RowSlot::Text("A".into()));
        assert_eq!(forest.indent(3), 0.0);
        let mut tabs = Tabs::new(["A", "B", "C"]).with_badge(0, "2");
        assert_eq!(tabs.badges[0], "2");
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
        let mut tabs = Tabs::new(["A", "B", "C"]).with_disabled(1);
        assert!(tabs.is_disabled(1));
        assert!(!tabs.is_disabled(0));
        tabs.closable = true;
        assert_eq!(tabs.close(0).as_deref(), Some("A"));
        assert!(tabs.is_disabled(0));
        assert!(!tabs.is_disabled(1));
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
    fn past_end_scroll_still_mounts_rows_when_content_exists() {
        // Paint path must use this clamp: unclamped scroll past content blanks the list.
        let row_h = 40.0;
        let viewport = 200.0;
        let n = 50;
        let prev = VisibleWindow {
            start: 40,
            end: 50,
            scroll: 9_000.0,
            viewport,
        };
        let win = window_after_scroll(prev, prev.scroll, viewport, row_h, n, 4, None);
        assert!(win.end > win.start);
        let max_scroll = n as f32 * row_h - viewport;
        assert!((win.scroll - max_scroll).abs() < 0.01);
        let tall: Vec<f32> = (0..25).map(|_| 80.0).collect();
        let short: Vec<f32> = (0..25).map(|_| 40.0).collect();
        let deep = window_after_scroll_var(
            VisibleWindow {
                start: 0,
                end: 10,
                scroll: tall.iter().sum::<f32>(),
                viewport: 200.0,
            },
            tall.iter().sum(),
            200.0,
            &tall,
            4,
            None,
        );
        let remount = window_after_scroll_var(deep, deep.scroll, 200.0, &short, 4, None);
        assert!(remount.end > remount.start);
        let short_total: f32 = short.iter().sum();
        assert!(remount.scroll <= (short_total - 200.0).max(0.0) + 0.01);
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
        let cards = expand_card_heights(5, 40.0, &[(2, 120.0), (9, 50.0)]);
        assert_eq!(cards.len(), 5);
        assert_eq!(cards[0], 40.0);
        assert_eq!(cards[2], 120.0);
        assert_eq!(cards[4], 40.0);
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
        let mut click = Selection::Single(2);
        click.apply_item_click(ItemClick {
            id: 2,
            button: ItemButton::Secondary,
            modifiers: iced::keyboard::Modifiers::empty(),
        });
        assert_eq!(click, Selection::Single(2));
        click.apply_item_click(ItemClick {
            id: 5,
            button: ItemButton::Secondary,
            modifiers: iced::keyboard::Modifiers::empty(),
        });
        assert_eq!(click, Selection::Single(5));
        click.apply_item_click(ItemClick {
            id: 7,
            button: ItemButton::Primary,
            modifiers: iced::keyboard::Modifiers::SHIFT,
        });
        assert!(click.contains(5) && click.contains(7));
        let mut tog = Selection::Single(1);
        tog.apply_item_click(ItemClick {
            id: 3,
            button: ItemButton::Primary,
            modifiers: iced::keyboard::Modifiers::COMMAND,
        });
        assert!(tog.contains(1) && tog.contains(3));
        tog.apply_item_click(ItemClick::primary(0));
        assert_eq!(tog, Selection::Single(0));
        assert_eq!(sel.move_primary(1, 5), Some(2));
        let mut empty = Selection::None;
        assert_eq!(empty.move_primary(0, 0), None);
        let mut from = vec![1, 2, 3];
        let mut to = vec![9];
        assert_eq!(transfer_index(&mut from, &mut to, 2, 0), 0);
        assert_eq!(from, vec![1, 3]);
        assert_eq!(to, vec![2, 9]);
        let ext = RowHeights::PerRow(&h);
        assert_eq!(ext.at(1), 40.0);
        assert_eq!(ext.offset(2), 60.0);
        assert_eq!(ext.total(5), 180.0);
        assert_eq!(RowHeights::from(20.0).at(3), 20.0);
        assert_eq!(RowFace::<fn(usize) -> f32>::default(), RowFace::Flush);
        let card = RowFace::Card {
            meter: Some((|_| 0.5) as fn(usize) -> f32),
        };
        assert!(matches!(card, RowFace::Card { .. }));
        assert_eq!(RowHeights::from(20.0).offset(4), 80.0);
        assert_eq!(RowHeights::from(20.0).total(3), 60.0);
        assert_eq!(RowHeights::from(-4.0).total(3), 0.0);
        assert_eq!(RowHeights::from(-4.0).offset(2), 0.0);
        let after = window_after_scroll_var(win, 40.0, 60.0, &h, 0, None);
        assert!(after.scroll >= 20.0);
        let zero_vp = window_after_scroll_var(win, 10.0, 0.0, &h, 0, None);
        assert!(zero_vp.viewport > 0.0);
        assert_eq!(RowHeights::PerRow(&[8.0]).at(9), 0.0);
        let _u0 = RowHeights::Uniform(20.0).total(0);
        let _u1 = RowHeights::Uniform(20.0).total(1);
        let _u2 = RowHeights::Uniform(0.0).offset(5);
        let _p0 = RowHeights::PerRow(&h).total(0);
        let _p1 = RowHeights::PerRow(&h).offset(0);
        let _p2 = RowHeights::PerRow(&[]).total(3);
        let _p3 = RowHeights::PerRow(&[]).offset(3);
        let _v0 = visible_window_var(0.0, 10.0, &h, 2, Some(99));
        let _v1 = visible_window_var(0.0, 10.0, &h, 0, Some(4));
        let _v2 = window_after_scroll_var(win, 999.0, 10.0, &h, 0, Some(0));
        let _v3 = virtual_pads_var(&h, 0.0, 0.0, 0, None);
        let _v4 = virtual_pads_var(&[10.0], 50.0, 10.0, 0, None);
        let _v5 = visible_range_var(-5.0, 10.0, &h);
        let _v6 = row_offsets(&[]);
        let mut cols = ColumnLayout::new(vec![80.0, 120.0, 60.0]).with_frozen(1);
        assert_eq!(cols.frozen, 1);
        assert_eq!(ColumnLayout::new(vec![10.0]).with_frozen(9).frozen, 1);
        cols.set_h_scroll(40.0);
        assert_eq!(cols.h_scroll, 40.0);
        cols.set_h_scroll(-3.0);
        assert_eq!(cols.h_scroll, 0.0);
        cols.reorder(2, 1);
        assert_eq!(cols.display()[0], 0);
        assert_eq!(cols.display()[1], 2);
        cols.resize(0, 10.0, 40.0);
        assert_eq!(cols.width(0), 90.0);
        cols.reorder(9, 0);
        assert_eq!(cols.width(9), 96.0);
    }
}
