use iced::widget::space::Space;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

use crate::editors::hex_editor::inspector::ENTRIES;
use crate::editors::hex_editor::HexEditorMessage;
use crate::editors::hex_editor::HexEditorState;
use crate::editors::hex_editor::HexProvider;
use crate::message::{Message, MessageExt};

const PANEL_WIDTH: f32 = 280.0;

pub fn view(editor: &HexEditorState) -> Element<'_, Message> {
    let header = container(text("Data inspector").size(11).font(Font::MONOSPACE))
        .padding([6, 12])
        .width(Fill);

    let rows: Element<'_, Message> = if editor.provider.is_empty() {
        container(text("(empty file)").size(11).font(Font::MONOSPACE))
            .padding([4, 12])
            .into()
    } else {
        let cursor = editor.selection.cursor;
        let len = editor.provider.len();
        let avail = (len - cursor) as usize;
        // Read a generous tail (max needed across decoders is 64 for cstr).
        let read_end = (cursor + 64).min(len);
        let bytes = editor.provider.read(cursor..read_end);

        let mut col = column![].spacing(2).padding([4, 12]);
        for (idx, entry) in ENTRIES.iter().enumerate() {
            let value = if avail >= entry.min_size {
                (entry.decode)(bytes)
            } else {
                "—".to_string()
            };
            let editable = entry.encode.is_some() && avail >= entry.min_size;
            col = col.push(inspector_row(entry.name, &value, idx, editable));
        }
        col.into()
    };

    container(column![header, scrollable(rows).height(Length::Fill)])
        .width(Length::Fixed(PANEL_WIDTH))
        .height(Fill)
        .into()
}

fn inspector_row<'a>(
    name: &'a str,
    value: &str,
    idx: usize,
    editable: bool,
) -> Element<'a, Message> {
    let edit_btn: Element<'a, Message> = if editable {
        button(text("✎").size(10).font(Font::MONOSPACE))
            .padding([0, 4])
            .on_press(Message::hex_editor(HexEditorMessage::BeginInspectorEdit(
                idx,
            )))
            .into()
    } else {
        Space::default().width(Length::Fixed(16.0)).into()
    };
    row![
        container(text(name.to_string()).size(10).font(Font::MONOSPACE)).width(Length::Fixed(60.0)),
        container(text(value.to_string()).size(11).font(Font::MONOSPACE)).width(Fill),
        edit_btn,
        Space::default().width(Length::Fixed(4.0)),
    ]
    .spacing(8)
    .into()
}
