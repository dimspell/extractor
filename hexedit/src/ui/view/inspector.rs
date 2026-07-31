use iced::widget::space::Space;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

use crate::config::HexEditorConfig;
use crate::inspector::ENTRIES;
use crate::state::InspectorSource;
use crate::{HexEditorMessage, HexEditorState, HexProvider};

pub fn view<'a>(
    editor: &'a HexEditorState,
    config: &HexEditorConfig,
) -> Element<'a, HexEditorMessage> {
    let has_comparison = editor.comparison_file.is_some();
    let header = container(
        row![
            text("Data inspector").size(11).font(Font::MONOSPACE),
            if has_comparison {
                source_toggle(editor)
            } else {
                Space::default().into()
            },
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .padding([6, 12])
    .width(Fill);

    let (src_len, src_bytes, editable) = match editor.inspector_source {
        InspectorSource::Baseline => (
            editor.provider.len(),
            editor.provider.as_slice(),
            true,
        ),
        InspectorSource::Comparison => {
            let data = editor
                .comparison_file
                .as_ref()
                .map(|cf| cf.data.as_slice())
                .unwrap_or(&[]);
            (data.len() as u64, data, false)
        }
    };

    let rows: Element<'_, HexEditorMessage> = if src_len == 0 {
        container(text("(empty file)").size(11).font(Font::MONOSPACE))
            .padding([4, 12])
            .into()
    } else {
        let cursor = editor.selection.cursor;
        let avail = src_len.saturating_sub(cursor) as usize;
        let read_end = (cursor + 64).min(src_len);
        let bytes = &src_bytes[cursor as usize..read_end as usize];

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
            let editable = editable && entry.encode.is_some() && avail >= entry.min_size;
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
                let editable = editable && entry.encode.is_some() && avail >= entry.min_size;
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
        .width(Fill)
        .height(Fill)
        .into()
}

/// A/B toggle letting the inspector decode from the main file or the
/// comparison file loaded for the diff view.
fn source_toggle<'a>(editor: &'a HexEditorState) -> Element<'a, HexEditorMessage> {
    let (a_active, b_active) = match editor.inspector_source {
        InspectorSource::Baseline => (true, false),
        InspectorSource::Comparison => (false, true),
    };
    row![
        toggle_button("A", a_active, InspectorSource::Baseline),
        toggle_button("B", b_active, InspectorSource::Comparison),
    ]
    .spacing(4)
    .into()
}

fn toggle_button<'a>(
    label: &'a str,
    active: bool,
    source: InspectorSource,
) -> Element<'a, HexEditorMessage> {
    let style = if active { button::primary } else { button::secondary };
    button(text(label).size(10).font(Font::MONOSPACE))
        .padding([1, 6])
        .style(style)
        .on_press(HexEditorMessage::SetInspectorSource(source))
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
    // Copy button only when the value is actually decoded; hidden when it's
    // a "—" placeholder (insufficient bytes at cursor).
    let can_copy = value != "—";
    let copy_btn: Element<'static, HexEditorMessage> = if can_copy {
        button(text("c").size(10).font(Font::MONOSPACE))
            .padding([0, 4])
            .on_press(HexEditorMessage::CopyInspectorValue(idx))
            .into()
    } else {
        Space::default().width(Length::Fixed(16.0)).into()
    };
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
