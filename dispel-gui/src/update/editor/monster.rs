use std::path::PathBuf;
// Monster editor handlers
use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::loading_state::LoadingState;
use crate::message::editor::monster::MonsterEditorMessage;
use crate::message::MessageExt;
use dispel_core::{Extractor, Monster};
use iced::Task;

pub fn handle(message: MonsterEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        MonsterEditorMessage::LoadCatalog | MonsterEditorMessage::ScanMonsters => {
            // Load or scan monster catalog
            if app.state.shared_game_path.is_empty() {
                app.state.monster_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            app.state.monster_editor.loading_state = LoadingState::Loading;
            app.state.monster_spreadsheet.is_loading = true;
            app.state.monster_editor.status_msg = "Loading monster catalog...".into();

            let path = PathBuf::from(&app.state.shared_game_path);

            Task::perform(
                async move {
                    Monster::read_file(&path.join("MonsterInGame").join("Monster.db"))
                        .map_err(|e: std::io::Error| e.to_string())
                },
                |result: Result<Vec<dispel_core::Monster>, String>| {
                    crate::message::Message::monster(MonsterEditorMessage::Scanned(result))
                },
            )
        }
        MonsterEditorMessage::Scanned(result) => {
            app.state.monster_editor.loading_state = LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.monster_editor.catalog = Some(catalog.clone());
                    app.state.monster_editor.status_msg =
                        format!("Monster catalog loaded: {} monsters", catalog.len());
                    app.state.monster_editor.refresh();
                    app.state.monster_editor.init_pane_state();
                    // Initialize spreadsheet with all records and caches
                    app.state.monster_spreadsheet.apply_filter(&catalog);
                    app.state.monster_spreadsheet.compute_all_caches(&catalog);
                    app.state.monster_spreadsheet.is_loading = false;
                    let tab_id = app
                        .state
                        .workspace
                        .active()
                        .map(|t| t.id)
                        .unwrap_or(usize::MAX);
                    if let Some(ed) = app.state.monster_ref_editors.get_mut(&tab_id) {
                        ed.editor.refresh();
                    }
                }
                Err(e) => {
                    app.state.monster_editor.status_msg =
                        format!("Error loading monster catalog: {}", e);
                    app.state.monster_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        MonsterEditorMessage::SelectMonster(index) => {
            // Select monster at index
            app.state.monster_editor.selected_idx = Some(index);
            if let Some(catalog) = &app.state.monster_editor.catalog {
                if index < catalog.len() {
                    app.state.monster_editor.status_msg =
                        format!("Selected monster: {}", catalog[index]);
                }
            }
            Task::none()
        }
        MonsterEditorMessage::FieldChanged(index, field, value) => {
            // Update monster field
            eprintln!("Updated field {} for monster {}: {}", field, index, value);
            app.state.monster_editor.update_field(index, &field, value);
            Task::none()
        }
        MonsterEditorMessage::Save => {
            // Save monster changes
            if app.state.shared_game_path.is_empty() {
                app.state.monster_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            // if let Some(ref catalog) = app.state.monster_editor.catalog {
            app.state.monster_editor.loading_state = LoadingState::Loading;
            app.state.monster_editor.status_msg = "Saving monster changes...".into();

            let result = app
                .state
                .monster_editor
                .save(&app.state.shared_game_path, "MonsterInGame/Monster.db");

            // Simulate async save operation
            Task::perform(async { result }, |result: Result<(), String>| {
                crate::message::Message::monster(MonsterEditorMessage::Saved(result))
            })
            // }
            // app.state.monster_editor.status_msg = "No catalog to save".into();
            // Task::none()
        }
        MonsterEditorMessage::Saved(result) => {
            match result {
                Ok(_) => {
                    if let Some(ref catalog) = app.state.monster_editor.catalog {
                        app.state.monster_editor.status_msg =
                            format!("Successfully saved {} monsters", catalog.len());
                    } else {
                        app.state.monster_editor.status_msg = "Save completed".into();
                    }
                }
                Err(e) => {
                    app.state.monster_editor.status_msg = format!("Error saving monsters: {}", e);
                }
            }
            app.state.monster_editor.loading_state = LoadingState::Loaded(());
            Task::none()
        }
        MonsterEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                monster_spreadsheet,
                monster_editor,
                |index, field, value| {
                    crate::message::Message::monster(MonsterEditorMessage::FieldChanged(
                        index, field, value,
                    ))
                },
                msg
            );
            Task::none()
        }
        MonsterEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.monster_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        MonsterEditorMessage::PaneClicked(pane) => {
            app.state.monster_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
