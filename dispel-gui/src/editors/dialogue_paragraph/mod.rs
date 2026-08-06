use dispel_core::{DialogueParagraph, Extractor};
use iced::Task;

use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::message::{Message, MessageExt};
use crate::update::editor::tab;

mod component;

crate::define_tab_editor! {
    name: dialogue_paragraph,
    name_pascal: DialogueParagraph,
    record: DialogueParagraph,
    field: dialogue_paragraph_editor,
    empty_text: "Dialogue Paragraph file not loaded",
    save_success_msg: "Texts saved successfully.",
    save_error_msg: "Error saving texts",
    extra_variants: {
        ScanCatalog,
        CatalogLoaded(usize, Result<Vec<DialogueParagraph>, String>),
    },
}

mod view;
pub use view::view;

pub fn handle(msg: DialogueParagraphEditorMessage, app: &mut App) -> Task<Message> {
    let tab_id = tab::get_tab_id(&app.state.workspace);

    match msg {
        DialogueParagraphEditorMessage::ScanCatalog => {
            if let Some(editor) = app
                .state
                .editors
                .dialogue_paragraph_editor
                .editors
                .get_mut(&tab_id)
                && let Some(path) = editor.current_file.clone()
            {
                editor.editor.loading_state = LoadingState::Loading;
                return Task::perform(
                    async move { DialogueParagraph::read_file(&path).map_err(|e| e.to_string()) },
                    move |result| {
                        Message::dialogue_paragraph(DialogueParagraphEditorMessage::CatalogLoaded(
                            tab_id, result,
                        ))
                    },
                );
            }
            Task::none()
        }
        DialogueParagraphEditorMessage::CatalogLoaded(id, result) => {
            if let Some(editor) = app
                .state
                .editors
                .dialogue_paragraph_editor
                .editors
                .get_mut(&id)
            {
                editor.editor.loading_state = LoadingState::Loaded(());
                match result {
                    Ok(catalog) => {
                        crate::components::item_catalog::ensure_item_lookups(
                            &app.state.shared_game_path,
                            &mut app.state.lookups,
                        );
                        editor.editor.status_msg =
                            format!("Text catalog loaded: {} entries", catalog.len());
                        editor.editor.catalog = Some(catalog);
                        editor.editor.refresh();
                        if let Some(spreadsheet) = app
                            .state
                            .editors
                            .dialogue_paragraph_editor
                            .spreadsheets
                            .get_mut(&id)
                        {
                            spreadsheet.active = true;
                            spreadsheet.init_filter(editor.editor.catalog.as_ref().unwrap());
                            spreadsheet.compute_all_caches(
                                editor.editor.catalog.as_ref().unwrap(),
                                &app.state.lookups,
                            );
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
        msg => handle_core(msg, app, tab_id),
    }
}
