use crate::app::App;
use crate::components::modal::modal;
use crate::message::editor::tileset::{TileExportFormat, TilesetEditorMessage};
use crate::message::{Message, MessageExt};
use crate::state::tileset_editor::{
    ExportDialogState, ExportStatus, TilesetEditorState, TilesetFileType,
};
use crate::style;
use crate::utils::horizontal_rule;
use iced::widget::{
    button, column, container, image, responsive, row, scrollable, slider, text, Space,
};
use iced::{Alignment, Element, Fill, Length, Size};

fn te(m: TilesetEditorMessage) -> Message {
    Message::tileset_editor(m)
}

impl App {
    pub fn view_tileset_editor_tab(&self) -> Element<'_, Message> {
        let tab_id = self
            .state
            .workspace
            .active()
            .map(|t| t.id)
            .unwrap_or(usize::MAX);

        let Some(editor) = self.state.tileset_editors.get(&tab_id) else {
            return container(text("Tileset not loaded").size(14))
                .width(Fill)
                .height(Fill)
                .padding(16)
                .into();
        };

        if let Some(ref err) = editor.error {
            return container(
                column![
                    text("Failed to load tileset").size(14),
                    text(err.as_str()).size(12).style(style::subtle_text),
                ]
                .spacing(8),
            )
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into();
        }

        let base = view_main(editor);

        if let Some(ref dlg) = editor.export_dialog {
            modal(
                base,
                view_export_dialog(dlg),
                || te(TilesetEditorMessage::CloseExportDialog),
                0.5,
            )
        } else {
            base
        }
    }
}

// ── Main content ──────────────────────────────────────────────────────────────

fn view_main(editor: &TilesetEditorState) -> Element<'_, Message> {
    column![
        view_header(editor),
        horizontal_rule(1),
        view_tile_grid(editor),
    ]
    .spacing(0)
    .height(Fill)
    .into()
}

// ── Header ────────────────────────────────────────────────────────────────────

fn view_header(editor: &TilesetEditorState) -> Element<'_, Message> {
    let type_label = match editor.file_type {
        TilesetFileType::Gtl => "GTL",
        TilesetFileType::Btl => "BTL",
    };
    let zoom_pct = (editor.zoom * 100.0).round() as u32;

    let export_btn = button(text("Export…").size(12))
        .on_press(te(TilesetEditorMessage::ShowExportDialog))
        .padding([4, 10])
        .style(style::export_button);

    container(
        row![
            text(editor.name.to_uppercase())
                .size(13)
                .style(style::primary_text),
            text(type_label).size(11).style(style::subtle_text),
            text(format!("{} tiles", editor.tiles.len()))
                .size(11)
                .style(style::subtle_text),
            Space::new().width(Fill),
            text(format!("Zoom: {}%", zoom_pct)).size(11),
            slider(0.5f32..=4.0, editor.zoom, |z| te(
                TilesetEditorMessage::SetZoom(z)
            ))
            .width(120),
            export_btn,
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .padding([6, 12]),
    )
    .width(Fill)
    .into()
}

// ── Tile grid ─────────────────────────────────────────────────────────────────

fn view_tile_grid(editor: &TilesetEditorState) -> Element<'_, Message> {
    if editor.tiles.is_empty() {
        return container(text("No tiles found").size(12).style(style::subtle_text))
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .into();
    }

    let tile_w = (62.0 * editor.zoom).round() as u32;
    let tile_h = (32.0 * editor.zoom).round() as u32;
    let cell_w = tile_w + 4;
    let tiles: Vec<iced::widget::image::Handle> =
        editor.tiles.iter().map(|t| t.image.clone()).collect();

    let grid: Element<'_, Message> = responsive(move |size: Size| {
        let cols = ((size.width - 16.0) / cell_w as f32).max(1.0) as usize;

        let rows: Vec<Element<'_, Message>> = tiles
            .chunks(cols)
            .enumerate()
            .map(|(row_idx, chunk)| {
                let cells: Vec<Element<'_, Message>> = chunk
                    .iter()
                    .enumerate()
                    .map(|(col_idx, handle)| {
                        let idx = row_idx * cols + col_idx;
                        button(
                            column![
                                image(handle.clone())
                                    .width(Length::Fixed(tile_w as f32))
                                    .height(Length::Fixed(tile_h as f32)),
                                text(format!("#{}", idx)).size(9).style(style::subtle_text),
                            ]
                            .spacing(2)
                            .align_x(Alignment::Center),
                        )
                        .padding(2)
                        .style(style::chip)
                        .into()
                    })
                    .collect();
                row(cells).spacing(2).into()
            })
            .collect();

        column(rows).spacing(2).padding([8, 8]).into()
    })
    .into();

    scrollable(grid).width(Fill).height(Fill).into()
}

// ── Export dialog ─────────────────────────────────────────────────────────────

fn view_export_dialog(dlg: &ExportDialogState) -> Element<'_, Message> {
    let title = text("Export Tiles").size(14).style(style::primary_text);

    let fmt_tiles = button(text("Separate Files").size(12))
        .on_press(te(TilesetEditorMessage::SetExportFormat(
            TileExportFormat::SeparateTiles,
        )))
        .padding([5, 12])
        .style(if dlg.format == TileExportFormat::SeparateTiles {
            style::active_chip
        } else {
            style::chip
        });

    let fmt_atlas = button(text("Atlas PNG").size(12))
        .on_press(te(TilesetEditorMessage::SetExportFormat(
            TileExportFormat::Atlas,
        )))
        .padding([5, 12])
        .style(if dlg.format == TileExportFormat::Atlas {
            style::active_chip
        } else {
            style::chip
        });

    let format_hint = match dlg.format {
        TileExportFormat::SeparateTiles => "One PNG per tile in a subdirectory",
        TileExportFormat::Atlas => "All tiles packed into a single 48-column PNG",
    };

    let dir_label = text(
        dlg.export_dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "No folder selected".to_string()),
    )
    .size(11)
    .style(style::subtle_text);

    let choose_dir = button(text("Choose Folder…").size(12))
        .on_press(te(TilesetEditorMessage::ChooseExportDir))
        .padding([4, 10])
        .style(style::chip);

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
            .on_press(te(TilesetEditorMessage::ExportConfirm))
            .padding([5, 16])
            .style(style::export_button)
    } else {
        button(text("Export").size(12))
            .padding([5, 16])
            .style(style::chip)
    };

    let cancel_btn = button(text("Cancel").size(12))
        .on_press(te(TilesetEditorMessage::CloseExportDialog))
        .padding([5, 12])
        .style(style::playback_button);

    let body = column![
        title,
        horizontal_rule(1),
        text("Format").size(11).style(style::subtle_text),
        row![fmt_tiles, fmt_atlas].spacing(8),
        text(format_hint).size(10).style(style::subtle_text),
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
    .width(Length::Fixed(400.0));

    container(body).style(style::export_dialog_container).into()
}
