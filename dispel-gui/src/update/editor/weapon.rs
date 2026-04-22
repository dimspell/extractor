// Weapon editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::loading_state::LoadingState;
use crate::message::editor::weapon::WeaponEditorMessage;
use crate::message::MessageExt;
use dispel_core::{Extractor, WeaponItem};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: WeaponEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        WeaponEditorMessage::ScanWeapons => {
            // Scan weapons from game files
            if app.state.shared_game_path.is_empty() {
                app.state.weapon_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            app.state.weapon_editor.loading_state = LoadingState::Loading;
            app.state.weapon_editor.status_msg = "Scanning weapons...".into();
            app.state.weapon_spreadsheet.is_loading = true;

            let path = PathBuf::from(&app.state.shared_game_path);

            // Use a simple future that doesn't capture any references
            Task::perform(
                async move {
                    WeaponItem::read_file(&path.join("CharacterInGame").join("weaponItem.db"))
                        .map_err(|e: std::io::Error| e.to_string())
                },
                move |result: Result<Vec<dispel_core::WeaponItem>, String>| {
                    crate::message::Message::weapon(WeaponEditorMessage::Scanned(result))
                },
            )
        }
        WeaponEditorMessage::Scanned(res) => {
            // Handle catalog loaded
            app.state.weapon_editor.loading_state = LoadingState::Loaded(());
            match res {
                Ok(catalog) => {
                    app.state.weapon_editor.catalog = Some(catalog.clone());
                    app.state.weapon_editor.status_msg =
                        format!("Weapon catalog loaded: {} weapons", catalog.len());
                    app.state.weapon_editor.refresh();
                    app.state.weapon_editor.init_pane_state();
                    app.state.weapon_spreadsheet.apply_filter(&catalog);
                    app.state.weapon_spreadsheet.compute_all_caches(&catalog);
                    app.state.weapon_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.weapon_editor.status_msg =
                        format!("Error loading weapon catalog: {}", e);
                    app.state.weapon_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        WeaponEditorMessage::SelectWeapon(index) => {
            // Select a weapon at the index
            app.state.weapon_editor.selected_idx = Some(index);
            eprintln!("[DEBUG]: Selected weapon {}", index);
            Task::none()
        }
        WeaponEditorMessage::FieldChanged(index, field, value) => {
            // Update weapon field
            eprintln!("Updated field {} for weapon {}: {}", field, index, value);
            app.state.weapon_editor.update_field(index, &field, value);
            Task::none()
        }
        WeaponEditorMessage::Save => {
            // Save weapon changes
            if app.state.shared_game_path.is_empty() {
                app.state.weapon_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            if app.state.weapon_editor.catalog.is_some() {
                app.state.weapon_editor.status_msg = "Saving weapon changes...".into();
                app.state.weapon_editor.loading_state = crate::loading_state::LoadingState::Loading;

                let result = app
                    .state
                    .weapon_editor
                    .save(&app.state.shared_game_path, "CharacterInGame/weaponItem.db");

                return Task::perform(async { result }, |result: Result<(), String>| {
                    crate::message::Message::weapon(WeaponEditorMessage::Saved(result))
                });
            }

            app.state.weapon_editor.status_msg = "No weapon selected to save".into();
            Task::none()
        }
        WeaponEditorMessage::Saved(result) => {
            app.state.weapon_editor.loading_state = crate::loading_state::LoadingState::Loaded(());

            match result {
                Ok(_) => {
                    app.state.weapon_editor.status_msg = "Weapon changes saved successfully".into();
                }
                Err(e) => {
                    app.state.weapon_editor.status_msg = format!("Error saving weapons: {}", e);
                }
            }
            Task::none()
        }
        WeaponEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                weapon_spreadsheet,
                weapon_editor,
                |index, field, value| {
                    crate::message::Message::weapon(WeaponEditorMessage::FieldChanged(
                        index, field, value,
                    ))
                },
                msg
            );
            Task::none()
        }

        WeaponEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.weapon_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.weapon_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        WeaponEditorMessage::PaneClicked(pane) => {
            app.state.weapon_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
