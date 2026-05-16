use iced::widget::space::Space;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

use crate::editors::hex_editor::pattern::{pattern_bg, pattern_fg};
use crate::editors::hex_editor::{HexEditorMessage, HexEditorState, Pattern};
use crate::message::{Message, MessageExt};

pub fn view(editor: &HexEditorState) -> Element<'_, Message> {
    let count = editor.patterns.len();

    let header = row![
        text(format!("Patterns ({})", count))
            .size(11)
            .font(Font::MONOSPACE),
        Space::default().width(Fill),
        button(text("✕").size(10).font(Font::MONOSPACE))
            .padding([2, 6])
            .on_press(Message::hex_editor(HexEditorMessage::TogglePatternList)),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let header = container(header).padding([4, 12]).width(Fill);

    if count == 0 {
        return container(column![
            header,
            text("No patterns defined").size(11).font(Font::MONOSPACE),
        ])
        .width(Fill)
        .into();
    }

    let body: Element<'_, Message> = {
        let mut col = column![].spacing(1).padding([2, 12]);
        for pat in &editor.patterns {
            col = col.push(pattern_row(pat));
        }
        scrollable(col).height(Length::Shrink).into()
    };

    container(column![header, body])
        .width(Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(iced::color!(0x1e1e1e))),
            border: iced::Border {
                color: iced::color!(0x3d3d3d),
                width: 1.0,
                radius: 0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn pattern_row<'a>(pat: &'a Pattern) -> Element<'a, Message> {
    let (bg, fg) = (pattern_bg(pat.color_idx), pattern_fg(pat.color_idx));

    let swatch = container(text("  ").size(8))
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(12.0))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border {
                color: fg,
                width: 1.0,
                radius: 2.into(),
            },
            ..Default::default()
        });

    let start = text(format!("0x{:08X}", pat.start))
        .size(10)
        .font(Font::MONOSPACE);
    let end = text(format!("0x{:08X}", pat.end))
        .size(10)
        .font(Font::MONOSPACE);
    let size = text(pat.len().to_string()).size(10).font(Font::MONOSPACE);

    let remove_btn = button(text("✕").size(9).font(Font::MONOSPACE))
        .padding([1, 4])
        .on_press(Message::hex_editor(HexEditorMessage::RemovePattern(pat.id)));

    let inner = row![
        swatch,
        container(start).width(Length::Fixed(80.0)),
        container(end).width(Length::Fixed(80.0)),
        container(size).width(Length::Fixed(40.0)),
        remove_btn,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    button(inner)
        .on_press(Message::hex_editor(HexEditorMessage::NavigateToPattern(
            pat.id,
        )))
        .padding([3, 6])
        .width(Fill)
        .style(button::text)
        .into()
}
