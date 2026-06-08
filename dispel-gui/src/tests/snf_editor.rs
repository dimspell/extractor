#[cfg(test)]
mod snf_editor_tests {
    use crate::app::App;
    use crate::editors::snf_editor::{self, ExportStatus, SnfEditorMessage, SnfEditorState};
    use crate::workspace::{EditorType, Workspace, WorkspaceTab};
    use std::path::PathBuf;

    /// Create an App with a single SnfEditor tab with default dummy state.
    /// The editor has no audio file loaded (snf = None, playback = None).
    fn app_with_snf_editor() -> App {
        let mut app = App::test_new(Workspace::new());
        let tab_id = 1;

        app.state.workspace.tabs.push(WorkspaceTab {
            id: tab_id,
            label: "test.snf".into(),
            path: None,
            editor_type: EditorType::SnfEditor,
            modified: false,
            pinned: false,
        });
        app.state.workspace.active_tab = Some(0);

        app.state.editors.snf_editors.insert(
            tab_id,
            SnfEditorState {
                path: PathBuf::from("test.snf"),
                name: "test.snf".into(),
                snf: None,
                waveform: Vec::new(),
                error: None,
                playback: None,
                is_looping: false,
                volume: 0.5,
                export_status: ExportStatus::Idle,
            },
        );

        app
    }

    // ── ToggleLoop ─────────────────────────────────────────────────────────

    #[test]
    fn test_snf_editor_toggle_loop() {
        let mut app = app_with_snf_editor();
        let tab_id = 1;

        // Initially false
        assert!(!app.state.editors.snf_editors[&tab_id].is_looping);

        // Toggle on
        let task = snf_editor::handle(SnfEditorMessage::ToggleLoop, &mut app);
        assert_eq!(task.units(), 0);
        assert!(app.state.editors.snf_editors[&tab_id].is_looping);

        // Toggle off
        let task = snf_editor::handle(SnfEditorMessage::ToggleLoop, &mut app);
        assert_eq!(task.units(), 0);
        assert!(!app.state.editors.snf_editors[&tab_id].is_looping);
    }

    // ── SetVolume ──────────────────────────────────────────────────────────

    #[test]
    fn test_snf_editor_set_volume() {
        let mut app = app_with_snf_editor();
        let tab_id = 1;

        // Initial default is 0.5
        assert_eq!(app.state.editors.snf_editors[&tab_id].volume, 0.5);

        // Set to 0.5
        let task = snf_editor::handle(SnfEditorMessage::SetVolume(0.5), &mut app);
        assert_eq!(task.units(), 0);
        assert_eq!(app.state.editors.snf_editors[&tab_id].volume, 0.5);

        // Set to 0.0
        let task = snf_editor::handle(SnfEditorMessage::SetVolume(0.0), &mut app);
        assert_eq!(task.units(), 0);
        assert_eq!(app.state.editors.snf_editors[&tab_id].volume, 0.0);
    }

    #[test]
    fn test_snf_editor_set_volume_clamps() {
        let mut app = app_with_snf_editor();
        let tab_id = 1;

        // Above 1.0 clamps to 1.0
        let task = snf_editor::handle(SnfEditorMessage::SetVolume(1.5), &mut app);
        assert_eq!(task.units(), 0);
        assert_eq!(app.state.editors.snf_editors[&tab_id].volume, 1.0);

        // Below 0.0 clamps to 0.0
        let task = snf_editor::handle(SnfEditorMessage::SetVolume(-0.5), &mut app);
        assert_eq!(task.units(), 0);
        assert_eq!(app.state.editors.snf_editors[&tab_id].volume, 0.0);
    }

    // ── Stop ───────────────────────────────────────────────────────────────

    #[test]
    fn test_snf_editor_stop_no_playback_noop() {
        let mut app = app_with_snf_editor();
        let tab_id = 1;

        // playback is already None
        assert!(app.state.editors.snf_editors[&tab_id].playback.is_none());

        let task = snf_editor::handle(SnfEditorMessage::Stop, &mut app);
        assert_eq!(task.units(), 0);
        assert!(app.state.editors.snf_editors[&tab_id].playback.is_none());
    }

    // ── Pause ──────────────────────────────────────────────────────────────

    #[test]
    fn test_snf_editor_pause_no_playback_noop() {
        let mut app = app_with_snf_editor();
        let tab_id = 1;

        // Pause when no active playback — should be a no-op
        let task = snf_editor::handle(SnfEditorMessage::Pause, &mut app);
        assert_eq!(task.units(), 0);
        assert!(app.state.editors.snf_editors[&tab_id].playback.is_none());
    }

    // ── Tick ───────────────────────────────────────────────────────────────

    #[test]
    fn test_snf_editor_tick_no_playback_noop() {
        let mut app = app_with_snf_editor();
        let tab_id = 1;

        let original_name = app.state.editors.snf_editors[&tab_id].name.clone();

        let task = snf_editor::handle(SnfEditorMessage::Tick, &mut app);
        assert_eq!(task.units(), 0);
        assert!(app.state.editors.snf_editors[&tab_id].playback.is_none());
        assert_eq!(app.state.editors.snf_editors[&tab_id].name, original_name);
    }

    // ── No active tab ──────────────────────────────────────────────────────

    #[test]
    fn test_snf_editor_no_active_tab_returns_none() {
        let mut app = App::test_new(Workspace::new());
        // No tabs at all — active_tab is None

        let task = snf_editor::handle(SnfEditorMessage::ToggleLoop, &mut app);
        assert_eq!(task.units(), 0);
    }

    #[test]
    fn test_snf_editor_unknown_tab_id_returns_none() {
        let mut app = App::test_new(Workspace::new());
        // Tab exists but snf_editors has no entry for it
        app.state.workspace.tabs.push(WorkspaceTab {
            id: 42,
            label: "mystery.snf".into(),
            path: None,
            editor_type: EditorType::SnfEditor,
            modified: false,
            pinned: false,
        });
        app.state.workspace.active_tab = Some(0);

        let task = snf_editor::handle(SnfEditorMessage::Stop, &mut app);
        assert_eq!(task.units(), 0);
    }

    // ── ExportStatus transitions ───────────────────────────────────────────

    #[test]
    fn test_snf_editor_export_status_defaults_to_idle() {
        let app = app_with_snf_editor();
        let tab_id = 1;

        match app.state.editors.snf_editors[&tab_id].export_status {
            ExportStatus::Idle => {} // expected
            _ => panic!("export_status should be Idle initially"),
        }
    }
}
