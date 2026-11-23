use crate::app::App;
use crate::components::audio_playback::start_playback;
use crate::editors::audio_viewer::AudioViewerMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(message: AudioViewerMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = match &message {
        AudioViewerMessage::PlaybackReady(id, _) => *id,
        _ => app.state.workspace.active_tab_id().as_usize(),
    };
    let Some(editor) = app.state.editors.audio_viewers.get_mut(&tab_id) else {
        return Task::none();
    };

    match message {
        AudioViewerMessage::Play => {
            if editor
                .playback
                .as_ref()
                .is_some_and(|p| !p.player.is_paused() && !p.player.empty())
            {
                return Task::none();
            }
            editor.playback = None;

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
                        tab_id, result,
                    ))
                },
            );
        }

        AudioViewerMessage::PlaybackReady(_, result) => match result {
            Ok(pending) => {
                if let Some(handle) = pending.take()
                    && let Some(target) = app.state.editors.audio_viewers.get_mut(&tab_id)
                {
                    target.playback = Some(handle);
                }
                // A closed tab drops the handle, which stops playback.
            }
            Err(e) => eprintln!("audio viewer: {e}"),
        },

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

        AudioViewerMessage::Tick => {
            if editor.playback.as_ref().is_some_and(|pb| pb.player.empty()) {
                editor.playback = None;
            }
        }
    }

    Task::none()
}
