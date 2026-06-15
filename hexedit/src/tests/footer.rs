use super::*;

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
// Footer — dirty count display
// ============================================================================

#[test]
fn test_footer_shows_dirty_after_edit() {
    let mut state = make_state(vec![0x00, 0x00, 0x00]);
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::WriteBytes {
            addr: 0,
            bytes: vec![0xAA, 0xBB],
        },
    );
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
    ui.find(expected)
        .expect("footer should show dirty: 1 after inline edit");
}

// ============================================================================
// Write-mode (state-level — Iced's pick_list uses canvas rendering so the
// selected text cannot be found with the `text == "..."` selector)
// ============================================================================

#[test]
fn test_set_write_mode_changes_state() {
    let mut state = make_state(vec![]);
    let config = default_config();
    assert_eq!(state.write_mode, WriteMode::Hex);
    send(&mut state, &config, HexEditorMessage::SetWriteMode(WriteMode::Ascii));
    assert_eq!(state.write_mode, WriteMode::Ascii, "SetWriteMode should update state");
    send(&mut state, &config, HexEditorMessage::SetWriteMode(WriteMode::EucKr));
    assert_eq!(state.write_mode, WriteMode::EucKr);
    send(&mut state, &config, HexEditorMessage::SetWriteMode(WriteMode::Hex));
    assert_eq!(state.write_mode, WriteMode::Hex, "should be able to switch back");
}
