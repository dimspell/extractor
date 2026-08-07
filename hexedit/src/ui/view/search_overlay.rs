use iced::widget::{button, container, row, text, text_input};
use iced::{Element, Fill, Font, Length};

use crate::HexEditorMessage;
use crate::search::{SearchMode, SearchState};
use crate::ui::theme::HexEditorTheme;

/// Search overlay bar rendered above the hex matrix.
pub fn view<'a>(
    state: &'a SearchState,
    theme: &'a HexEditorTheme,
) -> Element<'a, HexEditorMessage> {
    let mode_label = match state.mode {
        SearchMode::Hex => "HEX",
        SearchMode::Ascii => "TXT",
        SearchMode::Decimal => "DEC",
    };

    let mode_btn = button(text(mode_label).size(10).font(Font::MONOSPACE))
        .padding([2, 6])
        .on_press(HexEditorMessage::ToggleSearchMode);

    let search_input = text_input("Find...", &state.query)
        .on_input(HexEditorMessage::Search)
        .padding(4)
        .size(11)
        .width(Length::Fixed(160.0));

    let count_text = {
        let label = if state.has_results() {
            let cur = state
                .current_idx()
                .map(|i| i + 1)
                .map_or("-".to_string(), |n| n.to_string());
            format!("{}/{}", cur, state.count())
        } else if state.query.is_empty() {
            String::new()
        } else {
            "0 matches".to_string()
        };
        text(label).size(10).font(Font::MONOSPACE)
    };

    let prev_btn = button(text("<").size(10).font(Font::MONOSPACE))
        .padding([2, 6])
        .on_press(HexEditorMessage::SearchPrev);

    let next_btn = button(text(">").size(10).font(Font::MONOSPACE))
        .padding([2, 6])
        .on_press(HexEditorMessage::SearchNext);

    let close_btn = button(text("✕").size(10).font(Font::MONOSPACE))
        .padding([2, 6])
        .on_press(HexEditorMessage::CloseSearch);

    // Decimal-specific controls: byte-width selector and endianness toggle.
    let decimal_controls = if state.mode == SearchMode::Decimal {
        let mut widths: Vec<Element<'_, HexEditorMessage>> = vec![];
        for w in [1u8, 2, 4, 8] {
            let active = state.width == w;
            let mut b = button(text(w.to_string()).size(10).font(Font::MONOSPACE))
                .padding([2, 5])
                .on_press(HexEditorMessage::SetSearchWidth(w));
            if active {
                let bg = theme.search_overlay_border;
                let fg = theme.search_current_fg;
                b = b.style(move |_: &iced::Theme, _: iced::widget::button::Status| {
                    iced::widget::button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: fg,
                        border: iced::Border {
                            color: bg,
                            width: 1.0,
                            radius: 2.0.into(),
                        },
                        ..iced::widget::button::Style::default()
                    }
                });
            }
            widths.push(b.into());
        }
        let endian_label = if state.little_endian { "LE" } else { "BE" };
        let endian_btn = button(text(endian_label).size(10).font(Font::MONOSPACE))
            .padding([2, 6])
            .on_press(HexEditorMessage::ToggleSearchEndian);
        Some(row![row(widths).spacing(2), endian_btn].spacing(6))
    } else {
        None
    };

    let content = row![mode_btn, search_input, count_text, prev_btn, next_btn,];
    // Insert decimal controls before the close button when active.
    let content = if let Some(decimal_controls) = decimal_controls {
        content.push(decimal_controls)
    } else {
        content
    };
    let content = content
        .push(close_btn)
        .spacing(6)
        .align_y(iced::Alignment::Center);

    container(content)
        .padding([4, 12])
        .width(Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(theme.search_overlay_bg.into()),
            border: iced::Border {
                color: theme.search_overlay_border,
                width: 1.0,
                radius: 0.into(),
            },
            ..container::Style::default()
        })
        .into()
}
