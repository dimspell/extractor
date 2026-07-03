use crate::app::App;
use crate::editors::snf_editor::waveform::waveform_canvas;
use crate::editors::snf_editor::ExportStatus;
use crate::editors::snf_editor::SnfEditorMessage;
use crate::message::{Message, MessageExt};
use gui_widgets::components::toast;
use iced::widget::{button, column, container, progress_bar, row, slider, text, Space};
use iced::{Alignment, Element, Fill, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);

    let Some(editor) = app.state.editors.snf_editors.get(&tab_id) else {
        return container(text("SNF not loaded").size(14))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .accessible_label("SNF audio editor")
            .into();
    };

    if let Some(ref err) = editor.error {
        return container(
            column![text("Failed to load SNF").size(14), text(err).size(12)].spacing(8),
        )
        .width(Fill)
        .height(Fill)
        .padding(16)
        .accessible_label("SNF audio editor")
        .into();
    }

    let Some(ref snf) = editor.snf else {
        return container(text("Loading…").size(12))
            .padding(16)
            .accessible_label("SNF audio editor")
            .into();
    };

    // Header row with filename, import/save/export buttons
    let header = row![
        text(&editor.name).size(13),
        Space::new().width(Fill),
        button(text("📂 Import WAV…").size(12))
            .on_press(Message::snf_editor(SnfEditorMessage::ImportWav)),
        button(text("Export WAV…").size(12))
            .on_press(Message::snf_editor(SnfEditorMessage::ExportWav)),
        if editor.modified {
            button(text("💾 Save").size(12))
                .style(crate::style::active_chip)
                .on_press(Message::snf_editor(SnfEditorMessage::Save))
        } else {
            button(text("💾 Save").size(12))
                .style(crate::style::chip)
        }
    ]
    .spacing(8)
    .padding([6, 12])
    .align_y(Alignment::Center);

    // Metadata panel
    let meta = column![
        text(format!("Sample Rate: {} Hz", snf.sample_rate)).size(12),
        text(format!("Channels: {}", snf.number_of_channels)).size(12),
        text(format!("Bits/Sample: {}", snf.bits_per_sample)).size(12),
        text(format!("Duration: {:.2}s", snf.duration_secs())).size(12),
        text(format!("Data Size: {} bytes", snf.data_size)).size(12),
    ]
    .spacing(4)
    .padding([8, 12]);

    // Waveform canvas
    let waveform = container(waveform_canvas(&editor.waveform))
        .width(Fill)
        .height(Length::Fixed(120.0))
        .padding([8, 12]);

    // Playback state
    let is_playing = editor
        .playback
        .as_ref()
        .is_some_and(|p| !p.player.is_paused() && !p.player.empty());

    // Timeline: position / total with a progress bar
    let duration = snf.duration_secs();
    let pos_secs = editor
        .playback
        .as_ref()
        .map(|pb| pb.player.get_pos().as_secs_f32())
        .unwrap_or(0.0);
    let progress = if duration > 0.0 {
        (pos_secs / duration).clamp(0.0, 1.0)
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
        text(fmt_time(duration)).size(11),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding([4, 12]);

    // Playback controls
    let play_pause_btn = if is_playing {
        button(text("⏸ Pause").size(12)).on_press(Message::snf_editor(SnfEditorMessage::Pause))
    } else {
        button(text("▶ Play").size(12)).on_press(Message::snf_editor(SnfEditorMessage::Play))
    };

    let stop_btn =
        button(text("■ Stop").size(12)).on_press(Message::snf_editor(SnfEditorMessage::Stop));

    let loop_btn = button(text("↺ Loop").size(11))
        .style(if editor.is_looping {
            crate::style::active_chip
        } else {
            crate::style::chip
        })
        .on_press(Message::snf_editor(SnfEditorMessage::ToggleLoop));

    let volume_slider = slider(0.0f32..=1.0, editor.volume, |v| {
        Message::snf_editor(SnfEditorMessage::SetVolume(v))
    })
    .width(Length::Fixed(120.0));

    let controls = container(
        row![
            play_pause_btn,
            stop_btn,
            loop_btn,
            text("Vol:").size(11),
            volume_slider
        ]
        .spacing(8)
        .padding([6, 12])
        .align_y(Alignment::Center),
    )
    .width(Fill);

    let status_line: Element<'_, Message> = match &editor.export_status {
        ExportStatus::Idle => Space::new().height(Length::Fixed(0.0)).into(),
        ExportStatus::Done(p) => row![text(p.as_str()).size(11)].padding([2, 12]).into(),
        ExportStatus::Error(e) => row![text(e.as_str()).size(11)].padding([2, 12]).into(),
    };

    let content: Element<'_, Message> = column![
        header,
        meta,
        waveform,
        timeline,
        controls,
        status_line,
    ]
    .spacing(0)
    .height(Fill)
    .accessible_label("SNF audio editor")
    .into();

    // Wrap in toast Manager — toasts auto-dismiss after 4 seconds
    toast::Manager::new(content, &editor.toasts, |i| {
        Message::snf_editor(SnfEditorMessage::DismissToast(i))
    })
    .timeout(toast::DEFAULT_TIMEOUT)
    .into()
}
