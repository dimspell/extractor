// MessageScr editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::loading_state::LoadingState;
use crate::message::editor::messagescr::MessageScrEditorMessage;
use dispel_core::{Extractor, Message};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: MessageScrEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        MessageScrEditorMessage::LoadCatalog => {
            if app.state.shared_game_path.is_empty() {
                app.state.message_scr_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.message_scr_editor.loading_state =
                crate::loading_state::LoadingState::Loading;
            app.state.message_scr_spreadsheet.is_loading = true;
            let path = PathBuf::from(&app.state.shared_game_path)
                .join("ExtraInGame")
                .join("Message.scr");
            Task::perform(
                async move { Message::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                move |result: Result<Vec<dispel_core::Message>, String>| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::MessageScr(
                            MessageScrEditorMessage::CatalogLoaded(result),
                        ),
                    )
                },
            )
        }
        MessageScrEditorMessage::CatalogLoaded(result) => {
            app.state.message_scr_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.message_scr_editor.catalog = Some(catalog.clone());
                    app.state.message_scr_editor.status_msg =
                        format!("Message catalog loaded: {} entries", catalog.len());
                    app.state.message_scr_editor.refresh_messages();
                    app.state.message_scr_editor.init_pane_state();
                    app.state.message_scr_spreadsheet.apply_filter(&catalog);
                    app.state.message_scr_spreadsheet.compute_all_caches(&catalog);
                    app.state.message_scr_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.message_scr_editor.status_msg =
                        format!("Error loading message catalog: {}", e);
                    app.state.message_scr_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        MessageScrEditorMessage::Select(index) => {
            app.state.message_scr_editor.selected_idx = Some(index);
            app.state.message_scr_editor.select(index);
            Task::none()
        }
        MessageScrEditorMessage::FieldChanged(index, field, value) => {
            app.state
                .message_scr_editor
                .update_field(index, &field, value);
            Task::none()
        }
        MessageScrEditorMessage::Save => {
            if app.state.shared_game_path.is_empty() {
                app.state.message_scr_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.message_scr_editor.loading_state = LoadingState::Loading;
            let result = app
                .state
                .message_scr_editor
                .save_messages(&app.state.shared_game_path);
            app.state.message_scr_editor.loading_state = LoadingState::Loaded(());
            match result {
                Ok(_) => {
                    app.state.message_scr_editor.status_msg = "Messages saved successfully.".into()
                }
                Err(e) => {
                    app.state.message_scr_editor.status_msg =
                        format!("Error saving messages: {}", e)
                }
            }
            Task::none()
        }
        MessageScrEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                message_scr_spreadsheet,
                message_scr_editor,
                |index, field, value| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::MessageScr(
                            MessageScrEditorMessage::FieldChanged(index, field, value),
                        ),
                    )
                },
                msg
            );
            Task::none()
        }
        MessageScrEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.message_scr_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.message_scr_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        MessageScrEditorMessage::PaneClicked(pane) => {
            app.state.message_scr_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
