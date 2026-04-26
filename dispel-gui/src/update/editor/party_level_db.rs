use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::party_level_db::PartyLevelDbEditorMessage;
use crate::message::Message;
use dispel_core::{Extractor, PartyLevelNpc};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: PartyLevelDbEditorMessage, app: &mut App) -> Task<Message> {
    match message {
        PartyLevelDbEditorMessage::LoadCatalog => {
            if app.state.shared_game_path.is_empty() {
                app.state.party_level_db_editor.status_msg =
                    "Please select game path first.".into();
                return Task::none();
            }
            app.state.party_level_db_editor.loading_state =
                crate::loading_state::LoadingState::Loading;
            let level_path = PathBuf::from(&app.state.shared_game_path)
                .join("NpcInGame")
                .join("PrtLevel.db");
            Task::perform(
                async move {
                    PartyLevelNpc::read_file(&level_path).map_err(|e: std::io::Error| e.to_string())
                },
                |result| {
                    Message::Editor(crate::message::editor::EditorMessage::PartyLevelDb(
                        PartyLevelDbEditorMessage::CatalogLoaded(result),
                    ))
                },
            )
        }

        PartyLevelDbEditorMessage::CatalogLoaded(result) => {
            app.state.party_level_db_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(npcs) => {
                    app.state.party_level_db_editor.catalog = Some(npcs.clone());
                    app.state.party_level_db_editor.selected_npc_idx = None;
                    app.state.party_level_db_level_editor.catalog = None;
                    app.state.party_level_db_level_editor.status_msg =
                        format!("Party levels loaded: {} NPCs", npcs.len());
                    app.state.party_level_db_editor.status_msg =
                        format!("Loaded {} NPCs — select one to edit levels", npcs.len());
                    app.state.party_level_db_spreadsheet = crate::view::editor::SpreadsheetState::new();
                }
                Err(e) => {
                    let msg = format!("Error loading party levels: {}", e);
                    app.state.party_level_db_editor.status_msg = msg.clone();
                    app.state.party_level_db_level_editor.status_msg = msg;
                }
            }
            Task::none()
        }

        PartyLevelDbEditorMessage::SelectNpc(npc_idx) => {
            app.state.party_level_db_editor.selected_npc_idx = Some(npc_idx);

            let records = app
                .state
                .party_level_db_editor
                .catalog
                .as_ref()
                .and_then(|c| c.get(npc_idx))
                .map(|npc| npc.records.clone())
                .unwrap_or_default();

            app.state.party_level_db_level_editor.catalog = Some(records.clone());
            app.state.party_level_db_level_editor.refresh();
            app.state.party_level_db_level_editor.selected_idx = None;
            app.state.party_level_db_level_editor.edit_buffers.clear();
            app.state.party_level_db_level_editor.status_msg =
                format!("NPC {} — {} levels", npc_idx, records.len());

            app.state.party_level_db_spreadsheet.apply_filter(&records);
            app.state.party_level_db_spreadsheet.compute_all_caches(&records);
            app.state.party_level_db_spreadsheet.init_pane_state();
            app.state.party_level_db_spreadsheet.selected_row = None;
            app.state.party_level_db_spreadsheet.inspector_textarea_contents.clear();

            Task::none()
        }

        PartyLevelDbEditorMessage::FieldChanged(level_idx, field, value) => {
            app.state
                .party_level_db_level_editor
                .update_field(level_idx, &field, value);

            // Sync updated level records back to the NPC catalog
            if let Some(npc_idx) = app.state.party_level_db_editor.selected_npc_idx {
                if let (Some(level_catalog), Some(npc_catalog)) = (
                    app.state.party_level_db_level_editor.catalog.clone(),
                    app.state.party_level_db_editor.catalog.as_mut(),
                ) {
                    if let Some(npc) = npc_catalog.get_mut(npc_idx) {
                        npc.records = level_catalog;
                    }
                }
            }

            // Refresh spreadsheet display caches
            if let Some(catalog) = &app.state.party_level_db_level_editor.catalog {
                let catalog = catalog.clone();
                app.state.party_level_db_spreadsheet.compute_all_caches(&catalog);
            }

            Task::none()
        }

        PartyLevelDbEditorMessage::Save => {
            if app.state.shared_game_path.is_empty() {
                app.state.party_level_db_editor.status_msg =
                    "Please select game path first.".into();
                return Task::none();
            }
            match app.state.party_level_db_editor.save_levels(&app.state.shared_game_path) {
                Ok(_) => {
                    app.state.party_level_db_editor.status_msg =
                        "Party levels saved successfully.".into();
                    app.state.party_level_db_level_editor.status_msg =
                        "Party levels saved successfully.".into();
                }
                Err(e) => {
                    let msg = format!("Error saving party levels: {}", e);
                    app.state.party_level_db_editor.status_msg = msg.clone();
                    app.state.party_level_db_level_editor.status_msg = msg;
                }
            }
            Task::none()
        }

        PartyLevelDbEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                party_level_db_spreadsheet,
                party_level_db_level_editor,
                |index, field, value| {
                    Message::Editor(crate::message::editor::EditorMessage::PartyLevelDb(
                        PartyLevelDbEditorMessage::FieldChanged(index, field, value),
                    ))
                },
                msg
            );
            Task::none()
        }

        PartyLevelDbEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.party_level_db_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }

        PartyLevelDbEditorMessage::PaneClicked(_pane) => {
            Task::none()
        }
    }
}
