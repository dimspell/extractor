#[cfg(test)]
mod snf_editor_tests {
    use crate::app::App;
    use crate::editors::snf_editor::{self, ExportStatus, SnfEditorMessage, SnfEditorState};
    use crate::workspace::{EditorType, Workspace, WorkspaceTab};
    use dispel_core::snf::SnfFile;
    use std::path::PathBuf;
    use std::time::Instant;

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
                playback_generation: 0,
                is_looping: false,
                volume: 0.5,
                export_status: ExportStatus::Idle,
                modified: false,
                toasts: Vec::new(),
            },
        );

        app
    }

    /// An app with an SNF editor that has a small synthetic tone loaded.
    fn app_with_snf_editor_loaded() -> App {
        let mut app = app_with_snf_editor();
        let tone = SnfFile {
            pcmaudio_format: 1,
            number_of_channels: 1,
            sample_rate: 8000,
            byte_rate: 8000,
            block_align: 1,
            bits_per_sample: 8,
            data_size: 800,
            pcm_data: vec![128; 800],
        };
        let editor = app.state.editors.snf_editors.get_mut(&1).unwrap();
        editor.snf = Some(tone);
        editor.waveform = vec![(0.0, 0.5), (1.0, 0.5)];
        app
    }

    /// Build a `PendingPlayback` backed by an in-memory mixer, so tests can
    /// deliver a successful `PlaybackReady` without opening an audio device.
    fn pending_without_device() -> crate::components::audio_playback::PendingPlayback {
        use crate::components::audio_playback::{PendingPlayback, PlaybackHandle};
        use std::sync::Arc;
        use std::sync::atomic::AtomicBool;

        let (mixer, _source) = rodio::mixer::mixer(
            std::num::NonZero::new(1).unwrap(),
            std::num::NonZero::new(8000).unwrap(),
        );
        let player = Arc::new(rodio::Player::connect_new(&mixer));
        let handle = PlaybackHandle::new(
            player,
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicBool::new(false)),
            std::thread::spawn(|| {}),
        );
        PendingPlayback::new(handle)
    }

    // ── Play / PlaybackReady ───────────────────────────────────────────────

    #[test]
    fn test_snf_editor_stop_during_device_open_does_not_start_audio() {
        let mut app = app_with_snf_editor_loaded();
        let tab_id = 1;

        // Play defers the device open to a background task.
        let _ = snf_editor::handle(SnfEditorMessage::Play, &mut app);
        assert!(app.state.editors.snf_editors[&tab_id].playback.is_none());
        let play_generation = app.state.editors.snf_editors[&tab_id].playback_generation;

        // The user stops while the device is still opening.
        let _ = snf_editor::handle(SnfEditorMessage::Stop, &mut app);

        // The deferred result arrives after the Stop and must be discarded.
        let _ = snf_editor::handle(
            SnfEditorMessage::PlaybackReady(tab_id, play_generation, Ok(pending_without_device())),
            &mut app,
        );
        assert!(
            app.state.editors.snf_editors[&tab_id].playback.is_none(),
            "a PlaybackReady resolving after Stop must not start audio"
        );
    }

    #[test]
    fn test_snf_editor_playback_ready_installs_current_generation() {
        let mut app = app_with_snf_editor_loaded();
        let tab_id = 1;

        let _ = snf_editor::handle(SnfEditorMessage::Play, &mut app);
        let generation = app.state.editors.snf_editors[&tab_id].playback_generation;

        let _ = snf_editor::handle(
            SnfEditorMessage::PlaybackReady(tab_id, generation, Ok(pending_without_device())),
            &mut app,
        );
        assert!(
            app.state.editors.snf_editors[&tab_id].playback.is_some(),
            "the current generation's result must install a handle"
        );
    }

    #[test]
    fn test_snf_editor_play_defers_device_init_to_task() {
        let mut app = app_with_snf_editor_loaded();
        let tab_id = 1;

        // Play must only queue the async work: no device open on the UI thread,
        // so the handler returns quickly and playback is not set synchronously.
        let started = Instant::now();
        let task = snf_editor::handle(SnfEditorMessage::Play, &mut app);
        let elapsed = started.elapsed();

        assert!(
            elapsed.as_millis() < 250,
            "Play blocked the UI thread for {:?}",
            elapsed
        );
        assert_eq!(task.units(), 1, "Play must queue one async task");
        assert!(
            app.state.editors.snf_editors[&tab_id].playback.is_none(),
            "playback is installed only when PlaybackReady arrives"
        );

        // The deferred failure path is wired and leaves the editor untouched.
        let task = snf_editor::handle(
            SnfEditorMessage::PlaybackReady(
                tab_id,
                app.state.editors.snf_editors[&tab_id].playback_generation,
                Err("device unavailable".into()),
            ),
            &mut app,
        );
        assert_eq!(task.units(), 0);
        assert!(app.state.editors.snf_editors[&tab_id].playback.is_none());
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

#[cfg(test)]
mod audio_viewer_tests {
    use crate::app::App;
    use crate::editors::audio_viewer::{self, AudioFormat, AudioViewerMessage, AudioViewerState};
    use crate::workspace::{EditorType, Workspace, WorkspaceTab};
    use dispel_core::snf::SnfFile;
    use std::io::{Cursor, Write};
    use std::path::Path;
    use std::time::Instant;
    use tempfile::Builder;

    fn synthetic_tone() -> SnfFile {
        SnfFile {
            pcmaudio_format: 1,
            number_of_channels: 1,
            sample_rate: 8000,
            byte_rate: 8000,
            block_align: 1,
            bits_per_sample: 8,
            data_size: 800,
            pcm_data: vec![128; 800],
        }
    }

    fn wav_fixture() -> tempfile::NamedTempFile {
        let mut file = Builder::new()
            .prefix("invented_tone_")
            .suffix(".wav")
            .tempfile()
            .unwrap();
        file.write_all(&synthetic_tone().to_wav_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    fn app_with_audio_viewer(state: AudioViewerState) -> App {
        let mut app = App::test_new(Workspace::new());
        let tab_id = 1;
        app.state.workspace.tabs.push(WorkspaceTab {
            id: tab_id,
            label: "invented_tone.wav".into(),
            path: None,
            editor_type: EditorType::AudioViewer,
            modified: false,
            pinned: false,
        });
        app.state.workspace.active_tab = Some(0);
        app.state.editors.audio_viewers.insert(tab_id, state);
        app
    }

    #[test]
    fn test_audio_viewer_loads_wav_waveform_and_decoder() {
        let file = wav_fixture();
        let state = AudioViewerState::load_from_path(file.path());

        assert_eq!(state.format, Some(AudioFormat::Wav));
        assert!(state.error.is_none());
        assert!(state.snf.is_some());
        assert!(!state.waveform.is_empty());
        assert!(state.duration_secs > 0.0);
        assert!(
            rodio::Decoder::new(Cursor::new(state.audio_bytes.clone())).is_ok(),
            "loaded WAV bytes must be decodable"
        );
    }

    #[test]
    fn test_audio_viewer_detects_ogg_magic_bytes() {
        let mut file = Builder::new()
            .prefix("invented_track_")
            .suffix(".ogg")
            .tempfile()
            .unwrap();
        file.write_all(b"OggS synthetic fixture").unwrap();
        file.flush().unwrap();

        let state = AudioViewerState::load_from_path(file.path());

        assert_eq!(state.format, Some(AudioFormat::Ogg));
        assert!(state.error.is_none());
        assert!(state.snf.is_none());
        assert!(state.audio_bytes.starts_with(b"OggS"));
    }

    #[test]
    fn test_audio_viewer_editor_type_from_path() {
        assert_eq!(
            EditorType::from_path(Path::new("invented_track.ogg")),
            EditorType::AudioViewer
        );
        assert_eq!(
            EditorType::from_path(Path::new("invented_tone.wav")),
            EditorType::AudioViewer
        );
    }

    #[test]
    fn test_audio_viewer_render_does_not_panic() {
        let file = wav_fixture();
        let state = AudioViewerState::load_from_path(file.path());
        let app = app_with_audio_viewer(state);

        let _ = audio_viewer::view(&app);
    }

    /// A `PendingPlayback` backed by an in-memory mixer (no audio device).
    fn pending_without_device() -> crate::components::audio_playback::PendingPlayback {
        use crate::components::audio_playback::{PendingPlayback, PlaybackHandle};
        use std::sync::Arc;
        use std::sync::atomic::AtomicBool;

        let (mixer, _source) = rodio::mixer::mixer(
            std::num::NonZero::new(1).unwrap(),
            std::num::NonZero::new(8000).unwrap(),
        );
        let player = Arc::new(rodio::Player::connect_new(&mixer));
        let handle = PlaybackHandle::new(
            player,
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicBool::new(false)),
            std::thread::spawn(|| {}),
        );
        PendingPlayback::new(handle)
    }

    #[test]
    fn test_audio_viewer_stop_during_device_open_does_not_start_audio() {
        let file = wav_fixture();
        let mut app = app_with_audio_viewer(AudioViewerState::load_from_path(file.path()));
        let tab_id = 1;

        let _ = audio_viewer::handle(AudioViewerMessage::Play, &mut app);
        assert!(app.state.editors.audio_viewers[&tab_id].playback.is_none());
        let play_generation = app.state.editors.audio_viewers[&tab_id].playback_generation;

        let _ = audio_viewer::handle(AudioViewerMessage::Stop, &mut app);

        let _ = audio_viewer::handle(
            AudioViewerMessage::PlaybackReady(
                tab_id,
                play_generation,
                Ok(pending_without_device()),
            ),
            &mut app,
        );
        assert!(
            app.state.editors.audio_viewers[&tab_id].playback.is_none(),
            "a PlaybackReady resolving after Stop must not start audio"
        );
    }

    #[test]
    fn test_audio_viewer_playback_ready_installs_current_generation() {
        let file = wav_fixture();
        let mut app = app_with_audio_viewer(AudioViewerState::load_from_path(file.path()));
        let tab_id = 1;

        let _ = audio_viewer::handle(AudioViewerMessage::Play, &mut app);
        let generation = app.state.editors.audio_viewers[&tab_id].playback_generation;

        let _ = audio_viewer::handle(
            AudioViewerMessage::PlaybackReady(tab_id, generation, Ok(pending_without_device())),
            &mut app,
        );
        assert!(
            app.state.editors.audio_viewers[&tab_id].playback.is_some(),
            "the current generation's result must install a handle"
        );
    }

    #[test]
    fn test_audio_viewer_playback_state_transitions_without_device() {
        let file = wav_fixture();
        let state = AudioViewerState::load_from_path(file.path());
        let mut app = app_with_audio_viewer(state);
        let tab_id = 1;

        // Play only queues the device open; nothing is installed synchronously.
        let started = Instant::now();
        let task = audio_viewer::handle(AudioViewerMessage::Play, &mut app);
        assert!(
            started.elapsed().as_millis() < 250,
            "Play must not block the UI thread"
        );
        assert_eq!(task.units(), 1);
        assert!(app.state.editors.audio_viewers[&tab_id].playback.is_none());

        // The deferred failure path leaves state untouched.
        let task = audio_viewer::handle(
            AudioViewerMessage::PlaybackReady(
                tab_id,
                app.state.editors.audio_viewers[&tab_id].playback_generation,
                Err("device unavailable".into()),
            ),
            &mut app,
        );
        assert_eq!(task.units(), 0);
        assert!(app.state.editors.audio_viewers[&tab_id].playback.is_none());

        // Controls work with no audio device open.
        assert_eq!(
            audio_viewer::handle(AudioViewerMessage::ToggleLoop, &mut app).units(),
            0
        );
        assert!(app.state.editors.audio_viewers[&tab_id].is_looping);

        assert_eq!(
            audio_viewer::handle(AudioViewerMessage::SetVolume(2.0), &mut app).units(),
            0
        );
        assert_eq!(app.state.editors.audio_viewers[&tab_id].volume, 1.0);

        for msg in [
            AudioViewerMessage::Pause,
            AudioViewerMessage::Stop,
            AudioViewerMessage::Tick,
        ] {
            assert_eq!(audio_viewer::handle(msg, &mut app).units(), 0);
        }
        assert!(app.state.editors.audio_viewers[&tab_id].playback.is_none());
    }

    #[test]
    fn test_audio_viewer_missing_file_reports_error() {
        let state = AudioViewerState::load_from_path(Path::new(
            "/definitely/not/a/real/invented_track.ogg",
        ));
        assert!(state.error.is_some());
        assert!(state.format.is_none());
        assert!(state.audio_bytes.is_empty());
    }
}
