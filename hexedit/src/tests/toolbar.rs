use super::*;

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
