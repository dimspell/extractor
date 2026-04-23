// DialogueText editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages_tab;
use crate::message::editor::dialogue_paragraph::DialogueParagraphEditorMessage;
use crate::message::MessageExt;
use dispel_core::DialogueParagraph;
use iced::Task;

pub fn handle(
    message: DialogueParagraphEditorMessage,
    app: &mut App,
) -> Task<crate::message::Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);

    match message {
        DialogueParagraphEditorMessage::ScanCatalog => {
            if let Some(editor) = app.state.dialogue_paragraphs_editors.get_mut(&tab_id) {
                editor.editor.loading_state = crate::loading_state::LoadingState::Loading;
                let path = editor.current_file.clone();
                let tab_for_response = tab_id;
                return Task::perform(
                    async move {
                        let path = std::path::PathBuf::from(path);
                        let texts: std::io::Result<Vec<DialogueParagraph>> =
                            <DialogueParagraph as dispel_core::references::extractor::Extractor>::read_file(
                                &path,
                            );
                        texts.map_err(|e| e.to_string())
                    },
                    move |res: Result<Vec<DialogueParagraph>, String>| {
                        <crate::message::Message as crate::message::MessageExt>::dialogue_paragraph(
                            DialogueParagraphEditorMessage::CatalogLoaded(tab_for_response, res),
                        )
                    },
                );
            }
            Task::none()
        }
        DialogueParagraphEditorMessage::CatalogLoaded(_id, result) => {
            if let Some(editor) = app.state.dialogue_paragraphs_editors.get_mut(&tab_id) {
                let _ = _id;
                editor.editor.loading_state = crate::loading_state::LoadingState::Loaded(());
                match result {
                    Ok(catalog) => {
                        editor.editor.status_msg =
                            format!("Text catalog loaded: {} entries", catalog.len());
                        editor.editor.catalog = Some(catalog);
                        editor.editor.refresh();
                        if let Some(spreadsheet) =
                            app.state.dialogue_paragraph_spreadsheets.get_mut(&tab_id)
                        {
                            spreadsheet.active = true;
                            spreadsheet.init_filter(editor.editor.catalog.as_ref().unwrap());
                            spreadsheet.compute_all_caches(editor.editor.catalog.as_ref().unwrap());
                            spreadsheet.init_pane_state();
                        }
                    }
                    Err(e) => {
                        editor.editor.status_msg = format!("Error loading text catalog: {}", e);
                    }
                }
            }
            Task::none()
        }
        DialogueParagraphEditorMessage::Select(index) => {
            if let Some(editor) = app.state.dialogue_paragraphs_editors.get_mut(&tab_id) {
                editor.select(index);
            }
            Task::none()
        }
        DialogueParagraphEditorMessage::FieldChanged(index, field, value) => {
            if let Some(editor) = app.state.dialogue_paragraphs_editors.get_mut(&tab_id) {
                editor.update_field(index, &field, value);
            }
            Task::none()
        }
        DialogueParagraphEditorMessage::Save => {
            if let Some(editor) = app.state.dialogue_paragraphs_editors.get_mut(&tab_id) {
                editor.editor.loading_state = crate::loading_state::LoadingState::Loading;
                let result = editor.save();
                editor.editor.loading_state = crate::loading_state::LoadingState::Loaded(());
                match result {
                    Ok(_) => editor.editor.status_msg = "Texts saved successfully.".into(),
                    Err(e) => editor.editor.status_msg = format!("Error saving texts: {}", e),
                }
            }
            Task::none()
        }
        DialogueParagraphEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages_tab!(
                app,
                dialogue_paragraph_spreadsheets,
                dialogue_paragraphs_editors,
                &tab_id,
                |index, field, value| crate::message::Message::dialogue_paragraph(
                    DialogueParagraphEditorMessage::FieldChanged(index, field, value)
                ),
                msg
            );
            Task::none()
        }
        DialogueParagraphEditorMessage::PaneResized(event) => {
            if let Some(ed) = app.state.dialogue_paragraphs_editors.get_mut(&tab_id) {
                if let Some(ref mut ps) = ed.editor.pane_state {
                    ps.resize(event.split, event.ratio);
                }
            }
            if let Some(ss) = app.state.dialogue_paragraph_spreadsheets.get_mut(&tab_id) {
                if let Some(ref mut ps) = ss.pane_state {
                    ps.resize(event.split, event.ratio);
                }
            }
            Task::none()
        }
        DialogueParagraphEditorMessage::PaneClicked(pane) => {
            if let Some(ed) = app.state.dialogue_paragraphs_editors.get_mut(&tab_id) {
                ed.editor.pane_focus = Some(pane);
            }
            Task::none()
        }
    }
}
