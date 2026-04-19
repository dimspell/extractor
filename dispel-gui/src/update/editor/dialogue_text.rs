// DialogueText editor handlers

use crate::app::App;
use crate::message::editor::dialoguetext::DialogueTextEditorMessage;
use iced::Task;

pub fn handle(message: DialogueTextEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);

    match message {
        DialogueTextEditorMessage::CatalogLoaded(id, result) => {
            if let Some(editor) = app.state.dialogue_text_editors.get_mut(&id) {
                editor.loading_state = crate::loading_state::LoadingState::Loaded(());
                match result {
                    Ok(catalog) => {
                        editor.status_msg =
                            format!("Text catalog loaded: {} entries", catalog.len());
                        editor.catalog = Some(catalog);
                        editor.refresh_texts();
                    }
                    Err(e) => {
                        editor.status_msg = format!("Error loading text catalog: {}", e);
                    }
                }
            }
            Task::none()
        }
        DialogueTextEditorMessage::SelectText(index) => {
            if let Some(editor) = app.state.dialogue_text_editors.get_mut(&tab_id) {
                editor.select_text(index);
            }
            Task::none()
        }
        DialogueTextEditorMessage::FieldChanged(index, field, value) => {
            if let Some(editor) = app.state.dialogue_text_editors.get_mut(&tab_id) {
                editor.update_field(index, &field, value);
            }
            Task::none()
        }
        DialogueTextEditorMessage::TextAction(index, action) => {
            use iced::widget::text_editor;
            if let text_editor::Action::Edit(_) = action {
                if let Some(editor) = app.state.dialogue_text_editors.get_mut(&tab_id) {
                    editor.update_text_content(index);
                }
            }
            Task::none()
        }
        DialogueTextEditorMessage::CommentAction(index, action) => {
            use iced::widget::text_editor;
            if let text_editor::Action::Edit(_) = action {
                if let Some(editor) = app.state.dialogue_text_editors.get_mut(&tab_id) {
                    editor.update_comment_content(index);
                }
            }
            Task::none()
        }
        DialogueTextEditorMessage::Save => {
            if let Some(editor) = app.state.dialogue_text_editors.get_mut(&tab_id) {
                editor.loading_state = crate::loading_state::LoadingState::Loading;
                let result = editor.save();
                editor.loading_state = crate::loading_state::LoadingState::Loaded(());
                match result {
                    Ok(_) => editor.status_msg = "Texts saved successfully.".into(),
                    Err(e) => editor.status_msg = format!("Error saving texts: {}", e),
                }
            }
            Task::none()
        }
    }
}
