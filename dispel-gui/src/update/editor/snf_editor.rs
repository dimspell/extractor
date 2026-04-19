use crate::app::App;
use crate::message::editor::snf::SnfEditorMessage;
use crate::message::MessageExt;
use crate::state::snf_editor::PlaybackHandle;
use iced::Task;
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

pub fn handle(message: SnfEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);
    let Some(editor) = app.state.snf_editors.get_mut(&tab_id) else {
        return Task::none();
    };

    match message {
        SnfEditorMessage::Play => {
            // Do nothing if already playing (not paused, not empty).
            if editor
                .playback
                .as_ref()
                .is_some_and(|p| !p.sink.is_paused() && !p.sink.empty())
            {
                return Task::none();
            }

            // Stop any previous playback cleanly before starting a new one.
            editor.playback = None;

            let Some(ref snf) = editor.snf else {
                return Task::none();
            };

            let wav_bytes = snf.to_wav_bytes();
            let is_looping = editor.is_looping;
            let volume = editor.volume;

            let (sink_tx, sink_rx) = mpsc::sync_channel::<Arc<rodio::Sink>>(1);
            let stop_flag = Arc::new(AtomicBool::new(false));
            let loop_flag = Arc::new(AtomicBool::new(is_looping));
            let stop_flag_thread = Arc::clone(&stop_flag);
            let loop_flag_thread = Arc::clone(&loop_flag);

            let thread = std::thread::spawn(move || {
                let stream_handle = match rodio::OutputStreamBuilder::open_default_stream() {
                    Ok(h) => h,
                    Err(e) => {
                        eprintln!("SNF: failed to open audio output: {e}");
                        return;
                    }
                };

                let sink = Arc::new(rodio::Sink::connect_new(&stream_handle.mixer().clone()));
                sink.set_volume(volume);

                if sink_tx.send(Arc::clone(&sink)).is_err() {
                    // Receiver dropped (timed out) — abort before playing anything.
                    return;
                }

                loop {
                    if stop_flag_thread.load(Ordering::Relaxed) {
                        break;
                    }
                    let cursor = Cursor::new(wav_bytes.clone());
                    match rodio::Decoder::new(cursor) {
                        Ok(source) => sink.append(source),
                        Err(e) => {
                            eprintln!("SNF: decoder error: {e}");
                            break;
                        }
                    }
                    sink.sleep_until_end();
                    if !loop_flag_thread.load(Ordering::Relaxed)
                        || stop_flag_thread.load(Ordering::Relaxed)
                    {
                        break;
                    }
                }
            });

            match sink_rx.recv_timeout(std::time::Duration::from_millis(500)) {
                Ok(sink) => {
                    editor.playback = Some(PlaybackHandle::new(sink, stop_flag, loop_flag, thread));
                }
                Err(_) => {
                    eprintln!("SNF: timed out waiting for audio thread to start");
                }
            }
        }

        SnfEditorMessage::Pause => {
            if let Some(ref pb) = editor.playback {
                if pb.sink.is_paused() {
                    pb.sink.play();
                } else {
                    pb.sink.pause();
                }
            }
        }

        SnfEditorMessage::Stop => {
            editor.playback = None;
        }

        SnfEditorMessage::ToggleLoop => {
            editor.is_looping = !editor.is_looping;
            if let Some(ref pb) = editor.playback {
                pb.set_looping(editor.is_looping);
            }
        }

        SnfEditorMessage::SetVolume(v) => {
            editor.volume = v.clamp(0.0, 1.0);
            if let Some(ref pb) = editor.playback {
                pb.sink.set_volume(editor.volume);
            }
        }

        SnfEditorMessage::Tick => {
            if editor.playback.as_ref().is_some_and(|pb| pb.sink.empty()) {
                editor.playback = None;
            }
        }

        SnfEditorMessage::ExportWav => {
            let path = editor.path.clone();
            let stem = editor.name.clone();

            return Task::perform(
                async move {
                    let handle = rfd::AsyncFileDialog::new()
                        .set_file_name(format!("{stem}.wav"))
                        .add_filter("WAV Audio", &["wav"])
                        .save_file()
                        .await;

                    match handle {
                        Some(h) => {
                            let out = h.path().to_path_buf();
                            dispel_core::snf::extract(&path, &out)
                                .map(|_| out.to_string_lossy().to_string())
                                .map_err(|e| e.to_string())
                        }
                        None => Err("Export cancelled".into()),
                    }
                },
                |r| crate::message::Message::snf_editor(SnfEditorMessage::ExportWavDone(r)),
            );
        }

        SnfEditorMessage::ExportWavDone(result) => {
            use crate::state::snf_editor::ExportStatus;
            editor.export_status = match result {
                Ok(p) => ExportStatus::Done(p),
                Err(e) => ExportStatus::Error(e),
            };
        }
    }

    Task::none()
}
