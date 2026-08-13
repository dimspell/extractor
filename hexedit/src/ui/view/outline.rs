//! Navigable outline for parser-supplied binary layouts.

use gui_widgets::sweeten::list::List;
use iced::widget::{Space, button, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

use crate::{HexEditorMessage, HexEditorState};

pub fn view<'a>(state: &'a HexEditorState) -> Element<'a, HexEditorMessage> {
    if state.layout.is_none() {
        return container(
            text("No structure layout for this file.")
                .size(11)
                .font(Font::MONOSPACE),
        )
        .padding(12)
        .width(Fill)
        .height(Fill)
        .into();
    };
    if state.outline.is_empty() {
        return container(
            text("No recognized sections.")
                .size(11)
                .font(Font::MONOSPACE),
        )
        .padding(12)
        .width(Fill)
        .height(Fill)
        .into();
    }
    let list = List::new(&state.outline, |_index, item| {
        let indent = f32::from(item.depth.min(6)) * 12.0;
        let index = if item.ty == "record" {
            format!(" #{}", item.record_index)
        } else {
            String::new()
        };
        let label = format!("{}{}  {:08X}", item.name, index, item.range.start);
        let caret: Element<'_, HexEditorMessage> = if item.has_children {
            button(
                text(if item.expanded { "⌄" } else { "›" })
                    .size(12)
                    .font(Font::MONOSPACE),
            )
            .padding([0, 4])
            .style(button::text)
            .on_press(HexEditorMessage::ToggleOutline(item.id))
            .into()
        } else {
            Space::default().width(Length::Fixed(18.0)).into()
        };
        row![
            Space::default().width(Length::Fixed(indent)),
            caret,
            button(text(label).size(10).font(Font::MONOSPACE))
                .padding([2, 4])
                .width(Fill)
                .on_press(HexEditorMessage::JumpToLayout(item.range.start))
                .style(button::text),
        ]
        .align_y(iced::Alignment::Center)
        .into()
    })
    .spacing(1);
    container(scrollable(list).height(Length::Fill))
        .width(Fill)
        .height(Fill)
        .into()
}
