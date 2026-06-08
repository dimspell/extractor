use iced::widget::space::Space;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

use crate::config::HexEditorConfig;
use crate::inspector::ENTRIES;
use crate::{HexEditorMessage, HexEditorState, HexProvider};

const PANEL_WIDTH: f32 = 280.0;

pub fn view<'a>(
    editor: &'a HexEditorState,
    config: &HexEditorConfig,
) -> Element<'a, HexEditorMessage> {
    let header = container(text("Data inspector").size(11).font(Font::MONOSPACE))
        .padding([6, 12])
        .width(Fill);

    let rows: Element<'_, HexEditorMessage> = if editor.provider.is_empty() {
        container(text("(empty file)").size(11).font(Font::MONOSPACE))
            .padding([4, 12])
            .into()
    } else {
        let cursor = editor.selection.cursor;
        let len = editor.provider.len();
        let avail = (len - cursor) as usize;
        let read_end = (cursor + 64).min(len);
        let bytes = editor.provider.read(cursor..read_end);

        let mut col = column![].spacing(1).padding([4, 12]);
        let mut last_category: Option<&str> = None;

        for (idx, entry) in ENTRIES.iter().enumerate() {
            if last_category != Some(entry.category.as_str()) {
                last_category = Some(entry.category.as_str());
                col = col.push(category_header(&entry.category));
            }
            let value = if avail >= entry.min_size {
                (entry.decode)(bytes)
            } else {
                "—".to_string()
            };
            let editable = entry.encode.is_some() && avail >= entry.min_size;
            col = col.push(inspector_row(
                &entry.name,
                &value,
                idx,
                editable,
                &entry.description,
            ));
        }

        if !config.extra_entries.is_empty() {
            col = col.push(category_header("Custom"));
            for (i, entry) in config.extra_entries.iter().enumerate() {
                let idx = ENTRIES.len() + i;
                if last_category != Some(entry.category.as_str()) {
                    last_category = Some(entry.category.as_str());
                }
                let value = if avail >= entry.min_size {
                    (entry.decode)(bytes)
                } else {
                    "—".to_string()
                };
                let editable = entry.encode.is_some() && avail >= entry.min_size;
                col = col.push(inspector_row(
                    &entry.name,
                    &value,
                    idx,
                    editable,
                    &entry.description,
                ));
            }
        }

        col.into()
    };

    container(column![header, scrollable(rows).height(Length::Fill)])
        .width(Length::Fixed(PANEL_WIDTH))
        .height(Fill)
        .into()
}

fn category_header(category: &str) -> Element<'static, HexEditorMessage> {
    container(
        text(format!("── {category} ──"))
            .size(9)
            .font(Font::MONOSPACE),
    )
    .padding([4, 0])
    .width(Fill)
    .into()
}

fn inspector_row(
    name: &str,
    value: &str,
    idx: usize,
    editable: bool,
    _description: &str,
) -> Element<'static, HexEditorMessage> {
    let edit_btn: Element<'static, HexEditorMessage> = if editable {
        button(text("✎").size(10).font(Font::MONOSPACE))
            .padding([0, 4])
            .on_press(HexEditorMessage::BeginInspectorEdit(idx))
            .into()
    } else {
        Space::default().width(Length::Fixed(16.0)).into()
    };
    let copy_btn = button(text("c").size(10).font(Font::MONOSPACE))
        .padding([0, 4])
        .on_press(HexEditorMessage::CopyInspectorValue(idx));
    row![
        container(text(name.to_string()).size(10).font(Font::MONOSPACE)).width(Length::Fixed(60.0)),
        container(text(value.to_string()).size(11).font(Font::MONOSPACE)).width(Fill),
        copy_btn,
        edit_btn,
        Space::default().width(Length::Fixed(4.0)),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center)
    .into()
}
