// System message handlers

use crate::app::App;
use crate::components::FileTree;
use crate::components::utils::browse_folder;
use crate::editor_catalog::save_task_for_editor;
use crate::editors::map_editor::MapEditorMessage;
use crate::indexation::file_index_cache::FileIndexCacheManager;
use crate::message::{Message, MessageExt, system::SystemMessage};
use crate::workspace::EditorType;
use iced::Task;
use std::path::PathBuf;

/// Prompt shown when the user requests closing. Both the workspace tab flags
/// and the dirty state kept by custom editors are reported so the user is
/// warned before discarding unsaved file edits.
fn close_dialog_text(app: &App) -> (&'static str, &'static str) {
    let tab_modified = app.state.workspace.tabs.iter().any(|tab| tab.modified);
    let has_open_tab = |id: &usize| app.state.workspace.tabs.iter().any(|t| t.id == *id);
    let has_editor = |kind: EditorType| {
        app.state
            .workspace
            .tabs
            .iter()
            .any(|t| t.editor_type == kind)
    };
    // Some editors own their dirty flag instead of setting `tab.modified`.
    // Per-tab editors are mapped back through their tab id; single-instance
    // editors are checked only while a tab of that kind is open.
    let custom_dirty = app
        .state
        .editors
        .snf_editors
        .iter()
        .any(|(id, editor)| editor.modified && has_open_tab(id))
        || app
            .state
            .editors
            .hex_editors
            .iter()
            .any(|(id, editor)| editor.provider.dirty_count() > 0 && has_open_tab(id))
        || (app.state.editors.event_scr_editor.modified && has_editor(EditorType::EventScrEditor))
        || (app
            .state
            .editors
            .save_ifo_editor
            .data
            .as_ref()
            .is_some_and(|data| data.dirty)
            && has_editor(EditorType::SaveIfoEditor));

    if tab_modified || custom_dirty {
        (
            "Unsaved files",
            "Some open files have unsaved changes. Closing will discard those file edits; Yes saves workspace state and closes, No closes without saving it.",
        )
    } else {
        (
            "Save workspace?",
            "Do you want to save your workspace before closing?",
        )
    }
}

pub fn handle(message: SystemMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        SystemMessage::CloseRequested => {
            // Handle close requested with the confirmation dialog
            use rfd::{MessageButtons, MessageDialog, MessageDialogResult};
            let (title, description) = close_dialog_text(app);
            let dialog = MessageDialog::new()
                .set_title(title)
                .set_description(description)
                .set_buttons(MessageButtons::YesNoCancel);
            let result = dialog.show();
            match result {
                MessageDialogResult::Yes => {
                    app.save_workspace();
                    // Save a search index before closing
                    let index = app.search_index.clone();
                    Task::perform(
                        async move {
                            index.save(&crate::indexation::search_index::SearchIndex::index_path())
                        },
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
            let Some((editor_type, tab_id)) =
                app.state.workspace.active().map(|t| (t.editor_type, t.id))
            else {
                app.state.status_msg = "Nothing to undo".to_string();
                return Task::none();
            };
            if editor_type == EditorType::MapEditor {
                return Task::done(Message::map_editor(MapEditorMessage::Undo(tab_id)));
            }
            let result = app.state.undo_active(editor_type, tab_id);
            app.state.status_msg = result.unwrap_or_else(|| "Nothing to undo".to_string());
            Task::none()
        }
        SystemMessage::Redo => {
            let Some((editor_type, tab_id)) =
                app.state.workspace.active().map(|t| (t.editor_type, t.id))
            else {
                app.state.status_msg = "Nothing to redo".to_string();
                return Task::none();
            };
            if editor_type == EditorType::MapEditor {
                return Task::done(Message::map_editor(MapEditorMessage::Redo(tab_id)));
            }
            let result = app.state.redo_active(editor_type, tab_id);
            app.state.status_msg = result.unwrap_or_else(|| "Nothing to redo".to_string());
            Task::none()
        }
        SystemMessage::Save => {
            if let Some(tab) = app.state.workspace.active() {
                if let Some(task) = save_task_for_editor(tab.editor_type, tab.id) {
                    return task;
                }
                app.state.status_msg = "This editor does not support saving".to_string();
            } else {
                app.state.status_msg = "No active tab to save".to_string();
            }
            Task::none()
        }
        SystemMessage::FileSelected { field, path } => {
            let path = match path {
                Some(p) if !p.as_os_str().is_empty() => p,
                _ => return Task::none(),
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
                    app.save_workspace();
                    // Clear old index and trigger re-index
                    app.search_index.clear();
                    app.search_index.game_path = Some(s.clone());

                    // Async file tree scan (off UI thread)
                    let cache_mgr = app.state.file_index_cache_manager.clone();
                    app.file_tree.set_loading(true);
                    let ft_path = pathbuf.clone();
                    let file_tree_task = Task::perform(
                        async move {
                            crate::components::file_tree::FileTree::scan_with_cache(
                                &ft_path, &cache_mgr,
                            )
                        },
                        |tree| {
                            crate::message::Message::System(SystemMessage::FileTreeScanned(tree))
                        },
                    );

                    // Async search index build
                    let gp = pathbuf.clone();
                    let index_task = Task::perform(
                        async move { crate::indexation::search_index::build_index(&gp).await },
                        |index| {
                            crate::message::Message::System(SystemMessage::IndexLoaded(Ok(index)))
                        },
                    );

                    return Task::batch([file_tree_task, index_task]);
                }
                _ => {}
            }
            Task::none()
        }
        SystemMessage::BrowseSharedGamePath => browse_folder("workspace_game_path"),
        SystemMessage::FileTreeScanned(tree) => {
            app.file_tree = tree;
            Task::none()
        }
        SystemMessage::RebuildIndex => {
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
                    async move { crate::indexation::search_index::build_index(&gp).await },
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
                                    crate::indexation::indexation_service::IndexationService::new(
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
            app.state.status_msg = format!(
                "Index complete: {} files indexed",
                app.search_index.file_mappings.len()
            );
            Task::none()
        }
        SystemMessage::CacheIndexationComplete(cache) => {
            // Cache indexation completed successfully
            if let Some(ref mut cache_manager) = app.state.file_index_cache_manager {
                if let Err(e) = cache_manager.save_cache(&cache) {
                    eprintln!("Failed to save file index cache: {}", e);
                }
                // Update file tree with cached data
                if let Some(ref game_path) = app.state.workspace.game_path {
                    app.file_tree = FileTree::scan_with_cache(
                        game_path,
                        &app.state.file_index_cache_manager.clone(),
                    );
                } else {
                    eprintln!(
                        "File index cache completed without a configured game path; file tree not refreshed"
                    );
                }
            } else {
                eprintln!(
                    "File index cache completed without a configured cache manager; file tree not refreshed"
                );
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
                async move { index.save(&crate::indexation::search_index::SearchIndex::index_path()) },
                |result| match result {
                    Ok(()) => crate::message::Message::System(SystemMessage::IndexSaveComplete),
                    Err(e) => {
                        eprintln!("Failed to save search index: {}", e);
                        crate::message::Message::System(SystemMessage::IndexSaveComplete)
                    }
                },
            )
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
            .editors
            .map_editors
            .insert(1, crate::editors::map_editor::MapEditorState::default());
        app.state.lookups.insert(
            "test".to_string(),
            vec![("key1".to_string(), "value1".to_string())],
        );

        // Verify initial state (after adding our test tab and editors)
        let initial_tab_count = app.state.workspace.tabs.len();
        let initial_map_editor_count = app.state.editors.map_editors.len();
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
            app.state.editors.map_editors.len(),
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
        let _initial_map_editor_count = app.state.editors.map_editors.len();
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
        assert_eq!(app.state.editors.map_editors.len(), 0);
    }

    fn push_weapon_tab(app: &mut App) {
        app.state.workspace.tabs.clear();
        app.state
            .workspace
            .tabs
            .push(crate::workspace::WorkspaceTab {
                id: 1,
                label: "WeaponItem".to_string(),
                path: None,
                editor_type: crate::workspace::EditorType::WeaponEditor,
                modified: false,
                pinned: false,
            });
        app.state.workspace.active_tab = Some(0);
    }

    #[test]
    fn test_undo_nothing_no_tab() {
        let mut app = App::new().0;
        app.state.workspace.tabs.clear();
        app.state.workspace.active_tab = None;
        let _ = handle(SystemMessage::Undo, &mut app);
        assert_eq!(app.state.status_msg, "Nothing to undo");
    }

    #[test]
    fn test_redo_nothing_no_tab() {
        let mut app = App::new().0;
        app.state.workspace.tabs.clear();
        app.state.workspace.active_tab = None;
        let _ = handle(SystemMessage::Redo, &mut app);
        assert_eq!(app.state.status_msg, "Nothing to redo");
    }

    #[test]
    fn test_undo_viewer_tab_returns_nothing() {
        let mut app = App::new().0;
        app.state.workspace.tabs.clear();
        app.state
            .workspace
            .tabs
            .push(crate::workspace::WorkspaceTab {
                id: 1,
                label: "Sprite".to_string(),
                path: None,
                editor_type: crate::workspace::EditorType::SpriteViewer,
                modified: false,
                pinned: false,
            });
        app.state.workspace.active_tab = Some(0);
        let _ = handle(SystemMessage::Undo, &mut app);
        assert_eq!(app.state.status_msg, "Nothing to undo");
    }

    #[test]
    fn test_undo_weapon_editor() {
        use dispel_core::WeaponItem;

        let mut app = App::new().0;
        push_weapon_tab(&mut app);

        // Load a weapon into the editor
        let weapon = WeaponItem {
            name: "Iron Sword".to_string(),
            ..Default::default()
        };
        app.state.editors.weapon_editor.catalog = Some(vec![weapon]);
        app.state.editors.weapon_editor.refresh();
        app.state.editors.weapon_editor.select(0);

        // Make a change
        app.state
            .editors
            .weapon_editor
            .update_field(0, "name", "Steel Sword".to_string());
        assert!(app.state.editors.weapon_editor.edit_history.can_undo());

        // Undo via system message
        let _ = handle(SystemMessage::Undo, &mut app);

        assert_eq!(
            app.state.editors.weapon_editor.filtered[0].1.name, "Iron Sword",
            "Field should revert after undo"
        );
        assert!(
            app.state.status_msg.starts_with("Undo:"),
            "Status should confirm undo"
        );
    }

    #[test]
    fn test_redo_weapon_editor() {
        use dispel_core::WeaponItem;

        let mut app = App::new().0;
        push_weapon_tab(&mut app);

        let weapon = WeaponItem {
            name: "Iron Sword".to_string(),
            ..Default::default()
        };
        app.state.editors.weapon_editor.catalog = Some(vec![weapon]);
        app.state.editors.weapon_editor.refresh();
        app.state.editors.weapon_editor.select(0);

        app.state
            .editors
            .weapon_editor
            .update_field(0, "name", "Steel Sword".to_string());

        let _ = handle(SystemMessage::Undo, &mut app);
        assert_eq!(
            app.state.editors.weapon_editor.filtered[0].1.name,
            "Iron Sword"
        );

        let _ = handle(SystemMessage::Redo, &mut app);
        assert_eq!(
            app.state.editors.weapon_editor.filtered[0].1.name, "Steel Sword",
            "Field should re-apply after redo"
        );
        assert!(app.state.status_msg.starts_with("Redo:"));
    }

    #[test]
    fn test_undo_weapon_editor_empty_history() {
        let mut app = App::new().0;
        push_weapon_tab(&mut app);
        let _ = handle(SystemMessage::Undo, &mut app);
        assert_eq!(app.state.status_msg, "Nothing to undo");
    }

    #[test]
    fn test_file_selected_cancelled_or_empty_path_is_noop() {
        let mut app = App::new().0;
        push_weapon_tab(&mut app);

        let game_path = std::path::PathBuf::from("/test/game");
        let game_path_str = game_path.to_string_lossy().to_string();
        app.state.workspace.game_path = Some(game_path.clone());
        app.state.shared_game_path = game_path_str.clone();
        app.search_index.game_path = Some(game_path_str.clone());
        app.state.status_msg = "unchanged".to_string();
        let tab_count = app.state.workspace.tabs.len();

        for path in [None, Some(std::path::PathBuf::new())] {
            let _ = handle(
                SystemMessage::FileSelected {
                    field: "workspace_game_path".to_string(),
                    path,
                },
                &mut app,
            );

            assert_eq!(app.state.workspace.game_path, Some(game_path.clone()));
            assert_eq!(app.state.shared_game_path, game_path_str);
            assert_eq!(app.search_index.game_path, Some(game_path_str.clone()));
            assert_eq!(app.state.workspace.tabs.len(), tab_count);
            assert_eq!(app.state.workspace.active_tab, Some(0));
            assert_eq!(app.state.status_msg, "unchanged");
        }
    }

    #[test]
    fn test_modified_tab_warns_file_edits_discarded() {
        let mut app = App::new().0;
        let (title, description) = close_dialog_text(&app);
        assert_eq!(title, "Save workspace?");
        assert!(!description.contains("unsaved"));

        app.state
            .workspace
            .tabs
            .push(crate::workspace::WorkspaceTab {
                id: 1,
                label: "Test".to_string(),
                path: None,
                editor_type: crate::workspace::EditorType::WeaponEditor,
                modified: true,
                pinned: false,
            });

        let (title, description) = close_dialog_text(&app);
        assert_eq!(title, "Unsaved files");
        assert!(
            description.contains("unsaved"),
            "close prompt must warn about unsaved files: {description}"
        );
        assert!(
            description.contains("discard those file edits"),
            "close prompt must state file edits are discarded: {description}"
        );
        assert!(
            description.contains("Yes saves workspace state and closes"),
            "close prompt must state Yes saves workspace state: {description}"
        );
        assert!(
            !description.contains("save the workspace before closing")
                && !description.contains("save your files"),
            "close prompt must not promise saving file edits: {description}"
        );
    }

    #[test]
    fn test_dirty_snf_tab_warns_without_tab_modified_flag() {
        let mut app = App::new().0;
        app.state.workspace.tabs.clear();
        app.state
            .workspace
            .open("test.snf".into(), Some(PathBuf::from("test.snf")));
        let tab_id = app.state.workspace.active_tab_id().as_usize();
        assert!(!app.state.workspace.tabs[0].modified);

        let mut state = crate::editors::snf_editor::SnfEditorState::load_from_path(
            std::path::Path::new("test.snf"),
        );
        state.modified = true;
        app.state.editors.snf_editors.insert(tab_id, state);

        let (title, description) = close_dialog_text(&app);
        assert_eq!(title, "Unsaved files");
        assert!(
            description.contains("unsaved"),
            "dirty SNF editor must trigger the file-edits warning: {description}"
        );
    }

    #[test]
    fn test_dirty_event_script_tab_warns_without_tab_modified_flag() {
        let mut app = App::new().0;
        app.state.workspace.tabs.clear();
        app.state
            .workspace
            .open("Event.scr".into(), Some(PathBuf::from("Event.scr")));
        app.state.workspace.tabs[0].editor_type = crate::workspace::EditorType::EventScrEditor;
        assert!(!app.state.workspace.tabs[0].modified);

        app.state.editors.event_scr_editor.modified = true;

        let (title, description) = close_dialog_text(&app);
        assert_eq!(title, "Unsaved files");
        assert!(
            description.contains("unsaved"),
            "dirty event-script editor must trigger the file-edits warning: {description}"
        );
    }
}
