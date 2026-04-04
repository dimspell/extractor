use crate::app::App;
use crate::generic_editor::{GenericEditorState, MultiFileEditorState};
use crate::message::Message;
use crate::style;
use crate::utils::{
    horizontal_rule, horizontal_space, labeled_input, labeled_select, vertical_space,
};
use dispel_core::references::editable::{EditableRecord, FieldDescriptor, FieldKind};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};
use std::collections::HashMap;

/// Build a generic editor view for any `EditableRecord` type.
///
/// This replaces the 28 duplicated `view_*_editor_tab` functions.
pub fn build_editor_view<'a, R: EditableRecord>(
    _app: &'a App,
    editor: &'a GenericEditorState<R>,
    scan_msg: Message,
    save_msg: Message,
    select_msg: fn(usize) -> Message,
    field_changed_msg: fn(usize, String, String) -> Message,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
) -> Element<'a, Message> {
    build_editor_view_inner(
        editor,
        scan_msg,
        save_msg,
        select_msg,
        field_changed_msg,
        lookups,
    )
}

fn build_editor_view_inner<'a, R: EditableRecord>(
    editor: &'a GenericEditorState<R>,
    scan_msg: Message,
    save_msg: Message,
    select_msg: fn(usize) -> Message,
    field_changed_msg: fn(usize, String, String) -> Message,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
) -> Element<'a, Message> {
    // Status bar
    let status_row = container(
        row![
            text(&editor.status_msg).size(13).style(style::subtle_text),
            horizontal_space(),
            if editor.is_loading {
                Element::from(text("Loading...").size(13))
            } else {
                Element::from(text(""))
            },
            horizontal_space().width(20),
            button(text(R::save_button_label()))
                .on_press(save_msg)
                .style(style::commit_button),
        ]
        .padding([10, 20])
        .align_y(iced::Alignment::Center),
    )
    .width(Fill)
    .style(style::status_bar);

    // Item list (left panel)
    let item_list: Vec<Element<Message>> = editor
        .filtered
        .iter()
        .enumerate()
        .map(|(idx, (_, record))| {
            let is_selected = editor.selected_idx == Some(idx);
            let label = record.list_label();
            let btn = button(text(label).size(11).font(Font::MONOSPACE))
                .width(Fill)
                .on_press(select_msg(idx));

            if is_selected {
                btn.style(style::active_chip).into()
            } else {
                btn.style(style::chip).into()
            }
        })
        .collect();

    let scroll = scrollable(column(item_list).spacing(4))
        .height(Length::Fill)
        .width(Length::Fill);

    let header = row![
        text("Items").size(14),
        horizontal_space(),
        button(text("Scan"))
            .on_press(scan_msg)
            .style(style::browse_button),
    ]
    .padding(12)
    .align_y(iced::Alignment::Center);

    let left_panel = column![horizontal_rule(1), header, scroll]
        .spacing(0)
        .width(Length::FillPortion(1));

    // Detail panel (right panel) - scrollable and wider
    let mut detail_content = column![
        text(R::detail_title()).size(16),
        // vertical_space().height(10)
    ];

    if let Some((orig_idx, _record)) = editor.selected_idx.and_then(|idx| editor.filtered.get(idx))
    {
        let descriptors = R::field_descriptors();
        for (i, descriptor) in descriptors.iter().enumerate() {
            let _field_name = descriptor.name.to_string();
            let value = editor.edit_buffers.get(i).map(|s| s.as_str()).unwrap_or("");
            detail_content = detail_content.push(build_field_input(
                descriptor,
                value,
                *orig_idx,
                lookups,
                field_changed_msg,
            ));
        }
    } else {
        detail_content = detail_content.push(text(R::empty_selection_text()).size(13));
    }

    let detail_scroll = scrollable(detail_content.spacing(8))
        .height(Length::Fill)
        .width(Length::Fill);

    let detail_panel = container(detail_scroll)
        .width(Length::FillPortion(2))
        .padding(16)
        .style(style::info_card);

    let main_content = row![left_panel, detail_panel].spacing(16);

    column![horizontal_rule(1), main_content, status_row]
        .spacing(0)
        .height(Fill)
        .into()
}

// ===========================================================================
// Multi-file editor view (3-panel: file list | record list | record editor)
// ===========================================================================

/// Build a 3-panel editor view for multi-file record types.
///
/// Panels:
/// 1. File list (left) — discovered files matching the type's pattern
/// 2. Record list (center) — records within the selected file
/// 3. Record editor (right) — editable fields for the selected record
pub fn build_multi_file_editor_view<'a, R: EditableRecord>(
    _app: &'a App,
    editor: &'a MultiFileEditorState<R>,
    scan_msg: Message,
    file_select_msg: fn(std::path::PathBuf) -> Message,
    save_msg: Message,
    add_msg: Message,
    remove_msg: fn(usize) -> Message,
    select_msg: fn(usize) -> Message,
    field_changed_msg: fn(usize, String, String) -> Message,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
) -> Element<'a, Message> {
    build_multi_file_editor_view_inner(
        editor,
        scan_msg,
        file_select_msg,
        save_msg,
        add_msg,
        remove_msg,
        select_msg,
        field_changed_msg,
        lookups,
    )
}

fn build_multi_file_editor_view_inner<'a, R: EditableRecord>(
    editor: &'a MultiFileEditorState<R>,
    scan_msg: Message,
    file_select_msg: fn(std::path::PathBuf) -> Message,
    save_msg: Message,
    add_msg: Message,
    remove_msg: fn(usize) -> Message,
    select_msg: fn(usize) -> Message,
    field_changed_msg: fn(usize, String, String) -> Message,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
) -> Element<'a, Message> {
    // Status bar
    let status_row = container(
        row![
            text(&editor.editor.status_msg)
                .size(13)
                .style(style::subtle_text),
            horizontal_space(),
            if editor.editor.is_loading {
                Element::from(text("Loading...").size(13))
            } else {
                Element::from(text(""))
            },
            horizontal_space().width(20),
            button(text(R::save_button_label()))
                .on_press(save_msg)
                .style(style::commit_button),
        ]
        .padding([10, 20])
        .align_y(iced::Alignment::Center),
    )
    .width(Fill)
    .style(style::status_bar);

    // Panel 1: File list (left)
    let file_items: Vec<Element<Message>> = editor
        .file_list
        .iter()
        .enumerate()
        .map(|(_idx, path)| {
            let is_selected = editor.current_file.as_ref() == Some(path);
            let label = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let btn = button(text(label).size(11).font(Font::MONOSPACE))
                .width(Fill)
                .on_press(file_select_msg(path.clone()));

            if is_selected {
                btn.style(style::active_chip).into()
            } else {
                btn.style(style::chip).into()
            }
        })
        .collect();

    let file_scroll = scrollable(column(file_items).spacing(4))
        .height(Length::Fill)
        .width(Length::Fill);

    let file_header = row![
        text("Files").size(14),
        horizontal_space(),
        button(text("Scan"))
            .on_press(scan_msg)
            .style(style::browse_button),
    ]
    .padding(12)
    .align_y(iced::Alignment::Center);

    let file_panel = column![horizontal_rule(1), file_header, file_scroll]
        .spacing(0)
        .width(Length::FillPortion(1));

    // Panel 2: Record list (center)
    let item_list: Vec<Element<Message>> = editor
        .editor
        .filtered
        .iter()
        .enumerate()
        .map(|(idx, (_, record))| {
            let is_selected = editor.editor.selected_idx == Some(idx);
            let label = record.list_label_with_lookups(lookups);
            let btn = button(text(label).size(11).font(Font::MONOSPACE))
                .width(Fill)
                .on_press(select_msg(idx));

            if is_selected {
                btn.style(style::active_chip).into()
            } else {
                btn.style(style::chip).into()
            }
        })
        .collect();

    let record_scroll = scrollable(column(item_list).spacing(4))
        .height(Length::Fill)
        .width(Length::Fill);

    let record_header = row![
        text("Records").size(14),
        horizontal_space(),
        text(format!("{} records", editor.editor.filtered.len()))
            .size(12)
            .style(style::subtle_text),
        horizontal_space().width(8),
        button(text("+").size(13))
            .on_press(add_msg)
            .style(style::browse_button),
        button(text("−").size(13))
            .on_press_maybe(editor.editor.selected_idx.map(|idx| remove_msg(idx)))
            .style(style::browse_button),
    ]
    .padding(12)
    .align_y(iced::Alignment::Center);

    let record_panel = column![horizontal_rule(1), record_header, record_scroll]
        .spacing(0)
        .width(Length::FillPortion(1));

    // Panel 3: Detail editor (right, scrollable, wider)
    let mut detail_content = column![
        text(R::detail_title()).size(16),
        vertical_space().height(10)
    ];

    if let Some((orig_idx, _record)) = editor
        .editor
        .selected_idx
        .and_then(|idx| editor.editor.filtered.get(idx))
    {
        let descriptors = R::field_descriptors();
        for (i, descriptor) in descriptors.iter().enumerate() {
            let value = editor
                .editor
                .edit_buffers
                .get(i)
                .map(|s| s.as_str())
                .unwrap_or("");
            detail_content = detail_content.push(build_field_input(
                descriptor,
                value,
                *orig_idx,
                lookups,
                field_changed_msg,
            ));
        }
    } else {
        detail_content = detail_content.push(text(R::empty_selection_text()).size(13));
    }

    let detail_scroll = scrollable(detail_content.spacing(8))
        .height(Length::Fill)
        .width(Length::Fill);

    let detail_panel = container(detail_scroll)
        .width(Length::FillPortion(2))
        .padding(16)
        .style(style::info_card);

    let main_content = row![file_panel, record_panel, detail_panel].spacing(16);

    column![horizontal_rule(1), main_content, status_row]
        .spacing(0)
        .height(Fill)
        .into()
}

// ===========================================================================
// Field input builder
// ===========================================================================

fn build_field_input<'a>(
    descriptor: &'a FieldDescriptor,
    value: &'a str,
    orig_idx: usize,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
    field_changed_msg: fn(usize, String, String) -> Message,
) -> Element<'a, Message> {
    match descriptor.kind {
        FieldKind::String | FieldKind::Integer => {
            labeled_input(descriptor.label, value, move |v| {
                Message::from(field_changed_msg(orig_idx, descriptor.name.to_string(), v))
            })
        }
        FieldKind::Enum { variants } => {
            let options: Vec<String> = variants.iter().map(|s| s.to_string()).collect();
            let selected = if options.contains(&value.to_string()) {
                Some(value.to_string())
            } else {
                options.first().cloned()
            };
            labeled_select(
                descriptor.label,
                selected.unwrap_or_default(),
                options,
                move |v| Message::from(field_changed_msg(orig_idx, descriptor.name.to_string(), v)),
            )
        }
        FieldKind::Lookup(lookup_key) => {
            let options = lookups.get(lookup_key).cloned().unwrap_or_default();
            if options.is_empty() {
                // Fallback to text input if lookup data not available
                labeled_input(descriptor.label, value, move |v| {
                    Message::from(field_changed_msg(orig_idx, descriptor.name.to_string(), v))
                })
            } else {
                // Use display names as options, but store the ID as the value
                let display_options: Vec<String> =
                    options.iter().map(|(_, name)| name.clone()).collect();
                // Find current display name
                let current_display = options
                    .iter()
                    .find(|(id, _)| id == value)
                    .map(|(_, name)| name.clone())
                    .unwrap_or_else(|| display_options.first().cloned().unwrap_or_default());
                labeled_select(
                    descriptor.label,
                    current_display,
                    display_options,
                    move |v| {
                        // Convert display name back to ID
                        let id = options
                            .iter()
                            .find(|(_, name)| name == &v)
                            .map(|(id, _)| id.clone())
                            .unwrap_or_else(|| v.clone());
                        Message::from(field_changed_msg(orig_idx, descriptor.name.to_string(), id))
                    },
                )
            }
        }
    }
}
