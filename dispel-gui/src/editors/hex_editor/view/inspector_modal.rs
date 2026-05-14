use iced::widget::{button, column, container, row, text, text_input};
use iced::{color, Element, Font, Length};

use crate::editors::hex_editor::editing::InspectorEditState;
use crate::editors::hex_editor::inspector::ENTRIES;
use crate::editors::hex_editor::HexEditorMessage;
use crate::message::{Message, MessageExt};

/// Modal body shown when an inspector "Edit" button is pressed.
pub fn view(state: &InspectorEditState) -> Element<'_, Message> {
    let entry_name = ENTRIES.get(state.entry_idx).map(|e| e.name).unwrap_or("?");
    let title = format!("Edit {entry_name} at 0x{:X}", state.addr);

    let input = text_input("value", &state.draft)
        .on_input(|s| Message::hex_editor(HexEditorMessage::SetInspectorDraft(s)))
        .on_submit(Message::hex_editor(HexEditorMessage::CommitInspectorEdit))
        .padding(6)
        .size(13);

    let error: Element<'_, Message> = if let Some(err) = &state.error {
        text(err.clone())
            .size(11)
            .color(color!(0xff8a6e))
            .font(Font::MONOSPACE)
            .into()
    } else {
        text("").size(11).into()
    };

    let buttons = row![
        button(text("Cancel").size(12))
            .padding([4, 12])
            .on_press(Message::hex_editor(HexEditorMessage::CloseInspectorEdit)),
        button(text("Apply").size(12))
            .padding([4, 12])
            .on_press(Message::hex_editor(HexEditorMessage::CommitInspectorEdit)),
    ]
    .spacing(8);

    container(
        column![
            text(title).size(13).font(Font::MONOSPACE),
            input,
            error,
            buttons,
        ]
        .spacing(10),
    )
    .padding(16)
    .width(Length::Fixed(360.0))
    .style(|_| container::Style {
        background: Some(color!(0x201b18).into()),
        border: iced::Border {
            color: color!(0x4a3f35),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}
