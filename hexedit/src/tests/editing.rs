use super::*;

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
    assert_eq!(
        state.selection.cursor, 0,
        "cursor should stay at committed addr"
    );
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
    assert!(
        state.edit_mode.is_none(),
        "clicking should cancel edit mode"
    );
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
    assert!(
        state.edit_mode.is_none(),
        "edit mode should exit at max_addr"
    );
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
// Text write mode
// ============================================================================

#[test]
fn test_text_mode_typing_writes_and_advances() {
    let mut state = make_state(vec![0x00; 4]);
    let config = default_config();
    state.write_mode = WriteMode::Ascii;
    send(&mut state, &config, HexEditorMessage::EditTypeChar('H'));
    assert_eq!(state.provider.as_slice()[0], b'H');
    assert_eq!(state.selection.cursor, 1);
}

#[test]
fn test_text_mode_backspace_moves_cursor_left() {
    let mut state = make_state(vec![0x00; 4]);
    let config = default_config();
    state.write_mode = WriteMode::Ascii;
    // Type two characters then backspace
    send(&mut state, &config, HexEditorMessage::EditTypeChar('A'));
    send(&mut state, &config, HexEditorMessage::EditTypeChar('B'));
    assert_eq!(state.selection.cursor, 2);
    send(&mut state, &config, HexEditorMessage::EditBackspace);
    assert_eq!(
        state.selection.cursor, 1,
        "backspace should move cursor left"
    );
    assert_eq!(
        state.provider.as_slice()[1],
        b'B',
        "byte at addr 1 should be unchanged"
    );
}

#[test]
fn test_text_mode_backspace_at_zero_stays_at_zero() {
    let mut state = make_state(vec![0x00; 2]);
    let config = default_config();
    state.write_mode = WriteMode::Ascii;
    send(&mut state, &config, HexEditorMessage::EditBackspace);
    assert_eq!(
        state.selection.cursor, 0,
        "backspace at zero should stay at zero"
    );
}

#[test]
fn test_hex_mode_backspace_still_pops_nibble() {
    // Ensure hex mode Backspace still works (no regression).
    let mut state = make_state(vec![0x00, 0x00]);
    let config = default_config();
    state.write_mode = WriteMode::Hex;
    send(&mut state, &config, HexEditorMessage::BeginEdit(0));
    send(&mut state, &config, HexEditorMessage::EditTypeChar('1'));
    assert_eq!(state.edit_mode.as_ref().unwrap().draft, "1");
    send(&mut state, &config, HexEditorMessage::EditBackspace);
    assert!(state.edit_mode.as_ref().unwrap().draft.is_empty());
}

#[test]
fn test_text_mode_writes_on_empty_file_does_nothing() {
    let mut state = make_state(vec![]);
    let config = default_config();
    state.write_mode = WriteMode::Ascii;
    send(&mut state, &config, HexEditorMessage::DeleteByteAtCursor);
    assert!(state.provider.is_empty(), "should not modify empty file");
}

#[test]
fn test_text_mode_delete_writes_zero_and_advances() {
    let mut state = make_state(vec![0xAB, 0xCD, 0xEF]);
    let config = default_config();
    state.write_mode = WriteMode::Ascii;
    send(&mut state, &config, HexEditorMessage::DeleteByteAtCursor);
    assert_eq!(state.provider.as_slice()[0], 0x00, "should write 0x00");
    assert_eq!(state.selection.cursor, 1, "cursor should advance");
    assert_eq!(state.provider.as_slice()[1], 0xCD, "other bytes unchanged");
}

#[test]
fn test_text_mode_delete_at_max_addr_stays_put() {
    let mut state = make_state(vec![0xAB]);
    let config = default_config();
    state.write_mode = WriteMode::Ascii;
    send(&mut state, &config, HexEditorMessage::DeleteByteAtCursor);
    assert_eq!(state.provider.as_slice()[0], 0x00);
    assert_eq!(state.selection.cursor, 0, "cursor stays at last byte");
}

#[test]
fn test_hex_mode_delete_does_nothing() {
    let mut state = make_state(vec![0xAB]);
    let config = default_config();
    state.write_mode = WriteMode::Hex;
    send(&mut state, &config, HexEditorMessage::DeleteByteAtCursor);
    assert_eq!(
        state.provider.as_slice()[0],
        0xAB,
        "byte unchanged (hex mode)"
    );
}

#[test]
fn test_text_mode_unencodable_shows_status() {
    let mut state = make_state(vec![0x00; 4]);
    let config = default_config();
    state.write_mode = WriteMode::Ascii;
    // Euro sign cannot be encoded in ASCII (non-ASCII).
    send(&mut state, &config, HexEditorMessage::EditTypeChar('€'));
    assert!(
        !state.status_msg.is_empty(),
        "should show status message for unencodable char"
    );
    assert_eq!(state.provider.as_slice()[0], 0x00, "byte should not change");
    assert_eq!(state.selection.cursor, 0, "cursor should not advance");
}

// ============================================================================
// Write operations — edge cases
// ============================================================================

#[test]
fn test_write_bytes_on_empty_file_is_noop() {
    let mut state = make_state(vec![]);
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::WriteBytes {
            addr: 0,
            bytes: vec![0xFF],
        },
    );
    assert!(state.provider.is_empty(), "should not modify empty file");
}

#[test]
fn test_write_bytes_empty_slice() {
    let mut state = make_state(vec![0x00; 4]);
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::WriteBytes {
            addr: 0,
            bytes: vec![],
        },
    );
    assert_eq!(
        state.provider.dirty_count(),
        0,
        "empty write should not dirty"
    );
    assert_eq!(state.provider.as_slice(), &[0x00, 0x00, 0x00, 0x00]);
}
