use iced::widget::{button, column, container, row, text, text_input};
use iced::{color, Element, Font, Length};

use crate::goto::GotoState;
use crate::HexEditorMessage;

/// Modal body shown when Ctrl+G is pressed.
pub fn view(state: &GotoState) -> Element<'_, HexEditorMessage> {
    let title = text("Go to address").size(13).font(Font::MONOSPACE);

    let input = text_input("0x100, 255, +10, -5", &state.draft)
        .id(crate::goto::GotoState::input_id())
        .on_input(HexEditorMessage::SetGotoDraft)
        .on_submit(HexEditorMessage::CommitGoto)
        .padding(6)
        .size(13);

    let error: Element<'_, HexEditorMessage> = if let Some(err) = &state.error {
        text(err.clone())
            .size(11)
            .color(color!(0xff8a6e))
            .font(Font::MONOSPACE)
            .into()
    } else {
        text("").size(11).into()
    };

    let hint = text("hex (0xFF), dec (255), relative (+10, -5)")
        .size(10)
        .color(color!(0x7a6f64))
        .font(Font::MONOSPACE);

    let buttons = row![
        button(text("Go").size(12))
            .padding([4, 12])
            .on_press(HexEditorMessage::CommitGoto),
        button(text("Cancel").size(12))
            .padding([4, 12])
            .on_press(HexEditorMessage::CloseGotoDialog),
    ]
    .spacing(8);

    container(column![title, input, hint, error, buttons].spacing(10))
        .padding(16)
        .width(Length::Fixed(360.0))
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
