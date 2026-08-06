use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Font, Length};

use crate::HexEditorMessage;
use crate::editing::InspectorEditState;
use crate::inspector::ENTRIES;
use crate::ui::theme::HexEditorTheme;

/// Modal body shown when an inspector "Edit" button is pressed.
pub fn view<'a>(
    state: &'a InspectorEditState,
    theme: &'a HexEditorTheme,
) -> Element<'a, HexEditorMessage> {
    let entry_name = ENTRIES
        .get(state.entry_idx)
        .map(|e| e.name.as_str())
        .unwrap_or("?");
    let title = format!("Edit {entry_name} at 0x{:X}", state.addr);

    let input = text_input("value", &state.draft)
        .on_input(HexEditorMessage::SetInspectorDraft)
        .on_submit(HexEditorMessage::CommitInspectorEdit)
        .padding(6)
        .size(13);

    let error: Element<'_, HexEditorMessage> = if let Some(err) = &state.error {
        text(err.clone())
            .size(11)
            .color(theme.modal_error_fg)
            .font(Font::MONOSPACE)
            .into()
    } else {
        text("").size(11).into()
    };

    let buttons = row![
        button(text("Apply").size(12))
            .padding([4, 12])
            .on_press(HexEditorMessage::CommitInspectorEdit),
        button(text("Cancel").size(12))
            .padding([4, 12])
            .on_press(HexEditorMessage::CloseInspectorEdit),
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
    .style(|_: &_| container::Style {
        background: Some(iced::Background::Color(theme.modal_bg)),
        border: iced::Border {
            color: theme.modal_border,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}
