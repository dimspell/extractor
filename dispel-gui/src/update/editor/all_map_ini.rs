// AllMapIni editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::loading_state::LoadingState;
use crate::message::editor::allmapini::AllMapIniEditorMessage;
use crate::message::MessageExt;
use dispel_core::{Extractor, Map};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: AllMapIniEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        AllMapIniEditorMessage::LoadCatalog => {
            // Load catalog
            if app.state.shared_game_path.is_empty() {
                app.state.all_map_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            app.state.all_map_ini_editor.loading_state = LoadingState::Loading;
            app.state.all_map_ini_spreadsheet.is_loading = true;
            let path = PathBuf::from(&app.state.shared_game_path).join("AllMap.ini");

            Task::perform(
                async move { Map::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                |result: Result<Vec<Map>, String>| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::AllMapIni(
                            AllMapIniEditorMessage::CatalogLoaded(result),
                        ),
                    )
                },
            )
        }
        AllMapIniEditorMessage::CatalogLoaded(res) => {
            app.state.all_map_ini_editor.loading_state = LoadingState::Loaded(());
            app.state.all_map_ini_spreadsheet.is_loading = false;
            match res {
                Ok(catalog) => {
                    app.state.all_map_ini_editor.catalog = Some(catalog.clone());
                    app.state.all_map_ini_editor.status_msg =
                        format!("Map catalog loaded: {} maps", catalog.len());
                    app.state.all_map_ini_editor.refresh();
                    app.state.all_map_ini_editor.init_pane_state();
                    app.state.all_map_ini_spreadsheet.apply_filter(&catalog);
                }
                Err(e) => {
                    app.state.all_map_ini_editor.status_msg =
                        format!("Error loading map catalog: {}", e)
                }
            }
            Task::none()
        }
        AllMapIniEditorMessage::SelectMap(index) => {
            // Select map
            app.state.all_map_ini_editor.selected_idx = Some(index);
            Task::none()
        }
        AllMapIniEditorMessage::FieldChanged(index, field, value) => {
            // Update field
            app.state
                .all_map_ini_editor
                .update_field(index, &field, value);
            Task::none()
        }
        AllMapIniEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                all_map_ini_spreadsheet,
                all_map_ini_editor,
                |index, field, value| {
                    crate::message::Message::all_map_ini(AllMapIniEditorMessage::FieldChanged(
                        index, field, value,
                    ))
                },
                msg
            );
            Task::none()
        }
        AllMapIniEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.all_map_ini_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.all_map_ini_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        AllMapIniEditorMessage::PaneClicked(pane) => {
            app.state.all_map_ini_editor.pane_focus = Some(pane);
            Task::none()
        }
        AllMapIniEditorMessage::Save => {
            // Save changes
            if app.state.shared_game_path.is_empty() {
                app.state.all_map_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            app.state.all_map_ini_editor.loading_state = LoadingState::Loading;
            let result = app
                .state
                .all_map_ini_editor
                .save(&app.state.shared_game_path, "AllMap.ini");
            app.state.all_map_ini_editor.loading_state = LoadingState::Loaded(());

            match result {
                Ok(_) => {
                    app.state.all_map_ini_editor.status_msg = "Maps saved successfully.".into()
                }

                Err(e) => {
                    app.state.all_map_ini_editor.status_msg = format!("Error saving maps: {}", e)
                }
            }

            Task::none()
        }
    }
}
