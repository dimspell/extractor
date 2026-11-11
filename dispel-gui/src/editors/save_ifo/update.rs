//! Message handling for the Save.ifo editor.

use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::editors::mod_packager::recording;
use crate::editors::save_ifo::message::{SaveIfoEditorMessage, TAIL_RECORD_ID};
use crate::editors::save_ifo::state::SaveIfoEditorState;
use crate::message::{Message, MessageExt};
use dispel_core::{Extractor, SaveIfo, SlotSummary, summarize_slots, swap_slots};
use iced::Task;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

type LoadResult = Result<(Vec<SlotSummary>, SaveIfo), String>;

/// Keep only the newest [`BACKUPS_TO_KEEP`] `Save.ifo.bak.*` files in `root`.
/// Pruning failures are logged, never fatal — a save must not fail because
/// old backups could not be removed.
const BACKUPS_TO_KEEP: usize = 5;

fn prune_old_backups(root: &Path) {
    let mut backups: Vec<std::path::PathBuf> = match std::fs::read_dir(root) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("Save.ifo.bak."))
            })
            .collect(),
        Err(e) => {
            eprintln!("save_ifo: failed to list backups for pruning: {e}");
            return;
        }
    };
    // Names embed a Unix-timestamp suffix, so lexical order == chronological.
    backups.sort();
    while backups.len() > BACKUPS_TO_KEEP {
        let oldest = backups.remove(0);
        if let Err(e) = std::fs::remove_file(&oldest) {
            eprintln!(
                "save_ifo: failed to remove old backup {}: {e}",
                oldest.display()
            );
        }
    }
}

/// Read per-slot summaries plus the `Save.ifo` record from a game root.
fn read_game_data(root: &Path) -> LoadResult {
    let summaries = summarize_slots(root).map_err(|e| e.to_string())?;
    let mut records = SaveIfo::read_file(&root.join("Save.ifo")).map_err(|e| e.to_string())?;
    let ifo = records
        .pop()
        .ok_or_else(|| "Save.ifo contained no records".to_string())?;
    Ok((summaries, ifo))
}

pub fn handle(message: SaveIfoEditorMessage, app: &mut App) -> Task<Message> {
    match message {
        SaveIfoEditorMessage::LoadCatalog => {
            if app.state.shared_game_path.is_empty() {
                app.state.editors.save_ifo_editor.status_msg =
                    "Please select game path first.".into();
                return Task::none();
            }
            if app.state.editors.save_ifo_editor.loading_state == LoadingState::Loading {
                app.state.editors.save_ifo_editor.status_msg =
                    "Operation already in progress.".into();
                return Task::none();
            }
            app.state.editors.save_ifo_editor.loading_state = LoadingState::Loading;
            app.state.editors.save_ifo_editor.status_msg = "Loading Save.ifo…".into();

            let root = PathBuf::from(&app.state.shared_game_path);
            Task::perform(async move { read_game_data(&root) }, |result| {
                Message::save_ifo(SaveIfoEditorMessage::CatalogLoaded(result))
            })
        }
        SaveIfoEditorMessage::CatalogLoaded(result) => {
            let editor = &mut app.state.editors.save_ifo_editor;
            match result {
                Ok((summaries, ifo)) => {
                    let occupied = summaries.iter().filter(|s| s.occupied).count();
                    editor.load_completed(summaries, ifo);
                    editor.loading_state = LoadingState::Loaded(());
                    editor.status_msg = format!("Loaded Save.ifo: {occupied} of 6 slots used");
                }
                Err(e) => {
                    editor.loading_state = LoadingState::Failed(e.clone());
                    editor.status_msg = format!("Failed to load Save.ifo: {e}");
                }
            }
            Task::none()
        }
        SaveIfoEditorMessage::FieldChanged(path, value) => {
            let old = app
                .state
                .editors
                .save_ifo_editor
                .data
                .as_ref()
                .and_then(|d| SaveIfoEditorState::field_value(&d.ifo, &path));
            app.state
                .editors
                .save_ifo_editor
                .update_field(path.clone(), value.clone());
            let new_value = app
                .state
                .editors
                .save_ifo_editor
                .data
                .as_ref()
                .and_then(|d| SaveIfoEditorState::field_value(&d.ifo, &path));
            match (old, new_value) {
                (Some(old), Some(new)) if old != new => recording::observe_field_change(
                    app,
                    "Save.ifo",
                    TAIL_RECORD_ID,
                    &path,
                    old,
                    new,
                ),
                _ => Task::none(),
            }
        }
        SaveIfoEditorMessage::SwapRequested(a, b) => {
            app.state.editors.save_ifo_editor.pending_swap = Some((a, b));
            Task::none()
        }
        SaveIfoEditorMessage::SwapCancel => {
            app.state.editors.save_ifo_editor.pending_swap = None;
            Task::none()
        }
        SaveIfoEditorMessage::SwapConfirm => {
            let editor = &mut app.state.editors.save_ifo_editor;
            if editor.loading_state == LoadingState::Loading {
                // Prevents a confirmed swap racing an in-flight save on Save.ifo.
                editor.status_msg = "Operation already in progress.".into();
                return Task::none();
            }
            let Some((a, b)) = editor.pending_swap.take() else {
                return Task::none();
            };
            if app.state.shared_game_path.is_empty() {
                editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            editor.loading_state = LoadingState::Loading;
            editor.status_msg = format!("Swapping slots {a} and {b}…");

            let root = PathBuf::from(&app.state.shared_game_path);
            Task::perform(
                async move {
                    swap_slots(&root, a, b).map_err(|e| e.to_string())?;
                    read_game_data(&root)
                },
                |result| Message::save_ifo(SaveIfoEditorMessage::SwapDone(result)),
            )
        }
        SaveIfoEditorMessage::SwapDone(result) => {
            let editor = &mut app.state.editors.save_ifo_editor;
            editor.pending_swap = None;
            match result {
                Ok((summaries, fresh)) => {
                    // A swap only rewrites slot records; keep any unsaved tail
                    // edits by splicing in just the fresh slot list.
                    if editor.data.is_some() {
                        editor.apply_swapped(summaries, fresh);
                    } else {
                        editor.load_completed(summaries, fresh);
                    }
                    editor.loading_state = LoadingState::Loaded(());
                    editor.status_msg = "Slots swapped".into();
                }
                Err(e) => {
                    // Leave loading_state untouched when nothing is loaded —
                    // there was no in-flight operation to complete.
                    if editor.data.is_some() {
                        editor.loading_state = LoadingState::Loaded(());
                    }
                    editor.status_msg = format!("Swap failed: {e}");
                }
            }
            Task::none()
        }
        SaveIfoEditorMessage::Save => {
            if app.state.shared_game_path.is_empty() {
                app.state.editors.save_ifo_editor.status_msg =
                    "Please select game path first.".into();
                return Task::none();
            }
            if app.state.editors.save_ifo_editor.loading_state == LoadingState::Loading {
                // Prevents Ctrl+S racing an in-flight swap writing the same file.
                app.state.editors.save_ifo_editor.status_msg =
                    "Operation already in progress.".into();
                return Task::none();
            }
            let Some(data) = app.state.editors.save_ifo_editor.data.clone() else {
                app.state.editors.save_ifo_editor.status_msg = "Nothing loaded to save.".into();
                return Task::none();
            };
            app.state.editors.save_ifo_editor.status_msg = "Saving Save.ifo…".into();
            app.state.editors.save_ifo_editor.loading_state = LoadingState::Loading;

            let root = PathBuf::from(&app.state.shared_game_path);
            Task::perform(
                async move {
                    let path = root.join("Save.ifo");
                    if path.exists() {
                        // Timestamped backup before overwriting.
                        let stamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map_err(|e| e.to_string())?
                            .as_secs();
                        let backup = root.join(format!("Save.ifo.bak.{stamp}"));
                        std::fs::copy(&path, &backup)
                            .map_err(|e| format!("Failed to create backup: {e}"))?;
                    }
                    Extractor::save_file(std::slice::from_ref(&data.ifo), &path)
                        .map_err(|e| e.to_string())?;
                    prune_old_backups(&root);
                    Ok(())
                },
                |result| Message::save_ifo(SaveIfoEditorMessage::Saved(result)),
            )
        }
        SaveIfoEditorMessage::Saved(result) => {
            let editor = &mut app.state.editors.save_ifo_editor;
            editor.loading_state = LoadingState::Loaded(());
            match result {
                Ok(()) => {
                    if let Some(data) = &mut editor.data {
                        data.dirty = false;
                    }
                    editor.sync_tail_buffers();
                    editor.status_msg = "Saved Save.ifo".into();
                }
                Err(e) => {
                    editor.status_msg = format!("Error saving Save.ifo: {e}");
                }
            }
            Task::none()
        }
    }
}
