//! Modal dialog for managing custom text encodings.
//!
//! The user can add any of the common encodings (ISO-8859-*, Windows-125*,
//! Shift_JIS, etc.) to their custom-encoding list, and remove entries they no
//! longer need.  Changes take effect immediately in the write-mode pick list.

use iced::widget::{button, column, container, pick_list, row, scrollable, text};
use iced::{Element, Font, Length};

use crate::domain::write_mode::COMMON_ENCODINGS;
use crate::{HexEditorMessage, HexEditorState};

/// View the encoding-settings modal body.
pub fn view(state: &HexEditorState) -> Element<'_, HexEditorMessage> {
    let title = text("Custom Encodings").size(13).font(Font::MONOSPACE);

    // ── Current custom encodings list ────────────────────────────────────
    let mut items: Vec<Element<'_, HexEditorMessage>> = Vec::new();
    if state.custom_encodings.is_empty() {
        items.push(
            text("No custom encodings added yet.")
                .size(11)
                .color(state.theme.modal_muted_fg)
                .into(),
        );
    } else {
        for (i, entry) in state.custom_encodings.iter().enumerate() {
            let label = text(&entry.label).size(11).font(Font::MONOSPACE);
            let remove_btn = button(text("✕").size(10).font(Font::MONOSPACE))
                .padding([2, 6])
                .on_press(HexEditorMessage::RemoveCustomEncoding(i));
            items.push(
                row![label, remove_btn]
                    .spacing(8)
                    .align_y(iced::Alignment::Center)
                    .into(),
            );
        }
    }

    let custom_list = scrollable(column(items).spacing(4)).height(Length::Fixed(160.0));

    // ── Add-new pick list ────────────────────────────────────────────────
    let add_label = text("Add encoding:").size(11).font(Font::MONOSPACE);

    let labels = crate::domain::write_mode::common_encoding_labels();

    let pick_list_selection = state
        .encoding_settings_selection
        .and_then(|idx| COMMON_ENCODINGS.get(idx))
        .map(|(l, _)| *l);

    let encoding_picker = pick_list(labels, pick_list_selection, |label| {
        // Find the index of the selected label.
        let idx = COMMON_ENCODINGS
            .iter()
            .position(|(l, _)| *l == label)
            .unwrap_or(0);
        HexEditorMessage::AddCustomEncoding(idx)
    })
    .font(Font::MONOSPACE)
    .text_size(11)
    .padding([2, 6]);

    // ── Close button ────────────────────────────────────────────────────
    let close_btn = button(text("Close").size(12))
        .padding([4, 14])
        .on_press(HexEditorMessage::CloseEncodingSettings);

    container(
        column![
            title,
            custom_list,
            add_label,
            encoding_picker,
            row![close_btn].spacing(8),
        ]
        .spacing(8)
        .width(Length::Fill),
    )
    .padding(16)
    .width(Length::Fixed(460.0))
    .style(|_: &_| container::Style {
        background: Some(iced::Background::Color(state.theme.modal_bg)),
        border: iced::Border {
            color: state.theme.modal_border,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    })
    .into()
}
