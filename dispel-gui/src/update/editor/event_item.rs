// EventItem editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::loading_state::LoadingState;
use crate::message::editor::eventitem::EventItemEditorMessage;
use dispel_core::{EventItem, Extractor};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: EventItemEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        EventItemEditorMessage::LoadCatalog | EventItemEditorMessage::ScanItems => {
            // Load event item catalog
            // Scan event items from game files
            if app.state.shared_game_path.is_empty() {
                app.state.event_item_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            app.state.event_item_editor.loading_state = LoadingState::Loading;
            app.state.event_item_spreadsheet.is_loading = true;
            let path = PathBuf::from(&app.state.shared_game_path);

            Task::perform(
                async move {
                    EventItem::read_file(&path.join("CharacterInGame").join("EventItem.db"))
                        .map_err(|e: std::io::Error| e.to_string())
                },
                move |result: Result<Vec<dispel_core::EventItem>, String>| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::EventItem(
                            EventItemEditorMessage::CatalogLoaded(result),
                        ),
                    )
                },
            )
        }
        EventItemEditorMessage::CatalogLoaded(result) => {
            app.state.event_item_editor.loading_state = LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.event_item_editor.catalog = Some(catalog.clone());
                    app.state.event_item_editor.status_msg =
                        format!("Edit item catalog loaded: {} items", catalog.len());
                    app.state.event_item_editor.refresh();
                    app.state.event_item_editor.init_pane_state();
                    app.state.event_item_spreadsheet.apply_filter(&catalog);
                    app.state.event_item_spreadsheet.compute_all_caches(&catalog);
                    app.state.event_item_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.event_item_editor.status_msg =
                        format!("Error loading edit item catalog: {}", e);
                    app.state.event_item_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        EventItemEditorMessage::SelectItem(index) => {
            // Select event item at index
            app.state.event_item_editor.select(index);
            Task::none()
        }
        EventItemEditorMessage::FieldChanged(index, field, value) => {
            // Update event item field
            app.state
                .event_item_editor
                .update_field(index, &field, value);
            Task::none()
        }
        EventItemEditorMessage::Save => {
            // Save event item changes
            if app.state.shared_game_path.is_empty() {
                app.state.event_item_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            app.state.event_item_editor.loading_state = LoadingState::Loading;
            let result = app
                .state
                .event_item_editor
                .save(&app.state.shared_game_path, "CharacterInGame/EventItem.db");
            app.state.event_item_editor.loading_state = LoadingState::Loaded(());

            match result {
                Ok(_) => {
                    app.state.event_item_editor.status_msg =
                        "Event items saved successfully.".into()
                }
                Err(e) => {
                    app.state.event_item_editor.status_msg =
                        format!("Error saving event items: {}", e)
                }
            }
            Task::none()
        }
        EventItemEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                event_item_spreadsheet,
                event_item_editor,
                |index, field, value| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::EventItem(
                            EventItemEditorMessage::FieldChanged(index, field, value),
                        ),
                    )
                },
                msg
            );
            Task::none()
        }
        EventItemEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.event_item_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.event_item_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        EventItemEditorMessage::PaneClicked(pane) => {
            app.state.event_item_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
