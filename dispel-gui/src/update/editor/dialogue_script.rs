// DialogueScript editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages_tab;
use crate::message::editor::dialogue_script::DialogueScriptEditorMessage;
use crate::message::MessageExt;
use dispel_core::DialogueScript;
use iced::Task;

pub fn handle(
    message: DialogueScriptEditorMessage,
    app: &mut App,
) -> Task<crate::message::Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);

    match message {
        DialogueScriptEditorMessage::LoadCatalog => {
            if let Some(editor) = app.state.dialogue_script_editors.get_mut(&tab_id) {
                editor.editor.loading_state = crate::loading_state::LoadingState::Loading;
                let path = editor.current_file.clone();
                return Task::perform(
                    async move {
                        let path = std::path::PathBuf::from(path);
                        let scripts: std::io::Result<Vec<DialogueScript>> =
                            <DialogueScript as dispel_core::references::extractor::Extractor>::read_file(
                                &path,
                            );
                        scripts.map_err(|e| e.to_string())
                    },
                    |res: Result<Vec<DialogueScript>, String>| {
                        <crate::message::Message as crate::message::MessageExt>::dialogue_script(
                            DialogueScriptEditorMessage::CatalogLoaded(res),
                        )
                    },
                );
            }
            Task::none()
        }
        DialogueScriptEditorMessage::CatalogLoaded(result) => {
            if let Some(editor) = app.state.dialogue_script_editors.get_mut(&tab_id) {
                editor.editor.loading_state = crate::loading_state::LoadingState::Loaded(());
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
            if let Some(editor) = app.state.dialogue_script_editors.get_mut(&tab_id) {
                editor.select_dialog(index);
            }
            Task::none()
        }
        DialogueScriptEditorMessage::FieldChanged(index, field, value) => {
            if let Some(editor) = app.state.dialogue_script_editors.get_mut(&tab_id) {
                editor.update_field(index, &field, value);
            }
            Task::none()
        }
        DialogueScriptEditorMessage::Save => {
            if let Some(editor) = app.state.dialogue_script_editors.get_mut(&tab_id) {
                editor.editor.loading_state = crate::loading_state::LoadingState::Loading;
                let result = editor.save();
                editor.editor.loading_state = crate::loading_state::LoadingState::Loaded(());
                match result {
                    Ok(_) => {
                        editor.editor.status_msg = "DialogueScripts saved successfully.".into()
                    }
                    Err(e) => {
                        editor.editor.status_msg = format!("Error saving dialogue scripts: {}", e)
                    }
                }
            }
            Task::none()
        }
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
        DialogueScriptEditorMessage::PaneResized(event) => {
            if let Some(ed) = app.state.dialogue_script_editors.get_mut(&tab_id) {
                if let Some(ref mut ps) = ed.editor.pane_state {
                    ps.resize(event.split, event.ratio);
                }
            }
            if let Some(ss) = app.state.dialogue_script_spreadsheets.get_mut(&tab_id) {
                if let Some(ref mut ps) = ss.pane_state {
                    ps.resize(event.split, event.ratio);
                }
            }
            Task::none()
        }
        DialogueScriptEditorMessage::PaneClicked(pane) => {
            if let Some(ed) = app.state.dialogue_script_editors.get_mut(&tab_id) {
                ed.editor.pane_focus = Some(pane);
            }
            Task::none()
        }
    }
}
