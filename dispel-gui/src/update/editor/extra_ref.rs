// ExtraRef editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages_tab;
use crate::message::editor::extraref::ExtraRefEditorMessage;
use crate::message::MessageExt;
use iced::Task;

pub fn handle(message: ExtraRefEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);

    match message {
        ExtraRefEditorMessage::CatalogLoaded(id, result) => {
            if let Some(editor) = app.state.extra_ref_editors.get_mut(&id) {
                editor.editor.loading_state = crate::loading_state::LoadingState::Loaded(());
                match result {
                    Ok(catalog) => {
                        editor.editor.status_msg =
                            format!("Extra ref catalog loaded: {} entries", catalog.len());
                        editor.editor.catalog = Some(catalog);
                        // Note: refresh_items() is handled by the generic editor now
                    }
                    Err(e) => {
                        editor.editor.status_msg =
                            format!("Error loading extra ref catalog: {}", e);
                    }
                }
            }
            Task::none()
        }
        ExtraRefEditorMessage::SelectItem(index) => {
            if let Some(editor) = app.state.extra_ref_editors.get_mut(&tab_id) {
                editor.select(index);
            }
            Task::none()
        }
        ExtraRefEditorMessage::FieldChanged(index, field, value) => {
            if let Some(editor) = app.state.extra_ref_editors.get_mut(&tab_id) {
                editor.update_field(index, &field, value);
            }
            Task::none()
        }
        ExtraRefEditorMessage::Save => {
            if let Some(editor) = app.state.extra_ref_editors.get_mut(&tab_id) {
                editor.editor.loading_state = crate::loading_state::LoadingState::Loading;
                let result = editor.save();
                editor.editor.loading_state = crate::loading_state::LoadingState::Loaded(());
                match result {
                    Ok(_) => editor.editor.status_msg = "Extra refs saved successfully.".into(),
                    Err(e) => editor.editor.status_msg = format!("Error saving extra refs: {}", e),
                }
            }
            Task::none()
        }
        ExtraRefEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages_tab!(
                app,
                extra_ref_spreadsheets,
                extra_ref_editors,
                &tab_id,
                |index, field, value| crate::message::Message::extra_ref(
                    ExtraRefEditorMessage::FieldChanged(index, field, value)
                ),
                msg
            );
            Task::none()
        }
        ExtraRefEditorMessage::PaneResized(_) => Task::none(),
        ExtraRefEditorMessage::PaneClicked(_) => Task::none(),
    }
}
