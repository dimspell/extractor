use super::*;

// ============================================================================
// Settings & configuration
// ============================================================================

#[test]
fn test_config_builder_sets_every_field() {
    let encodings = vec![crate::domain::write_mode::EncodingEntry {
        label: "ISO-8859-2".into(),
        encoding_name: "windows-1250".into(),
    }];
    let config = HexEditorConfig::new()
        .pane_gap(7)
        .on_save(std::sync::Arc::new(|_| iced::Task::none()))
        .with_save_label("Store")
        .can_save(true)
        .save_hint("hint")
        .custom_encodings(encodings.clone())
        .on_write_mode_changed(std::sync::Arc::new(|_| {}));

    assert_eq!(config.pane_gap, 7);
    assert!(config.has_save());
    assert_eq!(config.save_label(), "Store");
    assert!(config.can_save);
    assert_eq!(config.save_hint, "hint");
    assert_eq!(config.custom_encodings, encodings);
    assert!(config.on_write_mode_changed.is_some());
    assert!(config.extra_entries.is_empty());
}

#[test]
fn test_standalone_config_matches_builder_defaults() {
    // The standalone host assembles through `standalone_config`; this is the
    // same chain written out explicitly. If either path drifts (a field is
    // dropped or a default changes), this parity test fails.
    let encodings = vec![crate::domain::write_mode::EncodingEntry {
        label: "EUC-KR".into(),
        encoding_name: "euc-kr".into(),
    }];
    let save_port: crate::SavePort = std::sync::Arc::new(|_| iced::Task::none());
    let mode_port: crate::WriteModeCallback = std::sync::Arc::new(|_| {});

    let through_helper = crate::standalone_config(
        true,
        encodings.clone(),
        Vec::new(),
        save_port.clone(),
        Some(mode_port.clone()),
    );
    let through_builder = HexEditorConfig::new()
        .pane_gap(4)
        .on_save(save_port)
        .with_save_label("Save")
        .can_save(true)
        .save_hint("")
        .custom_encodings(encodings.clone())
        .on_write_mode_changed(mode_port);

    assert_eq!(through_helper.pane_gap, through_builder.pane_gap);
    assert_eq!(through_helper.save_label(), through_builder.save_label());
    assert_eq!(through_helper.can_save, through_builder.can_save);
    assert_eq!(through_helper.save_hint, through_builder.save_hint);
    assert_eq!(
        through_helper.custom_encodings,
        through_builder.custom_encodings
    );
    assert_eq!(
        through_helper.extra_entries.len(),
        through_builder.extra_entries.len()
    );
    assert!(through_helper.has_save() && through_builder.has_save());
    assert!(
        through_helper.on_write_mode_changed.is_some()
            && through_builder.on_write_mode_changed.is_some()
    );
}

#[test]
fn test_standalone_config_dirty_controls_save_hint() {
    let port: crate::SavePort = std::sync::Arc::new(|_| iced::Task::none());
    let dirty = crate::standalone_config(true, vec![], vec![], port.clone(), None);
    let clean = crate::standalone_config(false, vec![], vec![], port, None);
    assert_eq!(dirty.save_hint, "");
    assert_eq!(clean.save_hint, "  ·  no edits");
}

#[test]
fn test_write_mode_callback_receives_domain_value_only() {
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let seen_by_host = seen.clone();
    let config = HexEditorConfig::new().on_write_mode_changed(std::sync::Arc::new(move |mode| {
        // The callback signature carries only WriteMode — an internal
        // HexEditorMessage could not be received here.
        seen_by_host.lock().unwrap().push(mode);
    }));
    let mut state = make_state((0..64).collect());

    send(
        &mut state,
        &config,
        HexEditorMessage::SetWriteMode(WriteMode::Ascii),
    );
    assert_eq!(state.write_mode, WriteMode::Ascii);
    assert_eq!(*seen.lock().unwrap(), vec![WriteMode::Ascii]);

    // An unchanged selection must not fire the host callback again.
    send(
        &mut state,
        &config,
        HexEditorMessage::SetWriteMode(WriteMode::Ascii),
    );
    assert_eq!(*seen.lock().unwrap(), vec![WriteMode::Ascii]);
}

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
    state.bpr_input = "8".to_string();

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
        state.bpr_input,
        crate::state::DEFAULT_BYTES_PER_ROW.to_string(),
        "bpr_input should reset alongside bytes_per_row"
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
    assert_eq!(
        state.bpr_input,
        crate::state::DEFAULT_BYTES_PER_ROW.to_string()
    );
}

#[test]
fn test_reset_settings_matches_fresh_state_defaults() {
    let mut state = make_state((0..64).collect());
    state.color_scheme = ColorScheme::Rainbow;
    state.dim_nulls = false;
    state.show_decimal = true;
    state.bytes_per_row = 8;
    state.bpr_input = "8".to_string();
    state.show_entropy_band = false;
    state.theme_variant = crate::ui::theme::ThemeVariant::Light;
    state.theme = crate::ui::theme::ThemeVariant::Light.theme();

    let config = default_config();
    send(&mut state, &config, HexEditorMessage::ResetSettings);

    let fresh = HexEditorState::from_bytes("test.bin", vec![0; 64], None, None);
    assert_eq!(state.color_scheme, fresh.color_scheme);
    assert_eq!(state.dim_nulls, fresh.dim_nulls);
    assert_eq!(state.show_decimal, fresh.show_decimal);
    assert_eq!(state.bytes_per_row, fresh.bytes_per_row);
    assert_eq!(state.bpr_input, fresh.bpr_input);
    assert_eq!(state.show_entropy_band, fresh.show_entropy_band);
    assert_eq!(state.theme_variant, fresh.theme_variant);
    assert_eq!(state.theme, fresh.theme);
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
