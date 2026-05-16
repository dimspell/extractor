use iced::widget::{button, container, row, text, text_input};
use iced::{color, Element, Fill, Font, Length};

use crate::editors::hex_editor::search::SearchState;
use crate::editors::hex_editor::HexEditorMessage;
use crate::message::{Message, MessageExt};

/// Search overlay bar rendered above the hex matrix.
pub fn view(state: &SearchState) -> Element<'_, Message> {
    let mode_label = match state.mode {
        crate::editors::hex_editor::search::SearchMode::Hex => "HEX",
        crate::editors::hex_editor::search::SearchMode::Ascii => "TXT",
    };

    let mode_btn = button(text(mode_label).size(10).font(Font::MONOSPACE))
        .padding([2, 6])
        .on_press(Message::hex_editor(HexEditorMessage::ToggleSearchMode));

    let search_input = text_input("Find...", &state.query)
        .on_input(|s| Message::hex_editor(HexEditorMessage::Search(s)))
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
        .on_press(Message::hex_editor(HexEditorMessage::SearchPrev));

    let next_btn = button(text(">").size(10).font(Font::MONOSPACE))
        .padding([2, 6])
        .on_press(Message::hex_editor(HexEditorMessage::SearchNext));

    let close_btn = button(text("✕").size(10).font(Font::MONOSPACE))
        .padding([2, 6])
        .on_press(Message::hex_editor(HexEditorMessage::CloseSearch));

    let content = row![
        mode_btn,
        search_input,
        count_text,
        prev_btn,
        next_btn,
        close_btn,
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    container(content)
        .padding([4, 12])
        .width(Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(color!(0x1e1e1e).into()),
            border: iced::Border {
                color: color!(0x3d3d3d),
                width: 1.0,
                radius: 0.into(),
            },
            ..container::Style::default()
        })
        .into()
}
