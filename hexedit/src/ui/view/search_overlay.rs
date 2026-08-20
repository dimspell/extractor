use gui_widgets::lucide::{LUCIDE_FONT, icon_char};
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Element, Fill, Font, Length};
use lucide_icons::Icon;

use crate::HexEditorMessage;
use crate::search::{SearchMode, SearchState};
use crate::ui::theme::HexEditorTheme;

/// Search bar rendered above the hex matrix.
///
/// The layout deliberately keeps the query, result state, and navigation in
/// distinct groups. This makes the most common flow — type, inspect count,
/// then move through matches — easy to scan without turning decimal options
/// into permanent visual noise.
pub fn view<'a>(
    state: &'a SearchState,
    theme: &'a HexEditorTheme,
) -> Element<'a, HexEditorMessage> {
    let (mode_label, mode_hint, placeholder) = match state.mode {
        SearchMode::Hex => ("HEX", "byte sequence", "DE AD BE EF"),
        SearchMode::Ascii => ("TEXT", "ASCII text", "Text to find"),
        SearchMode::Decimal => ("NUMBER", "signed integer", "e.g. 1024"),
    };

    // The mode control remains a cycle button because the search model has a
    // single toggle action, but its secondary label makes the active syntax
    // explicit instead of relying on the abbreviated button label alone.
    let mode = button(
        row![
            text(icon_char(Icon::Search)).font(LUCIDE_FONT).size(14),
            column![
                text(mode_label).size(10).font(Font::MONOSPACE),
                text(mode_hint)
                    .size(9)
                    .font(Font::MONOSPACE)
                    .color(theme.modal_muted_fg),
            ]
            .spacing(1),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([4, 7])
    .on_press(HexEditorMessage::ToggleSearchMode);

    let query = text_input(placeholder, &state.query)
        .on_input(HexEditorMessage::Search)
        .on_submit(HexEditorMessage::SearchNext)
        .padding([6, 8])
        .size(12)
        .width(Length::Fixed(220.0));

    let result_status = match (state.query.is_empty(), state.has_results()) {
        (true, _) => "Enter a value".to_owned(),
        (false, true) => format!(
            "{} of {}",
            state.current_idx().map_or(0, |index| index + 1),
            state.count()
        ),
        (false, false) => "No matches".to_owned(),
    };
    let address = state.current_addr().map_or_else(
        || "Search results".to_owned(),
        |addr| format!("0x{addr:08X}"),
    );
    let results = container(
        column![
            text(result_status).size(11).font(Font::MONOSPACE),
            text(address)
                .size(9)
                .font(Font::MONOSPACE)
                .color(theme.modal_muted_fg),
        ]
        .spacing(1),
    )
    .width(Length::Fixed(88.0));

    let previous = button(text(icon_char(Icon::ChevronUp)).font(LUCIDE_FONT).size(14))
        .padding([1, 8])
        .on_press(HexEditorMessage::SearchPrev);
    let next = button(
        text(icon_char(Icon::ChevronDown))
            .font(LUCIDE_FONT)
            .size(14),
    )
    .padding([1, 8])
    .on_press(HexEditorMessage::SearchNext);
    let navigation = row![previous, next].spacing(2).align_y(Alignment::Center);

    let decimal_options = (state.mode == SearchMode::Decimal).then(|| {
        let width_picker = row([1_u8, 2, 4, 8].into_iter().map(|width| {
            let active = state.width == width;
            let background = if active {
                theme.search_current_bg
            } else {
                theme.search_overlay_bg
            };
            let foreground = if active {
                theme.search_current_fg
            } else {
                theme.modal_muted_fg
            };
            button(text(width.to_string()).size(10).font(Font::MONOSPACE))
                .padding([3, 6])
                .on_press(HexEditorMessage::SetSearchWidth(width))
                .style(move |_, _| iced::widget::button::Style {
                    background: Some(background.into()),
                    text_color: foreground,
                    border: iced::Border {
                        color: if active {
                            theme.search_current_bg
                        } else {
                            theme.search_overlay_border
                        },
                        width: 1.0,
                        radius: 3.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        }))
        .spacing(2);
        let endian = if state.little_endian {
            "Little-endian"
        } else {
            "Big-endian"
        };
        let endian_button = button(text(endian).size(10).font(Font::MONOSPACE))
            .padding([3, 7])
            .on_press(HexEditorMessage::ToggleSearchEndian);

        row![
            column![
                text("WIDTH")
                    .size(9)
                    .font(Font::MONOSPACE)
                    .color(theme.modal_muted_fg),
                width_picker,
            ]
            .spacing(3),
            column![
                text("ORDER")
                    .size(9)
                    .font(Font::MONOSPACE)
                    .color(theme.modal_muted_fg),
                endian_button,
            ]
            .spacing(3),
        ]
        .spacing(8)
        .align_y(Alignment::End)
    });

    let close = button(text(icon_char(Icon::X)).font(LUCIDE_FONT).size(14))
        .padding([1, 8])
        .on_press(HexEditorMessage::CloseSearch);

    let content = row![
        mode,
        query,
        results,
        navigation,
        decimal_options.unwrap_or_else(|| row![]),
        close,
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    container(content)
        .padding([6, 12])
        .width(Fill)
        .style(|_| container::Style {
            background: Some(theme.search_overlay_bg.into()),
            border: iced::Border {
                color: theme.search_overlay_border,
                width: 1.0,
                radius: 0.into(),
            },
            ..Default::default()
        })
        .into()
}
