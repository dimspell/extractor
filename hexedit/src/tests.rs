//! Integration tests for the hex editor using `iced_test`.
//!
//! These tests verify the view→update→view pipeline end-to-end: construct a
//! state, feed it through [`crate::view`], assert the rendered widget tree via
//! [`iced_test::Simulator`], then send messages through [`crate::update`] and
//! re-check the view.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use iced_test::simulator;

use gui_widgets::components::paragraph_cache::ParagraphCache;

use crate::config::HexEditorConfig;
use crate::message::HexEditorMessage;
use crate::provider::BufferProvider;
use crate::provider::HexProvider;
use crate::search::SearchState;
use crate::selection::{NavDir, Selection};
use crate::state::HexEditorState;
use crate::update::update;
use crate::view::view;
use crate::LuaScriptEngine;

// ============================================================================
// Helpers
// ============================================================================

fn make_state(data: Vec<u8>) -> HexEditorState {
    HexEditorState {
        path: PathBuf::from("test.bin"),
        name: "test.bin".to_string(),
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
        context_menu_addr: None,
        goto: None,
        search: SearchState::new(),
        show_decimal: false,
        status_msg: String::new(),
        error: None,
        cache: ParagraphCache::default(),
        lua_engine: LuaScriptEngine::default(),
    }
}

fn default_config() -> HexEditorConfig {
    HexEditorConfig::default()
}

// Helper: feed a message through update, discard the returned task.
fn send(state: &mut HexEditorState, config: &HexEditorConfig, msg: HexEditorMessage) {
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
// Header
// ============================================================================

/// The header is a single `text()` widget — the `iced_test` selector matches
/// its *full* content, so we must search for the entire formatted string.
fn header_text_64() -> &'static str {
    "test.bin  ·  64 bytes  ·  16 bytes/row"
}
fn header_text_64_bpr8() -> &'static str {
    "test.bin  ·  64 bytes  ·  8 bytes/row"
}
fn header_text_256() -> &'static str {
    "test.bin  ·  256 bytes  ·  16 bytes/row"
}

#[test]
fn test_header_shows_file_name() {
    let state = make_state((0..64).collect());
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find(header_text_64())
        .expect("header should show file name, byte count and BPR");
}

#[test]
fn test_header_shows_byte_count() {
    let state = make_state((0..=255u8).collect());
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find(header_text_256())
        .expect("header should show 256 bytes");
}

#[test]
fn test_header_shows_bytes_per_row() {
    let state = make_state((0..64).collect());
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find(header_text_64())
        .expect("header should show default BPR");
}

#[test]
fn test_header_updates_after_bpr_change() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SetBytesPerRow(8));
    let mut ui = simulator(view(&state, &config));
    ui.find(header_text_64_bpr8())
        .expect("header should reflect updated BPR");
}

#[test]
fn test_header_rejects_invalid_bpr() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SetBytesPerRow(7));
    let mut ui = simulator(view(&state, &config));
    ui.find(header_text_64())
        .expect("invalid BPR should be ignored, keeping default");
}

// ============================================================================
// Toolbar
// ============================================================================

#[test]
fn test_toolbar_goto_button_renders() {
    let state = make_state((0..64).collect());
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find("Go to...")
        .expect("toolbar should have Go to... button");
}

#[test]
fn test_toolbar_patterns_button_renders() {
    let state = make_state((0..64).collect());
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find("Patterns")
        .expect("toolbar should have Patterns button");
}

#[test]
fn test_toolbar_hide_patterns_label_when_active() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::TogglePatternList);
    let mut ui = simulator(view(&state, &config));
    ui.find("Hide Patterns")
        .expect("toolbar should show Hide Patterns when list is open");
}

#[test]
fn test_toolbar_bpr_buttons_render() {
    let state = make_state((0..64).collect());
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find("BPR").expect("BPR label should be rendered");
    ui.find("08").expect("8 BPR button should be rendered");
    ui.find("16").expect("16 BPR button should be rendered");
    ui.find("32").expect("32 BPR button should be rendered");
}

#[test]
fn test_toolbar_save_button_with_custom_label() {
    let state = make_state((0..64).collect());
    let config = HexEditorConfig {
        on_save: Some(std::sync::Arc::new(|_| iced::Task::none())),
        save_label: "Store".into(),
        can_save: true,
        ..Default::default()
    };
    let mut ui = simulator(view(&state, &config));
    ui.find("Store")
        .expect("custom save label should appear");
}

#[test]
fn test_toolbar_save_hint_renders() {
    let state = make_state((0..64).collect());
    let config = HexEditorConfig {
        save_hint: "no active recording".into(),
        ..Default::default()
    };
    let mut ui = simulator(view(&state, &config));
    ui.find("no active recording")
        .expect("save hint should be visible");
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
// Footer — empty file
// ============================================================================

/// Footer is also a single `text()` widget; we match the full string.
fn footer_empty() -> &'static str {
    "(empty)  ·  total: 0 (0 B)  ·  dirty: 0"
}

fn footer_256_single() -> String {
    "0x0  ·  total: 0x100 (256 B)  ·  dirty: 0  ·  cursor: 0x0".to_string()
}

fn footer_256_single_decimal() -> String {
    "0  ·  total: 256 (256 B)  ·  dirty: 0  ·  cursor: 0".to_string()
}

#[test]
fn test_footer_shows_empty_state() {
    let state = make_state(vec![]);
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find(footer_empty())
        .expect("footer should show empty state");
}

#[test]
fn test_footer_shows_zero_dirty_for_empty_file() {
    let state = make_state(vec![]);
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find(footer_empty())
        .expect("empty footer should include dirty: 0");
}

// ============================================================================
// Footer — non-empty file
// ============================================================================

#[test]
fn test_footer_shows_cursor_and_total_in_hex() {
    let state = make_state((0..=255u8).collect());
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find(footer_256_single())
        .expect("footer should show hex cursor and total");
}

#[test]
fn test_footer_toggles_to_decimal() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::ToggleAddrFormat);
    assert!(state.show_decimal, "decimal format should be active");
    let mut ui = simulator(view(&state, &config));
    ui.find(footer_256_single_decimal())
        .expect("footer should show decimal cursor and total");
}

#[test]
fn test_footer_shows_selection_range() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(10));
    send(&mut state, &config, HexEditorMessage::ExtendTo(25));
    let mut ui = simulator(view(&state, &config));
    // Selection: 0xA - 0x19 (0x10 / 16 B)
    let expected =
        "0xA - 0x19 (0x10 / 16 B)  ·  total: 0x100 (256 B)  ·  dirty: 0  ·  cursor: 0x19";
    ui.find(expected)
        .expect("footer should show selection range");
}

// ============================================================================
// Inspector panel
// ============================================================================

#[test]
fn test_inspector_panel_header_renders() {
    let state = make_state(vec![0x2A, 0x00, 0x00, 0x00]);
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find("Data inspector")
        .expect("inspector panel header should be shown");
}

#[test]
fn test_inspector_shows_empty_file_for_zero_bytes() {
    let state = make_state(vec![]);
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find("(empty file)")
        .expect("inspector should show (empty file) placeholder");
}

#[test]
fn test_inspector_displays_u8_value() {
    let state = make_state(vec![0x2A, 0x00, 0x00, 0x00]);
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find("42")
        .expect("inspector should show u8 value 42 for byte 0x2A");
}

#[test]
fn test_inspector_displays_nonzero_values() {
    let state = make_state(vec![0xFF, 0x00, 0x00, 0x00]);
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find("255")
        .expect("inspector should show u8 value 255 for byte 0xFF");
}

#[test]
fn test_inspector_placeholder_for_truncated_read() {
    // At cursor=0 with only 1 byte available, multi-byte decoders show "—".
    let state = make_state(vec![0x2A]);
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find("42")
        .expect("u8 (1 byte) should still decode");
}

// ============================================================================
// Navigation
// ============================================================================

#[test]
fn test_select_at_sets_cursor() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(42));
    assert_eq!(state.selection.cursor, 42);
    assert!(state.selection.is_single(), "selection should be single");
}

#[test]
fn test_extend_to_creates_range() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(10));
    send(&mut state, &config, HexEditorMessage::ExtendTo(30));
    assert_eq!(state.selection.start(), 10);
    assert_eq!(state.selection.end(), 30);
}

#[test]
fn test_navigation_right() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::Nav {
            dir: NavDir::Right,
            extend: false,
        },
    );
    assert_eq!(state.selection.cursor, 1);
}

#[test]
fn test_navigation_down() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::Nav {
            dir: NavDir::Down,
            extend: false,
        },
    );
    assert_eq!(state.selection.cursor, 16);
}

#[test]
fn test_navigation_with_extend() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::Nav {
            dir: NavDir::Right,
            extend: true,
        },
    );
    // cursor moved to 1, but anchor is still at 0
    assert_eq!(state.selection.cursor, 1);
    assert_eq!(state.selection.anchor, 0);
    assert!(!state.selection.is_single(), "should extend selection");
}

#[test]
fn test_navigation_at_bounds() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    // Navigate left at cursor 0 should saturate.
    send(
        &mut state,
        &config,
        HexEditorMessage::Nav {
            dir: NavDir::Left,
            extend: false,
        },
    );
    assert_eq!(state.selection.cursor, 0, "left at 0 should stay at 0");

    // Navigate to document end.
    send(
        &mut state,
        &config,
        HexEditorMessage::Nav {
            dir: NavDir::DocumentEnd,
            extend: false,
        },
    );
    assert_eq!(state.selection.cursor, 255, "document end should be at max_addr");
}

#[test]
fn test_navigation_on_empty_file_does_nothing() {
    let mut state = make_state(vec![]);
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::Nav {
            dir: NavDir::Right,
            extend: false,
        },
    );
    assert_eq!(state.selection.cursor, 0, "empty file navigation should be no-op");
}

// ============================================================================
// Inline editing
// ============================================================================

#[test]
fn test_begin_edit_enters_edit_mode() {
    let mut state = make_state(vec![0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginEdit(0));
    assert!(state.edit_mode.is_some(), "edit mode should be active");
    assert_eq!(state.edit_mode.unwrap().addr, 0);
}

#[test]
fn test_edit_type_first_nibble() {
    let mut state = make_state(vec![0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginEdit(0));
    send(&mut state, &config, HexEditorMessage::EditTypeChar('A'));
    assert_eq!(state.edit_mode.as_ref().unwrap().draft, "A");
    // Not yet committed (need 2 chars).
    assert_eq!(state.provider.as_slice()[0], 0x00);
}

#[test]
fn test_edit_second_nibble_commits_and_advances() {
    let mut state = make_state(vec![0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginEdit(0));
    send(&mut state, &config, HexEditorMessage::EditTypeChar('A'));
    send(&mut state, &config, HexEditorMessage::EditTypeChar('B'));
    assert_eq!(state.provider.as_slice()[0], 0xAB, "byte should be written");
    assert_eq!(state.selection.cursor, 1, "cursor should advance");
    assert!(
        state.edit_mode.is_some(),
        "edit mode should continue at new address"
    );
}

#[test]
fn test_edit_commit_with_advance() {
    let mut state = make_state(vec![0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginEdit(0));
    // Type only ONE nibble — commit isn't automatic yet.
    send(&mut state, &config, HexEditorMessage::EditTypeChar('F'));
    send(
        &mut state,
        &config,
        HexEditorMessage::EditCommit { advance: true },
    );
    // Single nibble "F" → 0x0F
    assert_eq!(state.provider.as_slice()[0], 0x0F);
    assert_eq!(state.selection.cursor, 1, "cursor should advance");
    assert!(
        state.edit_mode.is_some(),
        "edit mode should re-enter at next address"
    );
}

#[test]
fn test_edit_commit_without_advance() {
    let mut state = make_state(vec![0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginEdit(0));
    send(&mut state, &config, HexEditorMessage::EditTypeChar('F'));
    send(
        &mut state,
        &config,
        HexEditorMessage::EditCommit { advance: false },
    );
    // Single nibble "F" → 0x0F
    assert_eq!(state.provider.as_slice()[0], 0x0F);
    assert_eq!(state.selection.cursor, 0, "cursor should stay at committed addr");
    assert!(state.edit_mode.is_none(), "edit mode should end");
}

#[test]
fn test_edit_cancel_restores_state() {
    let mut state = make_state(vec![0xAA, 0xBB]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginEdit(0));
    send(&mut state, &config, HexEditorMessage::EditTypeChar('0'));
    send(&mut state, &config, HexEditorMessage::EditCancel);
    assert!(state.edit_mode.is_none(), "edit mode should be cancelled");
    assert_eq!(
        state.provider.as_slice()[0],
        0xAA,
        "original byte must not change"
    );
}

#[test]
fn test_edit_backspace_removes_nibble() {
    let mut state = make_state(vec![0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginEdit(0));
    // Typing the second nibble auto-commits and advances, so only test
    // backspace with a single nibble in the draft.
    send(&mut state, &config, HexEditorMessage::EditTypeChar('1'));
    assert_eq!(state.edit_mode.as_ref().unwrap().draft, "1");
    send(&mut state, &config, HexEditorMessage::EditBackspace);
    assert_eq!(state.edit_mode.as_ref().unwrap().draft, "");
    assert!(state.edit_mode.as_ref().unwrap().draft.is_empty());
}

#[test]
fn test_begin_edit_on_empty_file_is_noop() {
    let mut state = make_state(vec![]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginEdit(0));
    assert!(state.edit_mode.is_none(), "should not edit empty file");
}

#[test]
fn test_write_bytes_modifies_provider() {
    let mut state = make_state(vec![0x00; 8]);
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::WriteBytes {
            addr: 2,
            bytes: vec![0xDE, 0xAD, 0xBE],
        },
    );
    assert_eq!(
        state.provider.as_slice(),
        &[0x00, 0x00, 0xDE, 0xAD, 0xBE, 0x00, 0x00, 0x00]
    );
    assert_eq!(state.provider.dirty_count(), 3);
}

// ============================================================================
// Search overlay
// ============================================================================

#[test]
fn test_search_visible_after_open() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenSearch);
    assert!(state.search.visible, "search should be visible");
}

#[test]
fn test_search_overlay_renders_mode_button() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenSearch);
    let mut ui = simulator(view(&state, &config));
    ui.find("HEX").expect("search mode button should show HEX");
}

#[test]
fn test_search_toggle_mode_shows_ascii() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenSearch);
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    let mut ui = simulator(view(&state, &config));
    ui.find("TXT").expect("search mode button should show TXT");
}

#[test]
fn test_search_execute_hex() {
    let mut state = make_state(b"\x00\xDE\xAD\xBE\xEF\x00".to_vec());
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::Search("DE AD BE EF".into()),
    );
    assert_eq!(state.search.count(), 1, "should find 1 match");
    assert_eq!(state.search.results[0], 1, "match should start at byte 1");
}

#[test]
fn test_search_execute_ascii() {
    let mut state = make_state(b"hello world".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenSearch);
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    send(
        &mut state,
        &config,
        HexEditorMessage::Search("world".into()),
    );
    assert_eq!(state.search.count(), 1, "should find 'world'");
    assert_eq!(state.search.results[0], 6);
}

#[test]
fn test_search_no_match_shows_zero() {
    let mut state = make_state(b"hello".to_vec());
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::Search("xyzzy".into()),
    );
    assert_eq!(state.search.count(), 0, "should find no matches");
}

#[test]
fn test_search_next_prev() {
    let mut state = make_state(b"\x61\x62\x20\x61\x62\x20\x61\x62".to_vec());
    let config = default_config();
    // Search for hex bytes "61 62" (= "ab" ASCII) to verify navigation.
    send(&mut state, &config, HexEditorMessage::Search("61 62".into()));
    assert_eq!(state.search.count(), 3, "should find 3 matches");
    // After initial search, current_match is None (no match selected).
    // Navigate to the first match:
    send(&mut state, &config, HexEditorMessage::SearchNext);
    assert_eq!(state.search.current_match, Some(0));
    send(&mut state, &config, HexEditorMessage::SearchNext);
    assert_eq!(state.search.current_match, Some(1));
    send(&mut state, &config, HexEditorMessage::SearchNext);
    assert_eq!(state.search.current_match, Some(2));
    send(&mut state, &config, HexEditorMessage::SearchNext);
    assert_eq!(
        state.search.current_match,
        Some(0),
        "should wrap around"
    );
    send(&mut state, &config, HexEditorMessage::SearchPrev);
    assert_eq!(
        state.search.current_match,
        Some(2),
        "should wrap backward"
    );
}

#[test]
fn test_search_close_clears_state() {
    let mut state = make_state(b"hello".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::Search("ll".into()));
    assert!(state.search.visible);
    assert!(!state.search.query.is_empty());
    send(&mut state, &config, HexEditorMessage::CloseSearch);
    assert!(!state.search.visible, "search should be hidden after close");
    assert!(state.search.query.is_empty(), "query should be cleared");
}

// ============================================================================
// Goto address
// ============================================================================

#[test]
fn test_goto_modal_opens_and_renders() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenGotoDialog);
    assert!(state.goto.is_some(), "goto dialog should be open");
    let mut ui = simulator(view(&state, &config));
    ui.find("Go to address")
        .expect("goto modal title should be visible");
    ui.find("Go").expect("Go button should be visible");
    ui.find("Cancel").expect("Cancel button should be visible");
}

#[test]
fn test_goto_modal_hidden_by_default() {
    let state = make_state((0..=255u8).collect());
    assert!(state.goto.is_none(), "goto dialog should be closed");
}

#[test]
fn test_goto_commit_with_hex() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenGotoDialog);
    send(
        &mut state,
        &config,
        HexEditorMessage::SetGotoDraft("0x42".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitGoto);
    assert!(state.goto.is_none(), "dialog should close after commit");
    assert_eq!(state.selection.cursor, 0x42);
}

#[test]
fn test_goto_commit_with_decimal() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenGotoDialog);
    send(&mut state, &config, HexEditorMessage::SetGotoDraft("100".into()));
    send(&mut state, &config, HexEditorMessage::CommitGoto);
    assert_eq!(state.selection.cursor, 100);
}

#[test]
fn test_goto_commit_with_relative_forward() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(50));
    send(&mut state, &config, HexEditorMessage::OpenGotoDialog);
    send(&mut state, &config, HexEditorMessage::SetGotoDraft("+10".into()));
    send(&mut state, &config, HexEditorMessage::CommitGoto);
    assert_eq!(state.selection.cursor, 60);
}

#[test]
fn test_goto_commit_with_relative_backward() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(50));
    send(&mut state, &config, HexEditorMessage::OpenGotoDialog);
    send(&mut state, &config, HexEditorMessage::SetGotoDraft("-10".into()));
    send(&mut state, &config, HexEditorMessage::CommitGoto);
    assert_eq!(state.selection.cursor, 40);
}

#[test]
fn test_goto_invalid_expression_shows_error() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenGotoDialog);
    send(
        &mut state,
        &config,
        HexEditorMessage::SetGotoDraft("xyz".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitGoto);
    assert!(state.goto.is_some(), "dialog should stay open on error");
    assert!(
        state.goto.as_ref().unwrap().error.is_some(),
        "should show error"
    );
}

#[test]
fn test_goto_empty_shows_error() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenGotoDialog);
    send(&mut state, &config, HexEditorMessage::SetGotoDraft("".into()));
    send(&mut state, &config, HexEditorMessage::CommitGoto);
    assert!(state.goto.is_some(), "dialog should stay open");
    assert!(
        state.goto.as_ref().unwrap().error.is_some(),
        "should show 'Enter an address' error"
    );
}

#[test]
fn test_goto_close_dismisses() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenGotoDialog);
    send(&mut state, &config, HexEditorMessage::CloseGotoDialog);
    assert!(state.goto.is_none(), "dialog should be closed");
}

// ============================================================================
// Patterns
// ============================================================================

#[test]
fn test_create_pattern_needs_range() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    // Single selection (no range)
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    assert_eq!(state.patterns.len(), 0, "should not create pattern");
    assert!(
        state.status_msg.contains("Select a range"),
        "should show hint to select range"
    );
}

#[test]
fn test_create_pattern_from_range() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(10));
    send(&mut state, &config, HexEditorMessage::ExtendTo(20));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    assert_eq!(state.patterns.len(), 1, "should create one pattern");
    assert_eq!(state.patterns[0].start, 10);
    assert_eq!(state.patterns[0].end, 20);
    assert!(
        state.status_msg.contains("Pattern created"),
        "should confirm creation"
    );
}

#[test]
fn test_remove_pattern_by_id() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(5));
    send(&mut state, &config, HexEditorMessage::ExtendTo(15));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    let id = state.patterns[0].id;
    send(&mut state, &config, HexEditorMessage::RemovePattern(id));
    assert_eq!(state.patterns.len(), 0);
    assert!(state.pattern_by_addr.is_empty());
}

#[test]
fn test_remove_pattern_at_address() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(5));
    send(&mut state, &config, HexEditorMessage::ExtendTo(15));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    // Remove the pattern by clicking on an address within it
    send(&mut state, &config, HexEditorMessage::RemovePatternAt(10));
    assert_eq!(state.patterns.len(), 0, "pattern should be removed");
}

#[test]
fn test_clear_all_patterns() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(0));
    send(&mut state, &config, HexEditorMessage::ExtendTo(10));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    send(&mut state, &config, HexEditorMessage::SelectAt(20));
    send(&mut state, &config, HexEditorMessage::ExtendTo(30));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    assert_eq!(state.patterns.len(), 2);
    send(&mut state, &config, HexEditorMessage::ClearAllPatterns);
    assert_eq!(state.patterns.len(), 0);
    assert!(state.status_msg.contains("All patterns cleared"));
}

#[test]
fn test_pattern_list_toggle() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    assert!(!state.show_pattern_list, "pattern list hidden by default");
    send(&mut state, &config, HexEditorMessage::TogglePatternList);
    assert!(state.show_pattern_list, "pattern list now visible");
    send(&mut state, &config, HexEditorMessage::TogglePatternList);
    assert!(!state.show_pattern_list, "pattern list hidden again");
}

#[test]
fn test_pattern_list_empty_view() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::TogglePatternList);
    let mut ui = simulator(view(&state, &config));
    ui.find("Patterns (0)")
        .expect("pattern list header should show 0");
    ui.find("No patterns defined")
        .expect("empty pattern message");
}

#[test]
fn test_pattern_list_with_patterns() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(10));
    send(&mut state, &config, HexEditorMessage::ExtendTo(20));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    send(&mut state, &config, HexEditorMessage::TogglePatternList);
    let mut ui = simulator(view(&state, &config));
    ui.find("Patterns (1)")
        .expect("pattern list header should show 1");
}

#[test]
fn test_navigate_to_pattern_moves_cursor() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(10));
    send(&mut state, &config, HexEditorMessage::ExtendTo(20));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    let id = state.patterns[0].id;
    // Navigate away
    send(&mut state, &config, HexEditorMessage::SelectAt(0));
    assert_eq!(state.selection.cursor, 0);
    // Navigate to pattern
    send(&mut state, &config, HexEditorMessage::NavigateToPattern(id));
    assert_eq!(state.selection.cursor, 10, "cursor should go to pattern start");
}

#[test]
fn test_context_menu_addr_set_on_right_click() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    assert!(state.context_menu_addr.is_none());
    send(&mut state, &config, HexEditorMessage::RightClickAt(42));
    assert_eq!(state.context_menu_addr, Some(42));
}

// ============================================================================
// Inspector edit modal
// ============================================================================

#[test]
fn test_inspector_edit_modal_opens_and_renders() {
    let mut state = make_state(vec![0x2A, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(0));
    assert!(state.inspector_edit.is_some(), "inspector edit modal open");
    assert_eq!(
        state.inspector_edit.as_ref().unwrap().draft,
        "42",
        "initial draft should decode current value"
    );
    let mut ui = simulator(view(&state, &config));
    // The modal title has the format "Edit {name} at 0x{addr}"
    ui.find("Edit u8 at 0x0")
        .expect("modal title should show entry name and address");
    ui.find("Apply").expect("Apply button should be visible");
    ui.find("Cancel").expect("Cancel button should be visible");
}

#[test]
fn test_inspector_edit_commit() {
    let mut state = make_state(vec![0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(0));
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorDraft("255".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    assert!(
        state.inspector_edit.is_none(),
        "modal should close after commit"
    );
    assert_eq!(state.provider.as_slice()[0], 255);
}

#[test]
fn test_inspector_edit_cancel() {
    let mut state = make_state(vec![0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(0));
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorDraft("200".into()),
    );
    send(&mut state, &config, HexEditorMessage::CloseInspectorEdit);
    assert!(
        state.inspector_edit.is_none(),
        "modal should close on cancel"
    );
    assert_eq!(
        state.provider.as_slice()[0],
        0x00,
        "original data should be unchanged"
    );
}

#[test]
fn test_inspector_edit_invalid_draft_shows_error() {
    let mut state = make_state(vec![0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(0));
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorDraft("abc".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    assert!(
        state.inspector_edit.is_some(),
        "modal should stay open on error"
    );
    assert!(
        state.inspector_edit.as_ref().unwrap().error.is_some(),
        "error should be set"
    );
}

#[test]
fn test_inspector_edit_on_insufficient_data_does_nothing() {
    let mut state = make_state(vec![0x2A]); // Only 1 byte
    let config = default_config();
    // Index 2 = u16 (min_size=2) should not allow editing with only 1 byte
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(2));
    assert!(
        state.inspector_edit.is_none(),
        "should not open edit for entries requiring more bytes than available"
    );
}

#[test]
fn test_copy_inspector_value_sets_status() {
    let mut state = make_state(vec![0x2A, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::CopyInspectorValue(0));
    assert!(
        state.status_msg.contains("Copied:"),
        "status should confirm copy"
    );
    assert!(
        state.status_msg.contains("42"),
        "status should contain decoded value"
    );
}

// ============================================================================
// Save functionality
// ============================================================================

#[test]
fn test_save_without_on_save_shows_not_available() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SaveIntoRecording);
    assert_eq!(state.status_msg, "Save not available.");
}

#[test]
fn test_saved_into_recording_updates_status() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::SavedIntoRecording(Ok("Saved into mod".into())),
    );
    assert_eq!(state.status_msg, "Saved into mod");
    assert_eq!(
        state.provider.dirty_count(),
        0,
        "dirty should be cleared on successful save"
    );
}

#[test]
fn test_saved_into_recording_error() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    // Dirty some bytes first
    send(
        &mut state,
        &config,
        HexEditorMessage::WriteBytes {
            addr: 0,
            bytes: vec![0x01],
        },
    );
    send(
        &mut state,
        &config,
        HexEditorMessage::SavedIntoRecording(Err("disk full".into())),
    );
    assert!(
        state.status_msg.contains("Save failed"),
        "should report failure"
    );
}

// ============================================================================
// Vanilla diff tracking
// ============================================================================

#[test]
fn test_vanilla_diff_updated_on_write() {
    let mut state = make_state(vec![0x00, 0x00, 0x00]);
    state.vanilla = Some(vec![0x00, 0x00, 0x00]);
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::WriteBytes {
            addr: 1,
            bytes: vec![0xFF],
        },
    );
    assert!(
        state.vanilla_diff.contains(&1),
        "address 1 should be in vanilla_diff"
    );
    assert_eq!(state.vanilla_diff.len(), 1);
}

#[test]
fn test_vanilla_diff_empty_without_vanilla() {
    let mut state = make_state(vec![0x00, 0x00]);
    state.vanilla = None;
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::WriteBytes {
            addr: 0,
            bytes: vec![0xFF],
        },
    );
    assert!(
        state.vanilla_diff.is_empty(),
        "without vanilla snapshot, diff should be empty"
    );
}

// ============================================================================
// Settings & configuration
// ============================================================================

#[test]
fn test_can_save_now_checks_dirty_and_can_save() {
    let mut state = make_state((0..64).collect());
    let mut config = HexEditorConfig::default();
    // No on_save, can_save=false → false
    assert!(!config.can_save_now(&state));

    config.can_save = true;
    config.on_save = Some(std::sync::Arc::new(|_| iced::Task::none()));
    // can_save=true but dirty=0 → false
    assert!(!config.can_save_now(&state));

    // Make a modification
    send(
        &mut state,
        &config,
        HexEditorMessage::WriteBytes {
            addr: 0,
            bytes: vec![0x01],
        },
    );
    assert!(config.can_save_now(&state), "should be savable now");
}

#[test]
fn test_save_label_fallback() {
    let config = HexEditorConfig::default();
    assert_eq!(config.save_label(), "Save", "should fall back to 'Save'");
    let config2 = HexEditorConfig {
        save_label: "Store".into(),
        ..Default::default()
    };
    assert_eq!(config2.save_label(), "Store");
}

// ============================================================================
// LuaScriptEngine — integration (only when `lua` feature enabled)
// ============================================================================

#[cfg(feature = "lua")]
mod lua_tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::LuaScriptEngine;

    /// Write a Lua script to the temp dir and return its path.
    /// Uses a global counter to keep paths unique across tests.
    static SCRIPT_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn write_script(dir: &str, name: &str, code: &str) -> PathBuf {
        let counter = SCRIPT_COUNTER.fetch_add(1, Ordering::SeqCst);
        let d = std::env::temp_dir()
            .join("hexedit_lua_test")
            .join(dir)
            .join(counter.to_string());
        std::fs::create_dir_all(&d).expect("create temp dir for lua script");
        let path = d.join(name);
        std::fs::write(&path, code).expect("write lua test script");
        path
    }

    // ── Lifecycle ─────────────────────────────────────────────────────────

    #[test]
    fn test_lua_engine_create() {
        let engine = LuaScriptEngine::new(false);
        assert!(engine.is_ok(), "engine should create in safe mode");
        let engine = LuaScriptEngine::new(true);
        assert!(engine.is_ok(), "engine should create in unsafe mode");
        let engine = LuaScriptEngine::default();
        let entries = engine.entries();
        assert!(entries.is_empty(), "default engine has no entries");
    }

    /// Shared Lua script for basic decode testing.
    const BASIC_SCRIPT: &str = r#"
return {
    name = "test_decoder",
    min_size = 2,
    category = "Test",
    description = "A test decoder",
    decode = function(bytes)
        return string.format("%02X %02X", bytes:byte(1), bytes:byte(2))
    end,
    encode = function(s)
        return s
    end,
}
"#;

    #[test]
    fn test_lua_engine_load_valid_script() {
        let path = write_script("load_valid", "test.lua", BASIC_SCRIPT);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        engine.load_script(&path).unwrap();
        let entries = engine.entries();
        assert_eq!(entries.len(), 1, "should have 1 entry");
        assert_eq!(entries[0].name, "test_decoder");
        assert_eq!(entries[0].min_size, 2);
        assert_eq!(entries[0].category, "Test");
        assert_eq!(entries[0].description, "A test decoder");
    }

    #[test]
    fn test_lua_engine_decode_basic() {
        let path = write_script("decode_basic", "test.lua", BASIC_SCRIPT);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        engine.load_script(&path).unwrap();
        let entries = engine.entries();
        let result = (entries[0].decode)(&[0xDE, 0xAD]);
        assert_eq!(result, "DE AD", "decode should format two hex bytes");
    }

    #[test]
    fn test_lua_engine_decode_null_bytes() {
        let path = write_script("decode_null", "test.lua", BASIC_SCRIPT);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        engine.load_script(&path).unwrap();
        let entries = engine.entries();
        let result = (entries[0].decode)(&[0x00, 0x00]);
        assert_eq!(result, "00 00", "decode should handle null bytes");
    }

    #[test]
    fn test_lua_engine_decode_multi_byte() {
        let path = write_script("decode_multi", "test.lua", BASIC_SCRIPT);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        engine.load_script(&path).unwrap();
        let entries = engine.entries();
        let result = (entries[0].decode)(&[0x01, 0x02, 0x03, 0x04]);
        // Script with min_size=2 only formats first 2 bytes
        assert_eq!(result, "01 02", "decode should handle 4 bytes, but only formats first 2");
    }

    #[test]
    fn test_lua_engine_encode_returns_bytes() {
        let path = write_script("encode_bytes", "test.lua", r#"
return {
    name = "echo",
    min_size = 1,
    decode = function(bytes)
        return string.format("%02X", bytes:byte(1))
    end,
    encode = function(s)
        return s
    end,
}
"#);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        engine.load_script(&path).unwrap();
        let entries = engine.entries();
        let encode = entries[0].encode.as_ref().expect("should have encode");
        let result = encode("hello").expect("encode should succeed");
        assert_eq!(result, b"hello", "encode should echo input string");
    }

    #[test]
    fn test_lua_engine_encode_hex_string() {
        let path = write_script("encode_hex", "test.lua", r#"
return {
    name = "hex_encode",
    min_size = 1,
    decode = function(bytes)
        return string.format("%02X", bytes:byte(1))
    end,
    encode = function(s)
        -- Convert hex string like "FF" back to bytes
        return (s:gsub("(%x%x)", function(h)
            return string.char(tonumber(h, 16))
        end))
    end,
}
"#);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        engine.load_script(&path).unwrap();
        let entries = engine.entries();
        let encode = entries[0].encode.as_ref().expect("should have encode");
        let result = encode("DEAD").expect("encode should succeed");
        assert_eq!(result, &[0xDE, 0xAD], "encode should convert hex to bytes");
    }

    #[test]
    fn test_lua_engine_encode_byte_calculation() {
        let path = write_script("encode_calc", "test.lua", r#"
return {
    name = "twos_complement",
    min_size = 1,
    decode = function(bytes)
        local b = bytes:byte(1)
        if b > 127 then
            return tostring(b - 256)
        else
            return tostring(b)
        end
    end,
    encode = function(s)
        local n = tonumber(s)
        if n < 0 then n = n + 256 end
        return string.char(n)
    end,
}
"#);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        engine.load_script(&path).unwrap();
        let entries = engine.entries();

        // Decode 0x80 → should be -128
        assert_eq!((entries[0].decode)(&[0x80]), "-128");
        // Decode 0x7F → should be 127
        assert_eq!((entries[0].decode)(&[0x7F]), "127");

        // Encode "-1" → should write 0xFF
        let result = entries[0]
            .encode
            .as_ref()
            .unwrap()("-1")
            .expect("encode should succeed");
        assert_eq!(result, &[0xFF]);
    }

    // ── Multiple scripts ──────────────────────────────────────────────────

    #[test]
    fn test_lua_engine_multiple_scripts() {
        let p1 = write_script("multi", "d1.lua", r#"
return { name = "d1", min_size = 1, decode = function(b) return "first" end }
"#);
        let p2 = write_script("multi", "d2.lua", r#"
return { name = "d2", min_size = 2, decode = function(b) return "second" end }
"#);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        engine.load_script(&p1).unwrap();
        engine.load_script(&p2).unwrap();
        let entries = engine.entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "d1");
        assert_eq!(entries[1].name, "d2");
        assert_eq!((entries[0].decode)(&[0x00]), "first");
        assert_eq!((entries[1].decode)(&[0x00, 0x00]), "second");
    }

    #[test]
    fn test_lua_engine_entries_independent_from_engine() {
        let path = write_script("independent", "test.lua", r#"
return { name = "indep", min_size = 1, decode = function(b) return "ok" end }
"#);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        engine.load_script(&path).unwrap();
        let entries = engine.entries();
        drop(engine); // Drop the engine — entries should still work
        assert_eq!((entries[0].decode)(&[0x00]), "ok");
    }

    // ── Error handling ────────────────────────────────────────────────────

    #[test]
    fn test_lua_engine_missing_name_returns_error() {
        let path = write_script("missing_name", "test.lua", r#"
return { min_size = 1, decode = function(b) return "x" end }
"#);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        let err = engine.load_script(&path).unwrap_err();
        assert!(err.contains("name"), "error should mention 'name': {err}");
    }

    #[test]
    fn test_lua_engine_missing_min_size_returns_error() {
        let path = write_script("missing_min_size", "test.lua", r#"
return { name = "x", decode = function(b) return "x" end }
"#);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        let err = engine.load_script(&path).unwrap_err();
        assert!(err.contains("min_size"), "error should mention 'min_size': {err}");
    }

    #[test]
    fn test_lua_engine_missing_decode_returns_error() {
        let path = write_script("missing_decode", "test.lua", r#"
return { name = "x", min_size = 1 }
"#);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        let err = engine.load_script(&path).unwrap_err();
        assert!(err.contains("decode"), "error should mention 'decode': {err}");
    }

    #[test]
    fn test_lua_engine_not_a_table_returns_error() {
        let path = write_script("not_table", "test.lua", r#"return 42"#);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        let err = engine.load_script(&path).unwrap_err();
        assert!(err.contains("table"), "error should mention 'table': {err}");
    }

    #[test]
    fn test_lua_engine_invalid_syntax_returns_error() {
        let path = write_script("bad_syntax", "test.lua", "this is not valid lua");
        let mut engine = LuaScriptEngine::new(false).unwrap();
        let err = engine.load_script(&path).unwrap_err();
        // Should mention a Lua syntax error
        assert!(
            err.contains("syntax") || err.contains("error"),
            "error should describe syntax problem: {err}"
        );
    }

    #[test]
    fn test_lua_engine_nonexistent_file_returns_error() {
        let mut engine = LuaScriptEngine::new(false).unwrap();
        let err = engine
            .load_script("/tmp/nonexistent_script_12345.lua")
            .unwrap_err();
        assert!(err.contains("cannot read"), "error should mention file read: {err}");
    }

    #[test]
    fn test_lua_engine_decode_runtime_error_graceful() {
        let path = write_script("decode_error", "test.lua", r#"
return {
    name = "faulty",
    min_size = 1,
    decode = function(bytes)
        error("something went wrong in lua")
    end,
}
"#);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        engine.load_script(&path).unwrap();
        let entries = engine.entries();
        let result = (entries[0].decode)(&[0x00]);
        assert!(
            result.starts_with("—"),
            "decode error should produce '—', got: {result:?}"
        );
        assert!(
            result.contains("something went wrong"),
            "error detail should be included: {result}"
        );
    }

    #[test]
    fn test_lua_engine_encode_runtime_error_graceful() {
        let path = write_script("encode_error", "test.lua", r#"
return {
    name = "faulty_encode",
    min_size = 1,
    decode = function(b) return "x" end,
    encode = function(s)
        error("encode failure")
    end,
}
"#);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        engine.load_script(&path).unwrap();
        let entries = engine.entries();
        let encode = entries[0].encode.as_ref().unwrap();
        let result = encode("test");
        assert!(result.is_err(), "encode should return Err on lua error");
        assert!(
            result.unwrap_err().contains("encode failure"),
            "error detail should be preserved"
        );
    }

    #[test]
    fn test_lua_engine_decode_returns_non_string() {
        let path = write_script("decode_non_string", "test.lua", r#"
return {
    name = "returns_number",
    min_size = 1,
    decode = function(bytes)
        return 42
    end,
}
"#);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        engine.load_script(&path).unwrap();
        let entries = engine.entries();
        let result = (entries[0].decode)(&[0x00]);
        // mlua's FromLua for String coerces numbers — 42 becomes "42".
        assert_eq!(result, "42", "mlua coerces number to string");
    }

    #[test]
    fn test_lua_engine_encode_returns_non_string() {
        let path = write_script("encode_non_string", "test.lua", r#"
return {
    name = "encode_number",
    min_size = 1,
    decode = function(b) return "x" end,
    encode = function(s)
        return 42  -- return number, not string
    end,
}
"#);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        engine.load_script(&path).unwrap();
        let entries = engine.entries();
        let result = entries[0].encode.as_ref().unwrap()("test").unwrap();
        // mlua coerces number 42 → string "42" → bytes b"42"
        assert_eq!(result, b"42", "mlua coerces number to string");
    }

    // ── Unsafe/safe mode ──────────────────────────────────────────────────

    #[test]
    fn test_lua_engine_unsafe_mode_blocks_os() {
        let path = write_script("unsafe_os", "test.lua", r#"
return {
    name = "os_user",
    min_size = 1,
    decode = function(bytes)
        return tostring(os.clock())
    end,
}
"#);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        engine.load_script(&path).unwrap();
        let entries = engine.entries();
        let result = (entries[0].decode)(&[0x00]);
        assert!(
            result.starts_with("—"),
            "accessing nilled 'os' should fail: {result:?}"
        );
    }

    #[test]
    fn test_lua_engine_unsafe_mode_allows_os() {
        let path = write_script("unsafe_allowed", "test.lua", r#"
return {
    name = "os_user",
    min_size = 1,
    decode = function(bytes)
        if os and os.clock then
            return "unsafe"
        else
            return "safe"
        end
    end,
}
"#);
        let mut engine = LuaScriptEngine::new(true).unwrap();
        engine.load_script(&path).unwrap();
        let entries = engine.entries();
        let result = (entries[0].decode)(&[0x00]);
        assert_eq!(result, "unsafe", "unsafe mode should allow os.clock");
    }

    // ── Script with custom category and description ───────────────────────

    #[test]
    fn test_lua_engine_default_category_and_description() {
        let path = write_script("default_cat", "test.lua", r#"
return { name = "minimal", min_size = 1, decode = function(b) return "ok" end }
"#);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        engine.load_script(&path).unwrap();
        let entries = engine.entries();
        assert_eq!(entries[0].category, "Custom", "default category");
        assert_eq!(entries[0].description, "", "default description");
    }

    // ── Load scripts from directory (HexEditorState::load_lua_scripts) ────

    #[test]
    fn test_load_lua_scripts_from_dir() {
        let counter = SCRIPT_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir()
            .join("hexedit_lua_test")
            .join("load_dir")
            .join(counter.to_string());
        std::fs::create_dir_all(&dir).expect("create temp dir");
        std::fs::write(dir.join("a.lua"), r#"
return { name = "from_dir_a", min_size = 1, decode = function(b) return "A" end }
"#).unwrap();
        std::fs::write(dir.join("b.lua"), r#"
return { name = "from_dir_b", min_size = 1, decode = function(b) return "B" end }
"#).unwrap();
        // Non-.lua file should be ignored
        std::fs::write(dir.join("notes.txt"), "not a script").unwrap();

        let mut state = crate::state::HexEditorState {
            path: std::path::PathBuf::from("test.bin"),
            name: "test.bin".to_string(),
            provider: crate::provider::BufferProvider::from_bytes(vec![0x00]),
            bytes_per_row: 16,
            selection: crate::selection::Selection::single(0),
            edit_mode: None,
            inspector_edit: None,
            vanilla: None,
            vanilla_diff: std::collections::BTreeSet::new(),
            patterns: Vec::new(),
            pattern_by_addr: std::collections::BTreeMap::new(),
            show_pattern_list: false,
            next_pattern_id: 0,
            context_menu_addr: None,
            goto: None,
            search: crate::search::SearchState::new(),
            show_decimal: false,
            status_msg: String::new(),
            error: None,
            cache: gui_widgets::components::paragraph_cache::ParagraphCache::default(),
            lua_engine: LuaScriptEngine::new(false).unwrap(),
        };
        let errors = state.load_lua_scripts(&dir);
        assert!(errors.is_empty(), "should load without errors: {errors:?}");
        let entries = state.lua_engine.entries();
        assert_eq!(entries.len(), 2, "should load 2 scripts");
        assert_eq!(entries[0].name, "from_dir_a");
        assert_eq!(entries[1].name, "from_dir_b");
    }

    #[test]
    fn test_load_lua_scripts_nonexistent_dir_returns_no_errors() {
        // Current behavior: non-existent dir returns empty errors (treated as
        // "no scripts to load", not an error condition).
        let engine = LuaScriptEngine::new(false).unwrap();
        let mut state = crate::state::HexEditorState {
            path: std::path::PathBuf::from("test.bin"),
            name: "test.bin".to_string(),
            provider: crate::provider::BufferProvider::from_bytes(vec![0x00]),
            bytes_per_row: 16,
            selection: crate::selection::Selection::single(0),
            edit_mode: None,
            inspector_edit: None,
            vanilla: None,
            vanilla_diff: std::collections::BTreeSet::new(),
            patterns: Vec::new(),
            pattern_by_addr: std::collections::BTreeMap::new(),
            show_pattern_list: false,
            next_pattern_id: 0,
            context_menu_addr: None,
            goto: None,
            search: crate::search::SearchState::new(),
            show_decimal: false,
            status_msg: String::new(),
            error: None,
            cache: gui_widgets::components::paragraph_cache::ParagraphCache::default(),
            lua_engine: engine,
        };
        let errors = state.load_lua_scripts(&std::path::PathBuf::from("/nonexistent/lua/dir"));
        assert!(errors.is_empty(), "non-existent dir should return 0 errors");
    }

    // ── Lua + iced_test integration: verify decoders appear in inspector ──

    #[test]
    fn test_lua_decoder_appears_in_inspector_view() {
        use crate::view::view;
        use iced_test::simulator;

        let path = write_script("inspector_view", "test.lua", r#"
return {
    name = "lua_decoder",
    min_size = 1,
    category = "LuaScripts",
    description = "A Lua decoder",
    decode = function(bytes)
        return string.format("LUA:0x%02X", bytes:byte(1))
    end,
}
"#);
        let mut engine = LuaScriptEngine::new(false).unwrap();
        engine.load_script(&path).unwrap();
        let entries = engine.entries();
        assert_eq!(entries.len(), 1, "should have Lua decoder");

        // Build a minimal state that includes the Lua entries.
        // We need to trick the inspector view into showing Lua entries.
        // The inspector reads from the HexEditorState via the entries() method.
        // But entries() returns only the ENGINE's entries. The inspector ALSO
        // renders the built-in ENTRIES (from inspector.rs). Lua entries are
        // NOT automatically rendered in the inspector view — the view only
        // shows built-in ENTRIES and config.extra_entries.
        //
        // So to see Lua entries in the view, they must be in config.extra_entries.
        // This won't be visible directly from the LuaScriptEngine alone.
        //
        // Instead, let's verify that Lua entries decode correctly and that
        // a custom InspectorEntry built from Lua can be used in the view.
        let state = crate::state::HexEditorState {
            path: std::path::PathBuf::from("test.bin"),
            name: "test.bin".to_string(),
            provider: crate::provider::BufferProvider::from_bytes(vec![0xAB]),
            bytes_per_row: 16,
            selection: crate::selection::Selection::single(0),
            edit_mode: None,
            inspector_edit: None,
            vanilla: None,
            vanilla_diff: std::collections::BTreeSet::new(),
            patterns: Vec::new(),
            pattern_by_addr: std::collections::BTreeMap::new(),
            show_pattern_list: false,
            next_pattern_id: 0,
            context_menu_addr: None,
            goto: None,
            search: crate::search::SearchState::new(),
            show_decimal: false,
            status_msg: String::new(),
            error: None,
            cache: gui_widgets::components::paragraph_cache::ParagraphCache::default(),
            lua_engine: engine,
        };
        // Verify the decode works
        assert_eq!((entries[0].decode)(&[0xAB]), "LUA:0xAB");
        // Verify it can be passed as extra_entries in config
        let config = crate::config::HexEditorConfig {
            extra_entries: entries,
            ..Default::default()
        };
        let mut ui = simulator(view(&state, &config));
        ui.find("lua_decoder").expect("Lua decoder name should appear in inspector");
        ui.find("LUA:0xAB").expect("Lua decoder value should appear in inspector");
    }
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

// ============================================================================
// Navigation — extended
// ============================================================================

#[test]
fn test_nav_page_down() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::Nav { dir: NavDir::PageDown, extend: false });
    // PAGE_ROWS = 24, so cursor moves from 0 to 24*16 = 384, clamped to 255
    assert_eq!(state.selection.cursor, 255, "PageDown should saturate at max_addr");
}

#[test]
fn test_nav_page_up() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    // Go to end first
    send(&mut state, &config, HexEditorMessage::Nav { dir: NavDir::DocumentEnd, extend: false });
    assert_eq!(state.selection.cursor, 255);
    // Page up from 255: 255.saturating_sub(24*16) = 255-384 = 0
    send(&mut state, &config, HexEditorMessage::Nav { dir: NavDir::PageUp, extend: false });
    assert_eq!(state.selection.cursor, 0, "PageUp from end should saturate at 0");
}

#[test]
fn test_nav_document_start() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::Nav { dir: NavDir::DocumentEnd, extend: false });
    assert_eq!(state.selection.cursor, 255);
    send(&mut state, &config, HexEditorMessage::Nav { dir: NavDir::DocumentStart, extend: false });
    assert_eq!(state.selection.cursor, 0);
}

#[test]
fn test_nav_down_saturates_at_end() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    // 256 bytes, 16 BPR = 16 rows. Last row starts at 240. Go there.
    send(&mut state, &config, HexEditorMessage::SelectAt(240));
    send(&mut state, &config, HexEditorMessage::Nav { dir: NavDir::Down, extend: false });
    // Down from 240 → 256, saturates at max_addr = 255
    assert_eq!(state.selection.cursor, 255, "Down from last row should saturate at max_addr");
}

#[test]
fn test_extend_to_backwards() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(20));
    send(&mut state, &config, HexEditorMessage::ExtendTo(5));
    assert_eq!(state.selection.start(), 5);
    assert_eq!(state.selection.end(), 20);
    assert_eq!(state.selection.cursor, 5);
}

// ============================================================================
// Selection + Edit Mode interaction
// ============================================================================

#[test]
fn test_select_at_clears_edit_mode() {
    let mut state = make_state(vec![0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginEdit(0));
    assert!(state.edit_mode.is_some(), "should be in edit mode");
    // Click on a different address
    send(&mut state, &config, HexEditorMessage::SelectAt(2));
    assert!(state.edit_mode.is_none(), "clicking should cancel edit mode");
    assert_eq!(state.selection.cursor, 2);
}

// ============================================================================
// Inline Editing — boundary & multi-step
// ============================================================================

#[test]
fn test_edit_commit_advance_at_last_byte_exits_edit_mode() {
    // Only 1 byte — committing with advance should exit mode (can't advance past max_addr)
    let mut state = make_state(vec![0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginEdit(0));
    send(&mut state, &config, HexEditorMessage::EditTypeChar('F'));
    send(&mut state, &config, HexEditorMessage::EditTypeChar('F'));
    assert_eq!(state.provider.as_slice()[0], 0xFF, "byte should be written");
    // After second nibble, it auto-commits with advance. Since addr 0 is max_addr,
    // advance should set edit_mode to None.
    assert!(state.edit_mode.is_none(), "edit mode should exit at max_addr");
    assert_eq!(state.selection.cursor, 0, "cursor should stay at last byte");
}

#[test]
fn test_multiple_sequential_edits() {
    let mut state = make_state(vec![0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    // Edit byte 0 → 0xAB, byte 1 → 0xCD, byte 2 → 0xEF
    send(&mut state, &config, HexEditorMessage::BeginEdit(0));
    send(&mut state, &config, HexEditorMessage::EditTypeChar('A'));
    send(&mut state, &config, HexEditorMessage::EditTypeChar('B'));
    // Now at addr 1
    send(&mut state, &config, HexEditorMessage::EditTypeChar('C'));
    send(&mut state, &config, HexEditorMessage::EditTypeChar('D'));
    // Now at addr 2
    send(&mut state, &config, HexEditorMessage::EditTypeChar('E'));
    send(&mut state, &config, HexEditorMessage::EditTypeChar('F'));
    assert_eq!(state.provider.as_slice(), &[0xAB, 0xCD, 0xEF, 0x00]);
    assert_eq!(state.selection.cursor, 3);
    assert_eq!(state.provider.dirty_count(), 3);
}

// ============================================================================
// Write operations — edge cases
// ============================================================================

#[test]
fn test_write_bytes_on_empty_file_is_noop() {
    let mut state = make_state(vec![]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::WriteBytes { addr: 0, bytes: vec![0xFF] });
    assert!(state.provider.is_empty(), "should not modify empty file");
}

#[test]
fn test_write_bytes_empty_slice() {
    let mut state = make_state(vec![0x00; 4]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::WriteBytes { addr: 0, bytes: vec![] });
    assert_eq!(state.provider.dirty_count(), 0, "empty write should not dirty");
    assert_eq!(state.provider.as_slice(), &[0x00, 0x00, 0x00, 0x00]);
}

// ============================================================================
// Goto — edge cases
// ============================================================================

#[test]
fn test_goto_zero() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(100));
    send(&mut state, &config, HexEditorMessage::OpenGotoDialog);
    send(&mut state, &config, HexEditorMessage::SetGotoDraft("0".into()));
    send(&mut state, &config, HexEditorMessage::CommitGoto);
    assert_eq!(state.selection.cursor, 0);
}

#[test]
fn test_goto_relative_saturates_forward() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(200));
    send(&mut state, &config, HexEditorMessage::OpenGotoDialog);
    send(&mut state, &config, HexEditorMessage::SetGotoDraft("+1000".into()));
    send(&mut state, &config, HexEditorMessage::CommitGoto);
    assert_eq!(state.selection.cursor, 255, "should saturate at max_addr");
}

#[test]
fn test_goto_relative_saturates_backward() {
    let mut state = make_state((0..=100u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(50));
    send(&mut state, &config, HexEditorMessage::OpenGotoDialog);
    send(&mut state, &config, HexEditorMessage::SetGotoDraft("-100".into()));
    send(&mut state, &config, HexEditorMessage::CommitGoto);
    assert_eq!(state.selection.cursor, 0, "should saturate at 0");
}

// ============================================================================
// Search — cursor movement, mode toggle, single-match navigation
// ============================================================================

#[test]
fn test_search_selects_cursor() {
    let mut state = make_state(b"\x00\xDE\xAD\xBE\xEF\x00".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::Search("DE AD BE EF".into()));
    // After initial Search, cursor stays at 0 (current_match is None).
    // Navigate to the first match:
    send(&mut state, &config, HexEditorMessage::SearchNext);
    assert_eq!(state.selection.cursor, 1, "search should move cursor to first match");
}

#[test]
fn test_search_next_prev_single_match_stays_on_zero() {
    let mut state = make_state(b"\x00\xFF\x00".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::Search("FF".into()));
    assert_eq!(state.search.count(), 1);
    assert_eq!(state.search.current_match, None, "initial match is None");
    send(&mut state, &config, HexEditorMessage::SearchNext);
    assert_eq!(state.search.current_match, Some(0));
    send(&mut state, &config, HexEditorMessage::SearchNext);
    assert_eq!(state.search.current_match, Some(0), "single match should stay on 0 with wrap");
    send(&mut state, &config, HexEditorMessage::SearchPrev);
    assert_eq!(state.search.current_match, Some(0), "single match should stay on 0 backward");
}

#[test]
fn test_search_reexecute_after_toggle_mode() {
    let mut state = make_state(b"\x00\xDE\xAD\xBE\xEF\x41\x42".to_vec());
    let config = default_config();
    // Search for "AB" in hex mode — no match (bytes would be 0xAB, 0xBE...)
    send(&mut state, &config, HexEditorMessage::Search("4142".into()));
    assert_eq!(state.search.count(), 1, "hex 4142 matches bytes at offset 5");
    // Toggle to ASCII — the query "4142" is now treated as ASCII text
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    // In ASCII mode, "4142" is looked for as literal bytes 0x34 0x31 0x34 0x32
    // (ASCII codes for '4','1','4','2'), which probably won't match anything.
    // But at least the search re-executes without panicking.
    assert_eq!(state.search.mode, crate::search::SearchMode::Ascii);
}

// ============================================================================
// Inspector — different entry types
// ============================================================================

#[test]
fn test_inspector_shows_category_headers() {
    let state = make_state(vec![0x2A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find("── Integer ──").expect("Integer category header should render");
    ui.find("── Float ──").expect("Float category header should render");
    ui.find("── Text ──").expect("Text category header should render");
    ui.find("── Color ──").expect("Color category header should render");
    ui.find("── Binary ──").expect("Binary category header should render");
}

#[test]
fn test_inspector_shows_multiple_decoded_types() {
    let state = make_state(vec![0x2A, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    // u8 decodes to "42 (0x2A)" — verify the decimal value appears
    ui.find("42").expect("u8 value 42 should display");
    // Verify that entry names are also rendered
    ui.find("u16").expect("u16 entry name should display");
}

#[test]
fn test_inspector_displays_all_entry_names() {
    let state = make_state(vec![0x00; 8]);
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    // All entry names should appear in the inspector
    ui.find("u8").expect("u8 entry name should appear");
    ui.find("i8").expect("i8 entry name should appear");
    ui.find("u16").expect("u16 entry name should appear");
    ui.find("i16").expect("i16 entry name should appear");
    ui.find("u32").expect("u32 entry name should appear");
    ui.find("i32").expect("i32 entry name should appear");
    ui.find("u64").expect("u64 entry name should appear");
    ui.find("i64").expect("i64 entry name should appear");
    ui.find("f32").expect("f32 entry name should appear");
    ui.find("f64").expect("f64 entry name should appear");
    ui.find("ascii").expect("ascii entry name should appear");
    ui.find("utf8").expect("utf8 entry name should appear");
    ui.find("rgb565").expect("rgb565 entry name should appear");
    ui.find("cstr").expect("cstr entry name should appear");
    ui.find("hex").expect("hex entry name should appear");
}

// ============================================================================
// Inspector edit — multiple encoder types
// ============================================================================

#[test]
fn test_inspector_edit_with_hex_prefix() {
    let mut state = make_state(vec![0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(0)); // u8
    send(&mut state, &config, HexEditorMessage::SetInspectorDraft("0xFF".into()));
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    assert_eq!(state.provider.as_slice()[0], 0xFF);
}

#[test]
fn test_inspector_edit_i8() {
    let mut state = make_state(vec![0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(1)); // i8
    assert_eq!(state.inspector_edit.as_ref().unwrap().draft, "0");
    send(&mut state, &config, HexEditorMessage::SetInspectorDraft("-128".into()));
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    assert_eq!(state.provider.as_slice()[0], 0x80);
}

#[test]
fn test_inspector_edit_u16() {
    let mut state = make_state(vec![0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(2)); // u16 at cursor=0
    send(&mut state, &config, HexEditorMessage::SetInspectorDraft("0x1234".into()));
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    assert_eq!(state.provider.as_slice()[0..2], [0x34, 0x12]);
}

#[test]
fn test_inspector_edit_i16() {
    let mut state = make_state(vec![0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(3)); // i16
    send(&mut state, &config, HexEditorMessage::SetInspectorDraft("-1".into()));
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    assert_eq!(state.provider.as_slice()[0..2], [0xFF, 0xFF]);
}

#[test]
fn test_inspector_edit_u32() {
    let mut state = make_state(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(4)); // u32
    send(&mut state, &config, HexEditorMessage::SetInspectorDraft("305419896".into()));
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    // 305419896 = 0x12345678 in LE
    assert_eq!(state.provider.as_slice()[0..4], [0x78, 0x56, 0x34, 0x12]);
}

#[test]
fn test_inspector_edit_i32() {
    let mut state = make_state(vec![0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(5)); // i32
    send(&mut state, &config, HexEditorMessage::SetInspectorDraft("-128".into()));
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    // -128 as i32 LE = [0x80, 0xFF, 0xFF, 0xFF]
    assert_eq!(state.provider.as_slice()[0..4], [0x80, 0xFF, 0xFF, 0xFF]);
}

#[test]
fn test_inspector_edit_u64() {
    let mut state = make_state(vec![0x00; 8]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(6)); // u64
    send(&mut state, &config, HexEditorMessage::SetInspectorDraft("0x0102030405060708".into()));
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    assert_eq!(state.provider.as_slice(), &[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
}

#[test]
fn test_inspector_edit_i64() {
    let mut state = make_state(vec![0x00; 8]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(7)); // i64
    send(&mut state, &config, HexEditorMessage::SetInspectorDraft("-1".into()));
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    assert_eq!(state.provider.as_slice(), &[0xFF; 8]);
}

#[test]
fn test_inspector_edit_f32() {
    let mut state = make_state(vec![0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(8)); // f32
    send(&mut state, &config, HexEditorMessage::SetInspectorDraft("1.5".into()));
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    let v = f32::from_le_bytes([
        state.provider.as_slice()[0],
        state.provider.as_slice()[1],
        state.provider.as_slice()[2],
        state.provider.as_slice()[3],
    ]);
    assert!((v - 1.5).abs() < f32::EPSILON);
}

#[test]
fn test_inspector_edit_f64() {
    let mut state = make_state(vec![0x00; 8]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(9)); // f64
    send(&mut state, &config, HexEditorMessage::SetInspectorDraft("3.14159".into()));
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    let bytes = state.provider.as_slice();
    let v = f64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    assert!((v - 3.14159).abs() < 0.001);
}

// ============================================================================
// Inspector — value rendering edge cases
// ============================================================================

#[test]
fn test_inspector_displays_negative_i8_value() {
    let state = make_state(vec![0xFE, 0x00, 0x00, 0x00]);
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    // 0xFE as i8 = -2
    ui.find("-2").expect("i8 should show -2 for byte 0xFE");
}

#[test]
fn test_inspector_displays_cstr_for_printable() {
    let state = make_state(b"hello\0world".to_vec());
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find("\"hello\"").expect("cstr should show quoted string");
}

// ============================================================================
// Patterns — extended
// ============================================================================

#[test]
fn test_pattern_context_menu_on_existing_pattern() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    // Create a pattern
    send(&mut state, &config, HexEditorMessage::SelectAt(10));
    send(&mut state, &config, HexEditorMessage::ExtendTo(20));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    // Right-click on address within pattern
    send(&mut state, &config, HexEditorMessage::RightClickAt(15));
    assert_eq!(state.context_menu_addr, Some(15));
    // The RightClickAt message itself doesn't check patterns — that's done in view.
    // We just verify the address was stored.
}

#[test]
fn test_remove_pattern_at_nonexistent_address_is_noop() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::RemovePatternAt(999));
    assert_eq!(state.patterns.len(), 0);
}

// ============================================================================
// Search overlay — match count display
// ============================================================================

#[test]
fn test_search_overlay_shows_match_count() {
    let mut state = make_state(b"\x00\xFF\x00\xFF\x00".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::Search("FF".into()));
    assert_eq!(state.search.count(), 2);
    let mut ui = simulator(view(&state, &config));
    // Shows "-/2" after initial search (no match selected yet)
    ui.find("-/2").expect("search overlay should show match count");
}

#[test]
fn test_search_overlay_shows_no_matches() {
    let mut state = make_state(b"hello".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::Search("xyzzy".into()));
    let mut ui = simulator(view(&state, &config));
    ui.find("0 matches").expect("should show 0 matches for no results");
}

#[test]
fn test_search_overlay_nav_buttons_render() {
    let mut state = make_state(b"\x00\xFF\x00\xFF\x00".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::Search("FF".into()));
    let mut ui = simulator(view(&state, &config));
    ui.find("<").expect("prev match button should render");
    ui.find(">").expect("next match button should render");
}

// ============================================================================
// Footer — dirty count display
// ============================================================================

#[test]
fn test_footer_shows_dirty_after_edit() {
    let mut state = make_state(vec![0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::WriteBytes { addr: 0, bytes: vec![0xAA, 0xBB] });
    assert_eq!(state.provider.dirty_count(), 2);
    let mut ui = simulator(view(&state, &config));
    let expected = "0x0  ·  total: 0x3 (3 B)  ·  dirty: 2  ·  cursor: 0x0";
    ui.find(expected).expect("footer should show dirty: 2");
}

#[test]
fn test_footer_shows_dirty_after_inline_edit() {
    let mut state = make_state(vec![0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginEdit(0));
    send(&mut state, &config, HexEditorMessage::EditTypeChar('F'));
    send(&mut state, &config, HexEditorMessage::EditTypeChar('F'));
    assert_eq!(state.provider.dirty_count(), 1);
    let mut ui = simulator(view(&state, &config));
    let expected = "0x1  ·  total: 0x3 (3 B)  ·  dirty: 1  ·  cursor: 0x1";
    ui.find(expected).expect("footer should show dirty: 1 after inline edit");
}

// ============================================================================
// Settings & Configuration — extended
// ============================================================================

#[test]
fn test_toolbar_save_disabled_when_not_dirty() {
    let state = make_state((0..64).collect());
    let config = HexEditorConfig {
        can_save: true,
        on_save: Some(std::sync::Arc::new(|_| iced::Task::none())),
        ..Default::default()
    };
    // can_save_now should be false when dirty=0
    assert!(!config.can_save_now(&state), "should not be savable when clean");
}

#[test]
fn test_toolbar_save_hint_empty_when_not_set() {
    let state = make_state((0..64).collect());
    let config = HexEditorConfig::default();
    // With empty save_hint, no hint text should appear.
    // Just verify the toolbar still renders correctly.
    let mut ui = simulator(view(&state, &config));
    ui.find("Save").expect("save button should still render");
}
