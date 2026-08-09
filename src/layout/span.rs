//! Grid cells with column/row span.

/// One cell in a spanned grid.
///
/// ```
/// use icedtea::layout::{GridCell, span_occupies};
/// let cells = [GridCell { col: 0, row: 0, col_span: 2, row_span: 1 }];
/// assert!(span_occupies(&cells, 1, 0));
/// assert!(!span_occupies(&cells, 2, 0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCell {
    pub col: u32,
    pub row: u32,
    pub col_span: u32,
    pub row_span: u32,
}

impl GridCell {
    pub fn new(col: u32, row: u32) -> Self {
        Self {
            col,
            row,
            col_span: 1,
            row_span: 1,
        }
    }

    pub fn span(mut self, cols: u32, rows: u32) -> Self {
        self.col_span = cols.max(1);
        self.row_span = rows.max(1);
        self
    }

    pub fn contains(self, col: u32, row: u32) -> bool {
        col >= self.col
            && col < self.col + self.col_span.max(1)
            && row >= self.row
            && row < self.row + self.row_span.max(1)
    }
}

pub fn span_occupies(cells: &[GridCell], col: u32, row: u32) -> bool {
    cells.iter().any(|c| c.contains(col, row))
}

/// Pixel box `(x, y, width, height)` for a spanned cell.
///
/// ```
/// use icedtea::layout::GridCell;
/// let (x, y, w, h) = icedtea::layout::cell_geometry(
///     &GridCell::new(1, 0).span(2, 1),
///     40.0,
///     20.0,
///     8.0,
/// );
/// assert!((x - 48.0).abs() < 0.01);
/// assert!((w - 88.0).abs() < 0.01);
/// assert!((h - 20.0).abs() < 0.01);
/// ```
pub fn cell_geometry(cell: &GridCell, cell_w: f32, cell_h: f32, gap: f32) -> (f32, f32, f32, f32) {
    let cs = cell.col_span.max(1);
    let rs = cell.row_span.max(1);
    let x = cell.col as f32 * (cell_w + gap);
    let y = cell.row as f32 * (cell_h + gap);
    let w = cs as f32 * cell_w + (cs.saturating_sub(1) as f32) * gap;
    let h = rs as f32 * cell_h + (rs.saturating_sub(1) as f32) * gap;
    (x, y, w, h)
}

/// Total size of a spanned grid.
pub fn grid_extent(cells: &[GridCell], cell_w: f32, cell_h: f32, gap: f32) -> (f32, f32) {
    let mut max_c = 0u32;
    let mut max_r = 0u32;
    for c in cells {
        max_c = max_c.max(c.col + c.col_span.max(1));
        max_r = max_r.max(c.row + c.row_span.max(1));
    }
    let w = if max_c == 0 {
        0.0
    } else {
        max_c as f32 * cell_w + (max_c.saturating_sub(1) as f32) * gap
    };
    let h = if max_r == 0 {
        0.0
    } else {
        max_r as f32 * cell_h + (max_r.saturating_sub(1) as f32) * gap
    };
    (w, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_covers_block() {
        let c = GridCell::new(1, 1).span(2, 2);
        assert!(c.contains(1, 1) && c.contains(2, 2));
        assert!(!c.contains(0, 1));
        assert!(!c.contains(3, 1));
        assert!(span_occupies(&[c], 2, 1));
        assert!(!span_occupies(&[c], 0, 0));
        let unit = GridCell::new(0, 0).span(0, 0);
        assert!(unit.contains(0, 0));
        let (x, y, w, h) = cell_geometry(&GridCell::new(1, 0).span(2, 1), 40.0, 20.0, 8.0);
        assert!((x - 48.0).abs() < 0.01);
        assert_eq!(y, 0.0);
        assert!((w - 88.0).abs() < 0.01);
        assert!((h - 20.0).abs() < 0.01);
        let (gw, gh) = grid_extent(&[GridCell::new(0, 0).span(2, 2)], 10.0, 10.0, 0.0);
        assert!((gw - 20.0).abs() < 0.01);
        assert!((gh - 20.0).abs() < 0.01);
        assert_eq!(grid_extent(&[], 10.0, 10.0, 2.0), (0.0, 0.0));
    }
}
