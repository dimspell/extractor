use iced::widget::{button, column, container, pick_list, row, text, toggler};
use iced::{color, Color, Element, Font, Length};

use crate::ui::coloring::{default_byte_colors, ColorScheme};
use crate::HexEditorMessage;
use crate::HexEditorState;

/// Modal body for the hex editor settings dialog.
///
/// Options are live-applied — toggling/selecting takes effect immediately.
/// The modal has no "Apply" button; just "Close" to dismiss.
pub fn view(state: &HexEditorState) -> Element<'_, HexEditorMessage> {
    let title = text("Hex Editor Settings").size(13).font(Font::MONOSPACE);

    // ── Colour-scheme pick list ────────────────────────────────────────
    let scheme_row = row![
            text("Byte colour scheme")
                .size(12)
                .width(Length::Fill),
            pick_list(
                &ColorScheme::ALL[..],
                Some(state.color_scheme),
                HexEditorMessage::SetColorScheme,
            )
            .font(Font::MONOSPACE)
            .text_size(12)
            .padding([2, 6]),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    // ── Dim-nulls toggle ───────────────────────────────────────────────
    let dim_toggle = toggler(state.dim_nulls)
        .label("Dim null bytes")
        .on_toggle(HexEditorMessage::SetDimNulls)
        .size(13)
        .spacing(8);

    // ── Palette preview ────────────────────────────────────────────────
    // Show a row of small coloured squares sampled across the byte range.
    // Uses the same provider chain as the matrix so the preview stays in sync.
    let swatch_size = 14.0;
    let palette: Vec<Element<'_, HexEditorMessage>> = (0..=15)
        .map(|i| {
            let b = (i * 17) as u8; // 0x00, 0x11, …, 0xFF
            let (fg_opt, _) = default_byte_colors(state.color_scheme, b, state.dim_nulls);
            let fg = fg_opt.unwrap_or(color!(0xd4cabd));
            container(
                text("  ") // invisible spacer – the swatch is the container bg
                    .size(10),
            )
            .style(move |_: &_| container::Style {
                background: Some(iced::Background::Color(fg)),
                border: iced::Border {
                    color: Color::BLACK,
                    width: 0.5,
                    radius: 2.0.into(),
                },
                ..Default::default()
            })
            .width(swatch_size)
            .height(swatch_size)
            .into()
        })
        .collect();

    let palette_row = row(palette).spacing(2);

    let palette_label = text("Palette preview (0x00 … 0xFF)")
        .size(10)
        .color(color!(0x7a6f64));

    // ── Close button ───────────────────────────────────────────────────
    let close_btn = button(text("Close").size(12))
        .padding([4, 14])
        .on_press(HexEditorMessage::CloseSettings);

    container(
        column![
            title,
            scheme_row,
            dim_toggle,
            palette_label,
            palette_row,
            row![close_btn].spacing(8),
        ]
        .spacing(8)
        .width(Length::Fill),
    )
    .padding(16)
    .width(Length::Fixed(420.0))
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


