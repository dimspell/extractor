#[cfg(test)]
mod sprite_viewer_tests {
    use crate::app::App;
    use crate::editors::sprite_editor::{SpriteFrame, SpriteViewerMessage, SpriteViewerState};
    use crate::workspace::{EditorType, Workspace, WorkspaceTab};
    use std::path::PathBuf;

    fn dummy_frame(frame_idx: usize) -> SpriteFrame {
        SpriteFrame {
            sequence_idx: 0,
            frame_idx,
            image: iced::widget::image::Handle::from_rgba(1, 1, vec![0, 0, 0, 0]),
            rgba_bytes: vec![],
            width: 1,
            height: 1,
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
                save_path: PathBuf::new(),
                name: "test.spr".into(),
                error: None,
                preserve_black: false,
                sprite_file: None,
                dirty: false,
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
                zoom: 1.0,
                fit_mode: false,
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
                save_path: PathBuf::new(),
                name: "test.spr".into(),
                error: None,
                preserve_black: false,
                sprite_file: None,
                dirty: false,
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
                zoom: 1.0,
                fit_mode: false,
            },
        );
        app
    }

    // ── Navigation ───────────────────────────────────────────────────────────

    #[test]
    fn test_sprite_viewer_select_sequence() {
        let mut app = app_with_sprite_viewer();
        let task =
            crate::editors::sprite_editor::handle(SpriteViewerMessage::SelectSequence(1), &mut app);
        assert_eq!(task.units(), 0);
        assert_eq!(app.state.editors.sprite_viewers[&0].selected_sequence, 1);
        // Frame resets to 0 when sequence changes
        assert_eq!(app.state.editors.sprite_viewers[&0].selected_frame, 0);
    }

    #[test]
    fn test_sprite_viewer_select_frame() {
        let mut app = app_with_sprite_viewer_frames();
        let task =
            crate::editors::sprite_editor::handle(SpriteViewerMessage::SelectFrame(5), &mut app);
        assert_eq!(task.units(), 0);
        assert_eq!(app.state.editors.sprite_viewers[&0].selected_frame, 5);
    }

    // ── Playback ─────────────────────────────────────────────────────────────

    #[test]
    fn test_sprite_viewer_play_pause_toggles() {
        let mut app = app_with_sprite_viewer();
        assert!(!app.state.editors.sprite_viewers[&0].is_playing);

        let task = crate::editors::sprite_editor::handle(SpriteViewerMessage::Play, &mut app);
        assert_eq!(task.units(), 0);
        assert!(app.state.editors.sprite_viewers[&0].is_playing);

        let task = crate::editors::sprite_editor::handle(SpriteViewerMessage::Pause, &mut app);
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

        let task = crate::editors::sprite_editor::handle(SpriteViewerMessage::Tick, &mut app);
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

        let task = crate::editors::sprite_editor::handle(SpriteViewerMessage::ToggleLoop, &mut app);
        assert_eq!(task.units(), 0);
        assert!(!app.state.editors.sprite_viewers[&0].is_looping);
    }

    #[test]
    fn test_sprite_viewer_set_speed() {
        let mut app = app_with_sprite_viewer();
        assert_eq!(app.state.editors.sprite_viewers[&0].speed_100x, 100);

        let task =
            crate::editors::sprite_editor::handle(SpriteViewerMessage::SetSpeed(200), &mut app);
        assert_eq!(task.units(), 0);
        assert_eq!(app.state.editors.sprite_viewers[&0].speed_100x, 200);
    }

    #[test]
    fn test_sprite_viewer_step_forward() {
        let mut app = app_with_sprite_viewer_frames();
        assert_eq!(app.state.editors.sprite_viewers[&0].selected_frame, 0);

        let task =
            crate::editors::sprite_editor::handle(SpriteViewerMessage::StepForward, &mut app);
        assert_eq!(task.units(), 0);
        assert_eq!(app.state.editors.sprite_viewers[&0].selected_frame, 1);
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

        let task = crate::editors::sprite_editor::handle(SpriteViewerMessage::StepBack, &mut app);
        assert_eq!(task.units(), 0);
        assert_eq!(app.state.editors.sprite_viewers[&0].selected_frame, 4);
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

        let task = crate::editors::sprite_editor::handle(SpriteViewerMessage::ScrubTo(3), &mut app);
        assert_eq!(task.units(), 0);
        assert_eq!(app.state.editors.sprite_viewers[&0].selected_frame, 3);
        assert!(
            !app.state.editors.sprite_viewers[&0].is_playing,
            "ScrubTo pauses playback"
        );
    }

    // ── Zoom / Fit ────────────────────────────────────────────────────────────

    #[test]
    fn test_zoom_to_fit_resets_manual_zoom() {
        let mut app = app_with_sprite_viewer_frames();
        for _ in 0..5 {
            let _ = crate::editors::sprite_editor::handle(SpriteViewerMessage::ZoomIn, &mut app);
        }
        assert!(app.state.editors.sprite_viewers[&0].zoom > 1.0);
        assert!(!app.state.editors.sprite_viewers[&0].fit_mode);

        let _ = crate::editors::sprite_editor::handle(SpriteViewerMessage::ZoomToFit, &mut app);
        let viewer = &app.state.editors.sprite_viewers[&0];
        assert!(viewer.fit_mode);
        assert_eq!(
            viewer.zoom, 1.0,
            "Fit must clear the manual multiplier so the effective scale is exactly fit_scale"
        );

        let viewport = iced::Size::new(640.0, 480.0);
        let frame = (viewer.frames[0].width, viewer.frames[0].height);
        let effective = crate::editors::sprite_editor::fit_scale(viewport, frame) * viewer.zoom;
        assert_eq!(
            effective,
            crate::editors::sprite_editor::fit_scale(viewport, frame)
        );
    }

    #[cfg(feature = "iced_test")]
    #[test]
    fn test_zoom_to_fit_clears_stale_percentage_label() {
        use iced_test::simulator;

        let mut app = app_with_sprite_viewer_frames();
        for _ in 0..5 {
            let _ = crate::editors::sprite_editor::handle(SpriteViewerMessage::ZoomIn, &mut app);
        }
        let stale = format!(
            "{}%",
            (app.state.editors.sprite_viewers[&0].zoom * 100.0).round() as u32
        );
        {
            let mut ui = simulator(app.view());
            ui.find(stale.clone())
                .expect("manual zoom shows a percentage");
        }

        let _ = crate::editors::sprite_editor::handle(SpriteViewerMessage::ZoomToFit, &mut app);
        let mut ui = simulator(app.view());
        ui.find(stale)
            .expect_err("Fit must not keep showing a stale manual zoom percentage");
    }

    // ── Export dialog ────────────────────────────────────────────────────────

    #[test]
    fn test_sprite_viewer_show_close_export_dialog() {
        let mut app = app_with_sprite_viewer();
        assert!(app.state.editors.sprite_viewers[&0].export_dialog.is_none());

        let task =
            crate::editors::sprite_editor::handle(SpriteViewerMessage::ShowExportDialog, &mut app);
        assert_eq!(task.units(), 0);
        assert!(
            app.state.editors.sprite_viewers[&0].export_dialog.is_some(),
            "export dialog shown"
        );

        let task =
            crate::editors::sprite_editor::handle(SpriteViewerMessage::CloseExportDialog, &mut app);
        assert_eq!(task.units(), 0);
        assert!(
            app.state.editors.sprite_viewers[&0].export_dialog.is_none(),
            "export dialog closed"
        );
    }

    // ── Edge cases ───────────────────────────────────────────────────────────

    #[test]
    fn test_sprite_viewer_no_active_tab_returns_none() {
        let mut app = App::test_new(Workspace::new());
        // No active tab, no sprite viewers inserted
        // handle() gets TabId::NONE for the tab id -> no viewer is found -> Task::none()
        let task = crate::editors::sprite_editor::handle(SpriteViewerMessage::Play, &mut app);
        assert_eq!(task.units(), 0, "no-op with no active tab");
    }
}

#[cfg(test)]
mod bmp_viewer_tests {
    use crate::app::App;
    use crate::editors::bmp_viewer::{
        self, BmpViewerMessage, BmpViewerState, MAX_ZOOM, MIN_ZOOM, fit_scale, rendered_size,
    };
    use crate::workspace::{EditorType, Workspace, WorkspaceTab};
    use iced::Size;
    use std::io::Write;
    use std::path::Path;
    use tempfile::Builder;

    /// A minimal valid 1x1 24-bit BMP (BGR pixel + row padding).
    fn synthetic_bmp() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"BM");
        bytes.extend_from_slice(&58u32.to_le_bytes()); // total file size
        bytes.extend_from_slice(&0u32.to_le_bytes()); // reserved
        bytes.extend_from_slice(&54u32.to_le_bytes()); // pixel data offset
        bytes.extend_from_slice(&40u32.to_le_bytes()); // DIB header size
        bytes.extend_from_slice(&1i32.to_le_bytes()); // width
        bytes.extend_from_slice(&1i32.to_le_bytes()); // height
        bytes.extend_from_slice(&1u16.to_le_bytes()); // planes
        bytes.extend_from_slice(&24u16.to_le_bytes()); // bits per pixel
        bytes.extend_from_slice(&0u32.to_le_bytes()); // compression: BI_RGB
        bytes.extend_from_slice(&4u32.to_le_bytes()); // image size
        bytes.extend_from_slice(&2835i32.to_le_bytes()); // x pixels per metre
        bytes.extend_from_slice(&2835i32.to_le_bytes()); // y pixels per metre
        bytes.extend_from_slice(&0u32.to_le_bytes()); // colors in palette
        bytes.extend_from_slice(&0u32.to_le_bytes()); // important colors
        bytes.extend_from_slice(&[0x11, 0x22, 0x33, 0x00]); // blue, green, red, padding
        bytes
    }

    fn bmp_fixture() -> tempfile::NamedTempFile {
        let mut file = Builder::new()
            .prefix("invented_pixel_")
            .suffix(".bmp")
            .tempfile()
            .unwrap();
        file.write_all(&synthetic_bmp()).unwrap();
        file.flush().unwrap();
        file
    }

    fn app_with_bmp_viewer(state: BmpViewerState, tab_id: usize) -> App {
        let mut app = App::test_new(Workspace::new());
        app.state.workspace.tabs.push(WorkspaceTab {
            id: tab_id,
            label: "invented_pixel.bmp".into(),
            path: None,
            editor_type: EditorType::BmpViewer,
            modified: false,
            pinned: false,
        });
        app.state.workspace.active_tab = Some(0);
        app.state.editors.bmp_viewers.insert(tab_id, state);
        app
    }

    #[test]
    fn test_bmp_viewer_loads_dimensions_and_handle() {
        let file = bmp_fixture();
        let state = BmpViewerState::load_from_path(file.path());

        assert!(state.error.is_none());
        assert_eq!(state.width, 1);
        assert_eq!(state.height, 1);
        assert!(state.handle.is_some());
        assert_eq!(
            EditorType::from_path(Path::new("invented_pixel.bmp")),
            EditorType::BmpViewer
        );
    }

    #[test]
    fn test_bmp_viewer_bad_bytes_report_error() {
        let mut file = Builder::new()
            .prefix("invented_broken_")
            .suffix(".bmp")
            .tempfile()
            .unwrap();
        file.write_all(b"not a bitmap").unwrap();
        file.flush().unwrap();

        let state = BmpViewerState::load_from_path(file.path());

        assert!(state.error.is_some());
        assert!(state.handle.is_none());
        assert_eq!(state.width, 0);
    }

    #[test]
    fn test_bmp_viewer_zoom_clamps_and_fit_computes() {
        let file = bmp_fixture();
        let state = BmpViewerState::load_from_path(file.path());
        let mut app = app_with_bmp_viewer(state, 1);

        for _ in 0..10 {
            let _ = bmp_viewer::handle(BmpViewerMessage::ZoomIn, &mut app);
        }
        assert_eq!(app.state.editors.bmp_viewers[&1].zoom, MAX_ZOOM);
        assert!(!app.state.editors.bmp_viewers[&1].fit_mode);

        for _ in 0..10 {
            let _ = bmp_viewer::handle(BmpViewerMessage::ZoomOut, &mut app);
        }
        assert_eq!(app.state.editors.bmp_viewers[&1].zoom, MIN_ZOOM);

        let _ = bmp_viewer::handle(BmpViewerMessage::ZoomReset, &mut app);
        assert_eq!(app.state.editors.bmp_viewers[&1].zoom, 1.0);

        let _ = bmp_viewer::handle(BmpViewerMessage::Fit, &mut app);
        assert!(app.state.editors.bmp_viewers[&1].fit_mode);

        // Fit uses the real viewport size and preserves the aspect ratio.
        assert_eq!(fit_scale(Size::new(800.0, 600.0), (1600, 800)), 0.5);
        assert_eq!(fit_scale(Size::new(100.0, 400.0), (1000, 1000)), 0.1);
    }

    #[test]
    fn test_bmp_viewer_fit_content_size_is_finite_and_exact() {
        let viewport = Size::new(800.0, 600.0);
        // An invented tall image: height must drive Fit, not width alone.
        let image = (1200u32, 3000u32);

        let scale = fit_scale(viewport, image);
        assert!(scale.is_finite() && scale > 0.0);
        assert_eq!(scale, 0.2);

        let content = rendered_size(viewport, image, true, 1.0);
        assert_eq!(content.width, image.0 as f32 * scale);
        assert_eq!(content.height, image.1 as f32 * scale);
        assert!(
            content.height <= viewport.height,
            "Fit must never produce content taller than a finite viewport"
        );

        // Zooming in keeps the content size exact so the scrollable can pan.
        let zoomed = rendered_size(viewport, image, true, 2.0);
        assert_eq!(zoomed.width, content.width * 2.0);
        assert_eq!(zoomed.height, content.height * 2.0);
    }

    #[test]
    fn test_bmp_viewer_render_does_not_panic() {
        let file = bmp_fixture();
        let state = BmpViewerState::load_from_path(file.path());
        let app = app_with_bmp_viewer(state, 1);

        let _ = bmp_viewer::view(&app);
    }

    #[test]
    fn test_bmp_viewer_per_tab_isolation() {
        let file = bmp_fixture();
        let loaded = BmpViewerState::load_from_path(file.path());
        let broken =
            BmpViewerState::load_from_path(Path::new("/definitely/not/a/real/invented_broken.bmp"));

        let mut app = app_with_bmp_viewer(loaded, 1);
        app.state.workspace.tabs.push(WorkspaceTab {
            id: 2,
            label: "invented_broken.bmp".into(),
            path: None,
            editor_type: EditorType::BmpViewer,
            modified: false,
            pinned: false,
        });
        app.state.editors.bmp_viewers.insert(2, broken);

        let _ = bmp_viewer::handle(BmpViewerMessage::ZoomIn, &mut app);

        assert_eq!(app.state.editors.bmp_viewers[&1].zoom, 1.25);
        assert_eq!(app.state.editors.bmp_viewers[&2].zoom, 1.0);
        assert!(app.state.editors.bmp_viewers[&2].error.is_some());
    }
}
