//! Right-hand inspector pane — one input widget per field of the
//! currently-selected record.

use crate::components::editable::{EditableRecord, FieldDescriptor, FieldKind};
use crate::components::generic_editor::GenericEditorState;
use crate::components::textarea::{self, TextAreaContent};
use crate::components::utils::horizontal_space;
use crate::message::Message;
use crate::style;
use crate::view::editor::spreadsheet::message::SpreadsheetMessage;
use crate::view::editor::spreadsheet::state::SpreadsheetState;
use iced::widget::{
    button, column, container, pick_list, row, scrollable, text, text_input, Column,
};
use iced::{Element, Fill, Length};
use std::collections::HashMap;

pub fn build_inspector_panel<'a, R: EditableRecord>(
    editor: &'a GenericEditorState<R>,
    spreadsheet: &'a SpreadsheetState,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
    field_changed_msg: fn(usize, String, String) -> Message,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    let descriptors = R::field_descriptors();

    let header = container(
        row![
            text("Inspector").size(13),
            horizontal_space(),
            button(text("✕").size(11))
                .on_press(spreadsheet_msg(SpreadsheetMessage::CloseInspector))
                .style(style::browse_button)
                .padding([2, 6]),
        ]
        .align_y(iced::Alignment::Center)
        .padding([6, 10]),
    )
    .width(Fill)
    .style(style::spreadsheet_header);

    let mut fields: Column<Message> = column![].spacing(6).padding([8, 12]);

    if let Some(orig_idx) = spreadsheet.selected_orig {
        if let Some(record) = editor.catalog.as_ref().and_then(|c| c.get(orig_idx)) {
            for desc in descriptors.iter() {
                let value = record.get_field(desc.name);
                let lookup_data = match &desc.kind {
                    FieldKind::Lookup(key) => lookups.get(*key).cloned(),
                    _ => None,
                };
                let validation_error = record.validate_field(desc.name, &value);
                fields = fields.push(build_inspector_field(
                    desc,
                    value,
                    orig_idx,
                    lookup_data,
                    validation_error,
                    field_changed_msg,
                    &spreadsheet.inspector_textarea_contents,
                    spreadsheet_msg,
                ));
            }
        }
    } else {
        fields = fields.push(
            text("Click a row to inspect")
                .size(12)
                .style(style::subtle_text),
        );
    }

    let scroll = scrollable(fields).height(Length::Fill);

    container(column![header, scroll])
        .width(Fill)
        .height(Fill)
        .style(style::sidebar_container)
        .into()
}

#[allow(clippy::too_many_arguments)]
fn build_inspector_field<'a>(
    descriptor: &'a FieldDescriptor,
    value: String,
    orig_idx: usize,
    lookups: Option<Vec<(String, String)>>,
    validation_error: Option<String>,
    field_changed_msg: fn(usize, String, String) -> Message,
    textarea_contents: &'a HashMap<String, TextAreaContent>,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
) -> Element<'a, Message> {
    let label = text(descriptor.label).size(11).style(style::subtle_text);

    let body: Element<'a, Message> = match &descriptor.kind {
        FieldKind::TextArea => {
            let field_name = descriptor.name.to_string();
            if let Some(tc) = textarea_contents.get(descriptor.name) {
                textarea::textarea(&tc.0, move |action| {
                    spreadsheet_msg(SpreadsheetMessage::TextAreaChanged(
                        orig_idx,
                        field_name.clone(),
                        action,
                    ))
                })
            } else {
                text_input("", &value)
                    .on_input(move |v| field_changed_msg(orig_idx, field_name.clone(), v))
                    .padding(4)
                    .size(11)
                    .width(Length::Fill)
                    .into()
            }
        }
        FieldKind::Lookup(_) => {
            if let Some(options) = lookups {
                let field_name = descriptor.name.to_string();
                let selected = options
                    .iter()
                    .find(|(id, _)| id == &value)
                    .map(|(_, name)| name.clone());

                let options_vec: Vec<String> =
                    options.iter().map(|(_, name)| name.clone()).collect();

                pick_list(options_vec, selected, move |selected_name| {
                    let selected_id = options
                        .iter()
                        .find(|(_, name)| name == &selected_name)
                        .map(|(id, _)| id.clone())
                        .unwrap_or_default();
                    spreadsheet_msg(SpreadsheetMessage::InspectorFieldChanged(
                        orig_idx,
                        field_name.clone(),
                        selected_id,
                    ))
                })
                .width(Length::Fill)
                .into()
            } else {
                text_input("", &value)
                    .padding(4)
                    .size(11)
                    .width(Length::Fill)
                    .into()
            }
        }
        FieldKind::Enum { variants } => {
            let field_name = descriptor.name.to_string();
            let selected = variants.iter().find(|&&v| v == value).copied();
            pick_list(*variants, selected, move |selected_variant| {
                spreadsheet_msg(SpreadsheetMessage::InspectorFieldChanged(
                    orig_idx,
                    field_name.clone(),
                    selected_variant.to_string(),
                ))
            })
            .width(Length::Fill)
            .into()
        }
        _ => {
            let field_name = descriptor.name.to_string();
            text_input("", &value)
                .on_input(move |v| {
                    spreadsheet_msg(SpreadsheetMessage::InspectorFieldChanged(
                        orig_idx,
                        field_name.clone(),
                        v,
                    ))
                })
                .padding(4)
                .size(11)
                .width(Length::Fill)
                .into()
        }
    };

    let mut field_column = column![label, body].spacing(4);
    if let Some(err) = validation_error {
        field_column =
            field_column.push(
                text(err)
                    .size(10)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(iced::color!(0xff5252)),
                    }),
            );
    }
    field_column.into()
}
