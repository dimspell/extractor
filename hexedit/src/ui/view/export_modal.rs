use iced::widget::{button, column, container, row, scrollable, text, toggler};
use iced::{color, Element, Font, Length};

use crate::domain::provider::HexProvider;
use crate::message::HexEditorMessage;
use crate::state::HexEditorState;
use crate::ui::update::format_hex_dump;

/// Maximum rows shown in the live preview.
const PREVIEW_ROWS: usize = 10;

/// Modal body for configuring hex dump text export options.
///
/// Provides toggles for:
/// - Show address (hex/decimal)
/// - Show ASCII column
///
/// A live preview of the first `PREVIEW_ROWS` lines is rendered on the right
/// so the user can see how the exported text will look before committing.
pub fn view(state: &HexEditorState) -> Element<'_, HexEditorMessage> {
    let cfg = state
        .export_config
        .as_ref()
        .expect("export_config is set when modal is shown");

    let title = text("Export as Text").size(13).font(Font::MONOSPACE);

    // ── Settings column (left) ────────────────────────────────────────
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

    let settings = column![title, show_addr, addr_fmt, show_ascii, buttons]
        .spacing(10)
        .width(Length::FillPortion(1));

    // ── Preview (right) ───────────────────────────────────────────────
    let bpr = state.bytes_per_row;
    let total = state.provider.len();
    let preview_len = (PREVIEW_ROWS as u64 * bpr as u64).min(total) as usize;
    let preview_bytes = &state.provider.as_slice()[..preview_len];
    let preview_text = format_hex_dump(preview_bytes, bpr, cfg);

    let preview = column![
        text("Preview").size(11).font(Font::MONOSPACE),
        scrollable(
            container(
                text(preview_text)
                    .font(Font::MONOSPACE)
                    .size(12)
                    .wrapping(text::Wrapping::None)
            )
            .padding(8)
            .style(|_: &_| container::Style {
                background: Some(iced::Background::Color(color!(0x15110e))),
                border: iced::Border {
                    color: color!(0x3a3026),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
        )
        .direction(scrollable::Direction::Both {
            vertical: Default::default(),
            horizontal: Default::default(),
        })
        .height(Length::Fixed(340.0))
        .width(Length::Fill),
    ]
    .spacing(6)
    .width(Length::FillPortion(2));

    // ── Outer container ───────────────────────────────────────────────
    container(row![settings, preview].spacing(16))
        .padding(16)
        .width(Length::Fixed(700.0))
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
