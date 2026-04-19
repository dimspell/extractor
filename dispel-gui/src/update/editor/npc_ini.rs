// NpcIni editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::loading_state::LoadingState;
use crate::message::editor::npcini::NpcIniEditorMessage;
use crate::message::MessageExt;
use dispel_core::{Extractor, NpcIni};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: NpcIniEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        NpcIniEditorMessage::LoadCatalog | NpcIniEditorMessage::ScanNpcs => {
            // Load NPC ini catalog
            if app.state.shared_game_path.is_empty() {
                app.state.npc_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.npc_ini_editor.loading_state = LoadingState::Loading;
            app.state.npc_ini_spreadsheet.is_loading = true;
            let path = PathBuf::from(&app.state.shared_game_path);
            Task::perform(
                async move {
                    NpcIni::read_file(&path.join("Npc.ini"))
                        .map_err(|e: std::io::Error| e.to_string())
                },
                move |result: Result<Vec<dispel_core::NpcIni>, String>| {
                    crate::message::Message::npc_ini(NpcIniEditorMessage::CatalogLoaded(result))
                },
            )
        }
        NpcIniEditorMessage::CatalogLoaded(result) => {
            app.state.npc_ini_editor.loading_state = LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.npc_ini_editor.catalog = Some(catalog.clone());
                    app.state.npc_ini_editor.status_msg =
                        format!("NPC catalog loaded: {} npcs", catalog.len());
                    app.state.npc_ini_editor.refresh_npcs();
                    app.state.npc_ini_editor.init_pane_state();
                    app.state.npc_ini_spreadsheet.apply_filter(&catalog);
                    app.state.npc_ini_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.npc_ini_editor.status_msg =
                        format!("Error loading NPC catalog: {}", e);
                    app.state.npc_ini_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        NpcIniEditorMessage::SelectNpc(index) => {
            // Select NPC at index\
            app.state.npc_ini_editor.select_npc(index);
            Task::none()
        }
        NpcIniEditorMessage::FieldChanged(index, field, value) => {
            // Update NPC field
            app.state.npc_ini_editor.update_field(index, &field, value);
            Task::none()
        }
        NpcIniEditorMessage::Save => {
            // Save NPC changes
            if app.state.shared_game_path.is_empty() {
                app.state.npc_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.npc_ini_editor.loading_state = LoadingState::Loading;
            let result = app
                .state
                .npc_ini_editor
                .save_npcs(&app.state.shared_game_path);
            app.state.npc_ini_editor.loading_state = LoadingState::Loaded(());
            match result {
                Ok(_) => app.state.npc_ini_editor.status_msg = "NPCs saved successfully.".into(),
                Err(e) => app.state.npc_ini_editor.status_msg = format!("Error saving NPCs: {}", e),
            }
            Task::none()
        }
        NpcIniEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                npc_ini_spreadsheet,
                npc_ini_editor,
                |index, field, value| {
                    crate::message::Message::npc_ini(NpcIniEditorMessage::FieldChanged(
                        index, field, value,
                    ))
                },
                msg
            );
            Task::none()
        }
        NpcIniEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.npc_ini_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        NpcIniEditorMessage::PaneClicked(pane) => {
            app.state.npc_ini_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
