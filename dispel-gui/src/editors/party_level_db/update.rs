use crate::app::App;
use crate::components::editable::EditableRecord;
use crate::editors::party_level_db::PartyLevelDbEditorMessage;
use crate::handle_spreadsheet_messages;
use crate::message::Message;
use dispel_core::{Extractor, PartyLevelNpc, PartyRef};
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
                crate::components::loading_state::LoadingState::Loading;
            let level_path = PathBuf::from(&app.state.shared_game_path)
                .join("NpcInGame")
                .join("PrtLevel.db");
            let party_ref_path = PathBuf::from(&app.state.shared_game_path)
                .join("Ref")
                .join("PartyRef.ref");
            Task::perform(
                async move {
                    let npcs = PartyLevelNpc::read_file(&level_path)
                        .map_err(|e: std::io::Error| e.to_string())?;
                    let refs = PartyRef::read_file(&party_ref_path).unwrap_or_default();
                    Ok((npcs, refs))
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
                crate::components::loading_state::LoadingState::Loaded(());
            match result {
                Ok((npcs, refs)) => {
                    app.state.party_level_db_editor.party_refs = refs;
                    app.state.party_level_db_editor.catalog = Some(npcs.clone());
                    app.state.party_level_db_editor.selected_npc_idx = None;
                    app.state.party_level_db_level_editor.catalog = None;
                    app.state.party_level_db_level_editor.status_msg =
                        format!("Party levels loaded: {} NPCs", npcs.len());
                    app.state.party_level_db_editor.status_msg =
                        format!("Loaded {} NPCs — select one to edit levels", npcs.len());
                    app.state.party_level_db_level_editor.spreadsheet =
                        crate::view::editor::SpreadsheetState::new();
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

            app.state
                .party_level_db_level_editor
                .spreadsheet
                .apply_filter(&records);
            app.state
                .party_level_db_level_editor
                .spreadsheet
                .compute_all_caches(&records);
            app.state
                .party_level_db_level_editor
                .spreadsheet
                .init_pane_state();
            app.state
                .party_level_db_level_editor
                .spreadsheet
                .selected_orig = None;
            app.state
                .party_level_db_level_editor
                .spreadsheet
                .inspector_textarea_contents
                .clear();

            Task::none()
        }

        PartyLevelDbEditorMessage::FieldChanged(level_idx, field, value) => {
            // Capture (npc, level, old) BEFORE the edit. Skip recording if
            // either the row or the selected NPC is missing — recording a
            // delta against a guessed npc_idx=0 would silently mis-address
            // the change.
            let captured = app
                .state
                .party_level_db_editor
                .selected_npc_idx
                .and_then(|npc_idx| {
                    let (level_orig_idx, record) = app
                        .state
                        .party_level_db_level_editor
                        .state
                        .filtered
                        .get(level_idx)?;
                    Some((
                        npc_idx as u32 * 20 + *level_orig_idx as u32,
                        record.get_field(&field),
                    ))
                });
            let new_value = value.clone();
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
                app.state
                    .party_level_db_level_editor
                    .spreadsheet
                    .compute_all_caches(&catalog);
            }

            // Observe for recording. PartyLevelDbPatcher packs the address
            // as `npc_idx * 20 + level_idx` (0-based on both axes).
            match captured {
                Some((record_id, old_value)) if old_value != new_value => {
                    crate::editors::mod_packager::recording::observe_field_change(
                        app,
                        "NpcInGame/PrtLevel.db",
                        record_id,
                        &field,
                        old_value,
                        new_value,
                    )
                }
                _ => Task::none(),
            }
        }

        PartyLevelDbEditorMessage::Save => {
            if app.state.shared_game_path.is_empty() {
                app.state.party_level_db_editor.status_msg =
                    "Please select game path first.".into();
                return Task::none();
            }
            match app
                .state
                .party_level_db_editor
                .save_levels(&app.state.shared_game_path)
            {
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
            if let Some(ref mut ps) = app.state.party_level_db_level_editor.spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }

        PartyLevelDbEditorMessage::PaneClicked(_pane) => Task::none(),
    }
}
