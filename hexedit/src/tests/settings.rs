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
    assert!(
        !config.can_save_now(&state),
        "should not be savable when clean"
    );
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

// ============================================================================
// Settings modal — new functionality (unit + smoke tests)
// ============================================================================

#[test]
fn test_set_addr_format_hex() {
    let mut state = make_state((0..64).collect());
    state.show_decimal = true; // start in decimal mode
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SetAddrFormat(false));
    assert!(!state.show_decimal, "should switch to hex address format");
}

#[test]
fn test_set_addr_format_decimal() {
    let mut state = make_state((0..64).collect());
    state.show_decimal = false; // start in hex mode
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SetAddrFormat(true));
    assert!(
        state.show_decimal,
        "should switch to decimal address format"
    );
}

#[test]
fn test_reset_settings_restores_defaults() {
    let mut state = make_state((0..64).collect());
    // Set non-default values
    state.color_scheme = ColorScheme::Rainbow;
    state.dim_nulls = false;
    state.show_decimal = true;
    state.bytes_per_row = 8;

    let config = default_config();
    send(&mut state, &config, HexEditorMessage::ResetSettings);

    assert_eq!(
        state.color_scheme,
        ColorScheme::Monochrome,
        "color scheme should reset to Monochrome"
    );
    assert!(state.dim_nulls, "dim_nulls should reset to true");
    assert!(!state.show_decimal, "show_decimal should reset to false");
    assert_eq!(
        state.bytes_per_row,
        crate::state::DEFAULT_BYTES_PER_ROW,
        "bytes_per_row should reset to default"
    );
    assert_eq!(
        state.status_msg, "Settings reset to defaults",
        "should set status message"
    );
}

#[test]
fn test_reset_settings_from_defaults_is_idempotent() {
    let mut state = make_state((0..64).collect());
    // Already at defaults — reset should be a no-op.
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::ResetSettings);

    assert_eq!(state.color_scheme, ColorScheme::Monochrome);
    assert!(state.dim_nulls);
    assert!(!state.show_decimal);
    assert_eq!(state.bytes_per_row, crate::state::DEFAULT_BYTES_PER_ROW);
}

#[test]
fn test_settings_modal_view_does_not_panic() {
    // Smoke test: the settings modal view function should not panic.
    let state = make_state((0..64).collect());
    let _element = crate::ui::view::settings_modal::view(&state);
}

#[test]
fn test_settings_modal_dim_nulls_true_does_not_panic() {
    // Smoke test: render with dim_nulls=true, color scheme = Nybble.
    let mut state = make_state((0..64).collect());
    state.dim_nulls = true;
    state.color_scheme = ColorScheme::Nybble;
    let _element = crate::ui::view::settings_modal::view(&state);
}
