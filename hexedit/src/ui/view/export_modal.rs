use iced::widget::{button, column, container, row, text, toggler};
use iced::{color, Element, Font, Length};

use crate::HexEditorMessage;
use crate::HexEditorState;

/// Modal body for configuring hex dump text export options.
///
/// Provides toggles for:
/// - Show address (hex/decimal)
/// - Show ASCII column
pub fn view(state: &HexEditorState) -> Element<'_, HexEditorMessage> {
    let cfg = state.export_config.as_ref().expect("export_config is set when modal is shown");

    let title = text("Export as Text").size(13).font(Font::MONOSPACE);

    let show_addr = toggler(cfg.show_address)
        .label("Show address")
        .on_toggle(HexEditorMessage::SetExportShowAddress)
        .size(13)
        .spacing(8);

    let addr_fmt = toggler(cfg.address_decimal)
        .label("Decimal addresses")
        .on_toggle(HexEditorMessage::SetExportAddressDecimal)
        .size(13)
        .spacing(8);

    let show_ascii = toggler(cfg.show_ascii)
        .label("Show ASCII column")
        .on_toggle(HexEditorMessage::SetExportShowAscii)
        .size(13)
        .spacing(8);

    let buttons = row![
        button(text("Export").size(12))
            .padding([4, 14])
            .on_press(HexEditorMessage::CommitExport),
        button(text("Cancel").size(12))
            .padding([4, 14])
            .on_press(HexEditorMessage::CloseExportConfig),
    ]
    .spacing(8);

    container(column![title, show_addr, addr_fmt, show_ascii, buttons].spacing(10))
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
