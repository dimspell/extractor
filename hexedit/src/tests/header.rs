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
