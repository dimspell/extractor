// PartyRef editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::partyref::PartyRefEditorMessage;
use dispel_core::{Extractor, PartyRef};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: PartyRefEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        PartyRefEditorMessage::LoadCatalog | PartyRefEditorMessage::ScanParty => {
            // Load party ref catalog
            if app.state.shared_game_path.is_empty() {
                app.state.party_ref_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.party_ref_editor.loading_state = crate::loading_state::LoadingState::Loading;
            app.state.party_ref_spreadsheet.is_loading = true;
            let path = PathBuf::from(&app.state.shared_game_path);
            Task::perform(
                async move {
                    PartyRef::read_file(&path.join("Ref").join("PartyRef.ref"))
                        .map_err(|e: std::io::Error| e.to_string())
                },
                move |result: Result<Vec<dispel_core::PartyRef>, String>| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::PartyRef(
                            PartyRefEditorMessage::CatalogLoaded(result),
                        ),
                    )
                },
            )
        }
        PartyRefEditorMessage::CatalogLoaded(result) => {
            app.state.party_ref_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.party_ref_editor.catalog = Some(catalog.clone());
                    app.state.party_ref_editor.status_msg =
                        format!("Party catalog loaded: {} members", catalog.len());
                    app.state.party_ref_editor.refresh();
                    app.state.party_ref_editor.init_pane_state();
                    app.state.party_ref_spreadsheet.apply_filter(&catalog);
                    app.state.party_ref_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.party_ref_editor.status_msg =
                        format!("Error loading party catalog: {}", e);
                    app.state.party_ref_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        PartyRefEditorMessage::SelectMember(index) => {
            // Select member at index
            app.state.party_ref_editor.select(index);
            Task::none()
        }
        PartyRefEditorMessage::FieldChanged(index, field, value) => {
            // Update member field
            app.state
                .party_ref_editor
                .update_field(index, &field, value);
            Task::none()
        }
        PartyRefEditorMessage::Save => {
            // Save party ref changes
            if app.state.shared_game_path.is_empty() {
                app.state.party_ref_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.party_ref_editor.loading_state = crate::loading_state::LoadingState::Loading;
            let result = app
                .state
                .party_ref_editor
                .save(&app.state.shared_game_path, "PartyRef.ref");
            app.state.party_ref_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(_) => app.state.party_ref_editor.status_msg = "Party saved successfully.".into(),
                Err(e) => {
                    app.state.party_ref_editor.status_msg = format!("Error saving party: {}", e)
                }
            }
            Task::none()
        }
        PartyRefEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                party_ref_spreadsheet,
                party_ref_editor,
                |index, field, value| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::PartyRef(
                            PartyRefEditorMessage::FieldChanged(index, field, value),
                        ),
                    )
                },
                msg
            );
            Task::none()
        }
        PartyRefEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.party_ref_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.party_ref_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        PartyRefEditorMessage::PaneClicked(pane) => {
            app.state.party_ref_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
