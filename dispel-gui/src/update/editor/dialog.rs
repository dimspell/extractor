// Dialog editor handlers

use crate::app::App;
use crate::message::editor::dialog::DialogEditorMessage;
use iced::Task;

pub fn handle(message: DialogEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);

    match message {
        DialogEditorMessage::CatalogLoaded(id, result) => {
            if let Some(editor) = app.state.dialog_editors.get_mut(&id) {
                editor.loading_state = crate::loading_state::LoadingState::Loaded(());
                match result {
                    Ok(catalog) => {
                        editor.status_msg =
                            format!("Dialog catalog loaded: {} entries", catalog.len());
                        editor.catalog = Some(catalog);
                        editor.refresh_dialogs();
                    }
                    Err(e) => {
                        editor.status_msg = format!("Error loading dialog catalog: {}", e);
                    }
                }
            }
            Task::none()
        }
        DialogEditorMessage::SelectDialog(index) => {
            if let Some(editor) = app.state.dialog_editors.get_mut(&tab_id) {
                editor.select_dialog(index);
            }
            Task::none()
        }
        DialogEditorMessage::FieldChanged(index, field, value) => {
            if let Some(editor) = app.state.dialog_editors.get_mut(&tab_id) {
                editor.update_field(index, &field, value);
            }
            Task::none()
        }
        DialogEditorMessage::Save => {
            if let Some(editor) = app.state.dialog_editors.get_mut(&tab_id) {
                editor.loading_state = crate::loading_state::LoadingState::Loading;
                let result = editor.save();
                editor.loading_state = crate::loading_state::LoadingState::Loaded(());
                match result {
                    Ok(_) => editor.status_msg = "Dialogs saved successfully.".into(),
                    Err(e) => editor.status_msg = format!("Error saving dialogs: {}", e),
                }
            }
            Task::none()
        }
    }
}
