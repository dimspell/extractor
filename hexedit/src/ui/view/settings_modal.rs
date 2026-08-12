use iced::widget::{button, column, container, pick_list, row, text, text_input, toggler};
use iced::{Color, Element, Font, Length};

use crate::HexEditorMessage;
use crate::HexEditorState;
use crate::ui::coloring::{ColorScheme, default_byte_colors};
use crate::ui::theme::{DARK_THEME, ThemeVariant};
use crate::ui::update::parse_bpr;

/// Colour for section heading text.
const HEADING_COLOR: Color = DARK_THEME.modal_heading_fg;
/// Colour for lighter secondary text.
const MUTED_COLOR: Color = DARK_THEME.modal_muted_fg;

/// A horizontal rule separator.
fn sep() -> iced::widget::rule::Rule<'static, iced::Theme> {
    iced::widget::rule::horizontal(1.0)
}

/// Section heading label.
fn section_label(label: &str) -> Element<'_, HexEditorMessage> {
    text(label)
        .size(11)
        .color(HEADING_COLOR)
        .font(Font::MONOSPACE)
        .into()
}

/// Modal body for the hex editor settings dialog.
///
/// Options are live-applied — toggling/selecting takes effect immediately.
/// The modal has no "Apply" button; just "Close" to dismiss.
pub fn view(state: &HexEditorState) -> Element<'_, HexEditorMessage> {
    let title = text("Hex Editor Settings").size(13).font(Font::MONOSPACE);

    // ── Byte Coloring section ──────────────────────────────────────────
    let coloring_label = section_label("Byte Coloring");

    // Colour-scheme pick list
    let scheme_row = row![
        text("Scheme").size(12).width(Length::Fill),
        pick_list(Some(state.color_scheme), &ColorScheme::ALL[..], |cs| cs
            .to_string(),)
        .on_select(HexEditorMessage::SetColorScheme)
        .font(Font::MONOSPACE)
        .text_size(12)
        .padding([2, 6]),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Dim-nulls toggle
    let dim_toggle = toggler(state.dim_nulls)
        .label("Dim null bytes")
        .on_toggle(HexEditorMessage::SetDimNulls)
        .size(13)
        .spacing(8);

    // Palette preview — a row of small coloured squares sampled across
    // the byte range. Uses the same provider chain as the matrix so the
    // preview stays in sync.
    let swatch_size = 14.0;
    let palette: Vec<Element<'_, HexEditorMessage>> = (0..=15)
        .map(|i| {
            let b = (i * 17) as u8;
            let (fg_opt, _) = default_byte_colors(state.color_scheme, b, state.dim_nulls);
            let fg = fg_opt.unwrap_or(state.theme.hex_fg);
            container(text("  ").size(10))
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
        .color(MUTED_COLOR);

    // ── Display section ────────────────────────────────────────────────
    let display_label = section_label("Display");

    // Address format pick list
    const ADDR_OPTIONS: [&str; 2] = ["Hex (default)", "Decimal"];
    let addr_current = if state.show_decimal {
        ADDR_OPTIONS[1]
    } else {
        ADDR_OPTIONS[0]
    };
    let addr_pick = pick_list(Some(addr_current), &ADDR_OPTIONS[..], |s| s.to_string())
        .on_select(|selected| HexEditorMessage::SetAddrFormat(selected == ADDR_OPTIONS[1]))
        .font(Font::MONOSPACE)
        .text_size(12)
        .padding([2, 6]);

    let addr_row = row![
        text("Address format").size(12).width(Length::Fill),
        addr_pick,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Bytes-per-row selector
    let bpr = state.bytes_per_row;
    let bpr_btn = |n: u8| {
        let label = format!("{:02}", n);
        let active = bpr == n;
        let mut btn = button(text(label).size(12).font(Font::MONOSPACE)).padding([3, 6]);
        if !active {
            btn = btn.style(button::text);
        }
        btn.on_press(HexEditorMessage::SetBytesPerRow(n))
    };

    // Custom bytes-per-row input — parse the draft on submit; a valid value
    // applies immediately, an invalid one surfaces a status message via
    // `BytesPerRowInputInvalid`.
    let custom_bpr_submit = match parse_bpr(&state.bpr_input) {
        Some(n) => HexEditorMessage::SetBytesPerRow(n),
        None => HexEditorMessage::BytesPerRowInputInvalid,
    };
    let custom_bpr = text_input(
        format!(
            "{}–{}",
            crate::state::MIN_BYTES_PER_ROW,
            crate::state::MAX_BYTES_PER_ROW
        ),
        &state.bpr_input,
    )
    .on_input(HexEditorMessage::BytesPerRowInputChanged)
    .on_submit(custom_bpr_submit)
    .font(Font::MONOSPACE)
    .size(12)
    .padding([2, 6])
    .width(Length::Fixed(48.0));

    let bpr_row = row![
        text("Bytes per row").size(12).width(Length::Fill),
        row![bpr_btn(8), bpr_btn(16), bpr_btn(32), custom_bpr].spacing(4),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Entropy band toggle
    let entropy_toggle = toggler(state.show_entropy_band)
        .label("Entropy colour band in gutter")
        .on_toggle(HexEditorMessage::SetShowEntropyBand)
        .size(13)
        .spacing(8);

    // Minimap toggle
    let minimap_toggle = toggler(state.show_minimap)
        .label("Minimap overview strip")
        .on_toggle(HexEditorMessage::SetShowMinimapEnabled)
        .size(13)
        .spacing(8);

    // ── Appearance section ─────────────────────────────────────────────
    let appearance_label = section_label("Appearance");

    // Theme pick list
    let theme_pick = pick_list(Some(state.theme_variant), &ThemeVariant::ALL[..], |tv| {
        tv.to_string()
    })
    .on_select(HexEditorMessage::SetTheme)
    .font(Font::MONOSPACE)
    .text_size(12)
    .padding([2, 6]);

    let theme_row = row![text("Theme").size(12).width(Length::Fill), theme_pick,]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    // ── Action buttons ─────────────────────────────────────────────────
    let reset_btn = button(text("Reset to Defaults").size(12))
        .padding([4, 14])
        .on_press(HexEditorMessage::ResetSettings);

    let close_btn = button(text("Close").size(12))
        .padding([4, 14])
        .on_press(HexEditorMessage::CloseSettings);

    let action_row = row![reset_btn, close_btn].spacing(12);

    // ── Assemble ───────────────────────────────────────────────────────
    container(
        column![
            title,
            sep(),
            coloring_label,
            scheme_row,
            dim_toggle,
            palette_label,
            palette_row,
            sep(),
            display_label,
            addr_row,
            bpr_row,
            entropy_toggle,
            minimap_toggle,
            sep(),
            appearance_label,
            theme_row,
            sep(),
            action_row,
        ]
        .spacing(8)
        .width(Length::Fill),
    )
    .padding(16)
    .width(Length::Fixed(420.0))
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
