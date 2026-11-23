use crate::app::App;
use crate::editors::audio_viewer::{AudioFormat, AudioViewerMessage};
use crate::editors::snf_editor::waveform_canvas;
use crate::message::{Message, MessageExt};
use gui_widgets::lucide::{LUCIDE_FONT, icon_char};
use iced::widget::{Space, button, column, container, progress_bar, row, slider, text};
use iced::{Alignment, Element, Fill, Length};
use lucide_icons::Icon;

pub fn view(app: &App) -> Element<'_, Message> {
    let tab_id = app.state.workspace.active_tab_id().as_usize();

    let Some(editor) = app.state.editors.audio_viewers.get(&tab_id) else {
        return container(text("Audio file not loaded").size(14))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .accessible_label("Audio viewer")
            .into();
    };

    if let Some(ref err) = editor.error {
        return container(
            column![text("Failed to load audio").size(14), text(err).size(12)].spacing(8),
        )
        .width(Fill)
        .height(Fill)
        .padding(16)
        .accessible_label("Audio viewer")
        .into();
    }

    let format_label = match editor.format {
        Some(AudioFormat::Wav) => "WAV",
        Some(AudioFormat::Ogg) => "OGG",
        None => "Unknown",
    };

    let header = row![
        text(&editor.name).size(13),
        text(format_label).size(11),
        Space::new().width(Fill),
    ]
    .spacing(8)
    .padding([6, 12])
    .align_y(Alignment::Center);

    let meta: Element<'_, Message> = match &editor.snf {
        Some(snf) => column![
            text(format!("Sample Rate: {} Hz", snf.sample_rate)).size(12),
            text(format!("Channels: {}", snf.number_of_channels)).size(12),
            text(format!("Bits/Sample: {}", snf.bits_per_sample)).size(12),
        ]
        .spacing(4)
        .padding([8, 12])
        .into(),
        None => container(text("Vorbis stream").size(12))
            .padding([8, 12])
            .into(),
    };

    let waveform: Element<'_, Message> = if editor.waveform.is_empty() {
        Space::new().height(Length::Fixed(120.0)).into()
    } else {
        container(waveform_canvas(&editor.waveform))
            .width(Fill)
            .height(Length::Fixed(120.0))
            .padding([8, 12])
            .into()
    };

    let is_playing = editor
        .playback
        .as_ref()
        .is_some_and(|p| !p.player.is_paused() && !p.player.empty());

    let pos_secs = editor
        .playback
        .as_ref()
        .map(|pb| pb.player.get_pos().as_secs_f32())
        .unwrap_or(0.0);
    let progress = if editor.duration_secs > 0.0 {
        (pos_secs / editor.duration_secs).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let fmt_time = |s: f32| -> String {
        let s = s as u32;
        format!("{}:{:02}", s / 60, s % 60)
    };

    let timeline = row![
        text(fmt_time(pos_secs)).size(11),
        container(progress_bar(0.0f32..=1.0, progress))
            .height(Length::Fixed(8.0))
            .width(Fill),
        text(fmt_time(editor.duration_secs)).size(11),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding([4, 12]);

    let play_pause_btn = if is_playing {
        button(
            row![
                text(icon_char(Icon::Pause)).font(LUCIDE_FONT).size(12),
                text(" Pause").size(12),
            ]
            .spacing(4),
        )
        .on_press(Message::audio_viewer(AudioViewerMessage::Pause))
    } else {
        button(
            row![
                text(icon_char(Icon::Play)).font(LUCIDE_FONT).size(12),
                text(" Play").size(12),
            ]
            .spacing(4),
        )
        .on_press(Message::audio_viewer(AudioViewerMessage::Play))
    };

    let stop_btn = button(
        row![
            text(icon_char(Icon::Square)).font(LUCIDE_FONT).size(12),
            text(" Stop").size(12),
        ]
        .spacing(4),
    )
    .on_press(Message::audio_viewer(AudioViewerMessage::Stop));

    let loop_btn = button(
        row![
            text(icon_char(Icon::Repeat)).font(LUCIDE_FONT).size(11),
            text(" Loop").size(11),
        ]
        .spacing(4),
    )
    .style(if editor.is_looping {
        crate::style::active_chip
    } else {
        crate::style::chip
    })
    .on_press(Message::audio_viewer(AudioViewerMessage::ToggleLoop));

    let volume_slider = slider(0.0f32..=1.0, editor.volume, |v| {
        Message::audio_viewer(AudioViewerMessage::SetVolume(v))
    })
    .width(Length::Fixed(120.0));

    let controls = container(
        row![play_pause_btn, stop_btn, loop_btn, text("Vol:").size(11), volume_slider]
            .spacing(8)
            .padding([6, 12])
            .align_y(Alignment::Center),
    )
    .width(Fill);

    column![header, meta, waveform, timeline, controls]
        .spacing(0)
        .height(Fill)
        .accessible_label("Audio viewer")
        .into()
}
