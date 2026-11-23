use crate::app::App;
use crate::components::audio_playback::start_playback;
use crate::editors::snf_editor::{ExportStatus, SnfEditorMessage};
use crate::message::MessageExt;
use gui_widgets::components::toast;
use iced::Task;

pub fn handle(message: SnfEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    // Save messages carry their tab id so a save that resolves after a tab
    // switch still reaches the editor the user actually saved.
    let tab_id = match &message {
        SnfEditorMessage::Save(id)
        | SnfEditorMessage::SaveDone(id, _)
        | SnfEditorMessage::PlaybackReady(id, _) => *id,
        _ => app.state.workspace.active_tab_id().as_usize(),
    };
    let Some(editor) = app.state.editors.snf_editors.get_mut(&tab_id) else {
        return Task::none();
    };

    match message {
        SnfEditorMessage::Play => {
            // Do nothing if already playing (not paused, not empty).
            if editor
                .playback
                .as_ref()
                .is_some_and(|p| !p.player.is_paused() && !p.player.empty())
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

            // Device initialization can block for hundreds of milliseconds. Run
            // it on a blocking worker so the UI thread stays responsive.
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        start_playback(wav_bytes, volume, is_looping)
                    })
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()))
                },
                move |result| {
                    crate::message::Message::snf_editor(SnfEditorMessage::PlaybackReady(
                        tab_id, result,
                    ))
                },
            );
        }

        SnfEditorMessage::PlaybackReady(_, result) => match result {
            Ok(pending) => {
                if let Some(handle) = pending.take()
                    && let Some(target) = app.state.editors.snf_editors.get_mut(&tab_id)
                {
                    target.playback = Some(handle);
                }
                // If the tab was closed while the device was opening, dropping
                // the handle stops playback.
            }
            Err(e) => eprintln!("SNF: {e}"),
        },

        SnfEditorMessage::Pause => {
            if let Some(ref pb) = editor.playback {
                if pb.player.is_paused() {
                    pb.player.play();
                } else {
                    pb.player.pause();
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
                pb.player.set_volume(editor.volume);
            }
        }

        SnfEditorMessage::Tick => {
            if editor.playback.as_ref().is_some_and(|pb| pb.player.empty()) {
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

        SnfEditorMessage::ExportWavDone(result) => match result {
            Ok(p) => {
                editor.export_status = ExportStatus::Done(p.clone());
                editor.toasts.push(toast::Toast::success("Exported", p));
            }
            Err(e) => {
                editor.export_status = ExportStatus::Error(e.clone());
                editor.toasts.push(toast::Toast::error("Export failed", e));
            }
        },

        SnfEditorMessage::ImportWav => {
            return Task::perform(
                async move {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("WAV Audio", &["wav"])
                        .set_file_name("sound.wav")
                        .pick_file()
                        .await;
                    match handle {
                        Some(h) => {
                            let wav_path = h.path().to_path_buf();
                            match dispel_core::snf::read_wav(&wav_path) {
                                Ok(snf) => Ok((snf, wav_path.to_string_lossy().to_string())),
                                Err(e) => Err(e.to_string()),
                            }
                        }
                        None => Err("Import cancelled".into()),
                    }
                },
                |r: Result<(dispel_core::snf::SnfFile, String), String>| match r {
                    Ok((snf, path)) => crate::message::Message::snf_editor(
                        SnfEditorMessage::ImportWavDone(Ok((snf, path))),
                    ),
                    Err(e) => {
                        crate::message::Message::snf_editor(SnfEditorMessage::ImportWavDone(Err(e)))
                    }
                },
            );
        }

        SnfEditorMessage::ImportWavDone(result) => match result {
            Ok((snf, path)) => {
                editor.snf = Some(snf.clone());
                editor.waveform = snf.waveform_points(1000);
                editor.modified = true;
                editor.export_status = ExportStatus::Done(format!("Imported: {}", path));
                editor.toasts.push(toast::Toast::success("Imported", path));
            }
            Err(e) => {
                editor.export_status = ExportStatus::Error(e.clone());
                editor.toasts.push(toast::Toast::error("Import failed", e));
            }
        },

        SnfEditorMessage::Save(_) => {
            let path = editor.path.clone();
            let snf = editor.snf.clone();
            return Task::perform(
                async move {
                    match snf {
                        Some(snf) => dispel_core::snf::save(&path, &snf)
                            .map(|_| path.to_string_lossy().to_string())
                            .map_err(|e| e.to_string()),
                        None => Err("No audio data loaded".into()),
                    }
                },
                move |r| crate::message::Message::snf_editor(SnfEditorMessage::SaveDone(tab_id, r)),
            );
        }

        SnfEditorMessage::SaveDone(_, result) => match result {
            Ok(p) => {
                editor.modified = false;
                editor.export_status = ExportStatus::Done(format!("Saved: {}", p));
                editor.toasts.push(toast::Toast::success("Saved", p));
            }
            Err(e) => {
                editor.export_status = ExportStatus::Error(e.clone());
                editor.toasts.push(toast::Toast::error("Save failed", e));
            }
        },

        SnfEditorMessage::DismissToast(index) => {
            if index < editor.toasts.len() {
                editor.toasts.remove(index);
            }
        }
    }

    Task::none()
}
