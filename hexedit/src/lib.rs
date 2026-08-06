//! A standalone, embeddable hex editor widget for Iced.
//!
//! # Usage
//!
//! ```rust,ignore
//! use hexedit::{HexEditorState, HexEditorConfig, HexEditorMessage};
//! use hexedit::{update, view};
//!
//! // In your app's update:
//! update(&mut state, &config, msg);
//!
//! // In your app's view:
//! view(&state, &config);
//! ```

// ── Module hierarchy ─────────────────────────────────────────────────────
//
//  domain/   — Pure data model (no internal Iced-widget dependencies)
//  ui/       — Iced-specific (widget, view, update, coloring)
//  Root      — config, message, state aggregate, lua_engine, standalone bin

pub mod domain;
pub mod ui;

mod app;
mod config;
pub mod lua_engine;
mod message;
mod state;

// ── Re-exports for downstream convenience ────────────────────────────────
//
// Keep the same public API surface that existed before the domain/ui split.

pub use app::{AppMessage, HexEditorApp, HexEditorDocument, app_update, app_view};
pub use config::{HexEditorConfig, OnSaveFn};
pub use message::HexEditorMessage;
pub use state::{ComparisonFile, DEFAULT_BYTES_PER_ROW, HexEditorState, InspectorSource};
pub use ui::update::update;
pub use ui::view::view;

// Type-level re-exports from domain.
pub use domain::byte_stats::{
    ByteStatistics, RowEntropyCache, StructureHeuristic, entropy_to_color,
};
pub use domain::editing::{EditState, InspectorEditState};
pub use domain::extend_dialog::ExtendDialog;
pub use domain::fill_dialog::FillDialog;
pub use domain::layout::{BinaryLayout, FieldSpan, LayoutRegistry};
pub use domain::panel::{HexPanel, HexPanelContent};
pub use domain::pattern::{Pattern, RepeatPatternDialog, RepeatedPatternGroup};
pub use domain::provider::{BufferProvider, HexProvider};
pub use domain::search::{SearchMode, SearchState};
pub use domain::selection::{NavDir, Selection};
pub use domain::vanilla_diff::compute_diff;
pub use domain::write_mode::{EncodingEntry, WriteMode};

// Type-level re-exports from ui.
pub use lua_engine::LuaScriptEngine;
pub use ui::coloring::CellColorProvider;
pub use ui::inspector::{EncodeFn, InspectorEntry};

// Module-level re-exports — allow `hexedit::selection::NavDir` and
// `crate::selection::*` to keep working inside the crate.
pub use domain::{
    byte_stats, editing, goto, layout, pattern, provider, search, selection, vanilla_diff,
    write_mode,
};
pub use ui::{coloring, inspector, update, view};

#[cfg(test)]
mod tests;
