// DrawItem editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::draw_item::DrawItemEditorMessage;
use dispel_core::{DrawItem, Extractor};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: DrawItemEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        DrawItemEditorMessage::LoadCatalog => {
            // Load catalog
            if app.state.shared_game_path.is_empty() {
                app.state.draw_item_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.draw_item_editor.loading_state = crate::loading_state::LoadingState::Loading;
            app.state.draw_item_spreadsheet.is_loading = true;
            let path = PathBuf::from(&app.state.shared_game_path)
                .join("Ref")
                .join("DRAWITEM.ref");
            Task::perform(
                async move { DrawItem::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                move |result: Result<Vec<dispel_core::DrawItem>, String>| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::DrawItem(
                            DrawItemEditorMessage::CatalogLoaded(result),
                        ),
                    )
                },
            )
        }
        DrawItemEditorMessage::CatalogLoaded(res) => {
            app.state.draw_item_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match res {
                Ok(catalog) => {
                    app.state.draw_item_editor.catalog = Some(catalog.clone());
                    app.state.draw_item_editor.status_msg =
                        format!("Draw item catalog loaded: {} entries", catalog.len());
                    app.state.draw_item_editor.refresh_items();
                    app.state.draw_item_editor.init_pane_state();
                    app.state.draw_item_spreadsheet.apply_filter(&catalog);
                    app.state.draw_item_spreadsheet.compute_all_caches(&catalog);
                    app.state.draw_item_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.draw_item_editor.status_msg =
                        format!("Error loading draw item catalog: {}", e);
                    app.state.draw_item_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        DrawItemEditorMessage::Select(index) => {
            // Select item
            app.state.draw_item_editor.select(index);
            Task::none()
        }
        DrawItemEditorMessage::FieldChanged(index, field, value) => {
            // Update field
            app.state
                .draw_item_editor
                .update_field(index, &field, value);
            Task::none()
        }
        DrawItemEditorMessage::Save => {
            // Save changes
            if app.state.shared_game_path.is_empty() {
                app.state.draw_item_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.draw_item_editor.loading_state = crate::loading_state::LoadingState::Loading;
            let result = app
                .state
                .draw_item_editor
                .save_items(&app.state.shared_game_path);
            app.state.draw_item_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(_) => {
                    app.state.draw_item_editor.status_msg = "Draw items saved successfully.".into()
                }
                Err(e) => {
                    app.state.draw_item_editor.status_msg =
                        format!("Error saving draw items: {}", e)
                }
            }
            Task::none()
        }
        DrawItemEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                draw_item_spreadsheet,
                draw_item_editor,
                |index, field, value| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::DrawItem(
                            DrawItemEditorMessage::FieldChanged(index, field, value),
                        ),
                    )
                },
                msg
            );
            Task::none()
        }
        DrawItemEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.draw_item_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.draw_item_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        DrawItemEditorMessage::PaneClicked(pane) => {
            app.state.draw_item_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
