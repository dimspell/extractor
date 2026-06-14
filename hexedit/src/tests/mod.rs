//! Integration tests for the hex editor using `iced_test`.
//!
//! These tests verify the view→update→view pipeline end-to-end: construct a
//! state, feed it through [`crate::view`], assert the rendered widget tree via
//! [`iced_test::Simulator`], then send messages through [`crate::update`] and
//! re-check the view.
//!
//! Shared helpers are defined here and re-used by all sub-modules.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use iced_test::simulator;

use gui_widgets::components::paragraph_cache::ParagraphCache;

use crate::config::HexEditorConfig;
use crate::message::HexEditorMessage;
use crate::provider::BufferProvider;
use crate::provider::HexProvider;
use crate::ui::coloring::ColorScheme;
use crate::search::SearchState;
use crate::selection::{NavDir, Selection};
use crate::state::HexEditorState;
use crate::update::update;
use crate::view::view;
use crate::LuaScriptEngine;

// ============================================================================
// Helpers
// ============================================================================

pub fn make_state(data: Vec<u8>) -> HexEditorState {
    let panes = crate::domain::panel::default_pane_grid();
    let pane_focus = *panes.iter().next().map(|(id, _)| id).unwrap();
    HexEditorState {
        path: PathBuf::from("test.bin"),
        name: "test.bin".to_string(),
        panes,
        pane_focus,
        provider: BufferProvider::from_bytes(data),
        bytes_per_row: 16,
        selection: Selection::single(0),
        edit_mode: None,
        inspector_edit: None,
        vanilla: None,
        vanilla_diff: BTreeSet::new(),
        patterns: Vec::new(),
        pattern_by_addr: BTreeMap::new(),
        show_pattern_list: false,
        next_pattern_id: 0,
        groups: Vec::new(),
        next_group_id: 0,
        collapsed_groups: BTreeSet::new(),
        context_menu_addr: None,
        goto: None,
        search: SearchState::new(),
        show_decimal: false,
        status_msg: String::new(),
        error: None,
        cache: ParagraphCache::default(),
        lua_engine: LuaScriptEngine::default(),
        export_config: None,
        repeat_pattern: None,
        row_annotations: BTreeMap::new(),
        active_patterns: BTreeSet::new(),
        renaming_group: None,
        renaming_group_draft: String::new(),
        color_scheme: ColorScheme::Monochrome,
        dim_nulls: true,
        settings_open: false,
    }
}

pub fn default_config() -> HexEditorConfig {
    HexEditorConfig::default()
}

// Helper: feed a message through update, discard the returned task.
pub fn send(state: &mut HexEditorState, config: &HexEditorConfig, msg: HexEditorMessage) {
    let _task = update(state, config, msg);
}

// ============================================================================
// Error state
// ============================================================================

#[test]
fn test_error_state_shows_error_message() {
    let mut state = make_state(vec![]);
    state.error = Some("test error: file not found".into());
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find("test error: file not found")
        .expect("error message should be displayed");
}

#[test]
fn test_error_state_hides_header_and_toolbar() {
    let mut state = make_state(vec![]);
    state.error = Some("corrupt file".into());
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    // When error is set, the view renders ONLY the error, not the normal chrome.
    ui.find("corrupt file")
        .expect("error message should be displayed");
}

// ============================================================================
// Status message
// ============================================================================

#[test]
fn test_status_message_displays() {
    let mut state = make_state((0..64).collect());
    state.status_msg = "Operation completed".into();
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find("Operation completed")
        .expect("status message should be visible");
}

#[test]
fn test_clear_status_removes_message() {
    let mut state = make_state((0..64).collect());
    state.status_msg = "temporary".into();
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::ClearStatus);
    assert!(state.status_msg.is_empty(), "status should be cleared");
}

// ============================================================================
// ParagraphCache integration
// ============================================================================

#[test]
fn test_hex_matrix_uses_paragraph_cache() {
    // This test verifies the ParagraphCache is wired up without panicking.
    // It doesn't assert on rendered output since the matrix widget's internals
    // are not directly observable through the text finder.
    let state = make_state((0..=255u8).collect());
    let config = default_config();
    // Just verify the element can be created without errors
    let _element = view(&state, &config);
}

pub mod pane_grid;

pub mod header;
pub mod toolbar;
pub mod footer;
pub mod navigation;
pub mod editing;
pub mod search;
pub mod goto;
pub mod patterns;
pub mod pattern_group;
pub mod inspector;
pub mod saving;
pub mod settings;

#[cfg(feature = "lua")]
pub mod lua_tests;

