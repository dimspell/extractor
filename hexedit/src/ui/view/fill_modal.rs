//! Modal dialog for the "Fill Selection" hex editor operation.
//!
//! Shown after selecting a byte range and choosing "Fill…" from the context
//! menu. The user enters hex bytes (e.g. `"00"` or `"DE AD BE EF"`) that
//! will be repeated across the selected range.

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Font, Length};

use crate::HexEditorMessage;
use crate::domain::fill_dialog::FillDialog;
use crate::ui::theme::HexEditorTheme;

/// Modal body shown when "Fill…" is selected from the context menu.
pub fn view<'a>(dlg: &'a FillDialog, theme: &'a HexEditorTheme) -> Element<'a, HexEditorMessage> {
    let title = text("Fill Selection").size(13).font(Font::MONOSPACE);

    let input = text_input("00 FF AA BB …", &dlg.draft)
        .id(FillDialog::input_id())
        .on_input(HexEditorMessage::SetFillDraft)
        .on_submit(HexEditorMessage::CommitFill)
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

    let hint = text("Enter hex bytes to repeat across the selection")
        .size(10)
        .color(theme.modal_muted_fg)
        .font(Font::MONOSPACE);

    let buttons = row![
        button(text("Fill").size(12))
            .padding([4, 12])
            .on_press(HexEditorMessage::CommitFill),
        button(text("Cancel").size(12))
            .padding([4, 12])
            .on_press(HexEditorMessage::CloseFill),
    ]
    .spacing(8);

    container(column![title, input, hint, error, buttons].spacing(10))
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
