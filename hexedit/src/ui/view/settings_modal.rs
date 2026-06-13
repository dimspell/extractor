use iced::widget::{button, column, container, row, text, toggler};
use iced::{color, Element, Font, Length};

use crate::HexEditorMessage;
use crate::HexEditorState;

/// Modal body for the hex editor settings dialog.
///
/// Options are live-applied — toggling a switch takes effect immediately.
/// The modal has no "Apply" button; just "Close" to dismiss.
pub fn view(state: &HexEditorState) -> Element<'_, HexEditorMessage> {
    let title = text("Hex Editor Settings").size(13).font(Font::MONOSPACE);

    let color_toggle = toggler(state.nybble_coloring)
        .label("Color-code bytes by high nybble")
        .on_toggle(HexEditorMessage::SetNybbleColoring)
        .size(13)
        .spacing(8);

    let close_btn = button(text("Close").size(12))
        .padding([4, 14])
        .on_press(HexEditorMessage::CloseSettings);

    container(
        column![title, color_toggle, row![close_btn].spacing(8)]
            .spacing(12)
            .width(Length::Fill),
    )
    .padding(16)
    .width(Length::Fixed(380.0))
    .style(|_: &_| container::Style {
        background: Some(iced::Background::Color(color!(0x201b18))),
        border: iced::Border {
            color: color!(0x4a3f35),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}
