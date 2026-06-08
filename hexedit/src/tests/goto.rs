use super::*;

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
