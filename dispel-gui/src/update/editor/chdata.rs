// ChData editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::message::editor::chdata::ChDataEditorMessage;
use crate::message::MessageExt;
use dispel_core::{ChData, Extractor};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: ChDataEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        ChDataEditorMessage::LoadCatalog => {
            if app.state.shared_game_path.is_empty() {
                app.state.chdata_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.chdata_editor.loading_state = crate::loading_state::LoadingState::Loading;
            app.state.chdata_spreadsheet.is_loading = true;
            let path = PathBuf::from(&app.state.shared_game_path)
                .join("CharacterInGame")
                .join("ChData.db");
            Task::perform(
                async move { ChData::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                move |result: Result<Vec<dispel_core::ChData>, String>| {
                    crate::message::Message::ch_data(ChDataEditorMessage::CatalogLoaded(result))
                },
            )
        }
        ChDataEditorMessage::CatalogLoaded(result) => {
            app.state.chdata_editor.loading_state = crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.chdata_editor.catalog = Some(catalog.clone());
                    app.state.chdata_editor.status_msg =
                        format!("Loaded {} ChData records.", catalog.len());
                    app.state.chdata_editor.init_pane_state();
                    app.state.chdata_spreadsheet.apply_filter(&catalog);
                    app.state.chdata_spreadsheet.is_loading = false;
                    if !catalog.is_empty() {
                        app.state.chdata_editor.select(0);
                    }
                }
                Err(e) => {
                    app.state.chdata_editor.status_msg = format!("Error loading ChData: {}", e);
                    app.state.chdata_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        ChDataEditorMessage::SelectData(index) => {
            app.state.chdata_editor.select(index);
            Task::none()
        }
        ChDataEditorMessage::FieldChanged(index, field, value) => {
            app.state.chdata_editor.update_field(index, &field, value);
            Task::none()
        }
        ChDataEditorMessage::Save => {
            if app.state.shared_game_path.is_empty() {
                app.state.chdata_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.chdata_editor.loading_state = crate::loading_state::LoadingState::Loading;
            let result = app
                .state
                .chdata_editor
                .save_data(&app.state.shared_game_path);
            app.state.chdata_editor.loading_state = crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(_) => app.state.chdata_editor.status_msg = "ChData saved successfully.".into(),
                Err(e) => {
                    app.state.chdata_editor.status_msg = format!("Error saving ChData: {}", e)
                }
            }
            Task::none()
        }
        ChDataEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                chdata_spreadsheet,
                chdata_editor,
                |index, field, value| {
                    crate::message::Message::ch_data(ChDataEditorMessage::FieldChanged(
                        index, field, value,
                    ))
                },
                msg
            );
            Task::none()
        }
        ChDataEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.chdata_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.chdata_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        ChDataEditorMessage::PaneClicked(pane) => {
            app.state.chdata_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
