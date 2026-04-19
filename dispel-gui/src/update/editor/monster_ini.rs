// MonsterIni editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::loading_state::LoadingState;
use crate::message::editor::monsterini::MonsterIniEditorMessage;
use crate::message::MessageExt;
use dispel_core::{Extractor, MonsterIni};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: MonsterIniEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        MonsterIniEditorMessage::LoadCatalog | MonsterIniEditorMessage::ScanMonsters => {
            // Load Monster INI catalog
            if app.state.shared_game_path.is_empty() {
                app.state.monster_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.monster_ini_editor.loading_state = LoadingState::Loading;
            app.state.monster_ini_spreadsheet.is_loading = true;
            let path = PathBuf::from(&app.state.shared_game_path);
            Task::perform(
                async move {
                    MonsterIni::read_file(&path.join("Monster.ini"))
                        .map_err(|e: std::io::Error| e.to_string())
                },
                move |result: Result<Vec<dispel_core::MonsterIni>, String>| {
                    crate::message::Message::monster_ini(MonsterIniEditorMessage::CatalogLoaded(
                        result,
                    ))
                },
            )
        }
        MonsterIniEditorMessage::CatalogLoaded(result) => {
            app.state.monster_ini_editor.loading_state = LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.monster_ini_editor.catalog = Some(catalog.clone());
                    app.state.monster_ini_editor.status_msg =
                        format!("Monster catalog loaded: {} monsters", catalog.len());
                    app.state.monster_ini_editor.refresh_monsters();
                    app.state.monster_ini_editor.init_pane_state();
                    app.state.monster_ini_spreadsheet.apply_filter(&catalog);
                    app.state.monster_ini_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.monster_ini_editor.status_msg =
                        format!("Error loading Monster catalog: {}", e);
                    app.state.monster_ini_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        MonsterIniEditorMessage::SelectMonster(index) => {
            // Select Monster at index
            app.state.monster_ini_editor.select_monster(index);
            Task::none()
        }
        MonsterIniEditorMessage::FieldChanged(index, field, value) => {
            // Update Monster field
            app.state
                .monster_ini_editor
                .update_field(index, &field, value);
            Task::none()
        }
        MonsterIniEditorMessage::Save => {
            // Save Monster changes
            if app.state.shared_game_path.is_empty() {
                app.state.monster_ini_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.monster_ini_editor.loading_state = LoadingState::Loading;
            let result = app
                .state
                .monster_ini_editor
                .save_monsters(&app.state.shared_game_path);
            app.state.monster_ini_editor.loading_state = LoadingState::Loaded(());
            match result {
                Ok(_) => {
                    app.state.monster_ini_editor.status_msg = "Monsters saved successfully.".into()
                }
                Err(e) => {
                    app.state.monster_ini_editor.status_msg =
                        format!("Error saving Monsters: {}", e)
                }
            }
            Task::none()
        }
        MonsterIniEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                monster_ini_spreadsheet,
                monster_ini_editor,
                |index, field, value| {
                    crate::message::Message::monster_ini(MonsterIniEditorMessage::FieldChanged(
                        index, field, value,
                    ))
                },
                msg
            );
            Task::none()
        }
        MonsterIniEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.monster_ini_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        MonsterIniEditorMessage::PaneClicked(pane) => {
            app.state.monster_ini_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
