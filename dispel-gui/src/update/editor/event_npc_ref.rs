// EventNpcRef editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::event_npc_ref::EventNpcRefEditorMessage;
use dispel_core::{EventNpcRef, Extractor};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: EventNpcRefEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        EventNpcRefEditorMessage::LoadCatalog => {
            if app.state.shared_game_path.is_empty() {
                app.state.event_npc_ref_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.event_npc_ref_editor.loading_state =
                crate::loading_state::LoadingState::Loading;
            app.state.event_npc_ref_spreadsheet.is_loading = true;
            let path = PathBuf::from(&app.state.shared_game_path)
                .join("NpcInGame")
                .join("Eventnpc.ref");
            Task::perform(
                async move { EventNpcRef::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                move |result: Result<Vec<dispel_core::EventNpcRef>, String>| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::EventNpcRef(
                            EventNpcRefEditorMessage::CatalogLoaded(result),
                        ),
                    )
                },
            )
        }
        EventNpcRefEditorMessage::CatalogLoaded(res) => {
            app.state.event_npc_ref_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match res {
                Ok(catalog) => {
                    app.state.event_npc_ref_editor.catalog = Some(catalog.clone());
                    app.state.event_npc_ref_editor.status_msg =
                        format!("Event NPC catalog loaded: {} entries", catalog.len());
                    app.state.event_npc_ref_editor.refresh_npcs();
                    app.state.event_npc_ref_editor.init_pane_state();
                    app.state.event_npc_ref_spreadsheet.apply_filter(&catalog);
                    app.state
                        .event_npc_ref_spreadsheet
                        .compute_all_caches(&catalog);
                    app.state.event_npc_ref_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.event_npc_ref_editor.status_msg =
                        format!("Error loading event NPC catalog: {}", e);
                    app.state.event_npc_ref_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        EventNpcRefEditorMessage::Select(index) => {
            app.state.event_npc_ref_editor.select(index);
            Task::none()
        }
        EventNpcRefEditorMessage::FieldChanged(index, field, value) => {
            app.state
                .event_npc_ref_editor
                .update_field(index, &field, value);
            Task::none()
        }
        EventNpcRefEditorMessage::Save => {
            if app.state.shared_game_path.is_empty() {
                app.state.event_npc_ref_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.event_npc_ref_editor.loading_state =
                crate::loading_state::LoadingState::Loading;
            let result = app
                .state
                .event_npc_ref_editor
                .save_npcs(&app.state.shared_game_path);
            app.state.event_npc_ref_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(_) => {
                    app.state.event_npc_ref_editor.status_msg =
                        "Event NPCs saved successfully.".into()
                }
                Err(e) => {
                    app.state.event_npc_ref_editor.status_msg =
                        format!("Error saving event NPCs: {}", e)
                }
            }
            Task::none()
        }
        EventNpcRefEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                event_npc_ref_spreadsheet,
                event_npc_ref_editor,
                |index, field, value| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::EventNpcRef(
                            EventNpcRefEditorMessage::FieldChanged(index, field, value),
                        ),
                    )
                },
                msg
            );
            Task::none()
        }
        EventNpcRefEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.event_npc_ref_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.event_npc_ref_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        EventNpcRefEditorMessage::PaneClicked(pane) => {
            app.state.event_npc_ref_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
