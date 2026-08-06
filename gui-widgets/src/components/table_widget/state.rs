//! Shared, app-owned scroll/viewport state for the table widget.
//!
//! Lives in the application model (e.g. `SpreadsheetState`, the save-file
//! viewer states) so programmatic navigation can write the scroll offset
//! directly and the widget reads it every frame. The widget is "controlled":
//! it never stores its own scroll offset — it renders from this struct and
//! publishes changes back through `on_scroll`.

use iced::Vector;

/// Scroll-offset and viewport-size bundle that the table widget rehydrates
/// from every frame.
///
/// ## Controlled widget pattern
///
/// The table widget does **not** own its own scroll offset. Instead:
///
/// 1. **Draw / layout** reads [`scroll_offset`](Self::scroll_offset) from
///    this struct (supplied via `TableWidget::table_state`).
/// 2. **User input** (wheel, drag, keyboard) computes a new offset and
///    publishes it to the application model via the `on_scroll` callback.
/// 3. **Programmatic navigation** (arrow keys, highlight jump, screen-reader
///    scroll-to-row) writes [`scroll_offset`](Self::scroll_offset) directly
///    through the normal application message flow.
///
/// This removes the previous `external_offset` / `sync_external` dance.
#[derive(Debug, Clone, Copy, Default)]
pub struct TableState {
    /// Current scroll offset in content-space pixels.
    pub scroll_offset: Vector,
    /// Height of the visible body region, as last reported by the widget
    /// through `on_scroll`.  Required for programmatic scroll-to-row math
    /// (`scroll_y_for_row`, `ensure_row_visible_y`).
    pub viewport_height: f32,
}
