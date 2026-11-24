//! Parity checks for the two hexedit embedders' config assembly.
//!
//! The GUI (`build_hex_config`) and the standalone binary
//! (`hexedit::standalone_config`) now go through the same
//! `HexEditorConfig::new()` builder. These tests pin the fields that must stay
//! identical in both hosts so neither can silently reintroduce a divergent
//! assembly path.

use crate::editors::hex_wrapper::build_hex_config;
use hexedit::{HexEditorConfig, HexEditorState};
use std::path::Path;
use std::sync::Arc;

fn state() -> HexEditorState {
    HexEditorState::load_from_path(Path::new("hex-config-parity.bin"))
}

#[test]
fn test_gui_hex_config_uses_shared_builder_defaults() {
    let state = state();
    let config = build_hex_config(&None, &None, &state);

    assert_eq!(config.pane_gap, 4, "both hosts must use the same pane gap");
    assert!(
        config.custom_encodings.is_empty(),
        "the GUI host does not persist custom encodings"
    );
    assert!(
        config.on_write_mode_changed.is_none(),
        "the GUI host has no write-mode persistence port"
    );
    assert!(
        config.extra_entries.is_empty(),
        "the GUI host has no custom inspector entries"
    );
    // The builder's fallback label and prerequisites still work.
    assert_eq!(config.save_label(), "Save into recording");
    assert!(!config.can_save);
}

#[test]
fn test_gui_and_standalone_config_parity_on_shared_fields() {
    let state = state();
    let gui = build_hex_config(&None, &None, &state);
    let standalone = hexedit::standalone_config(
        false,
        Vec::new(),
        Vec::new(),
        Arc::new(|_| iced::Task::none()),
        None,
    );

    assert_eq!(gui.pane_gap, standalone.pane_gap);
    assert_eq!(gui.custom_encodings, standalone.custom_encodings);
    assert_eq!(gui.extra_entries.len(), standalone.extra_entries.len());
    assert_eq!(
        gui.on_write_mode_changed.is_some(),
        standalone.on_write_mode_changed.is_some()
    );
    // Both are built from HexEditorConfig::new(), so their defaults agree.
    assert_eq!(
        HexEditorConfig::new().pane_gap,
        HexEditorConfig::default().pane_gap
    );
}
