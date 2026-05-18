use dispel_core::{DialogueScript, Extractor};
use iced::Task;

use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::message::{Message, MessageExt};
use crate::update::editor::tab;

mod component;

crate::define_tab_editor! {
    name: dialogue_script,
    name_pascal: DialogueScript,
    record: DialogueScript,
    field: dialogue_script_editor,
    empty_text: "DialogueScript file not loaded",
    save_success_msg: "DialogueScripts saved successfully.",
    save_error_msg: "Error saving dialogue scripts",
    extra_variants: {
        LoadCatalog,
        CatalogLoaded(Result<Vec<DialogueScript>, String>),
    },
}

mod view;
pub use view::view;

pub fn handle(msg: DialogueScriptEditorMessage, app: &mut App) -> Task<Message> {
    let tab_id = tab::get_tab_id(&app.state.workspace);

    match msg {
        DialogueScriptEditorMessage::LoadCatalog => {
            if let Some(editor) = app.state.dialogue_script_editor.editors.get_mut(&tab_id) {
                if let Some(path) = editor.current_file.clone() {
                    editor.editor.loading_state = LoadingState::Loading;
                    return Task::perform(
                        async move { DialogueScript::read_file(&path).map_err(|e| e.to_string()) },
                        move |result| {
                            Message::dialogue_script(DialogueScriptEditorMessage::CatalogLoaded(
                                result,
                            ))
                        },
                    );
                }
            }
            Task::none()
        }
        DialogueScriptEditorMessage::CatalogLoaded(result) => {
            if let Some(editor) = app.state.dialogue_script_editor.editors.get_mut(&tab_id) {
                editor.editor.loading_state = LoadingState::Loaded(());
                match result {
                    Ok(catalog) => {
                        editor.editor.status_msg =
                            format!("DialogueScript catalog loaded: {} entries", catalog.len());
                        editor.editor.catalog = Some(catalog);
                        editor.editor.refresh();
                        if let Some(spreadsheet) = app
                            .state
                            .dialogue_script_editor
                            .spreadsheets
                            .get_mut(&tab_id)
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
        msg => handle_core(msg, app, tab_id),
    }
}
