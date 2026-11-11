//! View for the Save.ifo editor.

use super::message::SaveIfoEditorMessage;
use super::state::{EditorData, SaveIfoEditorState};
use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::components::utils::horizontal_space;
use crate::message::{Message, MessageExt};
use crate::style;
use gui_widgets::components::modal::modal;
use gui_widgets::lucide::{LUCIDE_FONT, icon_char};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};
use lucide_icons::Icon;

const SLOT_COUNT: usize = 6;

pub fn view(app: &App) -> Element<'_, Message> {
    let editor = &app.state.editors.save_ifo_editor;
    let base: Element<'_, Message> = match &editor.loading_state {
        LoadingState::Failed(error) => failed_view(error),
        LoadingState::Loading => hint_view("Loading Save.ifo…"),
        _ => match &editor.data {
            Some(data) => loaded_view(data, editor),
            None => hint_view(
                "No Save.ifo loaded. Pick a game folder first, then reopen this tab to inspect its save slots.",
            ),
        },
    };

    match editor.pending_swap {
        Some((a, b)) => modal(
            base,
            swap_confirm_modal(a, b),
            || Message::save_ifo(SaveIfoEditorMessage::SwapCancel),
            0.6,
        ),
        None => base,
    }
}

fn loaded_view<'a>(data: &'a EditorData, editor: &'a SaveIfoEditorState) -> Element<'a, Message> {
    let occupied = data.summaries.iter().filter(|s| s.occupied).count();

    let header = row![
        column![
            text("Save slots").size(18),
            text(format!("{occupied} of {SLOT_COUNT} slots used"))
                .size(12)
                .style(style::subtle_text),
        ]
        .spacing(2),
        horizontal_space(),
        dirty_indicator(data.dirty),
        save_button(data.dirty),
    ]
    .spacing(8)
    .align_y(iced::alignment::Vertical::Center);

    let content = column![
        header,
        slot_table(&data.summaries),
        container(column![].height(6.0)),
        tail_section(editor),
    ]
    .spacing(14)
    .padding(12);

    let body = column![
        scrollable(content).width(Length::Fill).height(Length::Fill),
        status_bar(editor),
    ]
    .height(Length::Fill);

    body.into()
}

fn dirty_indicator(dirty: bool) -> Element<'static, Message> {
    if dirty {
        row![
            text(icon_char(Icon::Dot)).font(LUCIDE_FONT).size(12),
            text("unsaved changes").style(style::subtle_text),
        ]
        .spacing(4)
        .align_y(iced::alignment::Vertical::Center)
        .into()
    } else {
        horizontal_space().into()
    }
}

fn save_button(dirty: bool) -> Element<'static, Message> {
    button(text("Save"))
        .padding([4, 14])
        .style(style::commit_button)
        .on_press_maybe(dirty.then(|| Message::save_ifo(SaveIfoEditorMessage::Save)))
        .into()
}

fn slot_table(summaries: &[dispel_core::SlotSummary]) -> Element<'_, Message> {
    let mut table = column![slot_header_row()].spacing(2);
    for summary in summaries.iter().take(SLOT_COUNT) {
        table = table.push(slot_row(summary.index, summary));
    }
    container(table).into()
}

fn header_cell(label: &str, width: f32) -> Element<'_, Message> {
    container(text(label).style(style::subtle_text))
        .width(width)
        .into()
}

fn slot_header_row() -> Element<'static, Message> {
    row![
        header_cell("Slot", 40.0),
        header_cell("Used", 50.0),
        header_cell("Save file", 90.0),
        header_cell("Saved at", 110.0),
        header_cell("Tmp key", 70.0),
        header_cell("Map id", 70.0),
        header_cell("Swap", 80.0),
    ]
    .spacing(6)
    .into()
}

fn cell(value: String, width: f32) -> Element<'static, Message> {
    container(text(value)).width(width).into()
}

fn icon_cell(icon: Icon, width: f32) -> Element<'static, Message> {
    container(text(icon_char(icon)).font(LUCIDE_FONT))
        .width(width)
        .into()
}

fn saved_at_text(summary: &dispel_core::SlotSummary) -> String {
    if !summary.occupied {
        return "—".into();
    }
    format!(
        "{:02}/{:02} {:02}:{:02}",
        summary.month, summary.day, summary.hour, summary.minute
    )
}

fn sav_cell(summary: &dispel_core::SlotSummary) -> Element<'static, Message> {
    if summary.sav_present {
        icon_cell(Icon::Check, 90.0)
    } else if summary.occupied {
        container(
            row![
                text(icon_char(Icon::AlertTriangle))
                    .font(LUCIDE_FONT)
                    .size(11),
                text("missing").size(11),
            ]
            .spacing(3),
        )
        .width(90.0)
        .into()
    } else {
        cell("—".into(), 90.0)
    }
}

fn swap_button(icon: Icon, from: usize, to: usize) -> Element<'static, Message> {
    let request = (to < SLOT_COUNT && from != to)
        .then(|| Message::save_ifo(SaveIfoEditorMessage::SwapRequested(from, to)));
    button(text(icon_char(icon)).font(LUCIDE_FONT).size(11))
        .padding([2, 7])
        .style(style::browse_button)
        .on_press_maybe(request)
        .into()
}

fn slot_row(index: usize, summary: &dispel_core::SlotSummary) -> Element<'static, Message> {
    let opt = |value: Option<u32>| value.map_or_else(|| "—".to_string(), |v| v.to_string());
    row![
        cell(index.to_string(), 40.0),
        icon_cell(
            if summary.occupied {
                Icon::CircleDot
            } else {
                Icon::Circle
            },
            50.0,
        ),
        sav_cell(summary),
        cell(saved_at_text(summary), 110.0),
        cell(opt(summary.game_tmp_key), 70.0),
        cell(opt(summary.map_id), 70.0),
        row![
            swap_button(Icon::ChevronUp, index, index.wrapping_sub(1)),
            swap_button(Icon::ChevronDown, index, index + 1),
        ]
        .spacing(4),
    ]
    .spacing(6)
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

fn labeled_input<'a>(label: &'a str, value: &'a str, path: &'static str) -> Element<'a, Message> {
    row![
        container(text(label).style(style::subtle_text)).width(140.0),
        text_input("", value)
            .on_input(
                move |v| Message::save_ifo(SaveIfoEditorMessage::FieldChanged(path.to_string(), v))
            )
            .width(170.0),
    ]
    .spacing(8)
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

fn tail_section(editor: &SaveIfoEditorState) -> Element<'_, Message> {
    let buffers = &editor.tail_buffers;
    let counts = row![
        container(text("Payload counts").style(style::subtle_text)).width(140.0),
        text_input("", &buffers.payload_counts[0])
            .on_input(|v| count_msg(0, v))
            .width(90.0),
        text_input("", &buffers.payload_counts[1])
            .on_input(|v| count_msg(1, v))
            .width(90.0),
        text_input("", &buffers.payload_counts[2])
            .on_input(|v| count_msg(2, v))
            .width(90.0),
        text_input("", &buffers.payload_counts[3])
            .on_input(|v| count_msg(3, v))
            .width(90.0),
    ]
    .spacing(8)
    .align_y(iced::alignment::Vertical::Center);

    column![
        text("Global state").size(16),
        labeled_input("Game version", &buffers.game_version, "tail.game_version"),
        labeled_input("Tmp key", &buffers.game_tmp_key, "tail.game_tmp_key"),
        labeled_input("Map id", &buffers.map_id, "tail.map_id"),
        labeled_input("Reserved", &buffers.reserved, "tail.reserved"),
        counts,
    ]
    .spacing(8)
    .into()
}

fn count_msg(index: usize, value: String) -> Message {
    Message::save_ifo(SaveIfoEditorMessage::FieldChanged(
        format!("tail.payload_counts.{index}"),
        value,
    ))
}

fn status_bar(editor: &SaveIfoEditorState) -> Element<'_, Message> {
    container(
        row![
            dirty_indicator(editor.data.as_ref().is_some_and(|d| d.dirty)),
            horizontal_space(),
            text(&editor.status_msg).style(style::subtle_text),
        ]
        .spacing(8),
    )
    .padding([4, 10])
    .width(Length::Fill)
    .style(style::status_bar)
    .into()
}

fn swap_confirm_modal(a: usize, b: usize) -> Element<'static, Message> {
    let content = column![
        text(format!("Swap save slots {a} and {b}?")).size(16),
        text(
            "The two .sav files exchange positions immediately and the\n\
             slot metadata in Save.ifo is updated to match.\n\
             This cannot be undone with Ctrl+Z."
        )
        .style(style::subtle_text),
        row![
            button(text("Cancel"))
                .padding([4, 14])
                .style(style::browse_button)
                .on_press(Message::save_ifo(SaveIfoEditorMessage::SwapCancel)),
            button(text("Swap"))
                .padding([4, 14])
                .style(style::commit_button)
                .on_press(Message::save_ifo(SaveIfoEditorMessage::SwapConfirm)),
        ]
        .spacing(8),
    ]
    .spacing(10);

    container(content)
        .padding(16)
        .width(430.0)
        .style(style::modal_container)
        .into()
}

fn hint_view(message: &str) -> Element<'_, Message> {
    container(text(message).style(style::subtle_text))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .padding(24)
        .into()
}

fn failed_view(error: &str) -> Element<'_, Message> {
    container(
        column![
            text(format!("Failed to load Save.ifo: {error}")),
            button(text("Retry"))
                .padding([4, 14])
                .style(style::browse_button)
                .on_press(Message::save_ifo(SaveIfoEditorMessage::LoadCatalog)),
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .center_x(Length::Fill)
    .padding(24)
    .into()
}
