#[cfg(test)]
mod sprite_viewer_tests {
    use crate::app::App;
    use crate::editors::sprite_browser::{
        SpriteFrame, SpriteViewerMessage, SpriteViewerState,
    };
    use crate::workspace::{EditorType, Workspace, WorkspaceTab};
    use std::path::PathBuf;

    fn dummy_frame(frame_idx: usize) -> SpriteFrame {
        SpriteFrame {
            sequence_idx: 0,
            frame_idx,
            image: iced::widget::image::Handle::from_bytes(vec![]),
            png_bytes: vec![],
        }
    }

    /// App with a SpriteViewer tab and empty frames (state-only tests).
    fn app_with_sprite_viewer() -> App {
        let mut app = App::test_new(Workspace::new());
        let tab_id = 0;
        app.state.workspace.tabs.push(WorkspaceTab {
            id: tab_id,
            label: "test.spr".into(),
            path: None,
            editor_type: EditorType::SpriteViewer,
            modified: false,
            pinned: false,
        });
        app.state.workspace.active_tab = Some(0);

        app.state.editors.sprite_viewers.insert(
            tab_id,
            SpriteViewerState {
                sequence_count: 3,
                frame_counts: vec![10, 15, 8],
                selected_sequence: 0,
                selected_frame: 0,
                is_playing: false,
                is_looping: true,
                speed_100x: 100,
                ms_accumulated: 0.0,
                fps: 10.0,
                frames: vec![],
                export_dialog: None,
                path: PathBuf::new(),
                name: "test.spr".into(),
                error: None,
            },
        );
        app
    }

    /// App with a SpriteViewer tab populated with 10 dummy frames.
    fn app_with_sprite_viewer_frames() -> App {
        let mut app = App::test_new(Workspace::new());
        let tab_id = 0;
        app.state.workspace.tabs.push(WorkspaceTab {
            id: tab_id,
            label: "test.spr".into(),
            path: None,
            editor_type: EditorType::SpriteViewer,
            modified: false,
            pinned: false,
        });
        app.state.workspace.active_tab = Some(0);

        app.state.editors.sprite_viewers.insert(
            tab_id,
            SpriteViewerState {
                sequence_count: 3,
                frame_counts: vec![10, 15, 8],
                selected_sequence: 0,
                selected_frame: 0,
                is_playing: false,
                is_looping: true,
                speed_100x: 100,
                ms_accumulated: 0.0,
                fps: 10.0,
                frames: (0..10).map(dummy_frame).collect(),
                export_dialog: None,
                path: PathBuf::new(),
                name: "test.spr".into(),
                error: None,
            },
        );
        app
    }

    // ── Navigation ───────────────────────────────────────────────────────────

    #[test]
    fn test_sprite_viewer_select_sequence() {
        let mut app = app_with_sprite_viewer();
        let task = crate::editors::sprite_browser::handle(
            SpriteViewerMessage::SelectSequence(1),
            &mut app,
        );
        assert_eq!(task.units(), 0);
        assert_eq!(
            app.state.editors.sprite_viewers[&0].selected_sequence,
            1
        );
        // Frame resets to 0 when sequence changes
        assert_eq!(
            app.state.editors.sprite_viewers[&0].selected_frame,
            0
        );
    }

    #[test]
    fn test_sprite_viewer_select_frame() {
        let mut app = app_with_sprite_viewer_frames();
        let task = crate::editors::sprite_browser::handle(
            SpriteViewerMessage::SelectFrame(5),
            &mut app,
        );
        assert_eq!(task.units(), 0);
        assert_eq!(
            app.state.editors.sprite_viewers[&0].selected_frame,
            5
        );
    }

    // ── Playback ─────────────────────────────────────────────────────────────

    #[test]
    fn test_sprite_viewer_play_pause_toggles() {
        let mut app = app_with_sprite_viewer();
        assert!(!app.state.editors.sprite_viewers[&0].is_playing);

        let task = crate::editors::sprite_browser::handle(
            SpriteViewerMessage::Play,
            &mut app,
        );
        assert_eq!(task.units(), 0);
        assert!(app.state.editors.sprite_viewers[&0].is_playing);

        let task = crate::editors::sprite_browser::handle(
            SpriteViewerMessage::Pause,
            &mut app,
        );
        assert_eq!(task.units(), 0);
        assert!(!app.state.editors.sprite_viewers[&0].is_playing);
    }

    #[test]
    fn test_sprite_viewer_tick_advances_frame() {
        let mut app = app_with_sprite_viewer_frames();
        // Set is_playing and ms_accumulated just shy of a frame boundary.
        // At fps=10.0, each frame is 100ms. TICK_MS=16.0, so 100.0 - 16.0 = 84.0
        // means a single Tick pushes past the boundary.
        {
            let viewer = app.state.editors.sprite_viewers.get_mut(&0).unwrap();
            viewer.is_playing = true;
            viewer.ms_accumulated = 84.0;
        }

        let task = crate::editors::sprite_browser::handle(
            SpriteViewerMessage::Tick,
            &mut app,
        );
        assert_eq!(task.units(), 0);

        let viewer = app.state.editors.sprite_viewers.get(&0).unwrap();
        assert_eq!(
            viewer.selected_frame, 1,
            "Tick advanced past 100ms frame boundary at 10fps"
        );
        assert!(
            viewer.ms_accumulated < 0.001,
            "ms_accumulated reset after frame advance"
        );
    }

    #[test]
    fn test_sprite_viewer_toggle_loop() {
        let mut app = app_with_sprite_viewer();
        assert!(app.state.editors.sprite_viewers[&0].is_looping);

        let task = crate::editors::sprite_browser::handle(
            SpriteViewerMessage::ToggleLoop,
            &mut app,
        );
        assert_eq!(task.units(), 0);
        assert!(!app.state.editors.sprite_viewers[&0].is_looping);
    }

    #[test]
    fn test_sprite_viewer_set_speed() {
        let mut app = app_with_sprite_viewer();
        assert_eq!(app.state.editors.sprite_viewers[&0].speed_100x, 100);

        let task = crate::editors::sprite_browser::handle(
            SpriteViewerMessage::SetSpeed(200),
            &mut app,
        );
        assert_eq!(task.units(), 0);
        assert_eq!(app.state.editors.sprite_viewers[&0].speed_100x, 200);
    }

    #[test]
    fn test_sprite_viewer_step_forward() {
        let mut app = app_with_sprite_viewer_frames();
        assert_eq!(app.state.editors.sprite_viewers[&0].selected_frame, 0);

        let task = crate::editors::sprite_browser::handle(
            SpriteViewerMessage::StepForward,
            &mut app,
        );
        assert_eq!(task.units(), 0);
        assert_eq!(
            app.state.editors.sprite_viewers[&0].selected_frame,
            1
        );
        assert!(
            !app.state.editors.sprite_viewers[&0].is_playing,
            "StepForward pauses playback"
        );
    }

    #[test]
    fn test_sprite_viewer_step_back() {
        let mut app = app_with_sprite_viewer_frames();
        // Start at frame 5
        {
            let viewer = app.state.editors.sprite_viewers.get_mut(&0).unwrap();
            viewer.selected_frame = 5;
        }

        let task = crate::editors::sprite_browser::handle(
            SpriteViewerMessage::StepBack,
            &mut app,
        );
        assert_eq!(task.units(), 0);
        assert_eq!(
            app.state.editors.sprite_viewers[&0].selected_frame,
            4
        );
        assert!(
            !app.state.editors.sprite_viewers[&0].is_playing,
            "StepBack pauses playback"
        );
    }

    #[test]
    fn test_sprite_viewer_scrub_to_pauses() {
        let mut app = app_with_sprite_viewer_frames();
        // Start playing
        {
            let viewer = app.state.editors.sprite_viewers.get_mut(&0).unwrap();
            viewer.is_playing = true;
        }

        let task = crate::editors::sprite_browser::handle(
            SpriteViewerMessage::ScrubTo(3),
            &mut app,
        );
        assert_eq!(task.units(), 0);
        assert_eq!(
            app.state.editors.sprite_viewers[&0].selected_frame,
            3
        );
        assert!(
            !app.state.editors.sprite_viewers[&0].is_playing,
            "ScrubTo pauses playback"
        );
    }

    // ── Export dialog ────────────────────────────────────────────────────────

    #[test]
    fn test_sprite_viewer_show_close_export_dialog() {
        let mut app = app_with_sprite_viewer();
        assert!(app.state.editors.sprite_viewers[&0]
            .export_dialog
            .is_none());

        let task = crate::editors::sprite_browser::handle(
            SpriteViewerMessage::ShowExportDialog,
            &mut app,
        );
        assert_eq!(task.units(), 0);
        assert!(
            app.state.editors.sprite_viewers[&0]
                .export_dialog
                .is_some(),
            "export dialog shown"
        );

        let task = crate::editors::sprite_browser::handle(
            SpriteViewerMessage::CloseExportDialog,
            &mut app,
        );
        assert_eq!(task.units(), 0);
        assert!(
            app.state.editors.sprite_viewers[&0]
                .export_dialog
                .is_none(),
            "export dialog closed"
        );
    }

    // ── Edge cases ───────────────────────────────────────────────────────────

    #[test]
    fn test_sprite_viewer_no_active_tab_returns_none() {
        let mut app = App::test_new(Workspace::new());
        // No active tab, no sprite viewers inserted
        // handle() calls active() which returns None → tab_id = usize::MAX
        // → sprite_viewers.get_mut(usize::MAX) returns None → Task::none()
        let task = crate::editors::sprite_browser::handle(
            SpriteViewerMessage::Play,
            &mut app,
        );
        assert_eq!(task.units(), 0, "no-op with no active tab");
    }
}
