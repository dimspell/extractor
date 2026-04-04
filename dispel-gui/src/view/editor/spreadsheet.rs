use crate::generic_editor::GenericEditorState;
use crate::message::Message;
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space};
use dispel_core::references::editable::{EditableRecord, FieldDescriptor, FieldKind};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Fill, Font, Length};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct SpreadsheetState {
    pub sort_column: Option<usize>,
    pub sort_ascending: bool,
    pub filter_query: String,
    pub filtered_indices: Vec<usize>,
    pub selected_rows: Vec<usize>,
    pub last_selected: Option<usize>,
    pub editing_cell: Option<(usize, usize)>,
    pub edit_buffer: String,
    pub show_inspector: bool,
}

impl SpreadsheetState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_sort(&mut self, col: usize) {
        if self.sort_column == Some(col) {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = Some(col);
            self.sort_ascending = true;
        }
    }

    pub fn apply_filter<R: EditableRecord>(&mut self, catalog: &[R]) {
        self.filtered_indices.clear();
        if self.filter_query.is_empty() {
            self.filtered_indices = (0..catalog.len()).collect();
            return;
        }
        let query = self.filter_query.to_lowercase();
        for (idx, record) in catalog.iter().enumerate() {
            let label = record.list_label();
            if label.to_lowercase().contains(&query) {
                self.filtered_indices.push(idx);
            }
        }
    }

    pub fn apply_sort<R: EditableRecord>(&mut self, catalog: &[R]) {
        if let Some(col) = self.sort_column {
            let descriptors = R::field_descriptors();
            if col >= descriptors.len() {
                return;
            }
            let field = descriptors[col].name;
            self.filtered_indices.sort_by(|a, b| {
                let val_a = catalog[*a].get_field(field);
                let val_b = catalog[*b].get_field(field);
                let cmp = val_a.cmp(&val_b);
                if self.sort_ascending {
                    cmp
                } else {
                    cmp.reverse()
                }
            });
        }
    }

    pub fn toggle_row_selection(&mut self, filtered_idx: usize, modifiers: bool) {
        if modifiers {
            if let Some(last) = self.last_selected {
                let start = last.min(filtered_idx);
                let end = last.max(filtered_idx);
                self.selected_rows.clear();
                for i in start..=end {
                    if !self.selected_rows.contains(&i) {
                        self.selected_rows.push(i);
                    }
                }
            } else if !self.selected_rows.contains(&filtered_idx) {
                self.selected_rows.push(filtered_idx);
            }
        } else {
            self.selected_rows.clear();
            self.selected_rows.push(filtered_idx);
        }
        self.last_selected = Some(filtered_idx);
    }

    pub fn start_editing<R: EditableRecord>(
        &mut self,
        filtered_idx: usize,
        col: usize,
        catalog: &[R],
    ) {
        if let Some(&orig_idx) = self.filtered_indices.get(filtered_idx) {
            if let Some(record) = catalog.get(orig_idx) {
                let descriptors = R::field_descriptors();
                if let Some(desc) = descriptors.get(col) {
                    self.editing_cell = Some((filtered_idx, col));
                    self.edit_buffer = record.get_field(desc.name);
                }
            }
        }
    }

    pub fn commit_edit<R: EditableRecord>(
        &mut self,
        catalog: &mut [R],
        field_changed_msg: fn(usize, String, String) -> Message,
        orig_idx: usize,
    ) -> Option<Message> {
        if let Some((_, col)) = self.editing_cell.take() {
            let descriptors = R::field_descriptors();
            if let Some(desc) = descriptors.get(col) {
                let old_value = catalog[orig_idx].get_field(desc.name);
                let new_value = self.edit_buffer.clone();
                if old_value != new_value {
                    catalog[orig_idx].set_field(desc.name, new_value.clone());
                    return Some(field_changed_msg(
                        orig_idx,
                        desc.name.to_string(),
                        new_value,
                    ));
                }
            }
        }
        None
    }

    pub fn cancel_editing(&mut self) {
        self.editing_cell = None;
        self.edit_buffer.clear();
    }

    pub fn toggle_inspector(&mut self) {
        self.show_inspector = !self.show_inspector;
    }
}

#[derive(Debug, Clone)]
pub enum SpreadsheetMessage {
    SortColumn(usize),
    FilterChanged(String),
    SelectRow(usize, bool),
    StartEdit(usize, usize),
    EditCellInput(String),
    CommitEdit(usize),
    CancelEdit,
    ToggleInspector,
}

pub fn view_spreadsheet<'a, R: EditableRecord>(
    editor: &'a GenericEditorState<R>,
    spreadsheet: &'a SpreadsheetState,
    scan_msg: Message,
    save_msg: Message,
    _select_msg: fn(usize) -> Message,
    field_changed_msg: fn(usize, String, String) -> Message,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
) -> Element<'a, Message> {
    let descriptors = R::field_descriptors();

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
            button(text("Inspector"))
                .on_press(spreadsheet_msg(SpreadsheetMessage::ToggleInspector))
                .style(style::browse_button),
            button(text(R::save_button_label()))
                .on_press(save_msg)
                .style(style::commit_button),
        ]
        .padding([10, 20])
        .align_y(iced::Alignment::Center),
    )
    .width(Fill)
    .style(style::status_bar);

    let filter_bar = row![
        text("Filter:").size(12).style(style::subtle_text),
        text_input("Search records...", &spreadsheet.filter_query)
            .on_input(move |q| spreadsheet_msg(SpreadsheetMessage::FilterChanged(q)))
            .padding(6)
            .width(Length::FillPortion(2)),
        horizontal_space(),
        text(format!(
            "{} of {} records",
            spreadsheet.filtered_indices.len(),
            editor.catalog.as_ref().map(|c| c.len()).unwrap_or(0)
        ))
        .size(11)
        .style(style::subtle_text),
        button(text("Scan"))
            .on_press(scan_msg)
            .style(style::browse_button),
    ]
    .padding([8, 12])
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let header_cells: Vec<Element<Message>> = descriptors
        .iter()
        .enumerate()
        .map(|(col, desc)| {
            let sort_indicator = if spreadsheet.sort_column == Some(col) {
                if spreadsheet.sort_ascending {
                    " \u{25B2}"
                } else {
                    " \u{25BC}"
                }
            } else {
                ""
            };
            button(text(format!("{}{}", desc.label, sort_indicator)).size(11))
                .on_press(spreadsheet_msg(SpreadsheetMessage::SortColumn(col)))
                .style(style::chip)
                .padding([6, 8])
                .into()
        })
        .collect();

    let header_row = container(row(header_cells).spacing(2))
        .width(Fill)
        .style(style::status_bar);

    let catalog = editor.catalog.as_ref();
    let rows: Vec<Element<Message>> = if let Some(catalog) = catalog {
        spreadsheet
            .filtered_indices
            .iter()
            .enumerate()
            .map(|(filtered_idx, &orig_idx)| {
                let record = &catalog[orig_idx];
                let is_selected = spreadsheet.selected_rows.contains(&filtered_idx);
                let is_editing = spreadsheet.editing_cell.map(|(f, _)| f) == Some(filtered_idx);

                let cells: Vec<Element<Message>> = descriptors
                    .iter()
                    .enumerate()
                    .map(|(col, desc)| {
                        let value = record.get_field(desc.name);
                        let is_cell_editing =
                            is_editing && spreadsheet.editing_cell.map(|(_, c)| c) == Some(col);

                        if is_cell_editing {
                            text_input("", &spreadsheet.edit_buffer)
                                .on_input(move |v| {
                                    spreadsheet_msg(SpreadsheetMessage::EditCellInput(v))
                                })
                                .on_submit(spreadsheet_msg(SpreadsheetMessage::CommitEdit(
                                    orig_idx,
                                )))
                                .padding(4)
                                .size(11)
                                .font(Font::MONOSPACE)
                                .width(Length::Fill)
                                .into()
                        } else {
                            let display = if value.len() > 30 {
                                format!("{}...", &value[..30])
                            } else {
                                value
                            };
                            button(text(display).size(11).font(Font::MONOSPACE))
                                .on_press(spreadsheet_msg(SpreadsheetMessage::StartEdit(
                                    filtered_idx,
                                    col,
                                )))
                                .style(if is_selected {
                                    style::active_chip
                                } else {
                                    style::chip
                                })
                                .padding([4, 8])
                                .width(Length::Fill)
                                .into()
                        }
                    })
                    .collect();

                container(row(cells).spacing(2))
                    .width(Fill)
                    .style(if is_selected {
                        style::selected_row
                    } else {
                        style::normal_row
                    })
                    .into()
            })
            .collect()
    } else {
        vec![container(
            text("No data loaded. Click Scan to load records.")
                .size(13)
                .style(style::subtle_text),
        )
        .width(Fill)
        .padding(20)
        .into()]
    };

    let table_scroll = scrollable(column(rows).spacing(1))
        .height(Length::Fill)
        .width(Length::Fill);

    let main_content = if spreadsheet.show_inspector {
        let inspector_panel =
            build_inspector_panel(editor, spreadsheet, lookups, field_changed_msg);
        row![table_scroll, inspector_panel].spacing(16).width(Fill)
    } else {
        row![table_scroll].width(Fill)
    };

    column![
        horizontal_rule(1),
        filter_bar,
        horizontal_rule(1),
        header_row,
        main_content,
        status_row
    ]
    .spacing(0)
    .height(Fill)
    .into()
}

fn build_inspector_panel<'a, R: EditableRecord>(
    editor: &'a GenericEditorState<R>,
    spreadsheet: &'a SpreadsheetState,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
    field_changed_msg: fn(usize, String, String) -> Message,
) -> Element<'a, Message> {
    let descriptors = R::field_descriptors();

    let mut content = column![text("Inspector").size(14)].padding([8, 12]);

    if let Some(&orig_idx) = spreadsheet
        .selected_rows
        .first()
        .and_then(|f| spreadsheet.filtered_indices.get(*f))
    {
        if let Some(_record) = editor.catalog.as_ref().and_then(|c| c.get(orig_idx)) {
            for (i, desc) in descriptors.iter().enumerate() {
                let value = editor.edit_buffers.get(i).map(|s| s.as_str()).unwrap_or("");
                content = content.push(build_inspector_field(
                    desc,
                    value,
                    orig_idx,
                    lookups,
                    field_changed_msg,
                ));
            }
        }
    } else {
        content = content.push(text("No row selected").size(12).style(style::subtle_text));
    }

    container(content)
        .width(320)
        .style(style::sidebar_container)
        .into()
}

fn build_inspector_field<'a>(
    descriptor: &'a FieldDescriptor,
    value: &'a str,
    orig_idx: usize,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
    field_changed_msg: fn(usize, String, String) -> Message,
) -> Element<'a, Message> {
    let label = text(descriptor.label).size(11).style(style::subtle_text);

    match &descriptor.kind {
        FieldKind::Lookup(key) => {
            if let Some(options) = lookups.get(*key) {
                let items: Vec<Element<Message>> = options
                    .iter()
                    .map(|(id, name)| {
                        let is_selected = value == id;
                        button(text(name).size(11))
                            .width(Length::Fill)
                            .on_press(field_changed_msg(
                                orig_idx,
                                descriptor.name.to_string(),
                                id.clone(),
                            ))
                            .style(if is_selected {
                                style::active_chip
                            } else {
                                style::chip
                            })
                            .into()
                    })
                    .collect();
                column![label, column(items).spacing(2)].spacing(4).into()
            } else {
                column![
                    label,
                    text_input("", value)
                        .padding(4)
                        .size(11)
                        .width(Length::Fill)
                ]
                .spacing(4)
                .into()
            }
        }
        _ => column![
            label,
            text_input("", value)
                .on_input(move |v| field_changed_msg(orig_idx, descriptor.name.to_string(), v))
                .padding(4)
                .size(11)
                .width(Length::Fill)
        ]
        .spacing(4)
        .into(),
    }
}
