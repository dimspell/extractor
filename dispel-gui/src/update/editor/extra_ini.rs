// ExtraIni editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::extraini::ExtraIniEditorMessage;
use dispel_core::{Extra, Extractor};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: ExtraIniEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        ExtraIniEditorMessage::LoadCatalog => {
            if app.state.shared_game_path.is_empty() {
                app.state.extra_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.extra_ini_editor.loading_state = crate::loading_state::LoadingState::Loading;
            app.state.extra_ini_spreadsheet.is_loading = true;
            let path = PathBuf::from(&app.state.shared_game_path).join("Extra.ini");
            Task::perform(
                async move { Extra::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                move |result: Result<Vec<dispel_core::Extra>, String>| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::ExtraIni(
                            ExtraIniEditorMessage::CatalogLoaded(result),
                        ),
                    )
                },
            )
        }
        ExtraIniEditorMessage::CatalogLoaded(result) => {
            app.state.extra_ini_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.extra_ini_editor.catalog = Some(catalog.clone());
                    app.state.extra_ini_editor.status_msg =
                        format!("Extra catalog loaded: {} entries", catalog.len());
                    app.state.extra_ini_editor.refresh_extras();
                    app.state.extra_ini_editor.init_pane_state();
                    app.state.extra_ini_spreadsheet.apply_filter(&catalog);
                    app.state.extra_ini_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.extra_ini_editor.status_msg =
                        format!("Error loading extra catalog: {}", e);
                    app.state.extra_ini_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        ExtraIniEditorMessage::SelectExtra(index) => {
            app.state.extra_ini_editor.select(index);
            Task::none()
        }
        ExtraIniEditorMessage::FieldChanged(index, field, value) => {
            app.state
                .extra_ini_editor
                .update_field(index, &field, value);
            Task::none()
        }
        ExtraIniEditorMessage::Save => {
            if app.state.shared_game_path.is_empty() {
                app.state.extra_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.extra_ini_editor.loading_state = crate::loading_state::LoadingState::Loading;
            let result = app
                .state
                .extra_ini_editor
                .save_extras(&app.state.shared_game_path);
            app.state.extra_ini_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(_) => {
                    app.state.extra_ini_editor.status_msg = "Extras saved successfully.".into()
                }
                Err(e) => {
                    app.state.extra_ini_editor.status_msg = format!("Error saving extras: {}", e)
                }
            }
            Task::none()
        }
        ExtraIniEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                extra_ini_spreadsheet,
                extra_ini_editor,
                |index, field, value| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::ExtraIni(
                            ExtraIniEditorMessage::FieldChanged(index, field, value),
                        ),
                    )
                },
                msg
            );
            Task::none()
        }
        ExtraIniEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.extra_ini_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.extra_ini_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        ExtraIniEditorMessage::PaneClicked(pane) => {
            app.state.extra_ini_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
