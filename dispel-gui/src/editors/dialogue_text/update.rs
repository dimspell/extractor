use dispel_core::{DialogueParagraph, Extractor};
use iced::Task;

use crate::update::editor::tab;
use crate::app::App;
use crate::handle_spreadsheet_messages_tab;
use crate::loading_state::LoadingState;
use crate::editors::dialogue_text::DialogueParagraphEditorMessage;
use crate::message::MessageExt;

pub fn handle(msg: DialogueParagraphEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = tab::get_tab_id(&app.state.workspace);

    match msg {
        DialogueParagraphEditorMessage::ScanCatalog => {
            if let Some(editor) = app.state.dialogue_paragraphs_editors.get_mut(&tab_id) {
                if let Some(path) = editor.current_file.clone() {
                    editor.editor.loading_state = LoadingState::Loading;
                    return Task::perform(
                        async move { DialogueParagraph::read_file(&path).map_err(|e| e.to_string()) },
                        move |result| {
                            crate::message::Message::dialogue_paragraph(
                                DialogueParagraphEditorMessage::CatalogLoaded(tab_id, result),
                            )
                        },
                    );
                }
            }
            Task::none()
        }
        DialogueParagraphEditorMessage::CatalogLoaded(id, result) => {
            if let Some(editor) = app.state.dialogue_paragraphs_editors.get_mut(&id) {
                editor.editor.loading_state = LoadingState::Loaded(());
                match result {
                    Ok(catalog) => {
                        editor.editor.status_msg =
                            format!("Text catalog loaded: {} entries", catalog.len());
                        editor.editor.catalog = Some(catalog);
                        editor.editor.refresh();
                        if let Some(spreadsheet) =
                            app.state.dialogue_paragraph_spreadsheets.get_mut(&id)
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
            tab::select(&mut app.state.dialogue_paragraphs_editors, tab_id, index)
        }
        DialogueParagraphEditorMessage::FieldChanged(index, field, value) => tab::field_changed(
            &mut app.state.dialogue_paragraphs_editors,
            tab_id,
            index,
            field,
            value,
        ),
        DialogueParagraphEditorMessage::Save => tab::save(
            &mut app.state.dialogue_paragraphs_editors,
            tab_id,
            "Texts saved successfully.",
            "Error saving texts",
        ),
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
        DialogueParagraphEditorMessage::PaneResized(event) => tab::pane_resized(
            &mut app.state.dialogue_paragraphs_editors,
            &mut app.state.dialogue_paragraph_spreadsheets,
            tab_id,
            event,
        ),
        DialogueParagraphEditorMessage::PaneClicked(pane) => {
            tab::pane_clicked(&mut app.state.dialogue_paragraphs_editors, tab_id, pane)
        }
    }
}
