use crate::app::App;
use crate::editors::fog_data::message::FogDataMessage;
use crate::message::{Message, MessageExt};
use dispel_core::map::fogdata::{MAX_FACTOR, ROW_LEN, ROWS};
use iced::Task;
use std::path::PathBuf;

/// Parameters for recording a fogdata save into an active mod session.
#[derive(Debug, Clone)]
pub struct RecordingParams {
    pub workspace_root: PathBuf,
    pub game_path: PathBuf,
    pub mod_slug: String,
    pub relative_path: String,
}

/// Mark the workspace tab's modified flag.
///
/// Inlined at call sites (rather than taking `&mut App`) so the borrow
/// checker can split it from the outstanding `fog_editors` borrow.
macro_rules! mark_tab_modified {
    ($app:expr, $tab_id:expr, $modified:expr) => {
        if let Some(tab) = $app
            .state
            .workspace
            .tabs
            .iter_mut()
            .find(|t| t.id == $tab_id)
        {
            tab.modified = $modified;
        }
    };
}

pub fn handle(message: FogDataMessage, app: &mut App) -> Task<Message> {
    // Every variant names its tab, so we always mutate the right editor.
    let tab_id = message.tab_id();
    let Some(editor) = app.state.editors.fog_editors.get_mut(&tab_id) else {
        return Task::none();
    };

    match message {
        // ── Navigation ───────────────────────────────────────────────────────
        FogDataMessage::LevelSelected(_, level) => {
            if level >= 1 && (level as usize) <= ROWS {
                editor.selected_level = level;
                editor.sync_input_from_selection();
                return level_list_scroll_task(level);
            }
        }
        FogDataMessage::PairSelected(_, pair) => {
            if pair < ROW_LEN {
                editor.selected_pair = pair;
                editor.sync_input_from_selection();
            }
        }

        // ── Curve painting ───────────────────────────────────────────────────
        FogDataMessage::FactorPainted(_, pair, value) => {
            if value > MAX_FACTOR || pair >= ROW_LEN {
                return Task::none(); // Never trust the pointer beyond bounds.
            }
            editor.begin_stroke_if_needed();
            editor.selected_pair = pair;
            let changed = editor.paint_factor(pair, value);
            if changed {
                editor.dirty = true;
                editor.save_generation += 1;
                mark_tab_modified!(app, tab_id, true);
            }
            editor.sync_input_from_selection();
        }
        FogDataMessage::StrokeEnded(_) => {
            editor.end_stroke();
        }

        // ── Inspector edits ──────────────────────────────────────────────────
        FogDataMessage::FactorCommitted(_, pair, value) => {
            if value > MAX_FACTOR || pair >= ROW_LEN {
                return Task::none();
            }
            match editor.commit_factor(pair, value) {
                Ok(changed) => {
                    editor.input_error = None;
                    editor.sync_input_from_selection();
                    if changed {
                        editor.dirty = true;
                        editor.save_generation += 1;
                        mark_tab_modified!(app, tab_id, true);
                    }
                }
                Err(e) => editor.input_error = Some(e),
            }
        }
        FogDataMessage::ValueInputChanged(_, raw) => {
            editor.set_value_input(raw);
        }
        FogDataMessage::ValueSubmitted(_) => {
            if editor.submit_value_input().is_some() {
                editor.dirty = true;
                editor.save_generation += 1;
                mark_tab_modified!(app, tab_id, true);
            }
        }

        // ── Save ─────────────────────────────────────────────────────────────
        FogDataMessage::Save(_) => {
            // Capture the generation this save is based on; any edit (or
            // another save) bumps it and invalidates the completion.
            editor.save_generation += 1;
            let save_generation = editor.save_generation;
            let path = editor.save_path.clone();
            let fog = editor.fog.clone();
            let recording = mod_recording_params(app, &path);
            app.state.status_msg = "Saving fogdata.dat…".to_string();
            return Task::perform(
                async move { save_fog_data(path, fog, recording).await },
                move |result| {
                    Message::fog_data(FogDataMessage::SaveComplete(
                        tab_id,
                        save_generation,
                        result,
                    ))
                },
            );
        }
        FogDataMessage::SaveComplete(_, generation, result) => match result {
            Ok((msg, recording_error)) => {
                // Only mark clean when no edit landed while the save was in
                // flight — a stale completion must keep the tab dirty.
                if generation == editor.save_generation {
                    editor.mark_clean();
                    mark_tab_modified!(app, tab_id, false);
                }
                // The file on disk IS current even when only the mod
                // recording failed — report both facts separately.
                app.state.status_msg = match recording_error {
                    Some(err) => format!("{msg} — Mod recording failed: {err}"),
                    None => msg,
                };
            }
            Err(e) => {
                // Transient failure: keep the editor dirty and report via the
                // status bar; the load-error surface stays reserved for parse
                // failures.
                app.state.status_msg = format!("Save failed: {e}");
            }
        },

        // ── Revert ───────────────────────────────────────────────────────────
        FogDataMessage::Revert(_) => {
            if !editor.dirty {
                // Nothing to lose — reload straight away.
                match editor.reload_from_disk() {
                    Ok(()) => app.state.status_msg = "Reverted fogdata.dat".to_string(),
                    Err(e) => app.state.status_msg = format!("Revert failed: {e}"),
                }
            } else {
                editor.confirm_revert = true;
            }
        }
        FogDataMessage::RevertConfirmed(_) => match editor.reload_from_disk() {
            Ok(()) => {
                mark_tab_modified!(app, tab_id, false);
                app.state.status_msg = "Reverted fogdata.dat".to_string();
            }
            Err(e) => {
                editor.confirm_revert = false;
                app.state.status_msg = format!("Revert failed: {e}");
            }
        },
        FogDataMessage::RevertCancelled(_) => {
            editor.confirm_revert = false;
        }

        // ── Undo / redo (also reachable via the global Ctrl+Z/Ctrl+Y path) ───
        FogDataMessage::Undo(_) => {
            if editor.undo() {
                editor.save_generation += 1;
                mark_tab_modified!(app, tab_id, editor.dirty);
            } else {
                app.state.status_msg = "Nothing to undo".to_string();
            }
        }
        FogDataMessage::Redo(_) => {
            if editor.redo() {
                editor.save_generation += 1;
                mark_tab_modified!(app, tab_id, true);
            } else {
                app.state.status_msg = "Nothing to redo".to_string();
            }
        }
    }

    Task::none()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn mod_recording_params(app: &App, save_path: &std::path::Path) -> Option<RecordingParams> {
    let session = app.state.recording.as_ref()?;
    let game_path = PathBuf::from(&app.state.shared_game_path);
    let relative_path = relative_to(save_path, &game_path).unwrap_or_default();
    Some(RecordingParams {
        workspace_root: session.workspace_root.clone(),
        game_path,
        mod_slug: session.mod_slug.clone(),
        relative_path,
    })
}

fn relative_to(path: &std::path::Path, base: &std::path::Path) -> Option<String> {
    path.strip_prefix(base)
        .ok()
        .map(|r| r.to_string_lossy().replace('\\', "/"))
}

/// Keep the selected level visible in the scrollable list.
fn level_list_scroll_task(level: u32) -> Task<Message> {
    use iced::widget::scrollable::RelativeOffset;
    let fraction = ((level.saturating_sub(1)) as f32 / ROWS as f32).clamp(0.0, 1.0);
    iced::widget::operation::snap_to::<Message>(
        iced::widget::Id::new(crate::editors::fog_data::view::LEVEL_LIST_ID),
        RelativeOffset {
            x: None,
            y: Some(fraction),
        },
    )
}

// ── Save logic ────────────────────────────────────────────────────────────────

/// Save outcome: `(status message, optional mod-recording failure)`.
/// `Ok` always means the file was written to disk; a recording failure is
/// reported separately so it can't mask the successful save.
type SaveOutcome = (String, Option<String>);

pub async fn save_fog_data(
    path: PathBuf,
    fog: Option<dispel_core::map::fogdata::FogData>,
    recording: Option<RecordingParams>,
) -> Result<SaveOutcome, String> {
    let fog = fog.ok_or_else(|| "No fog data loaded".to_string())?;
    tokio::task::spawn_blocking(move || {
        save_to_disk(&path, &fog)?;

        let mut recording_error = None;
        if let Some(params) = recording {
            let current_bytes =
                std::fs::read(&path).map_err(|e| format!("Failed to read saved file: {e}"))?;
            if let Err(e) = crate::editors::mod_packager::recording::record_file_replace(
                &params.workspace_root,
                &params.game_path,
                &params.mod_slug,
                &params.relative_path,
                &current_bytes,
            ) {
                recording_error = Some(e);
            }
        }

        Ok((format!("Saved → {}", path.display()), recording_error))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Synchronous disk write — the unit of work behind [`save_fog_data`],
/// exposed for tests.
pub fn save_to_disk(
    path: &std::path::Path,
    fog: &dispel_core::map::fogdata::FogData,
) -> Result<(), String> {
    fog.save_file(path).map_err(|e| e.to_string())
}

impl FogDataMessage {
    /// The owning tab id carried by every variant.
    pub fn tab_id(&self) -> usize {
        match self {
            Self::LevelSelected(id, _)
            | Self::PairSelected(id, _)
            | Self::FactorPainted(id, _, _)
            | Self::StrokeEnded(id)
            | Self::FactorCommitted(id, _, _)
            | Self::ValueInputChanged(id, _)
            | Self::ValueSubmitted(id)
            | Self::Save(id)
            | Self::SaveComplete(id, _, _)
            | Self::Revert(id)
            | Self::RevertConfirmed(id)
            | Self::RevertCancelled(id)
            | Self::Undo(id)
            | Self::Redo(id) => *id,
        }
    }
}
