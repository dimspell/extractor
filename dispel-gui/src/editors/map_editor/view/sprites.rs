use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill};

use crate::components::utils::{horizontal_rule, horizontal_space};
use crate::message::{Message, MessageExt};
use crate::style;
use gui_widgets::components::modal::modal;

use super::super::message::MapEditorMessage;
use super::super::state::MapEditorState;
use super::super::state::{SpriteExportDialogState, SpriteExportStatus};

// ── Sprite browser ────────────────────────────────────────────────────────────

pub fn view_sprite_browser<'a>(
    state: &'a MapEditorState,
    tab_id: usize,
) -> Element<'a, Message> {
    use iced::widget::image;
    use iced::Length::Fixed;

    let handles = &state.data.sprite_sequence_handles;

    if handles.is_empty() {
        return container(
            text("No embedded sprites in this map.")
                .size(12)
                .style(style::subtle_text),
        )
        .padding(24)
        .into();
    }

    let selected = state.view.selected_sprite_sequence;

    let thumbnails: Vec<Element<'_, Message>> = handles
        .iter()
        .map(|s| {
            let thumb = column![
                image(s.handle.clone())
                    .width(Fixed(64.0))
                    .height(Fixed(64.0)),
                text(format!("#{}", s.sequence_idx)).size(10),
                text(format!("{}×{}", s.width, s.height))
                    .size(10)
                    .style(style::subtle_text),
                text(format!("×{} placed", s.placement_count))
                    .size(10)
                    .style(style::subtle_text),
            ]
            .spacing(2)
            .align_x(iced::Alignment::Center);

            let is_selected = selected == Some(s.sequence_idx);
            button(thumb)
                .on_press(Message::map_editor(MapEditorMessage::SelectSpriteSequence(
                    tab_id,
                    if is_selected {
                        None
                    } else {
                        Some(s.sequence_idx)
                    },
                )))
                .padding(6)
                .style(if is_selected {
                    style::active_chip
                } else {
                    style::chip
                })
                .into()
        })
        .collect();

    let header = row![
        text(format!(
            "{} sprite sequence{}",
            handles.len(),
            if handles.len() == 1 { "" } else { "s" }
        ))
        .size(11)
        .style(style::subtle_text),
        horizontal_space(),
        button(text("Export…").size(11))
            .on_press(Message::map_editor(
                MapEditorMessage::ShowSpriteExportDialog(tab_id),
            ))
            .padding([3, 8])
            .style(style::export_button),
    ]
    .padding([4, 16])
    .align_y(iced::Alignment::Center)
    .width(Fill);

    let grid: Element<'_, Message> = scrollable(
        column![
            container(text("Sprites").size(14).style(style::subtle_text)).padding([0, 16]),
            row(thumbnails).spacing(8).padding([8, 16]).wrap(),
        ]
        .spacing(4),
    )
    .width(Fill)
    .height(Fill)
    .into();

    let detail: Element<'_, Message> = if let Some(idx) = selected {
        if let Some(s) = handles.iter().find(|h| h.sequence_idx == idx) {
            let placement_items: Vec<Element<'_, Message>> = s
                .placements
                .iter()
                .map(|(x, y)| text(format!("  ({x}, {y})")).size(11).into())
                .collect();

            scrollable(
                column![
                    text(format!(
                        "Sprite #{} — {}×{}px — {} placement{}",
                        s.sequence_idx,
                        s.width,
                        s.height,
                        s.placement_count,
                        if s.placement_count == 1 { "" } else { "s" },
                    ))
                    .size(12),
                    column(placement_items).spacing(2),
                ]
                .spacing(8)
                .padding([8, 16]),
            )
            .width(Fill)
            .height(Fill)
            .into()
        } else {
            text("").into()
        }
    } else {
        container(
            text("Select a sprite to see placements.")
                .size(11)
                .style(style::subtle_text),
        )
        .padding([8, 16])
        .into()
    };

    // Split pane: grid on left (70%), detail on right (30%)
    let split_content: Element<'_, Message> = row![
        container(grid)
            .width(iced::Length::FillPortion(7))
            .height(Fill),
        container(detail)
            .width(iced::Length::FillPortion(3))
            .height(Fill),
    ]
    .width(Fill)
    .height(Fill)
    .spacing(0)
    .into();

    let base: Element<'_, Message> = column![header, split_content]
        .spacing(0)
        .width(Fill)
        .height(Fill)
        .into();

    // Wrap in sprite export dialog modal if open
    let base = if let Some(ref dlg) = state.data.sprite_export_dialog {
        modal(
            base,
            view_sprite_export_dialog(dlg, tab_id),
            move || Message::map_editor(MapEditorMessage::CloseSpriteExportDialog(tab_id)),
            0.5,
        )
    } else {
        base
    };

    base
}

fn view_sprite_export_dialog<'a>(
    dlg: &'a SpriteExportDialogState,
    tab_id: usize,
) -> Element<'a, Message> {
    let title = text("Export Map Sprites")
        .size(14)
        .style(style::primary_text);

    let dir_label = if let Some(ref p) = dlg.export_dir {
        text(p.display().to_string()).size(11)
    } else {
        text("No folder selected")
            .size(11)
            .style(style::subtle_text)
    };

    let choose_btn = button(text("Choose Folder…").size(11))
        .on_press(Message::map_editor(
            MapEditorMessage::ChooseSpriteExportDir(tab_id),
        ))
        .padding([4, 10]);

    let can_export = dlg.export_dir.is_some() && dlg.status != SpriteExportStatus::Exporting;

    let export_btn = if can_export {
        button(text("Export").size(12))
            .on_press(Message::map_editor(MapEditorMessage::ConfirmSpriteExport(
                tab_id,
            )))
            .padding([5, 16])
            .style(style::export_button)
    } else {
        button(text("Export").size(12)).padding([5, 16])
    };

    let cancel_btn = button(text("Cancel").size(12))
        .on_press(Message::map_editor(
            MapEditorMessage::CloseSpriteExportDialog(tab_id),
        ))
        .padding([5, 16]);

    let status_row: Element<'_, Message> = match &dlg.status {
        SpriteExportStatus::Idle => text("").size(11).into(),
        SpriteExportStatus::Exporting => {
            text("Exporting…").size(11).style(style::subtle_text).into()
        }
        SpriteExportStatus::Done(msg) => text(msg.as_str()).size(11).into(),
        SpriteExportStatus::Error(e) => text(e.as_str())
            .size(11)
            .color(iced::Color::from_rgb(0.8, 0.2, 0.2))
            .into(),
    };

    container(
        column![
            title,
            horizontal_rule(1),
            text("Output folder:").size(11).style(style::subtle_text),
            row![dir_label, horizontal_space(), choose_btn]
                .align_y(iced::Alignment::Center)
                .spacing(8),
            horizontal_rule(1),
            status_row,
            row![cancel_btn, export_btn].spacing(8),
        ]
        .spacing(12)
        .padding(20)
        .width(iced::Length::Fixed(400.0)),
    )
    .style(style::toolbar_container)
    .into()
}
