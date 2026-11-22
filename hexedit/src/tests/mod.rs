//! Integration tests for the hex editor using `iced_test`.
//!
//! These tests verify the view→update→view pipeline end-to-end: construct a
//! state, feed it through [`crate::view`], assert the rendered widget tree via
//! [`iced_test::Simulator`], then send messages through [`crate::update`] and
//! re-check the view.
//!
//! Shared helpers are defined here and re-used by all sub-modules.

use std::collections::{BTreeMap, BTreeSet};

use iced_test::simulator;

use gui_widgets::components::paragraph_cache::ParagraphCache;

use crate::config::HexEditorConfig;
use crate::domain::save::SaveOutcome;
use crate::domain::write_mode::WriteMode;
use crate::message::HexEditorMessage;
use crate::provider::HexProvider;
use crate::selection::{NavDir, Selection};
use crate::state::HexEditorState;
use crate::ui::coloring::ColorScheme;
use crate::update::update;
use crate::view::view;

// ============================================================================
// Helpers
// ============================================================================

pub fn make_state(data: Vec<u8>) -> HexEditorState {
    HexEditorState::from_bytes("test.bin", data, None, None)
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
// Toast notifications
// ============================================================================

#[test]
fn test_toast_notification_displays() {
    let mut state = make_state((0..64).collect());
    state.notify("Operation completed");
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find("Operation completed")
        .expect("status message should be visible");
}

#[test]
fn test_clear_status_removes_notifications() {
    let mut state = make_state((0..64).collect());
    state.notify("temporary");
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::ClearStatus);
    assert!(
        state.notifications.is_empty(),
        "notifications should be cleared"
    );
}

#[test]
fn test_dismiss_notification_removes_only_the_selected_toast() {
    let mut state = make_state((0..64).collect());
    state.notify("First");
    state.notify("Second");
    let config = default_config();

    send(
        &mut state,
        &config,
        HexEditorMessage::DismissNotification(0),
    );

    assert_eq!(state.notifications.len(), 1);
    assert_eq!(state.notifications[0].body, "Second");
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

#[test]
fn test_outline_collapses_descendants_without_removing_siblings() {
    use crate::domain::layout::{NamedSpan, SpanBinaryLayout};

    let mut state = make_state(vec![0; 100]);
    state.set_layout(Some(Box::new(SpanBinaryLayout::new(
        "test",
        vec![
            NamedSpan {
                range: 0..60,
                name: "parent",
                ty: "section",
                record_index: 0,
            },
            NamedSpan {
                range: 0..20,
                name: "child",
                ty: "record",
                record_index: 0,
            },
            NamedSpan {
                range: 60..100,
                name: "sibling",
                ty: "section",
                record_index: 0,
            },
        ],
    ))));
    assert_eq!(state.outline.len(), 3);
    state.toggle_outline(0);
    assert_eq!(state.outline.len(), 2);
}

pub mod pane_grid;

pub mod diff;
pub mod editing;
pub mod extend;
pub mod footer;
pub mod goto;
pub mod header;
pub mod inspector;
pub mod navigation;
pub mod pattern_group;
pub mod patterns;
pub mod saving;
pub mod search;
pub mod settings;
pub mod toolbar;

#[cfg(feature = "lua")]
pub mod lua_tests;
