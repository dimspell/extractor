//! Modal dialog for the "Extend File" hex editor operation.
//!
//! Shown after right-clicking a byte and choosing "Extend…" from the context
//! menu. The user enters a byte count and a hex fill pattern (e.g. `"00"` or
//! `"DE AD BE EF"`); committing inserts that many bytes at the cursor address,
//! shifting the rest of the file forward.

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Font, Length};

use crate::domain::extend_dialog::ExtendDialog;
use crate::ui::theme::HexEditorTheme;
use crate::HexEditorMessage;

/// Modal body shown when "Extend…" is selected from the context menu.
pub fn view<'a>(dlg: &'a ExtendDialog, theme: &'a HexEditorTheme) -> Element<'a, HexEditorMessage> {
    let title = text("Extend File").size(13).font(Font::MONOSPACE);

    let count_label = text("Bytes to add")
        .size(10)
        .color(theme.modal_muted_fg)
        .font(Font::MONOSPACE);
    let count_input = text_input("1024", &dlg.count_draft)
        .id(ExtendDialog::count_input_id())
        .on_input(HexEditorMessage::SetExtendCount)
        .on_submit(HexEditorMessage::CommitExtend)
        .padding(6)
        .size(13);

    let pattern_label = text("Fill pattern")
        .size(10)
        .color(theme.modal_muted_fg)
        .font(Font::MONOSPACE);
    let pattern_input = text_input("00 FF AA BB", &dlg.pattern_draft)
        .id(ExtendDialog::pattern_input_id())
        .on_input(HexEditorMessage::SetExtendPattern)
        .on_submit(HexEditorMessage::CommitExtend)
        .padding(6)
        .size(13);

    let error: Element<'a, HexEditorMessage> = if let Some(err) = &dlg.error {
        text(err.clone())
            .size(11)
            .color(theme.modal_error_fg)
            .font(Font::MONOSPACE)
            .into()
    } else {
        text("").size(11).into()
    };

    let hint = text("Insert bytes at the cursor position, shifting the rest of the file forward")
        .size(10)
        .color(theme.modal_muted_fg)
        .font(Font::MONOSPACE);

    let buttons = row![
        button(text("Extend").size(12))
            .padding([4, 12])
            .on_press(HexEditorMessage::CommitExtend),
        button(text("Cancel").size(12))
            .padding([4, 12])
            .on_press(HexEditorMessage::CloseExtend),
    ]
    .spacing(8);

    container(
        column![
            title,
            count_label,
            count_input,
            pattern_label,
            pattern_input,
            hint,
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
