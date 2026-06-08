use super::*;

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
