use crate::app::App;
use crate::components::audio_playback::start_playback;
use crate::editors::audio_viewer::AudioViewerMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(message: AudioViewerMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = match &message {
        AudioViewerMessage::PlaybackReady(id, _, _) | AudioViewerMessage::Tick(id) => *id,
        _ => app.state.workspace.active_tab_id().as_usize(),
    };
    let Some(editor) = app.state.editors.audio_viewers.get_mut(&tab_id) else {
        return Task::none();
    };

    match message {
        AudioViewerMessage::Play => {
            // The view offers "Play" while paused: resume the existing player
            // so its position is kept, mirroring the Pause toggle's resume arm.
            if let Some(pb) = &editor.playback
                && pb.player.is_paused()
                && !pb.player.empty()
            {
                pb.player.play();
                return Task::none();
            }

            if editor
                .playback
                .as_ref()
                .is_some_and(|p| !p.player.is_paused() && !p.player.empty())
            {
                return Task::none();
            }
            editor.playback = None;
            editor.playback_generation = editor.playback_generation.wrapping_add(1);
            let generation = editor.playback_generation;

            let audio_bytes = editor.audio_bytes.clone();
            let is_looping = editor.is_looping;
            let volume = editor.volume;

            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        start_playback(audio_bytes, volume, is_looping)
                    })
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()))
                },
                move |result| {
                    crate::message::Message::audio_viewer(AudioViewerMessage::PlaybackReady(
                        tab_id, generation, result,
                    ))
                },
            );
        }

        AudioViewerMessage::PlaybackReady(_, generation, result) => {
            if let Some(target) = app.state.editors.audio_viewers.get_mut(&tab_id)
                && target.playback_generation == generation
            {
                match result {
                    Ok(pending) => {
                        if let Some(handle) = pending.take() {
                            target.playback = Some(handle);
                        }
                    }
                    Err(e) => eprintln!("audio viewer: {e}"),
                }
            }
            // A stale generation means Stop or a tab switch happened while the
            // device was opening; dropping the pending handle stops playback.
        }

        AudioViewerMessage::Pause => {
            if let Some(pb) = &editor.playback {
                if pb.player.is_paused() {
                    pb.player.play();
                } else {
                    pb.player.pause();
                }
            }
        }

        AudioViewerMessage::Stop => {
            editor.playback = None;
            editor.playback_generation = editor.playback_generation.wrapping_add(1);
        }

        AudioViewerMessage::ToggleLoop => {
            editor.is_looping = !editor.is_looping;
            if let Some(pb) = &editor.playback {
                pb.set_looping(editor.is_looping);
            }
        }

        AudioViewerMessage::SetVolume(v) => {
            editor.volume = v.clamp(0.0, 1.0);
            if let Some(pb) = &editor.playback {
                pb.player.set_volume(editor.volume);
            }
        }

        AudioViewerMessage::Tick(_) => {
            if editor.playback.as_ref().is_some_and(|pb| pb.player.empty()) {
                editor.playback = None;
            }
        }
    }

    Task::none()
}
