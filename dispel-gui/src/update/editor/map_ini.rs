// MapIni editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::mapini::MapIniEditorMessage;
use crate::message::MessageExt;
use dispel_core::{Extractor, MapIni};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: MapIniEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        MapIniEditorMessage::LoadCatalog => {
            if app.state.shared_game_path.is_empty() {
                app.state.map_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.map_ini_editor.loading_state = crate::loading_state::LoadingState::Loading;
            app.state.map_ini_spreadsheet.is_loading = true;
            let path = PathBuf::from(&app.state.shared_game_path)
                .join("Ref")
                .join("Map.ini");
            Task::perform(
                async move { MapIni::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                move |result: Result<Vec<dispel_core::MapIni>, String>| {
                    crate::message::Message::map_ini(MapIniEditorMessage::CatalogLoaded(result))
                },
            )
        }
        MapIniEditorMessage::CatalogLoaded(result) => {
            app.state.map_ini_editor.loading_state = crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.map_ini_editor.catalog = Some(catalog.clone());
                    app.state.map_ini_editor.status_msg =
                        format!("Map ini catalog loaded: {} entries", catalog.len());
                    app.state.map_ini_editor.refresh_maps();
                    app.state.map_ini_editor.init_pane_state();
                    app.state.map_ini_spreadsheet.apply_filter(&catalog);
                    app.state.map_ini_spreadsheet.compute_all_caches(&catalog);
                    app.state.map_ini_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.map_ini_editor.status_msg =
                        format!("Error loading map ini catalog: {}", e);
                    app.state.map_ini_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        MapIniEditorMessage::Select(index) => {
            app.state.map_ini_editor.select(index);
            Task::none()
        }
        MapIniEditorMessage::FieldChanged(index, field, value) => {
            app.state.map_ini_editor.update_field(index, &field, value);
            Task::none()
        }
        MapIniEditorMessage::Save => {
            if app.state.shared_game_path.is_empty() {
                app.state.map_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.map_ini_editor.loading_state = crate::loading_state::LoadingState::Loading;
            let result = app
                .state
                .map_ini_editor
                .save_maps(&app.state.shared_game_path);
            app.state.map_ini_editor.loading_state = crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(_) => app.state.map_ini_editor.status_msg = "Map ini saved successfully.".into(),
                Err(e) => {
                    app.state.map_ini_editor.status_msg = format!("Error saving map ini: {}", e)
                }
            }
            Task::none()
        }
        MapIniEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                map_ini_spreadsheet,
                map_ini_editor,
                |index, field, value| {
                    crate::message::Message::map_ini(MapIniEditorMessage::FieldChanged(
                        index, field, value,
                    ))
                },
                msg
            );
            Task::none()
        }
        MapIniEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.map_ini_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.map_ini_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        MapIniEditorMessage::PaneClicked(pane) => {
            app.state.map_ini_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
