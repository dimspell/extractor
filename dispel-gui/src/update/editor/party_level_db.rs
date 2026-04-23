// PartyLevelDb editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::partyleveldb::PartyLevelDbEditorMessage;
use dispel_core::{Extractor, PartyLevelNpc};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: PartyLevelDbEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        PartyLevelDbEditorMessage::LoadCatalog => {
            if app.state.shared_game_path.is_empty() {
                app.state.party_level_db_editor.status_msg =
                    "Please select game path first.".into();
                return Task::none();
            }
            app.state.party_level_db_editor.loading_state =
                crate::loading_state::LoadingState::Loading;
            app.state.party_level_db_spreadsheet.is_loading = true;
            let level_path = PathBuf::from(&app.state.shared_game_path)
                .join("NpcInGame")
                .join("PrtLevel.db");
            Task::perform(
                async move {
                    PartyLevelNpc::read_file(&level_path).map_err(|e: std::io::Error| e.to_string())
                },
                move |result: Result<Vec<dispel_core::PartyLevelNpc>, String>| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::PartyLevelDb(
                            PartyLevelDbEditorMessage::CatalogLoaded(result),
                        ),
                    )
                },
            )
        }
        PartyLevelDbEditorMessage::CatalogLoaded(result) => {
            app.state.party_level_db_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(levels) => {
                    app.state.party_level_db_editor.catalog = Some(levels.clone());
                    app.state.party_level_db_editor.status_msg =
                        format!("Party level catalog loaded: {} NPCs", levels.len());
                    app.state.party_level_db_editor.init_pane_state();
                    app.state.party_level_db_spreadsheet.apply_filter(&levels);
                    app.state.party_level_db_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.party_level_db_editor.status_msg =
                        format!("Error loading party level catalog: {}", e);
                    app.state.party_level_db_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        PartyLevelDbEditorMessage::Select(index) => {
            app.state.party_level_db_editor.select(index);
            Task::none()
        }
        PartyLevelDbEditorMessage::FieldChanged(index, field, value) => {
            app.state
                .party_level_db_editor
                .update_field(index, &field, value);
            Task::none()
        }
        PartyLevelDbEditorMessage::Save => {
            if app.state.shared_game_path.is_empty() {
                app.state.party_level_db_editor.status_msg =
                    "Please select game path first.".into();
                return Task::none();
            }
            app.state.party_level_db_editor.loading_state =
                crate::loading_state::LoadingState::Loading;
            let result = app
                .state
                .party_level_db_editor
                .save_levels(&app.state.shared_game_path);
            app.state.party_level_db_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(_) => {
                    app.state.party_level_db_editor.status_msg =
                        "Party levels saved successfully.".into()
                }
                Err(e) => {
                    app.state.party_level_db_editor.status_msg =
                        format!("Error saving party levels: {}", e)
                }
            }
            Task::none()
        }
        PartyLevelDbEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                party_level_db_spreadsheet,
                party_level_db_editor,
                |index, field, value| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::PartyLevelDb(
                            PartyLevelDbEditorMessage::FieldChanged(index, field, value),
                        ),
                    )
                },
                msg
            );
            Task::none()
        }
        PartyLevelDbEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.party_level_db_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.party_level_db_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        PartyLevelDbEditorMessage::PaneClicked(pane) => {
            app.state.party_level_db_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
