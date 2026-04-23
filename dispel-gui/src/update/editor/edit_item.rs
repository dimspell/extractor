// EditItem editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::loading_state::LoadingState;
use crate::message::editor::edititem::EditItemEditorMessage;
use dispel_core::{EditItem, Extractor};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: EditItemEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        EditItemEditorMessage::LoadCatalog => {
            // Load edit item catalog
            if app.state.shared_game_path.is_empty() {
                app.state.edit_item_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            app.state.edit_item_editor.loading_state = LoadingState::Loading;
            app.state.edit_item_spreadsheet.is_loading = true;
            let path = PathBuf::from(&app.state.shared_game_path);

            Task::perform(
                async move {
                    EditItem::read_file(&path.join("CharacterInGame").join("EditItem.db"))
                        .map_err(|e: std::io::Error| e.to_string())
                },
                move |result: Result<Vec<dispel_core::EditItem>, String>| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::EditItem(
                            EditItemEditorMessage::CatalogLoaded(result),
                        ),
                    )
                },
            )
        }
        EditItemEditorMessage::CatalogLoaded(result) => {
            app.state.edit_item_editor.loading_state = LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.edit_item_editor.catalog = Some(catalog.clone());

                    app.state.edit_item_editor.status_msg =
                        format!("Edit item catalog loaded: {} items", catalog.len());

                    app.state.edit_item_editor.refresh();
                    app.state.edit_item_editor.init_pane_state();
                    app.state.edit_item_spreadsheet.apply_filter(&catalog);
                    app.state.edit_item_spreadsheet.compute_all_caches(&catalog);
                    app.state.edit_item_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.edit_item_editor.status_msg =
                        format!("Error loading edit item catalog: {}", e);
                    app.state.edit_item_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        EditItemEditorMessage::Select(index) => {
            // Select edit item at index
            app.state.edit_item_editor.select(index);
            Task::none()
        }
        EditItemEditorMessage::FieldChanged(index, field, value) => {
            // Update edit item field
            app.state
                .edit_item_editor
                .update_field(index, &field, value);
            Task::none()
        }
        EditItemEditorMessage::Save => {
            // Save edit item changes
            if app.state.shared_game_path.is_empty() {
                app.state.edit_item_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.edit_item_editor.loading_state = crate::loading_state::LoadingState::Loading;
            let result = app
                .state
                .edit_item_editor
                .save(&app.state.shared_game_path, "CharacterInGame/EditItem.db");
            app.state.edit_item_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(_) => {
                    app.state.edit_item_editor.status_msg = "Edit items saved successfully.".into()
                }
                Err(e) => {
                    app.state.edit_item_editor.status_msg =
                        format!("Error saving edit items: {}", e)
                }
            }
            Task::none()
        }
        EditItemEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                edit_item_spreadsheet,
                edit_item_editor,
                |index, field, value| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::EditItem(
                            EditItemEditorMessage::FieldChanged(index, field, value),
                        ),
                    )
                },
                msg
            );
            Task::none()
        }
        EditItemEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.edit_item_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.edit_item_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        EditItemEditorMessage::PaneClicked(pane) => {
            app.state.edit_item_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
