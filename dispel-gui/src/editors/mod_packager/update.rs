use std::path::PathBuf;
use std::sync::Arc;

use dispel_core::modding::{
    ApplyReport, ChangeAction, ChangeOp, ModManifest, PatcherRegistry, RevertReport, Workspace,
};
use iced::Task;

use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::editors::mod_packager::message::{
    ApplyOutcome, LibrarySnapshot, RevertOutcome, SelectedMod,
};
use crate::editors::mod_packager::recording::{ObservedAction, DEBOUNCE};
use crate::editors::mod_packager::ModPackagerMessage;
use crate::message::{Message, MessageExt};
use crate::state::{PendingEdit, RecordingKey, RecordingSession};

const WORKSPACE_SUBDIR: &str = ".dispel-mods";

pub fn handle(message: ModPackagerMessage, app: &mut App) -> Task<Message> {
    match message {
        // ----- Tabs -------------------------------------------------------
        ModPackagerMessage::TabSelected(tab) => {
            app.state.mod_packager_editor.tab = tab;
            Task::none()
        }

        // ----- Workspace lifecycle ---------------------------------------
        ModPackagerMessage::OpenWorkspace => {
            let initial = default_workspace_root(&app.state.shared_game_path);
            Task::perform(
                async move {
                    let mut dialog = rfd::AsyncFileDialog::new();
                    if let Some(p) = initial.as_ref().and_then(|p| p.parent()) {
                        dialog = dialog.set_directory(p);
                    }
                    dialog.pick_folder().await.map(|h| h.path().to_path_buf())
                },
                |picked| Message::mod_packager(ModPackagerMessage::WorkspacePicked(picked)),
            )
        }
        ModPackagerMessage::WorkspacePicked(path) => match path {
            None => Task::none(),
            Some(root) => open_workspace(app, root),
        },
        ModPackagerMessage::Refresh => {
            let Some(root) = app.state.mod_packager_editor.workspace_root.clone() else {
                return Task::none();
            };
            refresh_library(root)
        }
        ModPackagerMessage::Refreshed(result) => {
            let state = &mut app.state.mod_packager_editor;
            match result {
                Ok(snap) => {
                    let conflict_n = snap.conflicts.len();
                    state.mods = snap.mods;
                    state.conflicts = snap.conflicts;
                    state.status_msg = if conflict_n > 0 {
                        format!(
                            "{} mod(s) installed — {} conflict(s)",
                            state.mods.len(),
                            conflict_n
                        )
                    } else {
                        format!("{} mod(s) installed", state.mods.len())
                    };
                }
                Err(e) => {
                    state.status_msg = format!("Failed to list mods: {e}");
                }
            }
            Task::none()
        }

        // ----- Mod CRUD --------------------------------------------------
        ModPackagerMessage::CreateMod => {
            let Some(root) = app.state.mod_packager_editor.workspace_root.clone() else {
                app.state.mod_packager_editor.status_msg =
                    "Open a workspace first.".into();
                return Task::none();
            };
            Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        let ws = Workspace::open(root).map_err(|e| e.to_string())?;
                        ws.create_mod(ModManifest::new("New Mod"))
                            .map_err(|e| e.to_string())
                    })
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()))
                },
                |result| Message::mod_packager(ModPackagerMessage::Created(result)),
            )
        }
        ModPackagerMessage::Created(result) => match result {
            Ok(slug) => {
                app.state.mod_packager_editor.status_msg = format!("Created `{slug}`");
                let select = Message::mod_packager(ModPackagerMessage::SelectMod(slug));
                let refresh = Message::mod_packager(ModPackagerMessage::Refresh);
                Task::done(refresh).chain(Task::done(select))
            }
            Err(e) => {
                app.state.mod_packager_editor.status_msg = format!("Create failed: {e}");
                Task::none()
            }
        },

        ModPackagerMessage::SelectMod(slug) => {
            let Some(root) = app.state.mod_packager_editor.workspace_root.clone() else {
                return Task::none();
            };
            app.state.mod_packager_editor.tab = super::state::ModManagerTab::Detail;
            load_selected(root, slug)
        }
        ModPackagerMessage::Selected(result) => {
            let state = &mut app.state.mod_packager_editor;
            match result {
                Ok(SelectedMod {
                    slug,
                    manifest,
                    changes,
                }) => {
                    state.edit_name = manifest.name.clone();
                    state.edit_version = manifest.version.clone();
                    state.edit_author = manifest.author.clone();
                    state.edit_description = manifest.description.clone();
                    state.edit_dirty = false;
                    state.selected_changes = (*changes).clone();
                    state.selected_manifest = Some(manifest);
                    state.selected_slug = Some(slug);
                    state.status_msg.clear();
                }
                Err(e) => {
                    state.status_msg = format!("Failed to load mod: {e}");
                }
            }
            Task::none()
        }

        ModPackagerMessage::DeleteMod(slug) => {
            let Some(root) = app.state.mod_packager_editor.workspace_root.clone() else {
                return Task::none();
            };
            Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        let ws = Workspace::open(root).map_err(|e| e.to_string())?;
                        ws.delete_mod(&slug).map_err(|e| e.to_string())
                    })
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()))
                },
                |result| Message::mod_packager(ModPackagerMessage::Deleted(result)),
            )
        }
        ModPackagerMessage::Deleted(result) => {
            let state = &mut app.state.mod_packager_editor;
            match result {
                Ok(()) => {
                    state.status_msg = "Mod deleted".into();
                    state.selected_slug = None;
                    state.selected_manifest = None;
                    state.selected_changes.clear();
                    state.edit_dirty = false;
                    Task::done(Message::mod_packager(ModPackagerMessage::Refresh))
                }
                Err(e) => {
                    state.status_msg = format!("Delete failed: {e}");
                    Task::none()
                }
            }
        }

        // ----- Manifest editor -------------------------------------------
        ModPackagerMessage::NameChanged(v) => {
            app.state.mod_packager_editor.edit_name = v;
            app.state.mod_packager_editor.edit_dirty = true;
            Task::none()
        }
        ModPackagerMessage::VersionChanged(v) => {
            app.state.mod_packager_editor.edit_version = v;
            app.state.mod_packager_editor.edit_dirty = true;
            Task::none()
        }
        ModPackagerMessage::AuthorChanged(v) => {
            app.state.mod_packager_editor.edit_author = v;
            app.state.mod_packager_editor.edit_dirty = true;
            Task::none()
        }
        ModPackagerMessage::DescriptionChanged(v) => {
            app.state.mod_packager_editor.edit_description = v;
            app.state.mod_packager_editor.edit_dirty = true;
            Task::none()
        }
        ModPackagerMessage::SaveManifest => save_manifest(app),
        ModPackagerMessage::Saved(result) => {
            let state = &mut app.state.mod_packager_editor;
            match result {
                Ok(()) => {
                    state.edit_dirty = false;
                    state.status_msg = "Manifest saved".into();
                    Task::done(Message::mod_packager(ModPackagerMessage::Refresh))
                }
                Err(e) => {
                    state.status_msg = format!("Save failed: {e}");
                    Task::none()
                }
            }
        }

        // ----- Library actions -------------------------------------------
        ModPackagerMessage::ToggleEnabled(slug) => {
            let Some(root) = app.state.mod_packager_editor.workspace_root.clone() else {
                return Task::none();
            };
            let currently_enabled = app
                .state
                .mod_packager_editor
                .mods
                .iter()
                .find(|m| m.slug == slug)
                .map(|m| m.enabled)
                .unwrap_or(false);
            workspace_action(root, move |ws| {
                ws.set_enabled(&slug, !currently_enabled)
                    .map_err(|e| e.to_string())
            })
        }
        ModPackagerMessage::MoveUp(slug) => {
            let Some(root) = app.state.mod_packager_editor.workspace_root.clone() else {
                return Task::none();
            };
            workspace_action(root, move |ws| {
                ws.move_up(&slug).map_err(|e| e.to_string())
            })
        }
        ModPackagerMessage::MoveDown(slug) => {
            let Some(root) = app.state.mod_packager_editor.workspace_root.clone() else {
                return Task::none();
            };
            workspace_action(root, move |ws| {
                ws.move_down(&slug).map_err(|e| e.to_string())
            })
        }

        // ----- Apply / revert --------------------------------------------
        ModPackagerMessage::Apply => {
            let Some(root) = app.state.mod_packager_editor.workspace_root.clone() else {
                return Task::none();
            };
            let Some(game_dir) = nonempty_path(&app.state.shared_game_path) else {
                app.state.mod_packager_editor.status_msg =
                    "Set the game path before applying.".into();
                return Task::none();
            };
            app.state.mod_packager_editor.loading_state = LoadingState::Loading;
            app.state.mod_packager_editor.status_msg = "Applying mods…".into();
            Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        let ws = Workspace::open(root).map_err(|e| e.to_string())?;
                        ws.apply(&game_dir, &PatcherRegistry::with_defaults())
                            .map(|r: ApplyReport| ApplyOutcome {
                                actions_applied: r.actions_applied,
                                written: r.written.len(),
                                deleted: r.deleted.len(),
                            })
                            .map_err(|e| e.to_string())
                    })
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()))
                },
                |result| Message::mod_packager(ModPackagerMessage::Applied(result)),
            )
        }
        ModPackagerMessage::Applied(result) => {
            let state = &mut app.state.mod_packager_editor;
            match result {
                Ok(o) => {
                    state.status_msg = format!(
                        "Applied {} action(s); {} file(s) written, {} deleted",
                        o.actions_applied, o.written, o.deleted
                    );
                    state.loading_state = LoadingState::Loaded(());
                }
                Err(e) => {
                    state.status_msg = format!("Apply failed: {e}");
                    state.loading_state = LoadingState::Failed(e);
                }
            }
            Task::none()
        }

        ModPackagerMessage::Revert => {
            let Some(root) = app.state.mod_packager_editor.workspace_root.clone() else {
                return Task::none();
            };
            let Some(game_dir) = nonempty_path(&app.state.shared_game_path) else {
                app.state.mod_packager_editor.status_msg =
                    "Set the game path before reverting.".into();
                return Task::none();
            };
            app.state.mod_packager_editor.loading_state = LoadingState::Loading;
            app.state.mod_packager_editor.status_msg = "Reverting…".into();
            Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        let ws = Workspace::open(root).map_err(|e| e.to_string())?;
                        ws.revert(&game_dir)
                            .map(|r: RevertReport| RevertOutcome {
                                restored: r.restored.len(),
                            })
                            .map_err(|e| e.to_string())
                    })
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()))
                },
                |result| Message::mod_packager(ModPackagerMessage::Reverted(result)),
            )
        }
        ModPackagerMessage::Reverted(result) => {
            let state = &mut app.state.mod_packager_editor;
            match result {
                Ok(o) => {
                    state.status_msg = format!("Reverted {} file(s) to vanilla", o.restored);
                    state.loading_state = LoadingState::Loaded(());
                }
                Err(e) => {
                    state.status_msg = format!("Revert failed: {e}");
                    state.loading_state = LoadingState::Failed(e);
                }
            }
            Task::none()
        }

        // ----- Recording --------------------------------------------------
        ModPackagerMessage::StartRecording(slug) => {
            let Some(root) = app.state.mod_packager_editor.workspace_root.clone() else {
                app.state.mod_packager_editor.status_msg =
                    "Open a workspace first.".into();
                return Task::none();
            };
            let mod_name = app
                .state
                .mod_packager_editor
                .mods
                .iter()
                .find(|m| m.slug == slug)
                .map(|m| m.manifest.name.clone())
                .unwrap_or_else(|| slug.clone());
            // Flush any in-flight pending edits from a previous session
            // before swapping over.
            let flush = flush_all_pending(app);
            app.state.recording = Some(RecordingSession {
                workspace_root: root,
                mod_slug: slug.clone(),
                mod_name: mod_name.clone(),
                recorded_count: 0,
                pending: std::collections::HashMap::new(),
                next_generation: 0,
            });
            app.state.mod_packager_editor.status_msg =
                format!("Recording into `{mod_name}` — edits in any catalog editor are captured.");
            flush
        }
        ModPackagerMessage::StopRecording => {
            let flush = flush_all_pending(app);
            let stopped_name = app
                .state
                .recording
                .as_ref()
                .map(|s| s.mod_name.clone())
                .unwrap_or_default();
            let committed = app
                .state
                .recording
                .as_ref()
                .map(|s| s.recorded_count)
                .unwrap_or(0);
            let pending = app
                .state
                .recording
                .as_ref()
                .map(|s| s.pending.len())
                .unwrap_or(0);
            app.state.recording = None;
            if !stopped_name.is_empty() {
                let total = committed + pending;
                app.state.mod_packager_editor.status_msg = format!(
                    "Stopped recording `{stopped_name}` — {total} change(s) captured."
                );
            }
            flush.chain(Task::done(Message::mod_packager(
                ModPackagerMessage::Refresh,
            )))
        }
        ModPackagerMessage::RecordingObserved(observed) => observe_into_pending(app, observed),
        ModPackagerMessage::RecordingDebounceFired { key, generation } => {
            flush_one_pending(app, &key, Some(generation))
        }
        ModPackagerMessage::RecordingPersisted(result) => {
            match result {
                Ok(()) => {
                    if let Some(session) = app.state.recording.as_mut() {
                        session.recorded_count += 1;
                    }
                }
                Err(e) => {
                    app.state.mod_packager_editor.status_msg =
                        format!("Recording persist failed: {e}");
                }
            }
            Task::none()
        }

        // ----- Import / export -------------------------------------------
        ModPackagerMessage::ImportZip => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("Mod package", &["zip"])
                    .pick_file()
                    .await
                    .map(|h| h.path().to_path_buf())
            },
            |picked| Message::mod_packager(ModPackagerMessage::ImportPicked(picked)),
        ),
        ModPackagerMessage::ImportPicked(path) => {
            let Some(zip_path) = path else {
                return Task::none();
            };
            let Some(root) = app.state.mod_packager_editor.workspace_root.clone() else {
                app.state.mod_packager_editor.status_msg =
                    "Open a workspace before importing.".into();
                return Task::none();
            };
            Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        let ws = Workspace::open(root).map_err(|e| e.to_string())?;
                        ws.import_zip(&zip_path).map_err(|e| e.to_string())
                    })
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()))
                },
                |result| Message::mod_packager(ModPackagerMessage::Imported(result)),
            )
        }
        ModPackagerMessage::Imported(result) => {
            let state = &mut app.state.mod_packager_editor;
            match result {
                Ok(slug) => {
                    state.status_msg = format!("Imported `{slug}`");
                    Task::done(Message::mod_packager(ModPackagerMessage::Refresh))
                }
                Err(e) => {
                    state.status_msg = format!("Import failed: {e}");
                    Task::none()
                }
            }
        }

        ModPackagerMessage::ExportZip(slug) => {
            let default_name = format!("{slug}.zip");
            Task::perform(
                async move {
                    let picked = rfd::AsyncFileDialog::new()
                        .set_file_name(&default_name)
                        .add_filter("Mod package", &["zip"])
                        .save_file()
                        .await
                        .map(|h| h.path().to_path_buf());
                    (slug, picked)
                },
                |(slug, picked)| {
                    Message::mod_packager(ModPackagerMessage::ExportPicked(slug, picked))
                },
            )
        }
        ModPackagerMessage::ExportPicked(slug, path) => {
            let Some(dst) = path else {
                return Task::none();
            };
            let Some(root) = app.state.mod_packager_editor.workspace_root.clone() else {
                return Task::none();
            };
            Task::perform(
                async move {
                    let dst_clone = dst.clone();
                    tokio::task::spawn_blocking(move || {
                        let ws = Workspace::open(root).map_err(|e| e.to_string())?;
                        ws.export_zip(&slug, &dst_clone)
                            .map(|()| dst_clone)
                            .map_err(|e| e.to_string())
                    })
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()))
                },
                |result| Message::mod_packager(ModPackagerMessage::Exported(result)),
            )
        }
        ModPackagerMessage::Exported(result) => {
            let state = &mut app.state.mod_packager_editor;
            match result {
                Ok(path) => {
                    state.status_msg = format!("Exported to {}", path.display());
                }
                Err(e) => {
                    state.status_msg = format!("Export failed: {e}");
                }
            }
            Task::none()
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn open_workspace(app: &mut App, root: PathBuf) -> Task<Message> {
    // Workspace::open just creates a couple of directories; safe to run inline.
    match Workspace::open(root.clone()) {
        Ok(_) => {
            app.state.mod_packager_editor.workspace_root = Some(root.clone());
            app.state.mod_packager_editor.status_msg =
                format!("Workspace: {}", root.display());
            refresh_library(root)
        }
        Err(e) => {
            app.state.mod_packager_editor.status_msg = format!("Open failed: {e}");
            Task::none()
        }
    }
}

fn refresh_library(root: PathBuf) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || -> Result<LibrarySnapshot, String> {
                let ws = Workspace::open(root).map_err(|e| e.to_string())?;
                let mods = ws.list_mods().map_err(|e| e.to_string())?;
                let conflicts = ws.detect_conflicts().map_err(|e| e.to_string())?;
                Ok(LibrarySnapshot { mods, conflicts })
            })
            .await
            .unwrap_or_else(|e| Err(e.to_string()))
        },
        |result| Message::mod_packager(ModPackagerMessage::Refreshed(result)),
    )
}

fn workspace_action<F>(root: PathBuf, action: F) -> Task<Message>
where
    F: FnOnce(&Workspace) -> Result<(), String> + Send + 'static,
{
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || -> Result<(), String> {
                let ws = Workspace::open(root).map_err(|e| e.to_string())?;
                action(&ws)
            })
            .await
            .unwrap_or_else(|e| Err(e.to_string()))
        },
        |result| match result {
            Ok(()) => Message::mod_packager(ModPackagerMessage::Refresh),
            Err(e) => Message::mod_packager(ModPackagerMessage::Refreshed(Err(e))),
        },
    )
}

fn load_selected(root: PathBuf, slug: String) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || -> Result<SelectedMod, String> {
                let ws = Workspace::open(root).map_err(|e| e.to_string())?;
                let pkg = ws.read_mod(&slug).map_err(|e| e.to_string())?;
                Ok(SelectedMod {
                    slug,
                    manifest: pkg.manifest,
                    changes: Arc::new(pkg.changes.into_actions()),
                })
            })
            .await
            .unwrap_or_else(|e| Err(e.to_string()))
        },
        |result| Message::mod_packager(ModPackagerMessage::Selected(result)),
    )
}

fn save_manifest(app: &mut App) -> Task<Message> {
    let state = &mut app.state.mod_packager_editor;
    let Some(root) = state.workspace_root.clone() else {
        state.status_msg = "Open a workspace first.".into();
        return Task::none();
    };
    let Some(slug) = state.selected_slug.clone() else {
        return Task::none();
    };
    let Some(mut manifest) = state.selected_manifest.clone() else {
        return Task::none();
    };
    manifest.name = state.edit_name.clone();
    manifest.version = state.edit_version.clone();
    manifest.author = state.edit_author.clone();
    manifest.description = state.edit_description.clone();
    state.selected_manifest = Some(manifest.clone());

    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || -> Result<(), String> {
                let ws = Workspace::open(root).map_err(|e| e.to_string())?;
                let mut pkg = ws.read_mod(&slug).map_err(|e| e.to_string())?;
                pkg.manifest = manifest;
                ws.write_mod(&slug, &pkg).map_err(|e| e.to_string())
            })
            .await
            .unwrap_or_else(|e| Err(e.to_string()))
        },
        |result| Message::mod_packager(ModPackagerMessage::Saved(result)),
    )
}

/// Add or coalesce one observation into the recording session's pending
/// buffer, then schedule a delayed flush.
fn observe_into_pending(app: &mut App, observed: ObservedAction) -> Task<Message> {
    let Some(session) = app.state.recording.as_mut() else {
        return Task::none();
    };
    let ObservedAction { key, old, new } = observed;

    // Bump generation FIRST so any in-flight timer for this key becomes
    // stale and drops its flush attempt.
    session.next_generation += 1;
    let generation = session.next_generation;

    match session.pending.get_mut(&key) {
        Some(pending) => {
            // Preserve original_old; just update the latest_new + generation.
            pending.latest_new = new;
            pending.generation = generation;
        }
        None => {
            session.pending.insert(
                key.clone(),
                PendingEdit {
                    original_old: old,
                    latest_new: new,
                    generation,
                },
            );
        }
    }

    // Schedule a delayed flush.
    Task::perform(
        async move {
            tokio::time::sleep(DEBOUNCE).await;
            (key, generation)
        },
        |(key, generation)| {
            Message::mod_packager(ModPackagerMessage::RecordingDebounceFired { key, generation })
        },
    )
}

/// Flush one pending entry to disk, but only if its generation still matches
/// (i.e. no later edit has superseded it). When `expected_generation` is
/// `None`, flush regardless of generation — used by the bulk flush path.
fn flush_one_pending(
    app: &mut App,
    key: &RecordingKey,
    expected_generation: Option<u64>,
) -> Task<Message> {
    let Some(session) = app.state.recording.as_mut() else {
        return Task::none();
    };
    let workspace_root = session.workspace_root.clone();
    let mod_slug = session.mod_slug.clone();

    let pending = match session.pending.get(key) {
        Some(p) => p,
        None => return Task::none(),
    };
    if let Some(g) = expected_generation {
        if pending.generation != g {
            // A later keystroke superseded this timer; drop silently.
            return Task::none();
        }
    }

    // Skip no-op edits (user typed something then changed back to the original).
    if pending.original_old == pending.latest_new {
        session.pending.remove(key);
        return Task::none();
    }

    let action = ChangeAction::new(
        key.file_path.clone(),
        ChangeOp::FieldDelta {
            record_id: key.record_id,
            field: key.field.clone(),
            old: pending.original_old.clone(),
            new: pending.latest_new.clone(),
        },
    );
    session.pending.remove(key);

    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || -> Result<(), String> {
                let ws = Workspace::open(workspace_root).map_err(|e| e.to_string())?;
                ws.append_action(&mod_slug, action).map_err(|e| e.to_string())
            })
            .await
            .unwrap_or_else(|e| Err(e.to_string()))
        },
        |result| Message::mod_packager(ModPackagerMessage::RecordingPersisted(result)),
    )
}

/// Flush every pending entry immediately, in one workspace transaction, and
/// collapse repeated FieldDelta entries against the same field. Used when
/// the recording session ends or switches mods.
fn flush_all_pending(app: &mut App) -> Task<Message> {
    let Some(session) = app.state.recording.as_mut() else {
        return Task::none();
    };
    if session.pending.is_empty() {
        return Task::none();
    }

    let workspace_root = session.workspace_root.clone();
    let mod_slug = session.mod_slug.clone();

    // Drain the pending buffer into ChangeActions, skipping no-ops.
    let mut actions: Vec<ChangeAction> = Vec::with_capacity(session.pending.len());
    for (key, pending) in session.pending.drain() {
        if pending.original_old == pending.latest_new {
            continue;
        }
        actions.push(ChangeAction::new(
            key.file_path,
            ChangeOp::FieldDelta {
                record_id: key.record_id,
                field: key.field,
                old: pending.original_old,
                new: pending.latest_new,
            },
        ));
    }
    if actions.is_empty() {
        return Task::none();
    }

    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || -> Result<(), String> {
                let ws = Workspace::open(workspace_root).map_err(|e| e.to_string())?;
                ws.append_actions_and_flatten(&mod_slug, actions)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            })
            .await
            .unwrap_or_else(|e| Err(e.to_string()))
        },
        |result| Message::mod_packager(ModPackagerMessage::RecordingPersisted(result)),
    )
}

fn nonempty_path(s: &str) -> Option<PathBuf> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

fn default_workspace_root(game_path: &str) -> Option<PathBuf> {
    nonempty_path(game_path).map(|p| p.join(WORKSPACE_SUBDIR))
}

