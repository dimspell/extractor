// HealItem editor handlers

use crate::app::App;
use crate::handle_spreadsheet_messages;
use crate::loading_state::LoadingState;
use crate::message::editor::healitem::HealItemEditorMessage;
use crate::utils::browse_folder;
use dispel_core::{Extractor, HealItem};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: HealItemEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        HealItemEditorMessage::BrowseSpritePath => {
            // TODO: Remove this type after removing all references to the logic
            // Handle browse sprite path - open file dialog
            browse_folder("heal_item_sprite_path")
        }
        HealItemEditorMessage::ScanItems => {
            // Scan heal items from game files
            if app.state.shared_game_path.is_empty() {
                app.state.heal_item_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            app.state.heal_item_editor.loading_state = LoadingState::Loading;
            app.state.heal_item_spreadsheet.is_loading = true;
            app.state.heal_item_editor.status_msg = "Scanning heal items...".into();

            let path = PathBuf::from(&app.state.shared_game_path);

            // Simulate async heal item scanning
            Task::perform(
                async move {
                    HealItem::read_file(&path.join("CharacterInGame").join("HealItem.db"))
                        .map_err(|e: std::io::Error| e.to_string())
                    // Simulated catalog
                },
                |result: Result<Vec<dispel_core::HealItem>, String>| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::HealItem(
                            HealItemEditorMessage::Scanned(result), // former: HealItemCatalogLoaded
                        ),
                    )
                },
            )
        }
        HealItemEditorMessage::Scanned(result) => {
            app.state.heal_item_editor.loading_state =
                crate::loading_state::LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.heal_item_editor.catalog = Some(catalog.clone());
                    app.state.heal_item_editor.status_msg =
                        format!("Heal item catalog loaded: {} items", catalog.len());
                    app.state.heal_item_editor.refresh();
                    app.state.heal_item_editor.init_pane_state();
                    app.state.heal_item_spreadsheet.apply_filter(&catalog);
                    app.state.heal_item_spreadsheet.compute_all_caches(&catalog);
                    app.state.heal_item_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.heal_item_editor.status_msg =
                        format!("Error loading heal item catalog: {}", e);
                    app.state.heal_item_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        HealItemEditorMessage::SelectItem(index) => {
            // Select heal item at index
            app.state.heal_item_editor.selected_idx = Some(index);
            if let Some(catalog) = &app.state.heal_item_editor.catalog {
                if index < catalog.len() {
                    app.state.heal_item_editor.status_msg =
                        format!("Selected heal item: {}", catalog[index]);
                }
            }
            Task::none()
        }
        HealItemEditorMessage::FieldChanged(index, field, value) => {
            // Update heal item field
            eprintln!("Updated field {} for item {}: {}", field, index, value);
            app.state
                .heal_item_editor
                .update_field(index, &field, value);
            Task::none()
        }
        HealItemEditorMessage::Save => {
            // Save heal item changes
            if app.state.shared_game_path.is_empty() {
                app.state.heal_item_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            // Pre-save validation
            let validation_errors = app.state.heal_item_editor.validate_all();
            if !validation_errors.is_empty() {
                let error_summary: Vec<String> = validation_errors
                    .iter()
                    .take(5)
                    .map(|(idx, errs)| {
                        let record_label = format!("Record #{}", idx);

                        let field_errors: String = errs
                            .iter()
                            .map(|(f, e)| format!("{}: {}", f, e))
                            .collect::<Vec<_>>()
                            .join(", ");

                        format!("{}: {}", record_label, field_errors)
                    })
                    .collect();

                let error_msg = if validation_errors.len() > 5 {
                    format!(
                        "Found {} records with validation errors:\n{}\n... and {} more",
                        validation_errors.len(),
                        error_summary.join("\n"),
                        validation_errors.len() - 5
                    )
                } else {
                    format!(
                        "Found {} records with validation errors:\n{}",
                        validation_errors.len(),
                        error_summary.join("\n")
                    )
                };

                use rfd::MessageDialog;

                MessageDialog::new()
                    .set_title("Validation Errors")
                    .set_description(&error_msg)
                    .set_buttons(rfd::MessageButtons::Ok)
                    .show();

                return Task::none();
            }

            app.state.heal_item_editor.loading_state = LoadingState::Loading;
            let result = app
                .state
                .heal_item_editor
                .save(&app.state.shared_game_path, "CharacterInGame/HealItem.db");
            app.state.heal_item_editor.loading_state = LoadingState::Loaded(());

            match result {
                Ok(_) => {
                    app.state.heal_item_editor.status_msg = "Heal items saved successfully.".into()
                }
                Err(e) => {
                    app.state.heal_item_editor.status_msg =
                        format!("Error saving heal items: {}", e)
                }
            }
            Task::none()
        }
        HealItemEditorMessage::Saved(result) => {
            match result {
                Ok(_) => {
                    if let Some(ref catalog) = app.state.heal_item_editor.catalog {
                        app.state.heal_item_editor.status_msg =
                            format!("Successfully saved {} heal items", catalog.len());
                    } else {
                        app.state.heal_item_editor.status_msg = "Save completed".into();
                    }
                }
                Err(e) => {
                    app.state.heal_item_editor.status_msg =
                        format!("Error saving heal items: {}", e);
                }
            }
            Task::none()
        }
        HealItemEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                heal_item_spreadsheet,
                heal_item_editor,
                |index, field, value| {
                    crate::message::Message::Editor(
                        crate::message::editor::EditorMessage::HealItem(
                            HealItemEditorMessage::FieldChanged(index, field, value),
                        ),
                    )
                },
                msg
            );
            Task::none()
        }
        HealItemEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.heal_item_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.heal_item_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        HealItemEditorMessage::PaneClicked(pane) => {
            app.state.heal_item_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
