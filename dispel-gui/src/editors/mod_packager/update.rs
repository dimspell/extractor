use std::path::PathBuf;
use std::sync::Arc;

use dispel_core::modding::{
    ApplyReport, InstalledMod, ModManifest, PatcherRegistry, RevertReport, Workspace,
};
use iced::Task;

use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::editors::mod_packager::message::{ApplyOutcome, RevertOutcome, SelectedMod};
use crate::editors::mod_packager::ModPackagerMessage;
use crate::message::{Message, MessageExt};

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
                Ok(mods) => {
                    state.mods = mods;
                    state.status_msg = format!("{} mod(s) installed", state.mods.len());
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
            tokio::task::spawn_blocking(move || -> Result<Vec<InstalledMod>, String> {
                let ws = Workspace::open(root).map_err(|e| e.to_string())?;
                ws.list_mods().map_err(|e| e.to_string())
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

