//! Spreadsheet-style editor view.
//!
//! A richly-styled data grid inspired by Excel. Highlights:
//!
//! * Sticky header row with clickable sort arrows (`▲` / `▼`).
//! * Zebra striping, hover affordance, gold selection accent, red invalid cells.
//! * Frozen "#" column matching the row-number column of Excel.
//! * Dual-mode global filter (`Filter` / `Highlight`) with prev/next
//!   navigation through highlighted matches.
//! * Single-click-to-select, click-again-to-edit cell UX.
//! * VIM-style `NORMAL` / `EDIT` mode indicator in the status bar.
//! * One-click CSV export of the currently-filtered view.
//!
//! ## Module layout
//!
//! * [`state`] — [`SpreadsheetState`] and the methods that mutate it.
//! * [`message`] — [`SpreadsheetMessage`] enum, routed through
//!   `handle_spreadsheet_messages!` in `crate::update::editor::common`.
//! * [`caches`] — pure cache-build pipeline ([`compute_caches`]).
//! * [`view`] — pure render functions assembled by [`view_spreadsheet`].
//! * [`export`] — async CSV save dialog ([`export_csv_task`]).

pub mod caches;
pub mod constants;
pub mod export;
pub mod message;
pub mod state;
pub mod view;

pub use caches::{compute_caches, ComputedCaches};
pub use export::export_csv_task;
pub use message::SpreadsheetMessage;
pub use state::{
    ColumnDragState, ColumnFilterOption, GlobalFilterMode, SpreadsheetPaneContent, SpreadsheetState,
};
pub use view::view_spreadsheet;
