// System message handlers

use crate::app::App;
use crate::components::FileTree;
use crate::file_index_cache::FileIndexCacheManager;
use crate::message::editor::map_editor::MapEditorMessage;
use crate::message::{system::SystemMessage, Message, MessageExt};
use crate::utils::browse_folder;
use crate::workspace::EditorType;
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: SystemMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        SystemMessage::CloseRequested => {
            // Handle close requested with the confirmation dialog
            use rfd::{MessageButtons, MessageDialog, MessageDialogResult};
            let dialog = MessageDialog::new()
                .set_title("Save workspace?")
                .set_description("Do you want to save your workspace before closing?")
                .set_buttons(MessageButtons::YesNoCancel);
            let result = dialog.show();
            match result {
                MessageDialogResult::Yes => {
                    app.save_workspace();
                    // Save a search index before closing
                    let index = app.search_index.clone();
                    Task::perform(
                        async move { index.save(&crate::search_index::SearchIndex::index_path()) },
                        |_| Message::System(SystemMessage::CloseApp),
                    )
                }
                MessageDialogResult::No => Task::done(Message::System(SystemMessage::CloseApp)),
                _ => Task::none(),
            }
        }
        SystemMessage::CloseApp => {
            // Close the application
            iced::window::close(app.window_id)
        }
        SystemMessage::Undo => {
            if let Some(tab) = app.state.workspace.active() {
                if tab.editor_type == EditorType::MapEditor {
                    let tab_id = tab.id;
                    return Task::done(Message::map_editor(MapEditorMessage::Undo(tab_id)));
                }
            }
            app.state.status_msg = "Nothing to undo".to_string();
            Task::none()
        }
        SystemMessage::Redo => {
            if let Some(tab) = app.state.workspace.active() {
                if tab.editor_type == EditorType::MapEditor {
                    let tab_id = tab.id;
                    return Task::done(Message::map_editor(MapEditorMessage::Redo(tab_id)));
                }
            }
            app.state.status_msg = "Nothing to redo".to_string();
            Task::none()
        }
        SystemMessage::Save => {
            if let Some(tab) = app.state.workspace.active() {
                if tab.editor_type == EditorType::MapEditor {
                    let tab_id = tab.id;
                    return Task::done(Message::map_editor(MapEditorMessage::SaveEntities(tab_id)));
                }
            }
            Task::none()
        }
        SystemMessage::FileSelected { field, path } => {
            let path = match path {
                Some(p) => p,
                None => return Task::none(),
            };
            let s = path.to_string_lossy().to_string();
            match field.as_str() {
                "start_page_path" => {
                    app.start_page_input = s.clone();
                    return Task::none();
                }
                "shared_game_path" => app.state.shared_game_path = s.clone(),
                "workspace_game_path" => {
                    // Clear all editor states to prevent stale references
                    app.state.clear_editor_states();

                    // Clear all workspace tabs
                    app.state.workspace.clear_all_tabs();

                    let pathbuf = PathBuf::from(&s);
                    app.state.workspace.game_path = Some(pathbuf.clone());
                    app.state.shared_game_path = s.clone();
                    // Pass cache manager for cache-aware search
                    let cache_mgr = app.state.file_index_cache_manager.clone();
                    app.file_tree = crate::components::file_tree::FileTree::scan_with_cache(
                        &pathbuf, &cache_mgr,
                    );
                    app.save_workspace();
                    // Clear old index and trigger re-index
                    app.search_index.clear();
                    app.search_index.game_path = Some(s.clone());
                    let gp = pathbuf.clone();
                    return Task::perform(
                        async move { crate::search_index::build_index(&gp).await },
                        |index| {
                            crate::message::Message::System(SystemMessage::IndexLoaded(Ok(index)))
                        },
                    );
                }
                "viewer_db" => app.state.viewer.db_path = s,
                "chest_game_path" => app.state.shared_game_path = s,
                "chest_map_file" => app.state.chest_editor.current_map_file = s,
                "extra_ref_map_file" => {}
                "monster_ref_file" => {}
                _ => {}
            }
            Task::none()
        }
        SystemMessage::BrowseSharedGamePath => browse_folder("workspace_game_path"),
        SystemMessage::RebuildIndex => {
            // TODO: Write information what it does
            if let Some(ref gp) = app.state.workspace.game_path {
                app.search_index.clear();
                app.search_index.game_path = Some(gp.to_string_lossy().to_string());
                app.state.status_msg = "Rebuilding search index...".to_string();

                // Update last reindexed timestamp
                app.state.workspace.last_reindexed_at =
                    Some(FileIndexCacheManager::current_timestamp());

                let gp = gp.clone();
                let gp_cache = gp.clone();

                // Rebuild search index
                let search_index_task = Task::perform(
                    async move { crate::search_index::build_index(&gp).await },
                    |index| Message::System(SystemMessage::IndexLoaded(Ok(index))),
                );

                // Also rebuild file index cache if cache manager is available
                if let Some(ref cache_manager) = app.state.file_index_cache_manager {
                    let cache_manager = cache_manager.clone();
                    return Task::batch([
                        search_index_task,
                        Task::perform(
                            async move {
                                let indexation_service =
                                    crate::indexation_service::IndexationService::new(
                                        cache_manager,
                                    );
                                indexation_service
                                    .start_indexation_with_fallback(gp_cache)
                                    .await
                            },
                            |result| match result {
                                Ok(cache) => {
                                    Message::System(SystemMessage::CacheIndexationComplete(cache))
                                }
                                Err(e) => {
                                    eprintln!("Failed to rebuild file index cache: {}", e);
                                    Message::System(SystemMessage::CacheIndexationFailed)
                                }
                            },
                        ),
                    ]);
                } else {
                    return search_index_task;
                }
            } else {
                app.state.status_msg = "No game path set".to_string();
            }
            Task::none()
        }
        SystemMessage::ClearWorkspace => {
            app.state.status_msg = "Clearing workspace tabs and editor states...".to_string();

            // Clear all editor states to prevent stale references
            app.state.clear_editor_states();

            // Clear all workspace tabs
            app.state.workspace.clear_all_tabs();

            app.state.status_msg = "Workspace cleared. All tabs and editors reset.".to_string();
            Task::none()
        }
        SystemMessage::IndexLoaded(res) => {
            match res {
                Ok(index) => {
                    app.search_index = index;
                    app.state.status_msg = "Search index loaded".to_string();
                    return Task::done(Message::System(SystemMessage::IndexSaveRequested));
                }
                Err(e) => {
                    app.state.status_msg = format!("Failed to load search index: {}", e);
                }
            }
            Task::none()
        }
        SystemMessage::IndexComplete => {
            app.search_index.indexing = false;
            app.search_index.progress = 1.0;
            app.state.status_msg =
                format!("Index complete: {} entries", app.search_index.entries.len());
            Task::none()
        }
        SystemMessage::ClearLog => {
            app.state.log.clear();
            Task::none()
        }
        SystemMessage::CacheIndexationComplete(cache) => {
            eprintln!(
                "DEBUG: CacheIndexationComplete - {} files",
                cache.files.len()
            );
            // Cache indexation completed successfully
            if let Some(ref mut cache_manager) = app.state.file_index_cache_manager {
                eprintln!("DEBUG: Saving cache to disk...");
                if let Err(e) = cache_manager.save_cache(&cache) {
                    eprintln!("Failed to save file index cache: {}", e);
                } else {
                    eprintln!("DEBUG: Cache saved successfully!");
                }
                // Update file tree with cached data
                if let Some(ref game_path) = app.state.workspace.game_path {
                    eprintln!(
                        "DEBUG: Updating file tree with {} cached files",
                        cache.files.len()
                    );
                    app.file_tree = FileTree::scan_with_cache(
                        game_path,
                        &app.state.file_index_cache_manager.clone(),
                    );
                    eprintln!("DEBUG: File tree updated");
                } else {
                    eprintln!("DEBUG: No game_path in workspace");
                }
            } else {
                eprintln!("DEBUG: No cache_manager in state");
            }
            app.is_indexing = false;
            Task::none()
        }
        SystemMessage::CacheIndexationFailed => {
            eprintln!("File index cache indexation failed");
            app.is_indexing = false;
            Task::none()
        }
        SystemMessage::IndexSaveRequested => {
            let index = app.search_index.clone();
            Task::perform(
                async move { index.save(&crate::search_index::SearchIndex::index_path()) },
                |result| match result {
                    Ok(()) => crate::message::Message::System(SystemMessage::IndexSaveComplete),
                    Err(e) => {
                        eprintln!("Failed to save search index: {}", e);
                        crate::message::Message::System(SystemMessage::IndexSaveComplete)
                    }
                },
            )
        }
        SystemMessage::ToggleAutoSave => {
            app.draft_manager.toggle_auto_save();
            let status = if app.draft_manager.is_auto_save_enabled() {
                "Auto-save drafts enabled"
            } else {
                "Auto-save drafts disabled"
            };
            app.state.status_msg = status.to_string();
            Task::none()
        }
        SystemMessage::CheckDraftConflicts => {
            let conflicts = app.draft_manager.check_conflicts();
            if conflicts.is_empty() {
                app.state.status_msg = "No conflicts detected".to_string();
            } else {
                app.state.status_msg = format!("{} file(s) have conflicts", conflicts.len());
            }
            Task::none()
        }
        SystemMessage::ApplyDraft(file_path) => {
            let path = std::path::PathBuf::from(&file_path);
            match app.draft_manager.apply_draft(&path) {
                Ok(()) => {
                    app.draft_manager.discard_draft(&path);
                    app.state.status_msg = format!("Draft applied for {}", file_path);
                }
                Err(e) => {
                    app.state.status_msg = format!("Failed to apply draft: {}", e);
                }
            }
            Task::none()
        }
        SystemMessage::DiscardDraft(file_path) => {
            let path = std::path::PathBuf::from(&file_path);
            app.draft_manager.discard_draft(&path);
            app.state.status_msg = format!("Draft discarded for {}", file_path);
            Task::none()
        }
        SystemMessage::IndexSaveComplete => Task::none(),
        SystemMessage::ShowError(msg) => {
            app.error_dialog = Some(msg);
            Task::none()
        }
        SystemMessage::DismissError => {
            app.error_dialog = None;
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::message::system::SystemMessage;

    #[test]
    fn test_clear_workspace_message_handler() {
        // Create a test app with some state
        let mut app = App::new().0;

        // Add some workspace tabs
        app.state
            .workspace
            .tabs
            .push(crate::workspace::WorkspaceTab {
                id: 1,
                label: "Test Tab".to_string(),
                path: None,
                editor_type: crate::workspace::EditorType::WeaponEditor,
                modified: false,
                pinned: false,
            });
        app.state.workspace.active_tab = Some(0);

        // Add some editor states
        app.state
            .map_editors
            .insert(1, crate::state::map_editor::MapEditorState::default());
        app.state.lookups.insert(
            "test".to_string(),
            vec![("key1".to_string(), "value1".to_string())],
        );

        // Verify initial state (after adding our test tab and editors)
        let initial_tab_count = app.state.workspace.tabs.len();
        let initial_map_editor_count = app.state.map_editors.len();
        let initial_lookup_count = app.state.lookups.len();
        assert!(initial_tab_count > 0, "Should have some tabs initially");
        assert!(
            initial_map_editor_count > 0,
            "Should have some map editors initially"
        );
        assert!(
            initial_lookup_count > 0,
            "Should have some lookups initially"
        );

        // Handle the ClearWorkspace message
        let task = handle(SystemMessage::ClearWorkspace, &mut app);

        // Task was handled successfully (no panic)
        let _ = task;

        // Verify workspace was cleared
        assert_eq!(app.state.workspace.tabs.len(), 0, "Tabs should be cleared");
        assert_eq!(
            app.state.workspace.active_tab, None,
            "Active tab should be cleared"
        );
        assert_eq!(
            app.state.map_editors.len(),
            0,
            "Map editors should be cleared"
        );
        assert_eq!(app.state.lookups.len(), 0, "Lookups should be cleared");

        // Verify status message was set
        assert!(
            app.state.status_msg.contains("Workspace cleared"),
            "Status message should indicate workspace was cleared"
        );
    }

    #[test]
    fn test_clear_workspace_with_no_tabs() {
        // Create a test app with no tabs
        let mut app = App::new().0;

        // Note: App is created with some default tabs, so we just verify the clear works
        let _initial_tab_count = app.state.workspace.tabs.len();
        let _initial_map_editor_count = app.state.map_editors.len();
        // len() always returns non-negative values, so these assertions are removed
        // as they were causing compiler warnings without providing meaningful checks

        // Handle the ClearWorkspace message (should not panic)
        let task = handle(SystemMessage::ClearWorkspace, &mut app);

        // Task was handled successfully (no panic)
        let _ = task;

        // Verify status message was set
        assert!(
            app.state.status_msg.contains("Workspace cleared"),
            "Status message should indicate workspace was cleared"
        );
    }

    #[test]
    fn test_clear_workspace_preserves_game_path() {
        // Create a test app
        let mut app = App::new().0;

        // Set a game path
        app.state.workspace.game_path = Some(std::path::PathBuf::from("/test/game"));
        app.state.shared_game_path = "/test/game".to_string();

        // Handle the ClearWorkspace message
        let task = handle(SystemMessage::ClearWorkspace, &mut app);

        // Task was handled successfully (no panic)
        let _ = task;

        // Verify game path is preserved (only tabs and editors should be cleared)
        assert_eq!(
            app.state.workspace.game_path,
            Some(std::path::PathBuf::from("/test/game")),
            "Game path should be preserved"
        );
        assert_eq!(
            app.state.shared_game_path, "/test/game",
            "Shared game path should be preserved"
        );
    }

    #[test]
    fn test_clear_workspace_is_idempotent() {
        // Create a test app
        let mut app = App::new().0;

        // First clear
        let task1 = handle(SystemMessage::ClearWorkspace, &mut app);
        // Tasks were handled successfully (no panic)
        let _ = task1;

        // Second clear should not panic and should still work
        let task2 = handle(SystemMessage::ClearWorkspace, &mut app);
        let _ = task2;

        // State should still be valid
        assert_eq!(app.state.workspace.tabs.len(), 0);
        assert_eq!(app.state.map_editors.len(), 0);
    }
}
