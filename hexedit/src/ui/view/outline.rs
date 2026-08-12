//! Navigable outline for parser-supplied binary layouts.

use iced::widget::{button, column, container, scrollable, text};
use iced::{Element, Fill, Font, Length};

use crate::{HexEditorMessage, HexEditorState, HexProvider};

pub fn view<'a>(state: &'a HexEditorState) -> Element<'a, HexEditorMessage> {
    let Some(layout) = state.layout.as_deref() else {
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
    let mut entries = column![].spacing(1).padding([4, 8]);
    for item in layout.outline(state.provider.len()) {
        let indent = f32::from(item.depth.min(5)) * 12.0;
        let index = if item.ty == "record" {
            format!(" #{}", item.record_index)
        } else {
            String::new()
        };
        let label = format!("{}{}  {:08X}", item.name, index, item.range.start);
        entries = entries.push(
            button(text(label).size(10).font(Font::MONOSPACE))
                .padding([2, 4])
                .width(Fill)
                .on_press(HexEditorMessage::JumpToLayout(item.range.start))
                .style(button::text)
                .padding([2, indent as u16 + 4]),
        );
    }
    container(scrollable(entries).height(Length::Fill))
        .width(Fill)
        .height(Fill)
        .into()
}
