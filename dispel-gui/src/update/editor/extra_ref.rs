// ExtraRef editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages_tab;
use crate::message::editor::extra_ref::ExtraRefEditorMessage;
use crate::message::MessageExt;
use dispel_core::Extractor;
use iced::Task;

pub fn handle(message: ExtraRefEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);

    match message {
        ExtraRefEditorMessage::LoadCatalog(path) => {
            let tab_id = app
                .state
                .workspace
                .active()
                .map(|t| t.id)
                .unwrap_or(usize::MAX);

            let mut editor_state = crate::state::extra_ref_editor::ExtraRefEditorState::default();
            editor_state.current_file = Some(path.clone());

            // Load the catalog synchronously first to initialize the editor
            editor_state.select_file(path.clone());

            // Initialize spreadsheet state with the loaded catalog
            let mut ss = crate::view::editor::SpreadsheetState::new();
            if let Some(catalog) = editor_state.editor.catalog.as_ref() {
                ss.apply_filter(catalog);
                ss.init_pane_state();
            }

            app.state.extra_ref_editors.insert(tab_id, editor_state);
            app.state.extra_ref_spreadsheets.insert(tab_id, ss);

            // Also load asynchronously to ensure we have the latest data
            let path_buf = path.clone();
            Task::perform(
                async move {
                    dispel_core::ExtraRef::read_file(&path_buf)
                        .map_err(|e: std::io::Error| e.to_string())
                },
                move |result| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::ExtraRef(
                            ExtraRefEditorMessage::CatalogLoaded(tab_id, result),
                        ),
                    )
                },
            )
        }
        ExtraRefEditorMessage::CatalogLoaded(id, result) => {
            if let Some(editor) = app.state.extra_ref_editors.get_mut(&id) {
                editor.editor.loading_state = crate::loading_state::LoadingState::Loaded(());
                match result {
                    Ok(catalog) => {
                        editor.editor.status_msg =
                            format!("Extra ref catalog loaded: {} entries", catalog.len());
                        editor.editor.catalog = Some(catalog.clone());
                        // Update spreadsheet with the new catalog
                        if let Some(ss) = app.state.extra_ref_spreadsheets.get_mut(&id) {
                            ss.apply_filter(&catalog);
                            ss.compute_all_caches(&catalog);
                            ss.init_pane_state();
                        }
                    }
                    Err(e) => {
                        editor.editor.status_msg =
                            format!("Error loading extra ref catalog: {}", e);
                    }
                }
            }
            Task::none()
        }
        ExtraRefEditorMessage::Select(index) => {
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
