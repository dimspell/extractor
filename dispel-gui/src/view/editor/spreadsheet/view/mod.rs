//! Pure render functions. Mutating state belongs in
//! [`super::state::SpreadsheetState`]; this module only assembles widgets.

pub mod filter_bar;
pub mod filter_modal;
pub mod inspector;
pub mod status_bar;
pub mod table;

use super::message::SpreadsheetMessage;
use super::state::{SpreadsheetPaneContent, SpreadsheetState};
use crate::components::editable::EditableRecord;
use crate::components::modal;
use crate::generic_editor::GenericEditorState;
use crate::message::Message;
use crate::utils::horizontal_rule;
use iced::widget::column;
use iced::widget::pane_grid::{self, Pane};
use iced::{Element, Fill, Length};
use std::collections::HashMap;

#[allow(clippy::too_many_arguments)]
pub fn view_spreadsheet<'a, R: EditableRecord>(
    editor: &'a GenericEditorState<R>,
    spreadsheet: &'a SpreadsheetState,
    scan_msg: Message,
    save_msg: Message,
    _select_msg: fn(usize) -> Message,
    field_changed_msg: fn(usize, String, String) -> Message,
    spreadsheet_msg: fn(SpreadsheetMessage) -> Message,
    lookups: &'a HashMap<String, Vec<(String, String)>>,
    pane_resized_msg: fn(pane_grid::ResizeEvent) -> Message,
    pane_clicked_msg: fn(Pane) -> Message,
) -> Element<'a, Message> {
    let descriptors = R::field_descriptors();

    let status_row = status_bar::build_status_bar(editor, spreadsheet, save_msg, spreadsheet_msg);
    let filter_bar = filter_bar::build_filter_bar(editor, spreadsheet, scan_msg, spreadsheet_msg);

    let catalog = editor.catalog.as_ref();

    let main_content = if let Some(ref pane_state) = spreadsheet.pane_state {
        let pane_grid = pane_grid::PaneGrid::new(pane_state, |_id, pane_content, _is_maximized| {
            let content: Element<Message> = match pane_content {
                SpreadsheetPaneContent::Table => {
                    table::build_table_content(descriptors, catalog, spreadsheet, spreadsheet_msg)
                }
                SpreadsheetPaneContent::Inspector => inspector::build_inspector_panel(
                    editor,
                    spreadsheet,
                    lookups,
                    field_changed_msg,
                    spreadsheet_msg,
                ),
            };
            pane_grid::Content::new(content)
        })
        .on_click(pane_clicked_msg)
        .on_resize(4, pane_resized_msg)
        .height(Length::Fill)
        .width(Length::Fill);

        column![
            horizontal_rule(1),
            filter_bar,
            horizontal_rule(1),
            pane_grid,
            status_row,
        ]
        .spacing(0)
        .height(Fill)
        .into()
    } else {
        let table = table::build_table_content(descriptors, catalog, spreadsheet, spreadsheet_msg);
        column![
            horizontal_rule(1),
            filter_bar,
            horizontal_rule(1),
            table,
            status_row,
        ]
        .spacing(0)
        .height(Fill)
        .into()
    };

    if let Some(col) = spreadsheet.active_column_filter {
        let modal_content =
            filter_modal::build_column_filter_modal(col, spreadsheet, spreadsheet_msg);
        modal::modal(
            main_content,
            modal_content,
            move || spreadsheet_msg(SpreadsheetMessage::CloseColumnFilterModal),
            0.5,
        )
    } else {
        main_content
    }
}
