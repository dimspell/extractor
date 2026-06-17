//! Modal dialog for creating a repeated pattern group with a label.
//!
//! Shown after selecting a byte range and choosing "Add Repeated Pattern" from
//! the context menu. The user enters a label and a repeat count; the editor
//! creates a named group with that many pattern repetitions.

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Font, Length};

use crate::domain::pattern::RepeatPatternDialog;
use crate::ui::theme::HexEditorTheme;
use crate::HexEditorMessage;

/// Modal body shown when "Add Repeated Pattern" is selected from the context
/// menu with an active multi-byte selection.
pub fn view<'a>(dlg: &'a RepeatPatternDialog, theme: &'a HexEditorTheme) -> Element<'a, HexEditorMessage> {
    let title = text("Add Repeated Pattern").size(13).font(Font::MONOSPACE);

    let block_info = text(format!(
        "Block: {} bytes  ·  Start: 0x{:08X}",
        dlg.block_size, dlg.block_start
    ))
    .size(11)
    .color(theme.modal_muted_fg)
    .font(Font::MONOSPACE);

    // ── Label input ──────────────────────────────────────────────────────
    let label_input = text_input("Label (e.g. Monster HP array)", &dlg.label_draft)
        .on_input(HexEditorMessage::SetRepeatedPatternLabel)
        .padding(6)
        .size(13);

    let label_hint = text("Name for this pattern group")
        .size(10)
        .color(theme.modal_muted_fg)
        .font(Font::MONOSPACE);

    // ── Repeat count input ───────────────────────────────────────────────
    let count_input = text_input("Repeat count (e.g. 4)", &dlg.draft)
        .id(RepeatPatternDialog::input_id())
        .on_input(HexEditorMessage::SetRepeatedPatternDraft)
        .on_submit(HexEditorMessage::CommitRepeatedPattern)
        .padding(6)
        .size(13);

    let count_hint = text("How many times to repeat the selected block")
        .size(10)
        .color(theme.modal_muted_fg)
        .font(Font::MONOSPACE);

    let error: Element<'_, HexEditorMessage> = if let Some(err) = &dlg.error {
        text(err.clone())
            .size(11)
            .color(theme.modal_error_fg)
            .font(Font::MONOSPACE)
            .into()
    } else {
        text("").size(11).into()
    };

    let buttons = row![
        button(text("Add").size(12))
            .padding([4, 12])
            .on_press(HexEditorMessage::CommitRepeatedPattern),
        button(text("Cancel").size(12))
            .padding([4, 12])
            .on_press(HexEditorMessage::CloseRepeatedPattern),
    ]
    .spacing(8);

    container(
        column![
            title,
            block_info,
            label_input,
            label_hint,
            count_input,
            count_hint,
            error,
            buttons,
        ]
        .spacing(8),
    )
    .padding(16)
    .width(Length::Fixed(380.0))
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
