use crate::app::App;
use crate::components::utils::horizontal_rule;
use crate::editors::sprite_editor::{
    ExportDialogState, ExportFormat, ExportStatus, SpriteFrame, SpriteViewerMessage,
    SpriteViewerState,
};
use crate::message::{Message, MessageExt};
use crate::style;
use gui_widgets::components::context_menu::{ContextMenu, Entry};
use gui_widgets::components::modal::modal;
use gui_widgets::lucide::{icon_char, LUCIDE_FONT};
use iced::widget::{button, column, container, image, row, scrollable, slider, text, Space};
use iced::{Alignment, Element, Fill, Length};
use lucide_icons::Icon;

// Shorthand to wrap a SpriteViewerMessage into the top-level Message type.
fn sv(m: SpriteViewerMessage) -> Message {
    Message::sprite_viewer(m)
}

pub fn view(app: &App) -> Element<'_, Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);

    let Some(viewer) = app.state.editors.sprite_viewers.get(&tab_id) else {
        return container(text("Sprite not loaded").size(14))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .accessible_label("Sprite viewer")
            .into();
    };

    if let Some(ref err) = viewer.error {
        return container(
            column![
                text("Failed to load sprite").size(14),
                text(err).size(12).style(style::subtle_text),
            ]
            .spacing(8),
        )
        .width(Fill)
        .height(Fill)
        .padding(16)
        .accessible_label("Sprite viewer")
        .into();
    }

    let base = view_main(viewer);

    // Overlay the export dialog if it's open.
    if let Some(ref dlg) = viewer.export_dialog {
        modal(
            base,
            view_export_dialog(dlg),
            || sv(SpriteViewerMessage::CloseExportDialog),
            0.5,
        )
    } else {
        base
    }
}

// ── Main content ──────────────────────────────────────────────────────────────

fn view_main(viewer: &SpriteViewerState) -> Element<'_, Message> {
    let header = view_header(viewer);
    let sequence_row = view_sequence_chips(viewer);
    let preview = view_preview(viewer);
    let zoom_controls = view_zoom_controls(viewer);
    let edit_toolbar = view_edit_toolbar(viewer);
    let frame_strip = view_frame_strip(viewer);
    let scrubber = view_scrubber(viewer);
    let controls = view_playback_controls(viewer);

    column![
        header,
        horizontal_rule(1),
        sequence_row,
        horizontal_rule(1),
        preview,
        zoom_controls,
        horizontal_rule(1),
        frame_strip,
        edit_toolbar,
        horizontal_rule(1),
        scrubber,
        horizontal_rule(1),
        controls,
    ]
    .spacing(0)
    .height(Fill)
    .accessible_label("Sprite viewer")
    .into()
}

// ── Header bar ────────────────────────────────────────────────────────────────

fn view_header(viewer: &SpriteViewerState) -> Element<'_, Message> {
    let title: Element<'_, Message> = if viewer.dirty {
        row![
            text(&viewer.name).size(13).style(style::primary_text),
            text(" *").size(13).style(style::primary_text),
        ]
        .spacing(0)
        .align_y(Alignment::Center)
        .into()
    } else {
        text(&viewer.name)
            .size(13)
            .style(style::primary_text)
            .into()
    };

    // Undo/Redo buttons
    let undo_btn = button(text("↩").size(12))
        .on_press_maybe(if viewer.can_undo() {
            Some(sv(SpriteViewerMessage::Undo))
        } else {
            None
        })
        .padding([3, 7])
        .style(style::playback_button);

    let redo_btn = button(text("↪").size(12))
        .on_press_maybe(if viewer.can_redo() {
            Some(sv(SpriteViewerMessage::Redo))
        } else {
            None
        })
        .padding([3, 7])
        .style(style::playback_button);

    // Save button
    let save_label = if viewer.dirty { "Save *" } else { "Save" };
    let save_btn = button(text(save_label).size(12))
        .on_press(sv(SpriteViewerMessage::Save))
        .padding([4, 10])
        .style(style::export_button);

    let export_btn = button(text("Export…").size(12))
        .on_press(sv(SpriteViewerMessage::ShowExportDialog))
        .padding([4, 10])
        .style(style::export_button);

    container(
        row![
            title,
            Space::new().width(Fill),
            undo_btn,
            redo_btn,
            Space::new().width(4),
            save_btn,
            export_btn,
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([6, 12])
    .width(Fill)
    .into()
}

// ── Sequence chips ────────────────────────────────────────────────────────────

fn view_sequence_chips(viewer: &SpriteViewerState) -> Element<'_, Message> {
    use iced::widget::responsive;
    use iced::Size;

    let count = viewer.sequence_count;
    if count == 0 {
        return container(text("No sequences").size(11).style(style::subtle_text))
            .padding([4, 8])
            .into();
    }

    let selected = viewer.selected_sequence;
    let frame_counts = viewer.frame_counts.clone();

    let chips: Element<'_, Message> = responsive(move |size: Size| {
        let available = size.width - 8.0;
        let cols = (available / 110.0).max(1.0) as usize;
        let rows: Vec<Element<'_, Message>> = (0..count)
            .collect::<Vec<_>>()
            .chunks(cols)
            .map(|chunk| {
                let btns: Vec<Element<'_, Message>> = chunk
                    .iter()
                    .map(|&i| {
                        let fc = frame_counts.get(i).copied().unwrap_or(0);
                        button(text(format!("Seq {} ({}f)", i, fc)).size(11))
                            .padding([3, 8])
                            .on_press(sv(SpriteViewerMessage::SelectSequence(i)))
                            .style(if selected == i {
                                style::active_chip
                            } else {
                                style::chip
                            })
                            .into()
                    })
                    .collect();
                row(btns).spacing(4).into()
            })
            .collect();
        iced::Element::new(column(rows).spacing(4))
    })
    .into();

    scrollable(container(chips).padding([4, 8]))
        .height(Length::Fixed(72.0))
        .width(Fill)
        .into()
}

// ── Main preview ──────────────────────────────────────────────────────────────

fn view_preview(viewer: &SpriteViewerState) -> Element<'_, Message> {
    let inner: Element<'_, Message> =
        if let Some(frame) = viewer.frames.get(viewer.selected_frame_global()) {
            let w = (frame.width as f32 * viewer.zoom).ceil();
            let h = (frame.height as f32 * viewer.zoom).ceil();
            image(frame.image.clone())
                .width(Length::Fixed(w))
                .height(Length::Fixed(h))
                .into()
        } else {
            text("No frames loaded")
                .size(12)
                .style(style::subtle_text)
                .into()
        };

    container(inner)
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .center_y(Fill)
        .padding(8)
        .into()
}

// ── Zoom controls ─────────────────────────────────────────────────────────────

fn view_zoom_controls(viewer: &SpriteViewerState) -> Element<'_, Message> {
    let zoom_in = button(text("+").size(12))
        .on_press(sv(SpriteViewerMessage::ZoomIn))
        .padding([2, 7])
        .style(style::chip);

    let zoom_out = button(text("−").size(12))
        .on_press(sv(SpriteViewerMessage::ZoomOut))
        .padding([2, 7])
        .style(style::chip);

    let zoom_reset = button(text("1:1").size(10))
        .on_press(sv(SpriteViewerMessage::ZoomReset))
        .padding([2, 6])
        .style(style::chip);

    let zoom_fit = button(text("Fit").size(10))
        .on_press(sv(SpriteViewerMessage::ZoomToFit))
        .padding([2, 6])
        .style(style::chip);

    let zoom_label = text(format!("{}%", (viewer.zoom * 100.0).round() as u32))
        .size(11)
        .style(style::subtle_text);

    container(
        row![
            zoom_out,
            zoom_in,
            Space::new().width(4),
            zoom_reset,
            zoom_fit,
            Space::new().width(4),
            zoom_label,
        ]
        .spacing(2)
        .padding([2, 8])
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .into()
}

// ── Frame thumbnail strip ─────────────────────────────────────────────────────

fn view_frame_strip(viewer: &SpriteViewerState) -> Element<'_, Message> {
    if viewer.frames.is_empty() {
        return Space::new().into();
    }

    let seq_idx = viewer.selected_sequence;
    let selected = viewer.selected_frame;
    let sequence_frames: Vec<&SpriteFrame> = viewer
        .frames
        .iter()
        .filter(|f| f.sequence_idx == seq_idx)
        .collect();

    let max_frames = sequence_frames.len();
    let thumbs: Vec<Element<'_, Message>> = sequence_frames
        .iter()
        .enumerate()
        .map(|(i, frame)| {
            let thumb_btn: Element<'_, Message> = button(
                image(frame.image.clone())
                    .width(Length::Fixed(48.0))
                    .height(Length::Fixed(48.0)),
            )
            .padding(2)
            .on_press(sv(SpriteViewerMessage::SelectFrame(i)))
            .style(if selected == i {
                style::active_chip
            } else {
                style::chip
            })
            .into();

            // Right-click context menu on each thumbnail
            let context_entries = vec![
                Entry::item("Insert Frame", sv(SpriteViewerMessage::InsertFrame)),
                Entry::item("Duplicate Frame", sv(SpriteViewerMessage::DuplicateFrame)),
                Entry::separator(),
                if max_frames > 1 {
                    Entry::item("Delete Frame", sv(SpriteViewerMessage::DeleteFrame))
                } else {
                    Entry::disabled("Delete Frame")
                },
            ];

            ContextMenu::new(thumb_btn, context_entries).into()
        })
        .collect();

    scrollable(
        row(thumbs)
            .spacing(4)
            .padding([4, 8])
            .align_y(Alignment::Center),
    )
    .direction(scrollable::Direction::Horizontal(
        scrollable::Scrollbar::new(),
    ))
    .width(Fill)
    .height(Length::Fixed(64.0))
    .into()
}

// ── Editing toolbar ───────────────────────────────────────────────────────────

fn view_edit_toolbar(viewer: &SpriteViewerState) -> Element<'_, Message> {
    let has_frames = viewer.sprite_file.is_some()
        && viewer
            .frame_counts
            .get(viewer.selected_sequence)
            .copied()
            .unwrap_or(0)
            > 0;

    let insert_btn = if has_frames {
        button(text("+ Insert").size(11))
            .on_press(sv(SpriteViewerMessage::InsertFrame))
            .padding([3, 8])
            .style(style::chip)
    } else {
        button(text("+ Insert").size(11))
            .padding([3, 8])
            .style(style::chip)
    };

    let dup_btn = if has_frames {
        button(text("⧉ Dup").size(11))
            .on_press(sv(SpriteViewerMessage::DuplicateFrame))
            .padding([3, 8])
            .style(style::chip)
    } else {
        button(text("⧉ Dup").size(11))
            .padding([3, 8])
            .style(style::chip)
    };

    let del_btn = if has_frames {
        button(
            row![
                text(icon_char(Icon::X)).font(LUCIDE_FONT).size(11),
                text(" Del").size(11)
            ]
            .spacing(2),
        )
        .on_press(sv(SpriteViewerMessage::DeleteFrame))
        .padding([3, 8])
        .style(style::chip)
    } else {
        button(
            row![
                text(icon_char(Icon::X)).font(LUCIDE_FONT).size(11),
                text(" Del").size(11)
            ]
            .spacing(2),
        )
        .padding([3, 8])
        .style(style::chip)
    };

    // Move left/right buttons
    let move_left_btn = if has_frames && viewer.selected_frame > 0 {
        button(text(icon_char(Icon::ChevronLeft)).font(LUCIDE_FONT).size(11))
            .on_press(sv(SpriteViewerMessage::MoveFrameLeft))
            .padding([3, 7])
            .style(style::chip)
    } else {
        button(text(icon_char(Icon::ChevronLeft)).font(LUCIDE_FONT).size(11))
            .padding([3, 7])
            .style(style::chip)
    };

    let move_right_btn = if has_frames {
        let max_frames = viewer
            .frame_counts
            .get(viewer.selected_sequence)
            .copied()
            .unwrap_or(0);
        if viewer.selected_frame + 1 < max_frames {
            button(text(icon_char(Icon::ChevronRight)).font(LUCIDE_FONT).size(11))
                .on_press(sv(SpriteViewerMessage::MoveFrameRight))
                .padding([3, 7])
                .style(style::chip)
        } else {
            button(text(icon_char(Icon::ChevronRight)).font(LUCIDE_FONT).size(11))
                .padding([3, 7])
                .style(style::chip)
        }
    } else {
        button(text(icon_char(Icon::ChevronRight)).font(LUCIDE_FONT).size(11))
            .padding([3, 7])
            .style(style::chip)
    };

    let import_btn = if viewer.sprite_file.is_some() {
        button(text("📷 Import…").size(11))
            .on_press(sv(SpriteViewerMessage::ImportPngFrame))
            .padding([3, 8])
            .style(style::export_button)
    } else {
        button(text("📷 Import…").size(11))
            .padding([3, 8])
            .style(style::chip)
    };

    container(
        row![
            insert_btn,
            dup_btn,
            del_btn,
            Space::new().width(4),
            move_left_btn,
            move_right_btn,
            Space::new().width(Fill),
            import_btn,
        ]
        .spacing(4)
        .padding([4, 8])
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .into()
}

// ── Timeline scrubber ─────────────────────────────────────────────────────────

fn view_scrubber(viewer: &SpriteViewerState) -> Element<'_, Message> {
    let seq_frames = viewer.frames_in_sequence();
    if seq_frames <= 1 {
        return Space::new().into();
    }

    let max = (seq_frames - 1) as u32;
    let current = viewer.selected_frame as u32;

    let scrub = slider(0u32..=max, current, |v| {
        sv(SpriteViewerMessage::ScrubTo(v as usize))
    })
    .width(Fill);

    let label = text(format!(
        "Frame {}/{} (seq {})",
        viewer.selected_frame + 1,
        seq_frames,
        viewer.selected_sequence
    ))
    .size(11)
    .style(style::subtle_text);

    container(
        row![scrub, label]
            .spacing(8)
            .padding([4, 12])
            .align_y(Alignment::Center),
    )
    .width(Fill)
    .into()
}

// ── Playback controls ─────────────────────────────────────────────────────────

fn view_playback_controls(viewer: &SpriteViewerState) -> Element<'_, Message> {
    // Transport buttons
    let step_back = button(text(icon_char(Icon::ChevronLeft)).font(LUCIDE_FONT).size(12))
        .on_press(sv(SpriteViewerMessage::StepBack))
        .padding([4, 8])
        .style(style::playback_button);

    let play_pause = if viewer.is_playing {
        button(text(icon_char(Icon::Pause)).font(LUCIDE_FONT).size(14))
            .on_press(sv(SpriteViewerMessage::Pause))
            .padding([4, 10])
            .style(style::playback_button)
    } else {
        button(text(icon_char(Icon::Play)).font(LUCIDE_FONT).size(14))
            .on_press(sv(SpriteViewerMessage::Play))
            .padding([4, 10])
            .style(style::playback_button)
    };

    let step_fwd = button(
        row![
            text(icon_char(Icon::ChevronRight)).font(LUCIDE_FONT).size(12),
            text("|").size(12)
        ]
        .spacing(0),
    )
    .on_press(sv(SpriteViewerMessage::StepForward))
    .padding([4, 8])
    .style(style::playback_button);

    let loop_btn = button(
        row![
            text(icon_char(Icon::Repeat)).font(LUCIDE_FONT).size(11),
            text(" Loop").size(11)
        ]
        .spacing(2),
    )
    .on_press(sv(SpriteViewerMessage::ToggleLoop))
    .padding([4, 8])
    .style(if viewer.is_looping {
        style::playback_button_active
    } else {
        style::playback_button
    });

    // Speed buttons
    let speeds: &[(u32, &str)] = &[(25, "¼×"), (50, "½×"), (100, "1×"), (200, "2×")];
    let speed_row: Vec<Element<'_, Message>> = speeds
        .iter()
        .map(|&(val, label)| {
            button(text(label).size(11))
                .on_press(sv(SpriteViewerMessage::SetSpeed(val)))
                .padding([3, 7])
                .style(if viewer.speed_100x == val {
                    style::active_chip
                } else {
                    style::chip
                })
                .into()
        })
        .collect();

    let fps_label = text(format!("{:.0} fps", viewer.fps))
        .size(11)
        .style(style::subtle_text);

    container(
        row![
            step_back,
            play_pause,
            step_fwd,
            loop_btn,
            Space::new().width(8),
            row(speed_row).spacing(4),
            Space::new().width(Fill),
            fps_label,
        ]
        .spacing(4)
        .padding([6, 12])
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .into()
}

// ── Export dialog ─────────────────────────────────────────────────────────────

fn view_export_dialog(dlg: &ExportDialogState) -> Element<'_, Message> {
    let title = text("Export Sprite").size(14).style(style::primary_text);

    // Format selector
    let fmt_png = button(text("PNG Frames").size(12))
        .on_press(sv(SpriteViewerMessage::SetExportFormat(
            ExportFormat::PngFrames,
        )))
        .padding([5, 12])
        .style(if dlg.format == ExportFormat::PngFrames {
            style::active_chip
        } else {
            style::chip
        });

    let fmt_sheet = button(text("Sprite Sheet").size(12))
        .on_press(sv(SpriteViewerMessage::SetExportFormat(
            ExportFormat::SpriteSheet,
        )))
        .padding([5, 12])
        .style(if dlg.format == ExportFormat::SpriteSheet {
            style::active_chip
        } else {
            style::chip
        });

    // Destination folder
    let dir_label = text(
        dlg.export_dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "No folder selected".to_string()),
    )
    .size(11)
    .style(style::subtle_text);

    let choose_dir = button(text("Choose Folder…").size(12))
        .on_press(sv(SpriteViewerMessage::ChooseExportDir))
        .padding([4, 10])
        .style(style::chip);

    // Status / export button
    let (status_el, can_export): (Element<'_, Message>, bool) = match &dlg.status {
        ExportStatus::Idle => (Space::new().into(), dlg.export_dir.is_some()),
        ExportStatus::Exporting => (
            text("Exporting…").size(12).style(style::subtle_text).into(),
            false,
        ),
        ExportStatus::Done(msg) => (
            text(msg.as_str())
                .size(11)
                .style(style::section_header)
                .into(),
            true,
        ),
        ExportStatus::Error(e) => (
            text(format!("Error: {}", e))
                .size(11)
                .style(style::subtle_text)
                .into(),
            true,
        ),
    };

    let export_btn = if can_export {
        button(text("Export").size(12))
            .on_press(sv(SpriteViewerMessage::ExportConfirm))
            .padding([5, 16])
            .style(style::export_button)
    } else {
        button(text("Export").size(12))
            .padding([5, 16])
            .style(style::chip)
    };

    let cancel_btn = button(text("Cancel").size(12))
        .on_press(sv(SpriteViewerMessage::CloseExportDialog))
        .padding([5, 12])
        .style(style::playback_button);

    let body = column![
        title,
        horizontal_rule(1),
        text("Format").size(11).style(style::subtle_text),
        row![fmt_png, fmt_sheet].spacing(8),
        text("Destination").size(11).style(style::subtle_text),
        row![dir_label, Space::new().width(Fill), choose_dir]
            .spacing(8)
            .align_y(Alignment::Center),
        status_el,
        row![Space::new().width(Fill), cancel_btn, export_btn]
            .spacing(8)
            .align_y(Alignment::Center),
    ]
    .spacing(10)
    .padding(20)
    .width(Length::Fixed(380.0));

    container(body).style(style::export_dialog_container).into()
}
