// Magic editor handlers

use crate::app::App;
use crate::handle_field_changed;
use crate::handle_spreadsheet_messages;
use crate::loading_state::LoadingState;
use crate::message::editor::magic::MagicEditorMessage;
use crate::message::MessageExt;
use dispel_core::{Extractor, MagicSpell};
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: MagicEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        MagicEditorMessage::LoadCatalog => {
            // Load or scan magic catalog
            if app.state.shared_game_path.is_empty() {
                app.state.magic_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            app.state.magic_editor.loading_state = LoadingState::Loading;
            app.state.magic_spreadsheet.is_loading = true;
            app.state.magic_editor.status_msg = "Loading magic catalog...".into();

            let path = PathBuf::from(&app.state.shared_game_path);

            // Simulate async magic catalog loading
            Task::perform(
                async move {
                    MagicSpell::read_file(&path.join("MagicInGame").join("Magic.db"))
                        .map_err(|e: std::io::Error| e.to_string())
                },
                |result: Result<Vec<dispel_core::MagicSpell>, String>| {
                    crate::message::Message::magic(MagicEditorMessage::CatalogLoaded(result))
                },
            )
        }
        MagicEditorMessage::CatalogLoaded(result) => {
            app.state.magic_editor.loading_state = LoadingState::Loaded(());
            match result {
                Ok(catalog) => {
                    app.state.magic_editor.catalog = Some(catalog.clone());
                    app.state.magic_editor.status_msg =
                        format!("Magic catalog loaded: {} spells", catalog.len());
                    app.state.magic_editor.refresh();
                    app.state.magic_editor.init_pane_state();
                    app.state.magic_spreadsheet.apply_filter(&catalog);
                    app.state.magic_spreadsheet.compute_all_caches(&catalog);
                    app.state.magic_spreadsheet.is_loading = false;
                }
                Err(e) => {
                    app.state.magic_editor.status_msg =
                        format!("Error loading magic catalog: {}", e);
                    app.state.magic_spreadsheet.is_loading = false;
                }
            }
            Task::none()
        }
        MagicEditorMessage::Select(index) => {
            // Select spell at index
            app.state.magic_editor.selected_idx = Some(index);
            if let Some(catalog) = &app.state.magic_editor.catalog {
                if index < catalog.len() {
                    app.state.magic_editor.status_msg =
                        format!("Selected spell: {}", catalog[index]);
                }
            }
            Task::none()
        }
        MagicEditorMessage::FieldChanged(index, field, value) => {
            handle_field_changed!(app, magic_editor, index, field, value)
        }
        MagicEditorMessage::Save => {
            // Save spell changes
            if app.state.shared_game_path.is_empty() {
                app.state.magic_editor.status_msg = "Please select game path first.".into();

                return Task::none();
            }
            app.state.magic_editor.loading_state = LoadingState::Loading;

            // if let Some(ref catalog) = app.state.magic_editor.catalog {
            app.state.magic_editor.status_msg = "Saving spell changes...".into();

            // TODO: Use the async Task to perform saving the file
            let result = app
                .state
                .magic_editor
                .save(&app.state.shared_game_path, "MagicInGame/Magic.db");

            // Simulate async save operation
            Task::perform(async { result }, |result: Result<(), String>| {
                crate::message::Message::magic(MagicEditorMessage::Saved(result))
            })
            // }
            // app.state.magic_editor.status_msg = "No catalog to save".into();
            // Task::none()
        }
        MagicEditorMessage::Saved(result) => {
            app.state.magic_editor.loading_state = LoadingState::Loaded(());
            match result {
                Ok(_) => {
                    if let Some(ref catalog) = app.state.magic_editor.catalog {
                        app.state.magic_editor.status_msg =
                            format!("Successfully saved {} spells", catalog.len());
                    } else {
                        app.state.magic_editor.status_msg = "Save completed".into();
                    }
                }
                Err(e) => {
                    app.state.magic_editor.status_msg = format!("Error saving spells: {}", e);
                }
            }
            Task::none()
        }
        MagicEditorMessage::Spreadsheet(msg) => {
            handle_spreadsheet_messages!(
                app,
                magic_spreadsheet,
                magic_editor,
                |index, field, value| {
                    crate::message::Message::magic(MagicEditorMessage::FieldChanged(
                        index, field, value,
                    ))
                },
                msg
            );
            Task::none()
        }
        MagicEditorMessage::PaneResized(event) => {
            if let Some(ref mut ps) = app.state.magic_editor.pane_state {
                ps.resize(event.split, event.ratio);
            }
            if let Some(ref mut ps) = app.state.magic_spreadsheet.pane_state {
                ps.resize(event.split, event.ratio);
            }
            Task::none()
        }
        MagicEditorMessage::PaneClicked(pane) => {
            app.state.magic_editor.pane_focus = Some(pane);
            Task::none()
        }
    }
}
