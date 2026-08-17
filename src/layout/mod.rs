//! Layout recipes, size policy, and breakpoints.
//!
//! `dock`, `split_view`, `clamp`, `form`, and `pad` return iced
//! [`Element`](crate::Element)s. Split sash: the grip emits
//! [`SashEvent::Press`](split::SashEvent::Press) only; move and
//! release come from [`listen_sash`]. The sash paints a hairline and a
//! short handle from `Tokens`. Scroll with
//! [`crate::widget::themed_scroll`], not a raw iced scroller.
//!
//! ```
//! use icedtea::layout::{distribute, SizePolicy};
//! let sizes = distribute(100.0, &[SizePolicy::fixed(20.0), SizePolicy::expand(1.0)]);
//! assert_eq!(sizes[0], 20.0);
//! assert_eq!(sizes[1], 80.0);
//! ```

pub mod breakpoint;
pub mod recipes;
pub mod size;
pub mod span;
pub mod split;

pub use breakpoint::Breakpoint;
pub use recipes::{
    clamp, clamp_pad, clamp_width, column_box, dock, end_offset, fixed, form, form_columns, grid,
    grid_spanned, overlay_card, overlay_center, pad, padding, row_box, split_sizes, split_view,
    stack_child, stack_visible, stick_to_end, window_size_from_dock, wrap, wrap_per_row, wrap_rows,
    DockSpec, FILL, FORM_LABEL, LIST_PANE, SHRINK,
};
pub use size::{distribute, SizePolicy};
pub use span::{cell_geometry, grid_extent, span_occupies, GridCell};
pub use split::{
    listen_cursor, listen_sash, sash_from_window_event, sash_pointer_pos, Axis, CursorEvent,
    PointerDrive, SashDrag, SashEvent, SplitState,
};
