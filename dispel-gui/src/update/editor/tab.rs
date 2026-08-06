use std::path::PathBuf;

use crate::components::editable::EditableRecord;
use crate::components::generic_editor::{MultiFileEditorState, TabbedEditor};
use crate::components::loading_state::LoadingState;
use crate::message::Message;
use crate::view::editor::SpreadsheetState;
use crate::workspace::Workspace;
use dispel_core::Extractor;
use iced::Task;
use iced::widget::pane_grid;

pub fn get_tab_id(workspace: &Workspace) -> usize {
    workspace.active().map(|t| t.id).unwrap_or(usize::MAX)
}

pub fn select<T: EditableRecord + Extractor>(
    tabbed_editor: &mut TabbedEditor<T>,
    tab_id: usize,
    index: usize,
) -> Task<Message> {
    if let Some(editor) = tabbed_editor.editors.get_mut(&tab_id) {
        editor.select(index);
    }
    Task::none()
}

pub fn field_changed<T: EditableRecord + Extractor>(
    tabbed_editor: &mut TabbedEditor<T>,
    tab_id: usize,
    index: usize,
    field: String,
    value: String,
) -> Task<Message> {
    if let Some(editor) = tabbed_editor.editors.get_mut(&tab_id) {
        editor.update_field(index, &field, value);
    }
    Task::none()
}

pub fn save<T: EditableRecord + Extractor>(
    tabbed_editor: &mut TabbedEditor<T>,
    tab_id: usize,
    success_msg: &str,
    error_label: &str,
) -> Task<Message> {
    if let Some(editor) = tabbed_editor.editors.get_mut(&tab_id) {
        editor.editor.loading_state = LoadingState::Loading;
        let result = editor.save();
        editor.editor.loading_state = LoadingState::Loaded(());
        match result {
            Ok(_) => editor.editor.status_msg = success_msg.into(),
            Err(e) => editor.editor.status_msg = format!("{}: {}", error_label, e),
        }
    }
    Task::none()
}

pub fn add_entry<T: EditableRecord + Extractor>(
    tabbed_editor: &mut TabbedEditor<T>,
    tab_id: usize,
) -> Task<Message> {
    if let Some(editor) = tabbed_editor.editors.get_mut(&tab_id) {
        editor.add_record();
    }
    Task::none()
}

pub fn remove_entry<T: EditableRecord + Extractor>(
    tabbed_editor: &mut TabbedEditor<T>,
    tab_id: usize,
    index: usize,
) -> Task<Message> {
    if let Some(editor) = tabbed_editor.editors.get_mut(&tab_id)
        && let Err(msg) = editor.remove_record(index)
    {
        eprintln!("{}", msg);
    }
    Task::none()
}

pub fn pane_resized<T: EditableRecord>(
    tabbed_editor: &mut TabbedEditor<T>,
    tab_id: usize,
    event: pane_grid::ResizeEvent,
) -> Task<Message> {
    if let Some(editor) = tabbed_editor.editors.get_mut(&tab_id)
        && let Some(ref mut ps) = editor.editor.pane_state
    {
        ps.resize(event.split, event.ratio);
    }
    if let Some(ss) = tabbed_editor.spreadsheets.get_mut(&tab_id)
        && let Some(ref mut ps) = ss.pane_state
    {
        ps.resize(event.split, event.ratio);
    }
    Task::none()
}

pub fn pane_clicked<T: EditableRecord>(
    tabbed_editor: &mut TabbedEditor<T>,
    tab_id: usize,
    pane: pane_grid::Pane,
) -> Task<Message> {
    if let Some(editor) = tabbed_editor.editors.get_mut(&tab_id) {
        editor.editor.pane_focus = Some(pane);
    }
    Task::none()
}

pub fn load_catalog_sync<T: EditableRecord + Extractor>(
    path: PathBuf,
    tabbed_editor: &mut TabbedEditor<T>,
    tab_id: usize,
    lookups: &std::collections::HashMap<String, Vec<(String, String)>>,
) {
    let mut editor_state = MultiFileEditorState::<T>::default();
    editor_state.select_file(path);

    let mut ss = SpreadsheetState::new();
    if let Some(catalog) = editor_state.editor.catalog.as_ref() {
        ss.apply_filter(catalog);
        ss.compute_all_caches(catalog, lookups);
        ss.init_pane_state();
    }

    tabbed_editor.editors.insert(tab_id, editor_state);
    tabbed_editor.spreadsheets.insert(tab_id, ss);
}
