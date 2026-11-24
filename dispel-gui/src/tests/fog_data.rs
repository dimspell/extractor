//! State-level integration tests for the fog data editor
//! (`ExtraInGame/fogdata.dat`).
//!
//! These tests are fixture-free: they build the editor state in memory or
//! point it at a deliberately missing path, so they run without the
//! git-ignored `../fixtures/Dispel` game data.

#[cfg(test)]
mod fog_data_editor_tests {
    use crate::app::App;
    use crate::editors::fog_data::FogDataEditorState;
    use crate::message::Message;
    use crate::message::system::SystemMessage;
    use crate::tests::app_with_tab;
    use crate::workspace::{EditorType, Workspace};

    const TAB: usize = 1; // matches app_with_tab's tab id

    // ── Open + parse ──────────────────────────────────────────────────────

    #[test]
    fn test_from_path_routes_dat_fogdata_to_editor() {
        let p = std::path::Path::new("/game/ExtraInGame/fogdata.dat");
        assert_eq!(EditorType::from_path(p), EditorType::FogDataEditor);
    }

    #[test]
    fn test_open_missing_file_surfaces_error_without_panic() {
        let state = FogDataEditorState::load_from_path(std::path::Path::new(
            "/definitely/not/a/real/fogdata.dat",
        ));
        assert!(state.error.is_some(), "missing file → error surface");
        assert!(state.fog.is_none());
        // The view must render the error surface without panicking.
        let mut app = app_with_tab(EditorType::FogDataEditor);
        app.state.editors.fog_editors.insert(TAB, state);
        let _ = app.view();
    }

    // ── Undo / redo ───────────────────────────────────────────────────────

    #[test]
    fn test_registry_undo_redo_returns_none_with_empty_history() {
        use std::collections::HashMap;

        let mut registry = crate::editor_registry::EditorRegistry::default();
        let lookups: HashMap<String, Vec<(String, String)>> = HashMap::new();

        assert!(
            registry
                .undo_active(EditorType::FogDataEditor, TAB, &lookups)
                .is_none(),
            "no tab state → nothing to undo"
        );
        assert!(
            registry
                .redo_active(EditorType::FogDataEditor, TAB, &lookups)
                .is_none()
        );

        // With a tab present but empty stacks it must still report None.
        registry
            .fog_editors
            .insert(TAB, FogDataEditorState::default());
        assert!(
            registry
                .undo_active(EditorType::FogDataEditor, TAB, &lookups)
                .is_none(),
            "empty undo stack → None (so the status bar says 'Nothing to undo')"
        );
    }

    // ── System messages ───────────────────────────────────────────────────

    #[test]
    fn test_system_save_respects_capability() {
        let mut app = app_with_tab(EditorType::FogDataEditor);
        let _ = app.update(Message::System(SystemMessage::Save));
        assert_ne!(
            app.state.status_msg, "This editor does not support saving",
            "Ctrl+S must dispatch to the fog data editor"
        );
    }

    // ── Workspace lifecycle ───────────────────────────────────────────────

    #[test]
    fn test_remove_tab_clears_fog_editor_state() {
        let mut registry = crate::editor_registry::EditorRegistry::default();
        registry
            .fog_editors
            .insert(TAB, FogDataEditorState::default());

        registry.remove_tab(TAB);
        assert!(!registry.fog_editors.contains_key(&TAB), "tab cleanup");

        registry
            .fog_editors
            .insert(2, FogDataEditorState::default());
        registry.close_all_tabs();
        assert!(registry.fog_editors.is_empty(), "close_all_tabs cleanup");

        registry
            .fog_editors
            .insert(3, FogDataEditorState::default());
        registry.clear_all();
        assert!(registry.fog_editors.is_empty(), "clear_all cleanup");
    }

    #[test]
    fn test_workspace_reset_clears_open_fog_tabs() {
        let mut app = App::test_new(Workspace::new());
        app.state
            .editors
            .fog_editors
            .insert(TAB, Default::default());
        app.state.editors.clear_all();
        assert!(app.state.editors.fog_editors.is_empty());
    }
}
