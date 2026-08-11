use super::*;

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
fn header_text_64_bpr1() -> &'static str {
    "test.bin  ·  64 bytes  ·  1 bytes/row"
}
fn header_text_64_bpr20() -> &'static str {
    "test.bin  ·  64 bytes  ·  20 bytes/row"
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
    // 0 is below MIN_BYTES_PER_ROW (1) — must be ignored.
    send(&mut state, &config, HexEditorMessage::SetBytesPerRow(0));
    // 65 is above MAX_BYTES_PER_ROW (64) — must also be ignored.
    send(&mut state, &config, HexEditorMessage::SetBytesPerRow(65));
    let mut ui = simulator(view(&state, &config));
    ui.find(header_text_64())
        .expect("out-of-range BPR should be ignored, keeping default");
}

#[test]
fn test_header_accepts_custom_bpr() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    // 20 is not one of the preset buttons (8/16/32) but is within the
    // MIN..=MAX range, so it must be applied.
    send(&mut state, &config, HexEditorMessage::SetBytesPerRow(20));
    let mut ui = simulator(view(&state, &config));
    ui.find(header_text_64_bpr20())
        .expect("custom in-range BPR should be applied");
}

#[test]
fn test_header_accepts_max_boundary_bpr() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    // Upper boundary of the allowed range must be accepted.
    send(&mut state, &config, HexEditorMessage::SetBytesPerRow(64));
    let mut ui = simulator(view(&state, &config));
    ui.find("test.bin  ·  64 bytes  ·  64 bytes/row")
        .expect("MAX_BYTES_PER_ROW should be accepted");
}

#[test]
fn test_header_accepts_min_boundary_bpr() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    // Lower boundary of the allowed range must be accepted.
    send(&mut state, &config, HexEditorMessage::SetBytesPerRow(1));
    let mut ui = simulator(view(&state, &config));
    ui.find(header_text_64_bpr1())
        .expect("MIN_BYTES_PER_ROW should be accepted");
}

#[test]
fn test_set_bpr_syncs_input_draft() {
    // Regression: the settings-modal custom input must mirror the active
    // bytes-per-row so preset-button clicks don't leave a stale draft.
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SetBytesPerRow(20));
    assert_eq!(
        state.bpr_input, "20",
        "bpr_input should be synced to the active bytes_per_row"
    );
}

#[test]
fn test_reset_settings_syncs_bpr_input() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SetBytesPerRow(20));
    send(&mut state, &config, HexEditorMessage::ResetSettings);
    assert_eq!(
        state.bytes_per_row,
        crate::state::DEFAULT_BYTES_PER_ROW,
        "bytes_per_row should reset to default"
    );
    assert_eq!(
        state.bpr_input,
        crate::state::DEFAULT_BYTES_PER_ROW.to_string(),
        "bpr_input should reset alongside bytes_per_row"
    );
}
