// PartyIni editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::partyini::PartyIniEditorMessage;
use dispel_core::{Extractor, PartyIniNpc};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: PartyIniEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        PartyIniEditorMessage::LoadCatalog | PartyIniEditorMessage::ScanNpcs => {
            // Load party ini catalog
            if app.state.shared_game_path.is_empty() {
                app.state.party_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.party_ini_editor.loading_state = crate::loading_state::LoadingState::Loading;
            app.state.party_ini_spreadsheet.is_loading = true;
            let path = PathBuf::from(&app.state.shared_game_path);
            Task::perform(
                async move {
                    PartyIniNpc::read_file(&path.join("NpcInGame").join("PrtIni.db"))
                        .map_err(|e: std::io::Error| e.to_string())
                },
                move |result: Result<Vec<dispel_core::PartyIniNpc>, String>| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::PartyIni(
                            PartyIniEditorMessage::CatalogLoaded(result),
                        ),
                    )
                },
            )
        }
        PartyIniEditorMessage::CatalogLoaded(result) => {
            app.state.party_ini_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.party_ini_editor.catalog = Some(catalog.clone());
                    app.state.party_ini_editor.status_msg =
                        format!("Party ini catalog loaded: {} npcs", catalog.len());
                    app.state.party_ini_editor.refresh();
                    app.state.party_ini_editor.init_pane_state();
                    app.state.party_ini_spreadsheet.apply_filter(&catalog);
                    app.state.party_ini_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.party_ini_editor.status_msg =
                        format!("Error loading party ini catalog: {}", e);
                    app.state.party_ini_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        PartyIniEditorMessage::SelectNpc(index) => {
            // Select NPC at index
            app.state.party_ini_editor.select(index);
            Task::none()
        }
        PartyIniEditorMessage::FieldChanged(index, field, value) => {
            // Update NPC field
            app.state
                .party_ini_editor
                .update_field(index, &field, value);
            Task::none()
        }
        PartyIniEditorMessage::Save => {
            // Save party ini changes
            if app.state.shared_game_path.is_empty() {
                app.state.party_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.party_ini_editor.loading_state = crate::loading_state::LoadingState::Loading;
            let result = app
                .state
                .party_ini_editor
                .save_npcs(&app.state.shared_game_path);
            app.state.party_ini_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(_) => {
                    app.state.party_ini_editor.status_msg = "Party ini saved successfully.".into()
                }
                Err(e) => {
                    app.state.party_ini_editor.status_msg = format!("Error saving party ini: {}", e)
                }
            }
            Task::none()
        }
        PartyIniEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                party_ini_spreadsheet,
                party_ini_editor,
                |index, field, value| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::PartyIni(
                            PartyIniEditorMessage::FieldChanged(index, field, value),
                        ),
                    )
                },
                msg
            );
            Task::none()
        }
        PartyIniEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.party_ini_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.party_ini_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        PartyIniEditorMessage::PaneClicked(pane) => {
            app.state.party_ini_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
