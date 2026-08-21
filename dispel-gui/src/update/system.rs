// System message handlers

use crate::app::App;
use crate::components::FileTree;
use crate::components::utils::browse_folder;
use crate::editors::all_map_ini::AllMapIniEditorMessage;
use crate::editors::chdata::ChDataEditorMessage;
use crate::editors::draw_item::DrawItemEditorMessage;
use crate::editors::edit_item::EditItemEditorMessage;
use crate::editors::event_ini::EventIniEditorMessage;
use crate::editors::event_item::EventItemEditorMessage;
use crate::editors::event_npc_ref::EventNpcRefEditorMessage;
use crate::editors::event_scr::EventScrEditorMessage;
use crate::editors::extra_ini::ExtraIniEditorMessage;
use crate::editors::heal_item::HealItemEditorMessage;
use crate::editors::magic::MagicEditorMessage;
use crate::editors::map_editor::MapEditorMessage;
use crate::editors::map_ini::MapIniEditorMessage;
use crate::editors::message_scr::MessageScrEditorMessage;
use crate::editors::misc_item::MiscItemEditorMessage;
use crate::editors::monster::MonsterEditorMessage;
use crate::editors::monster_ini::MonsterIniEditorMessage;
use crate::editors::npc_ini::NpcIniEditorMessage;
use crate::editors::party_ini::PartyIniEditorMessage;
use crate::editors::party_level_db::PartyLevelDbEditorMessage;
use crate::editors::party_ref::PartyRefEditorMessage;
use crate::editors::quest_scr::QuestScrEditorMessage;
use crate::editors::store::StoreEditorMessage;
use crate::editors::wave_ini::WaveIniEditorMessage;
use crate::editors::weapon::WeaponEditorMessage;
use crate::indexation::file_index_cache::FileIndexCacheManager;
use crate::message::{Message, MessageExt, system::SystemMessage};
use crate::workspace::EditorType;
use iced::Task;
use std::path::PathBuf;

/// Returns a `Task` that dispatches `Save` to the given editor type, or
/// `None` if the editor type does not support saving via Ctrl+S.
fn save_task_for_editor(
    editor_type: EditorType,
    tab_id: usize,
) -> Option<Task<crate::message::Message>> {
    if !editor_type.supports_save() {
        return None;
    }
    Some(Task::done(match editor_type {
        // Standard editors (define_standard_editor! macro)
        EditorType::WeaponEditor => Message::weapon(WeaponEditorMessage::Save),
        EditorType::MonsterEditor => Message::monster(MonsterEditorMessage::Save),
        EditorType::MonsterIniEditor => Message::monster_ini(MonsterIniEditorMessage::Save),
        EditorType::HealItemEditor => Message::heal_item(HealItemEditorMessage::Save),
        EditorType::MiscItemEditor => Message::misc_item(MiscItemEditorMessage::Save),
        EditorType::EditItemEditor => Message::edit_item(EditItemEditorMessage::Save),
        EditorType::EventItemEditor => Message::event_item(EventItemEditorMessage::Save),
        EditorType::NpcIniEditor => Message::npc_ini(NpcIniEditorMessage::Save),
        EditorType::MagicEditor => Message::magic(MagicEditorMessage::Save),
        EditorType::PartyRefEditor => Message::party_ref(PartyRefEditorMessage::Save),
        EditorType::PartyIniEditor => Message::party_ini(PartyIniEditorMessage::Save),
        EditorType::AllMapIniEditor => Message::all_map_ini(AllMapIniEditorMessage::Save),
        EditorType::DrawItemEditor => Message::draw_item(DrawItemEditorMessage::Save),
        EditorType::EventIniEditor => Message::event_ini(EventIniEditorMessage::Save),
        EditorType::EventNpcRefEditor => Message::event_npc_ref(EventNpcRefEditorMessage::Save),
        EditorType::ExtraIniEditor => Message::extra_ini(ExtraIniEditorMessage::Save),
        EditorType::MapIniEditor => Message::map_ini(MapIniEditorMessage::Save),
        EditorType::MessageScrEditor => Message::message_scr(MessageScrEditorMessage::Save),
        EditorType::QuestScrEditor => Message::quest_scr(QuestScrEditorMessage::Save),
        EditorType::WaveIniEditor => Message::wave_ini(WaveIniEditorMessage::Save),
        EditorType::ChDataEditor => Message::chdata(ChDataEditorMessage::Save),

        // Custom editors with Save support
        EditorType::StoreEditor => Message::store(StoreEditorMessage::Save),
        EditorType::PartyLevelDbEditor => Message::party_level_db(PartyLevelDbEditorMessage::Save),
        EditorType::SaveIfoEditor => {
            Message::save_ifo(crate::editors::save_ifo::SaveIfoEditorMessage::Save)
        }

        // Sprite editor
        EditorType::SpriteViewer => {
            Message::sprite_viewer(crate::editors::sprite_editor::SpriteViewerMessage::Save)
        }

        // Map editor uses SaveEntities with tab_id
        EditorType::MapEditor => Message::map_editor(MapEditorMessage::SaveEntities(tab_id)),

        // EventScr script editor
        EditorType::EventScrEditor => Message::event_scr(EventScrEditorMessage::SaveScript),

        // Tabbed editors — they define a Save variant via define_tab_editor!
        EditorType::DialogueScriptEditor => Message::dialogue_script(
            crate::editors::dialogue_script::DialogueScriptEditorMessage::Save,
        ),
        EditorType::DialogueTextEditor => Message::dialogue_paragraph(
            crate::editors::dialogue_paragraph::DialogueParagraphEditorMessage::Save,
        ),
        EditorType::ExtraRefEditor => {
            Message::extra_ref(crate::editors::extra_ref::ExtraRefEditorMessage::Save)
        }
        EditorType::MonsterRefEditor => {
            Message::monster_ref(crate::editors::monster_ref::MonsterRefEditorMessage::Save)
        }
        EditorType::NpcRefEditor => {
            Message::npc_ref(crate::editors::npc_ref::NpcRefEditorMessage::Save)
        }

        // Safety net: supports_save() returned true but we're missing an arm.
        // This is a programming error — the editor should either be listed above
        // or removed from supports_save().
        _ => unreachable!(
            "save_task_for_editor: {:?} claims supports_save() but has no dispatch arm",
            editor_type
        ),
    }))
}

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
            if result.is_some() {
                app.state
                    .refresh_spreadsheet_after_undo_redo(editor_type, tab_id);
            }
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
            if result.is_some() {
                app.state
                    .refresh_spreadsheet_after_undo_redo(editor_type, tab_id);
            }
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
                "viewer_db" => app.state.editors.viewer.db_path = s,
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
}
