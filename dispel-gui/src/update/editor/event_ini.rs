// EventIni editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::loading_state::LoadingState;
use crate::message::editor::eventini::EventIniEditorMessage;
use dispel_core::{Event, Extractor};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: EventIniEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        EventIniEditorMessage::LoadCatalog => {
            if app.state.shared_game_path.is_empty() {
                app.state.event_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.event_ini_editor.loading_state = crate::loading_state::LoadingState::Loading;
            app.state.event_ini_spreadsheet.is_loading = true;
            let path = PathBuf::from(&app.state.shared_game_path).join("Event.ini");
            Task::perform(
                async move { Event::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                move |result: Result<Vec<dispel_core::Event>, String>| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::EventIni(
                            EventIniEditorMessage::CatalogLoaded(result),
                        ),
                    )
                },
            )
        }
        EventIniEditorMessage::CatalogLoaded(result) => {
            app.state.event_ini_editor.loading_state = LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.event_ini_editor.catalog = Some(catalog.clone());
                    app.state.event_ini_editor.status_msg =
                        format!("Event catalog loaded: {} entries", catalog.len());
                    app.state.event_ini_editor.refresh_events();
                    app.state.event_ini_editor.init_pane_state();
                    app.state.event_ini_spreadsheet.apply_filter(&catalog);
                    app.state.event_ini_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.event_ini_editor.status_msg =
                        format!("Error loading event catalog: {}", e);
                    app.state.event_ini_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        EventIniEditorMessage::SelectEvent(index) => {
            app.state.event_ini_editor.select(index);
            Task::none()
        }
        EventIniEditorMessage::FieldChanged(index, field, value) => {
            app.state
                .event_ini_editor
                .update_field(index, &field, value);
            Task::none()
        }
        EventIniEditorMessage::Save => {
            if app.state.shared_game_path.is_empty() {
                app.state.event_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.event_ini_editor.loading_state = crate::loading_state::LoadingState::Loading;
            let result = app
                .state
                .event_ini_editor
                .save_events(&app.state.shared_game_path);
            app.state.event_ini_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(_) => {
                    app.state.event_ini_editor.status_msg = "Events saved successfully.".into()
                }
                Err(e) => {
                    app.state.event_ini_editor.status_msg = format!("Error saving events: {}", e)
                }
            }
            Task::none()
        }
        EventIniEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                event_ini_spreadsheet,
                event_ini_editor,
                |index, field, value| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::EventIni(
                            EventIniEditorMessage::FieldChanged(index, field, value),
                        ),
                    )
                },
                msg
            );
            Task::none()
        }
        EventIniEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.event_ini_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.event_ini_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        EventIniEditorMessage::PaneClicked(pane) => {
            app.state.event_ini_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
