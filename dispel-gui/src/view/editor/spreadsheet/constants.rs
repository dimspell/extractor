//! Pixel-level layout constants shared between state, view, and the
//! `TableWidget`. Kept as a thin module so behavioural code does not have
//! to repeat magic numbers.

/// Height of every data row in the table (and the header row).
pub const ROW_HEIGHT: f32 = 24.0;

/// Width of the frozen `#` column on the left of the table.
pub const ID_COL_WIDTH_PX: f32 = 42.0;

/// Default width of a data column when the user has not resized it.
/// Fixed (rather than `FillPortion`) so the table can overflow horizontally
/// and become scrollable when there are many columns.
pub const COL_WIDTH: f32 = 140.0;

/// Lower bound for column width — also the floor enforced during resize.
pub const COL_WIDTH_MIN: f32 = 40.0;

/// Upper bound for column width.
pub const COL_WIDTH_MAX: f32 = 600.0;
