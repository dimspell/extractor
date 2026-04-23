// MiscItem editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::misc_item::MiscItemEditorMessage;
use dispel_core::{Extractor, MiscItem};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: MiscItemEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        MiscItemEditorMessage::LoadCatalog => {
            // Load misc item catalog
            if app.state.shared_game_path.is_empty() {
                app.state.misc_item_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.misc_item_editor.loading_state = crate::loading_state::LoadingState::Loading;
            app.state.misc_item_spreadsheet.is_loading = true;
            let path = PathBuf::from(&app.state.shared_game_path);
            Task::perform(
                async move {
                    MiscItem::read_file(&path.join("CharacterInGame").join("MiscItem.db"))
                        .map_err(|e: std::io::Error| e.to_string())
                },
                move |result: Result<Vec<dispel_core::MiscItem>, String>| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::MiscItem(
                            MiscItemEditorMessage::CatalogLoaded(result),
                        ),
                    )
                },
            )
        }
        MiscItemEditorMessage::CatalogLoaded(result) => {
            app.state.misc_item_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.misc_item_editor.catalog = Some(catalog.clone());
                    app.state.misc_item_editor.status_msg =
                        format!("Misc item catalog loaded: {} items", catalog.len());
                    app.state.misc_item_editor.refresh();
                    app.state.misc_item_editor.init_pane_state();
                    app.state.misc_item_spreadsheet.apply_filter(&catalog);
                    app.state.misc_item_spreadsheet.compute_all_caches(&catalog);
                    app.state.misc_item_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.misc_item_editor.status_msg =
                        format!("Error loading misc item catalog: {}", e);
                    app.state.misc_item_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        MiscItemEditorMessage::Select(index) => {
            // Select misc item at index
            app.state.misc_item_editor.select(index);
            Task::none()
        }
        MiscItemEditorMessage::FieldChanged(index, field, value) => {
            // Update misc item field
            app.state
                .misc_item_editor
                .update_field(index, &field, value);
            Task::none()
        }
        MiscItemEditorMessage::Save => {
            // Save misc item changes
            if app.state.shared_game_path.is_empty() {
                app.state.misc_item_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.misc_item_editor.loading_state = crate::loading_state::LoadingState::Loading;
            let result = app
                .state
                .misc_item_editor
                .save(&app.state.shared_game_path, "CharacterInGame/MiscItem.db");
            app.state.misc_item_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(_) => {
                    app.state.misc_item_editor.status_msg = "Misc items saved successfully.".into()
                }
                Err(e) => {
                    app.state.misc_item_editor.status_msg =
                        format!("Error saving misc items: {}", e)
                }
            }
            Task::none()
        }
        MiscItemEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                misc_item_spreadsheet,
                misc_item_editor,
                |index, field, value| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::MiscItem(
                            MiscItemEditorMessage::FieldChanged(index, field, value),
                        ),
                    )
                },
                msg
            );
            Task::none()
        }
        MiscItemEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.misc_item_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.misc_item_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        MiscItemEditorMessage::PaneClicked(pane) => {
            app.state.misc_item_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
