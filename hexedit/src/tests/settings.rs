use super::*;

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
