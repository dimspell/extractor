use super::*;

use crate::state::ComparisonFile;

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
    ui.find("42").expect("u8 (1 byte) should still decode");
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
// Inspector — different entry types
// ============================================================================

#[test]
fn test_inspector_shows_category_headers() {
    let state = make_state(vec![0x2A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find("── Integer ──")
        .expect("Integer category header should render");
    ui.find("── Float ──")
        .expect("Float category header should render");
    ui.find("── Text ──")
        .expect("Text category header should render");
    ui.find("── Color ──")
        .expect("Color category header should render");
    ui.find("── Binary ──")
        .expect("Binary category header should render");
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
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorDraft("0xFF".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    assert_eq!(state.provider.as_slice()[0], 0xFF);
}

#[test]
fn test_inspector_edit_i8() {
    let mut state = make_state(vec![0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(1)); // i8
    assert_eq!(state.inspector_edit.as_ref().unwrap().draft, "0");
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorDraft("-128".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    assert_eq!(state.provider.as_slice()[0], 0x80);
}

#[test]
fn test_inspector_edit_u16() {
    let mut state = make_state(vec![0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(2)); // u16 at cursor=0
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorDraft("0x1234".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    assert_eq!(state.provider.as_slice()[0..2], [0x34, 0x12]);
}

#[test]
fn test_inspector_edit_i16() {
    let mut state = make_state(vec![0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(3)); // i16
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorDraft("-1".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    assert_eq!(state.provider.as_slice()[0..2], [0xFF, 0xFF]);
}

#[test]
fn test_inspector_edit_u32() {
    let mut state = make_state(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(4)); // u32
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorDraft("305419896".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    // 305419896 = 0x12345678 in LE
    assert_eq!(state.provider.as_slice()[0..4], [0x78, 0x56, 0x34, 0x12]);
}

#[test]
fn test_inspector_edit_i32() {
    let mut state = make_state(vec![0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(5)); // i32
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorDraft("-128".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    // -128 as i32 LE = [0x80, 0xFF, 0xFF, 0xFF]
    assert_eq!(state.provider.as_slice()[0..4], [0x80, 0xFF, 0xFF, 0xFF]);
}

#[test]
fn test_inspector_edit_u64() {
    let mut state = make_state(vec![0x00; 8]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(6)); // u64
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorDraft("0x0102030405060708".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    assert_eq!(
        state.provider.as_slice(),
        &[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]
    );
}

#[test]
fn test_inspector_edit_i64() {
    let mut state = make_state(vec![0x00; 8]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(7)); // i64
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorDraft("-1".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    assert_eq!(state.provider.as_slice(), &[0xFF; 8]);
}

#[test]
fn test_inspector_edit_f32() {
    let mut state = make_state(vec![0x00, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(8)); // f32
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorDraft("1.5".into()),
    );
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
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorDraft("3.14159".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitInspectorEdit);
    let bytes = state.provider.as_slice();
    let v = f64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    assert!((v - std::f64::consts::PI).abs() < 0.001);
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
    ui.find("\"hello\"")
        .expect("cstr should show quoted string");
}

// ============================================================================
// Inspector — comparison-file source (A/B toggle)
// ============================================================================

fn state_with_comparison(baseline: Vec<u8>, comparison: Vec<u8>) -> HexEditorState {
    let mut state = make_state(baseline);
    let diff = state
        .provider
        .as_slice()
        .iter()
        .zip(comparison.iter())
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| i as u64)
        .collect();
    state.comparison_file = Some(ComparisonFile {
        name: "other.bin".into(),
        data: comparison,
        diff,
    });
    state
}

#[test]
fn test_inspector_toggle_button_click_emits_message() {
    let state = state_with_comparison(vec![0x2A, 0x00, 0x00, 0x00], vec![0x5A, 0x00, 0x00, 0x00]);
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.click("B")
        .expect("inspector should have a B toggle button");
    let messages: Vec<HexEditorMessage> = ui.into_messages().collect();
    assert!(
        messages
            .iter()
            .any(|m| matches!(m, HexEditorMessage::SetInspectorSource(InspectorSource::Comparison))),
        "clicking B should emit SetInspectorSource(Comparison), got {messages:?}"
    );
}

#[test]
fn test_inspector_tracks_cursor_position() {
    // Decoded values must follow the cursor, not the buffer start.
    let mut state = make_state(vec![0xAA, 0x2A, 0xBB, 0x00]);
    let config = default_config();
    state.selection = Selection::single(1);
    let mut ui = simulator(view(&state, &config));
    ui.find("42")
        .expect("u8 at cursor 1 (0x2A) should show 42, not the byte at offset 0");
}

#[test]
fn test_inspector_comparison_tracks_cursor_position() {
    let mut state = state_with_comparison(vec![0xAA, 0x2A, 0xBB, 0x00], vec![0xAA, 0x5A, 0xBB, 0x00]);
    let config = default_config();
    state.selection = Selection::single(1);
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorSource(InspectorSource::Comparison),
    );
    let mut ui = simulator(view(&state, &config));
    ui.find("90")
        .expect("comparison u8 at cursor 1 (0x5A) should show 90");
    ui.find("42").expect_err("must not decode from buffer start");
}

#[test]
fn test_inspector_shows_comparison_bytes_after_toggle() {
    let mut state = state_with_comparison(vec![0x2A, 0x00, 0x00, 0x00], vec![0x5A, 0x00, 0x00, 0x00]);
    let config = default_config();

    // Baseline (default): shows 42 for 0x2A.
    {
        let mut ui = simulator(view(&state, &config));
        ui.find("42").expect("baseline u8 should be 42");
    }

    // Switch to comparison source: shows 90 for 0x5A.
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorSource(InspectorSource::Comparison),
    );
    let mut ui = simulator(view(&state, &config));
    ui.find("90").expect("comparison u8 should be 90");
    ui.find("42").expect_err("baseline value must not be shown");
}

#[test]
fn test_inspector_toggle_hidden_without_comparison_file() {
    let state = make_state(vec![0x2A, 0x00, 0x00, 0x00]);
    let config = default_config();
    let mut ui = simulator(view(&state, &config));
    ui.find("A").expect_err("no A/B toggle without comparison file");
}

#[test]
fn test_inspector_edit_blocked_in_comparison_mode() {
    let mut state = state_with_comparison(vec![0x2A, 0x00, 0x00, 0x00], vec![0x5A, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorSource(InspectorSource::Comparison),
    );
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(0));
    assert!(
        state.inspector_edit.is_none(),
        "edit modal must not open for the read-only comparison file"
    );
}

#[test]
fn test_inspector_switching_source_closes_open_edit_modal() {
    let mut state = state_with_comparison(vec![0x2A, 0x00, 0x00, 0x00], vec![0x5A, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginInspectorEdit(0));
    assert!(state.inspector_edit.is_some());
    send(
        &mut state,
        &config,
        HexEditorMessage::SetInspectorSource(InspectorSource::Comparison),
    );
    assert!(
        state.inspector_edit.is_none(),
        "switching source must close the open edit modal"
    );
}

// ============================================================================
// Inspector — diff-view clicks drive the A/B source
// ============================================================================

#[test]
fn test_diff_click_b_side_switches_inspector_to_comparison() {
    let mut state = state_with_comparison(vec![0x2A, 0x00, 0x00, 0x00], vec![0x5A, 0x00, 0x00, 0x00]);
    let config = default_config();

    // Clicking the comparison (right) side must switch the inspector.
    send(
        &mut state,
        &config,
        HexEditorMessage::DiffAddrSelected { addr: 1, is_baseline: false },
    );
    assert_eq!(state.inspector_source, InspectorSource::Comparison);
    assert_eq!(state.selection.cursor, 1, "selection must follow the clicked address");

    // Clicking the baseline (left) side must switch back.
    send(
        &mut state,
        &config,
        HexEditorMessage::DiffAddrSelected { addr: 2, is_baseline: true },
    );
    assert_eq!(state.inspector_source, InspectorSource::Baseline);
}

#[test]
fn test_diff_drag_b_side_switches_inspector_to_comparison() {
    let mut state = state_with_comparison(vec![0x2A, 0x00, 0x00, 0x00], vec![0x5A, 0x00, 0x00, 0x00]);
    let config = default_config();

    // Drag-extend ending on the comparison side must switch the inspector.
    send(
        &mut state,
        &config,
        HexEditorMessage::DiffExtendTo { addr: 3, is_baseline: false },
    );
    assert_eq!(state.inspector_source, InspectorSource::Comparison);
    assert_eq!(state.selection.cursor, 3);

    // Drag ending on the baseline side must switch back.
    send(
        &mut state,
        &config,
        HexEditorMessage::DiffExtendTo { addr: 1, is_baseline: true },
    );
    assert_eq!(state.inspector_source, InspectorSource::Baseline);
}

#[test]
fn test_diff_click_b_side_updates_inspector_values_end_to_end() {
    // Full pipeline: B-side click message → update → inspector decodes
    // the comparison file's bytes.
    let mut state = state_with_comparison(vec![0x2A, 0x00, 0x00, 0x00], vec![0x5A, 0x00, 0x00, 0x00]);
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::DiffAddrSelected { addr: 0, is_baseline: false },
    );
    let mut ui = simulator(view(&state, &config));
    ui.find("90")
        .expect("inspector should decode comparison byte 0x5A after a B-side click");
    ui.find("42").expect_err("baseline value must not be shown after a B-side click");
}
