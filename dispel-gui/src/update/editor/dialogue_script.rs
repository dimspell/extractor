use dispel_core::{DialogueScript, Extractor};
use iced::Task;

use super::tab;
use crate::app::App;
use crate::handle_spreadsheet_messages_tab;
use crate::loading_state::LoadingState;
use crate::message::editor::dialogue_script::DialogueScriptEditorMessage;
use crate::message::MessageExt;

pub fn handle(msg: DialogueScriptEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = tab::get_tab_id(&app.state.workspace);

    match msg {
        DialogueScriptEditorMessage::LoadCatalog => {
            if let Some(editor) = app.state.dialogue_script_editors.get_mut(&tab_id) {
                if let Some(path) = editor.current_file.clone() {
                    editor.editor.loading_state = LoadingState::Loading;
                    return Task::perform(
                        async move { DialogueScript::read_file(&path).map_err(|e| e.to_string()) },
                        |result| {
                            crate::message::Message::dialogue_script(
                                DialogueScriptEditorMessage::CatalogLoaded(result),
                            )
                        },
                    );
                }
            }
            Task::none()
        }
        DialogueScriptEditorMessage::CatalogLoaded(result) => {
            if let Some(editor) = app.state.dialogue_script_editors.get_mut(&tab_id) {
                editor.editor.loading_state = LoadingState::Loaded(());
                match result {
                    Ok(catalog) => {
                        editor.editor.status_msg =
                            format!("DialogueScript catalog loaded: {} entries", catalog.len());
                        editor.editor.catalog = Some(catalog);
                        editor.editor.refresh();
                        if let Some(spreadsheet) =
                            app.state.dialogue_script_spreadsheets.get_mut(&tab_id)
                        {
                            spreadsheet.active = true;
                            spreadsheet.init_filter(editor.editor.catalog.as_ref().unwrap());
                            spreadsheet.compute_all_caches(editor.editor.catalog.as_ref().unwrap());
                            spreadsheet.init_pane_state();
                        }
                    }
                    Err(e) => {
                        editor.editor.status_msg =
                            format!("Error loading dialogue script catalog: {}", e);
                    }
                }
            }
            Task::none()
        }
        DialogueScriptEditorMessage::Select(index) => {
            tab::select(&mut app.state.dialogue_script_editors, tab_id, index)
        }
        DialogueScriptEditorMessage::FieldChanged(index, field, value) => tab::field_changed(
            &mut app.state.dialogue_script_editors,
            tab_id,
            index,
            field,
            value,
        ),
        DialogueScriptEditorMessage::Save => tab::save(
            &mut app.state.dialogue_script_editors,
            tab_id,
            "DialogueScripts saved successfully.",
            "Error saving dialogue scripts",
        ),
        DialogueScriptEditorMessage::Saved(_) => Task::none(),
        DialogueScriptEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages_tab!(
                app,
                dialogue_script_spreadsheets,
                dialogue_script_editors,
                &tab_id,
                |index, field, value| crate::message::Message::dialogue_script(
                    DialogueScriptEditorMessage::FieldChanged(index, field, value)
                ),
                msg
            );
            Task::none()
        }
        DialogueScriptEditorMessage::PaneResized(event) => tab::pane_resized(
            &mut app.state.dialogue_script_editors,
            &mut app.state.dialogue_script_spreadsheets,
            tab_id,
            event,
        ),
        DialogueScriptEditorMessage::PaneClicked(pane) => {
            tab::pane_clicked(&mut app.state.dialogue_script_editors, tab_id, pane)
        }
    }
}
