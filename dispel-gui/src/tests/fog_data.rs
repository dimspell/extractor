//! State-level integration tests for the fog data editor
//! (`ExtraInGame/fogdata.dat`).
//!
//! Fixture convention: `../fixtures/Dispel`, gracefully skipped when missing
//! (same as `map_editor.rs` / `update/map.rs`).

#[cfg(test)]
mod fog_data_editor_tests {
    use crate::app::App;
    use crate::editors::fog_data::{FogDataEditorState, FogDataMessage, handle};
    use crate::message::Message;
    use crate::message::system::SystemMessage;
    use crate::tests::app_with_tab;
    use crate::workspace::{EditorType, Workspace};
    use dispel_core::map::fogdata::{MAX_FACTOR, ROWS};
    use std::path::PathBuf;

    const TAB: usize = 1; // matches app_with_tab's tab id

    /// Fixture fade-table path, or `None` when fixtures are absent.
    fn fixture_fogdata_path() -> Option<PathBuf> {
        let p = PathBuf::from("../fixtures/Dispel/ExtraInGame/fogdata.dat");
        p.exists().then_some(p)
    }

    /// An App whose active tab is a FogDataEditor with the fixture loaded.
    fn app_with_fixture() -> Option<App> {
        let mut app = app_with_tab(EditorType::FogDataEditor);
        let path = fixture_fogdata_path()?;
        app.state
            .editors
            .fog_editors
            .insert(TAB, FogDataEditorState::load_from_path(&path));
        Some(app)
    }

    fn editor_of(app: &App) -> &FogDataEditorState {
        app.state.editors.fog_editors.get(&TAB).unwrap()
    }

    fn run(app: &mut App, msg: FogDataMessage) {
        let _ = handle(msg, app);
    }

    // ── Open + parse ──────────────────────────────────────────────────────

    #[test]
    fn test_open_fixture_parses_levels_and_spot_factors() {
        let Some(app) = app_with_fixture() else {
            eprintln!("Skipping: fixtures not found");
            return;
        };
        let editor = editor_of(&app);
        assert!(editor.error.is_none(), "fixture must parse cleanly");
        assert!(editor.fog.is_some(), "fade tables present");
        assert_eq!(editor.level_count(), ROWS, "123 light levels");
        assert_eq!(editor.current_row().len(), 512, "512 samples per level");
        // Known values from the shipped file.
        assert_eq!(
            editor.fog.as_ref().unwrap().factor(1, 0),
            2,
            "level 1 starts near-black"
        );
        assert_eq!(
            editor.fog.as_ref().unwrap().factor(65, 256),
            31,
            "level 65 mid-curve fully bright"
        );
        assert!(!editor.dirty, "freshly opened is clean");
    }

    #[test]
    fn test_from_path_routes_dat_fogdata_to_editor() {
        let p = PathBuf::from("/game/ExtraInGame/fogdata.dat");
        assert_eq!(EditorType::from_path(&p), EditorType::FogDataEditor);
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

    // ── Painting + dirty flag ─────────────────────────────────────────────

    #[test]
    fn test_paint_updates_state_dirty_and_tab_flag() {
        let mut app = app_with_fixture().expect("fixture available");
        let before = editor_of(&app).selected_factor();

        run(
            &mut app,
            FogDataMessage::FactorPainted(TAB, 100, MAX_FACTOR),
        );

        let editor = editor_of(&app);
        assert_eq!(editor.selected_pair, 100, "painting selects the pair");
        assert_eq!(
            editor.selected_factor(),
            Some(MAX_FACTOR),
            "painted value applied"
        );
        assert_ne!(editor.selected_factor(), before, "value actually changed");
        assert!(editor.dirty, "paint marks dirty");
        assert!(
            app.state.workspace.tabs[0].modified,
            "workspace tab flagged modified"
        );

        // Stroke end commits exactly one undo snapshot.
        run(&mut app, FogDataMessage::StrokeEnded(TAB));
        assert_eq!(editor_of(&app).undo_stack.len(), 1, "one stroke snapshot");
        assert!(editor_of(&app).can_undo());
    }

    #[test]
    fn test_paint_rejects_out_of_range_value() {
        let mut app = app_with_fixture().expect("fixture available");
        let before = editor_of(&app).selected_factor();

        run(&mut app, FogDataMessage::FactorPainted(TAB, 5, 32));
        run(&mut app, FogDataMessage::FactorPainted(TAB, 512, 1));

        let editor = editor_of(&app);
        assert_eq!(editor.selected_factor(), before, "no state change");
        assert!(!editor.dirty, "rejected paints stay clean");
        assert!(editor.undo_stack.is_empty(), "no undo entry created");
    }

    #[test]
    fn test_paint_stroke_without_change_creates_no_undo_entry() {
        let mut app = app_with_fixture().expect("fixture available");
        let value = editor_of(&app).selected_factor().unwrap();
        let pair = editor_of(&app).selected_pair;

        // Paint the exact same value — a no-op stroke.
        run(&mut app, FogDataMessage::FactorPainted(TAB, pair, value));
        run(&mut app, FogDataMessage::StrokeEnded(TAB));

        assert!(
            editor_of(&app).undo_stack.is_empty(),
            "unchanged curve must not push an undo snapshot"
        );
        assert!(!editor_of(&app).dirty);
    }

    // ── Inspector field ───────────────────────────────────────────────────

    #[test]
    fn test_value_input_validation_shows_inline_error() {
        let mut app = app_with_fixture().expect("fixture available");

        run(
            &mut app,
            FogDataMessage::ValueInputChanged(TAB, "99".into()),
        );
        assert!(
            editor_of(&app).input_error.is_some(),
            "99 out of range → inline error"
        );

        run(
            &mut app,
            FogDataMessage::ValueInputChanged(TAB, "abc".into()),
        );
        assert!(editor_of(&app).input_error.is_some(), "non-numeric → error");

        run(
            &mut app,
            FogDataMessage::ValueInputChanged(TAB, "17".into()),
        );
        assert!(
            editor_of(&app).input_error.is_none(),
            "valid input clears error"
        );
    }

    #[test]
    fn test_value_submit_commits_and_marks_dirty() {
        let mut app = app_with_fixture().expect("fixture available");
        let original = editor_of(&app).selected_factor().unwrap();
        let replacement = if original == 5 { 6 } else { 5 };

        run(
            &mut app,
            FogDataMessage::ValueInputChanged(TAB, replacement.to_string()),
        );
        run(&mut app, FogDataMessage::ValueSubmitted(TAB));

        let editor = editor_of(&app);
        assert_eq!(editor.selected_factor(), Some(replacement));
        assert!(editor.dirty, "committed edit marks dirty");
        assert_eq!(editor.undo_stack.len(), 1, "commit pushes one snapshot");

        // Submitting an invalid buffer must be rejected without state change.
        run(
            &mut app,
            FogDataMessage::ValueInputChanged(TAB, "40".into()),
        );
        run(&mut app, FogDataMessage::ValueSubmitted(TAB));
        let editor = editor_of(&app);
        assert_eq!(editor.selected_factor(), Some(replacement), "unchanged");
        assert!(editor.input_error.is_some());
        assert_eq!(editor.undo_stack.len(), 1, "no extra snapshot");
    }

    #[test]
    fn test_factor_committed_rejects_out_of_range() {
        let mut app = app_with_fixture().expect("fixture available");
        let before = editor_of(&app).selected_factor();

        run(&mut app, FogDataMessage::FactorCommitted(TAB, 3, 200));

        let editor = editor_of(&app);
        assert_eq!(editor.selected_factor(), before, "state untouched");
        assert!(!editor.dirty, "rejection stays clean");
        assert!(editor.undo_stack.is_empty());
    }

    #[test]
    fn test_level_selected_updates_selection_and_input_buffer() {
        let mut app = app_with_fixture().expect("fixture available");
        run(&mut app, FogDataMessage::LevelSelected(TAB, 65));
        assert_eq!(editor_of(&app).selected_level, 65);
        assert!(
            !editor_of(&app).value_input.is_empty(),
            "input buffer refreshed from new level"
        );

        // Out-of-range levels are ignored.
        run(&mut app, FogDataMessage::LevelSelected(TAB, 124));
        run(&mut app, FogDataMessage::LevelSelected(TAB, 0));
        assert_eq!(editor_of(&app).selected_level, 65, "clamped to valid range");
    }

    // ── Undo / redo round-trip ────────────────────────────────────────────

    #[test]
    fn test_undo_redo_round_trip_restores_prior_curve() {
        let mut app = app_with_fixture().expect("fixture available");
        let original = editor_of(&app).fog.as_ref().unwrap().clone();

        // Stroke A: paint pair 10.
        run(&mut app, FogDataMessage::FactorPainted(TAB, 10, 1));
        run(&mut app, FogDataMessage::StrokeEnded(TAB));
        let after_a = editor_of(&app).fog.as_ref().unwrap().clone();

        // Stroke B: paint pair 20 with a value guaranteed to differ from
        // whatever the fixture holds there.
        let cur_b = editor_of(&app).fog.as_ref().unwrap().factor(1, 20);
        let new_b = (cur_b + 7) % MAX_FACTOR;
        run(&mut app, FogDataMessage::FactorPainted(TAB, 20, new_b));
        run(&mut app, FogDataMessage::StrokeEnded(TAB));
        assert_ne!(editor_of(&app).fog.as_ref().unwrap(), &after_a);

        // Undo B → back to after-A.
        run(&mut app, FogDataMessage::Undo(TAB));
        assert_eq!(
            editor_of(&app).fog.as_ref().unwrap(),
            &after_a,
            "first undo restores prior stroke"
        );

        // Undo A → back to the pristine table.
        run(&mut app, FogDataMessage::Undo(TAB));
        assert_eq!(
            editor_of(&app).fog.as_ref().unwrap(),
            &original,
            "second undo restores the original curve"
        );
        assert!(!editor_of(&app).can_undo());

        // Nothing left to undo reports via status bar.
        run(&mut app, FogDataMessage::Undo(TAB));
        assert_eq!(app.state.status_msg, "Nothing to undo");

        // Redo reapplies both strokes in order.
        run(&mut app, FogDataMessage::Redo(TAB));
        assert_eq!(editor_of(&app).fog.as_ref().unwrap(), &after_a);
        run(&mut app, FogDataMessage::Redo(TAB));
        assert_ne!(
            editor_of(&app).fog.as_ref().unwrap(),
            &original,
            "redo restores edits"
        );
        assert!(editor_of(&app).dirty);
    }

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

    // ── Save round-trip ───────────────────────────────────────────────────

    #[test]
    fn test_save_writes_bytes_identical_to_original_when_unchanged() {
        let Some(path) = fixture_fogdata_path() else {
            eprintln!("Skipping: fixtures not found");
            return;
        };
        let original_bytes = std::fs::read(&path).expect("fixture readable");

        // Copy into a temp dir and open the copy.
        let dir = std::env::temp_dir().join(format!("dispel-fog-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir created");
        let copy = dir.join("fogdata.dat");
        std::fs::write(&copy, &original_bytes).expect("copy written");

        let state = FogDataEditorState::load_from_path(&copy);
        assert!(state.error.is_none());
        crate::editors::fog_data::save_to_disk(&copy, state.fog.as_ref().unwrap())
            .expect("save succeeds");

        let saved = std::fs::read(&copy).expect("saved file readable");
        assert_eq!(
            saved, original_bytes,
            "unmodified table round-trips byte-identical"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_message_dispatches_and_complete_marks_clean() {
        let mut app = app_with_fixture().expect("fixture available");
        run(&mut app, FogDataMessage::FactorPainted(TAB, 7, 9));
        run(&mut app, FogDataMessage::StrokeEnded(TAB));
        assert!(editor_of(&app).dirty);

        // Save returns an async task; simulate its completion.
        let save_gen = editor_of(&app).save_generation;
        run(
            &mut app,
            FogDataMessage::SaveComplete(TAB, save_gen, Ok(("Saved → somewhere".into(), None))),
        );
        assert!(!editor_of(&app).dirty, "successful save clears dirty");
        assert!(!app.state.workspace.tabs[0].modified);
        assert_eq!(app.state.status_msg, "Saved → somewhere");

        // A failed save keeps the editor dirty.
        run(&mut app, FogDataMessage::FactorPainted(TAB, 8, 10));
        run(&mut app, FogDataMessage::StrokeEnded(TAB));
        let save_gen = editor_of(&app).save_generation;
        run(
            &mut app,
            FogDataMessage::SaveComplete(TAB, save_gen, Err("disk on fire".into())),
        );
        assert!(editor_of(&app).dirty, "failed save stays dirty");
        assert!(app.state.workspace.tabs[0].modified);
        assert!(
            app.state.status_msg.contains("disk on fire"),
            "failure reported via status bar"
        );
    }

    #[test]
    fn test_save_completion_with_stale_generation_keeps_tab_dirty() {
        let mut app = app_with_fixture().expect("fixture available");
        run(&mut app, FogDataMessage::FactorPainted(TAB, 7, 9));
        run(&mut app, FogDataMessage::StrokeEnded(TAB));
        assert!(editor_of(&app).dirty);

        // Save dispatch bumps the generation; simulate the completion
        // carrying the pre-dispatch value (an edit landed while saving).
        let stale_gen = editor_of(&app).save_generation - 1;
        run(
            &mut app,
            FogDataMessage::SaveComplete(TAB, stale_gen, Ok(("Saved → x".into(), None))),
        );

        assert!(
            editor_of(&app).dirty,
            "stale save completion must not clear dirty"
        );
        assert!(app.state.workspace.tabs[0].modified, "tab stays modified");

        // A current-generation completion still marks clean.
        let save_gen = editor_of(&app).save_generation;
        run(
            &mut app,
            FogDataMessage::SaveComplete(TAB, save_gen, Ok(("Saved → x".into(), None))),
        );
        assert!(!editor_of(&app).dirty);
    }

    #[test]
    fn test_save_complete_recording_failure_reports_both_facts_and_marks_clean() {
        let mut app = app_with_fixture().expect("fixture available");
        run(&mut app, FogDataMessage::FactorPainted(TAB, 7, 9));
        run(&mut app, FogDataMessage::StrokeEnded(TAB));

        let save_gen = editor_of(&app).save_generation;
        run(
            &mut app,
            FogDataMessage::SaveComplete(
                TAB,
                save_gen,
                Ok(("Saved → x".into(), Some("changelog locked".into()))),
            ),
        );

        let status = &app.state.status_msg;
        assert!(status.contains("Saved"), "file-saved success is reported");
        assert!(
            status.contains("Mod recording failed") && status.contains("changelog locked"),
            "recording failure reported separately: {status}"
        );
        assert!(
            !editor_of(&app).dirty,
            "file on disk is current → editor marked clean"
        );
        assert!(!app.state.workspace.tabs[0].modified);
    }

    #[test]
    fn test_system_save_respects_capability() {
        let mut app = app_with_tab(EditorType::FogDataEditor);
        let _ = app.update(Message::System(SystemMessage::Save));
        assert_ne!(
            app.state.status_msg, "This editor does not support saving",
            "Ctrl+S must dispatch to the fog data editor"
        );
    }

    // ── Revert ────────────────────────────────────────────────────────────

    #[test]
    fn test_revert_discards_changes_and_restores_disk_state() {
        let Some(path) = fixture_fogdata_path() else {
            eprintln!("Skipping: fixtures not found");
            return;
        };
        let original_bytes = std::fs::read(&path).expect("fixture readable");
        let dir = std::env::temp_dir().join(format!("dispel-fog-revert-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir created");
        let copy = dir.join("fogdata.dat");
        std::fs::write(&copy, &original_bytes).expect("copy written");

        let mut app = app_with_tab(EditorType::FogDataEditor);
        app.state
            .editors
            .fog_editors
            .insert(TAB, FogDataEditorState::load_from_path(&copy));

        // Dirty the copy.
        run(&mut app, FogDataMessage::FactorPainted(TAB, 42, 30));
        run(&mut app, FogDataMessage::StrokeEnded(TAB));
        assert!(editor_of(&app).dirty);

        // Dirty revert asks for confirmation first.
        run(&mut app, FogDataMessage::Revert(TAB));
        assert!(editor_of(&app).confirm_revert, "confirm dialog opens");
        assert!(editor_of(&app).dirty, "nothing discarded yet");

        run(&mut app, FogDataMessage::RevertConfirmed(TAB));
        let editor = editor_of(&app);
        assert!(!editor.dirty, "revert restores clean state");
        assert!(!editor.confirm_revert);
        assert!(editor.undo_stack.is_empty(), "history cleared");
        assert_eq!(
            std::fs::read(&copy).unwrap(),
            original_bytes,
            "disk file untouched by revert"
        );

        // Clean revert reloads immediately without any dialog.
        run(&mut app, FogDataMessage::Revert(TAB));
        assert!(!editor_of(&app).confirm_revert);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_revert_cancelled_keeps_edits() {
        let mut app = app_with_fixture().expect("fixture available");
        run(&mut app, FogDataMessage::FactorPainted(TAB, 11, 3));
        run(&mut app, FogDataMessage::StrokeEnded(TAB));

        run(&mut app, FogDataMessage::Revert(TAB));
        assert!(editor_of(&app).confirm_revert);

        run(&mut app, FogDataMessage::RevertCancelled(TAB));
        let editor = editor_of(&app);
        assert!(!editor.confirm_revert);
        assert!(editor.dirty, "edits preserved after cancel");
        assert_eq!(editor.selected_factor(), Some(3), "painted value kept");
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
