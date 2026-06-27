//! Integration tests for system-message handlers.
//!
//! Coverage gap identified in the staging-branch review: 10 of ~20 system
//! messages had zero test coverage.  This file adds tests for the most
//! impactful missing handlers.
//!
//! Tests send messages through `app.update(Message::System(...))` rather
//! than calling system::handle directly (which is private).

use crate::app::App;
use crate::message::system::SystemMessage;
use crate::message::Message;
use crate::workspace::{EditorType, Workspace, WorkspaceTab};

// ============================================================================
// Undo / Redo / Save edge cases
// ============================================================================

mod undo_redo_save_edges {
    use super::*;

    #[test]
    fn undo_on_tabbed_editor_returns_nothing() {
        let mut app = app_with_tab(EditorType::ExtraRefEditor);
        let task = app.update(Message::System(SystemMessage::Undo));
        assert_eq!(app.state.status_msg, "Nothing to undo");
        let _ = task;
    }

    #[test]
    fn undo_on_store_editor_empty_history_returns_nothing() {
        let mut app = app_with_tab(EditorType::StoreEditor);
        let task = app.update(Message::System(SystemMessage::Undo));
        assert_eq!(app.state.status_msg, "Nothing to undo");
        let _ = task;
    }

    #[test]
    fn redo_on_tabbed_editor_returns_nothing() {
        let mut app = app_with_tab(EditorType::DialogueScriptEditor);
        let task = app.update(Message::System(SystemMessage::Redo));
        assert_eq!(app.state.status_msg, "Nothing to redo");
        let _ = task;
    }

    #[test]
    fn save_on_sprite_viewer_produces_task() {
        let mut app = app_with_tab(EditorType::SpriteViewer);
        let task = app.update(Message::System(SystemMessage::Save));
        assert!(task.units() > 0, "SpriteViewer Save produces task");
    }

    #[test]
    fn save_event_scr_returns_task() {
        let mut app = app_with_tab(EditorType::EventScrEditor);
        let task = app.update(Message::System(SystemMessage::Save));
        assert!(task.units() > 0, "EventScrEditor Save produces task");
    }

    #[test]
    fn save_map_editor_returns_task() {
        let mut app = app_with_tab(EditorType::MapEditor);
        let task = app.update(Message::System(SystemMessage::Save));
        assert!(task.units() > 0, "MapEditor Save produces task");
    }

    #[test]
    fn save_weapon_editor_returns_task() {
        let mut app = app_with_tab(EditorType::WeaponEditor);
        let task = app.update(Message::System(SystemMessage::Save));
        assert!(task.units() > 0, "WeaponEditor Save produces task");
    }
}

// ============================================================================
// Error dialog
// ============================================================================

mod error_dialog {
    use super::*;

    #[test]
    fn show_error_sets_dialog() {
        let mut app = App::test_new(Workspace::new());
        let task = app.update(Message::System(SystemMessage::ShowError(
            "Something broke".to_string(),
        )));
        assert_eq!(app.error_dialog.as_deref(), Some("Something broke"));
        let _ = task;
    }

    #[test]
    fn dismiss_error_clears_dialog() {
        let mut app = App::test_new(Workspace::new());
        app.error_dialog = Some("Existing".to_string());
        let task = app.update(Message::System(SystemMessage::DismissError));
        assert!(app.error_dialog.is_none());
        let _ = task;
    }

    #[test]
    fn dismiss_error_without_prior_does_not_panic() {
        let mut app = App::test_new(Workspace::new());
        let task = app.update(Message::System(SystemMessage::DismissError));
        assert!(app.error_dialog.is_none());
        let _ = task;
    }
}

// ============================================================================
// Index messages
// ============================================================================

mod index_messages {
    use super::*;
    use crate::indexation::search_index::SearchIndex;

    #[test]
    fn index_loaded_sets_index_and_triggers_save_request() {
        let mut app = App::test_new(Workspace::new());
        assert!(app.search_index.file_mappings.is_empty());

        let mut idx = SearchIndex::new();
        idx.game_path = Some("/fake/path".to_string());
        let task = app.update(Message::System(SystemMessage::IndexLoaded(Ok(idx))));

        assert!(task.units() > 0, "IndexLoaded triggers IndexSaveRequested");
        assert_eq!(app.state.status_msg, "Search index loaded");
    }

    #[test]
    fn index_loaded_error_sets_error_status() {
        let mut app = App::test_new(Workspace::new());
        let task = app.update(Message::System(SystemMessage::IndexLoaded(Err(
            "corrupt".to_string()
        ))));
        assert!(app.state.status_msg.contains("Failed to load"));
        let _ = task;
    }

    #[test]
    fn index_complete_updates_progress_and_flag() {
        let mut app = App::test_new(Workspace::new());
        app.search_index
            .file_mappings
            .push(crate::indexation::search_index::FileMapping {
                file_path: "test.file".to_string(),
                editor_type: "WeaponEditor".to_string(),
            });

        let task = app.update(Message::System(SystemMessage::IndexComplete));
        assert!(!app.search_index.indexing, "indexing flag cleared");
        assert_eq!(app.search_index.progress, 1.0, "progress = 1.0");
        assert!(app.state.status_msg.contains("Index complete"));
        let _ = task;
    }
}

// ============================================================================
// Draft manager messages
// ============================================================================

mod draft_messages {
    use super::*;

    #[test]
    fn toggle_auto_save_switches_state() {
        let mut app = App::test_new(Workspace::new());
        let was = app.draft_manager.is_auto_save_enabled();

        let task = app.update(Message::System(SystemMessage::ToggleAutoSave));
        assert_ne!(app.draft_manager.is_auto_save_enabled(), was, "toggled off");
        let _ = task;

        let task = app.update(Message::System(SystemMessage::ToggleAutoSave));
        assert_eq!(
            app.draft_manager.is_auto_save_enabled(),
            was,
            "toggled back"
        );
        let _ = task;
    }

    #[test]
    fn check_draft_conflicts_no_conflicts() {
        let mut app = App::test_new(Workspace::new());
        let task = app.update(Message::System(SystemMessage::CheckDraftConflicts));
        assert_eq!(app.state.status_msg, "No conflicts detected");
        let _ = task;
    }

    #[test]
    fn discard_draft_sets_status() {
        let mut app = App::test_new(Workspace::new());
        let task = app.update(Message::System(SystemMessage::DiscardDraft(
            "test.file".to_string(),
        )));
        assert!(app.state.status_msg.contains("Draft discarded"));
        let _ = task;
    }
}

// ============================================================================
// ClearWorkspace edge cases
// ============================================================================

mod clear_workspace_edges {
    use super::*;

    #[test]
    fn clear_workspace_clears_tabbed_editors() {
        let mut app = App::test_new(Workspace::new());

        app.state.workspace.tabs.push(WorkspaceTab {
            id: 1,
            label: "MonsterRef".to_string(),
            path: None,
            editor_type: EditorType::MonsterRefEditor,
            modified: false,
            pinned: false,
        });
        app.state.workspace.active_tab = Some(0);
        app.state.editors.monster_ref_editor.editors.insert(
            1,
            crate::components::generic_editor::MultiFileEditorState::default(),
        );

        let task = app.update(Message::System(SystemMessage::ClearWorkspace));
        let _ = task;

        assert_eq!(app.state.editors.monster_ref_editor.editors.len(), 0);
        assert_eq!(app.state.workspace.tabs.len(), 0);
    }

    #[test]
    fn clear_workspace_with_no_tabs_clears_status() {
        let mut app = App::test_new(Workspace::new());
        app.state.workspace.tabs.clear();
        app.state.workspace.active_tab = None;

        let task = app.update(Message::System(SystemMessage::ClearWorkspace));
        let _ = task;

        assert!(app.state.status_msg.contains("Workspace cleared"));
    }
}

// ============================================================================
// Cache indexation messages
// ============================================================================

mod cache_indexation {
    use super::*;
    use crate::indexation::file_index_cache::FileIndexCache;

    #[test]
    fn cache_indexation_complete_without_manager_does_not_panic() {
        let mut app = App::test_new(Workspace::new());
        app.state.file_index_cache_manager = None;

        let cache = FileIndexCache {
            game_path: std::path::PathBuf::from("/test"),
            last_indexed: 0,
            files: Vec::new(),
        };

        let task = app.update(Message::System(SystemMessage::CacheIndexationComplete(
            cache,
        )));
        let _ = task;
        assert!(!app.is_indexing, "indexing flag cleared");
    }

    #[test]
    fn cache_indexation_failed_clears_flag() {
        let mut app = App::test_new(Workspace::new());
        app.is_indexing = true;

        let task = app.update(Message::System(SystemMessage::CacheIndexationFailed));
        let _ = task;
        assert!(!app.is_indexing, "indexing flag cleared on failure");
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn app_with_tab(editor_type: EditorType) -> App {
    let mut workspace = Workspace::new();
    workspace.tabs.push(WorkspaceTab {
        id: 1,
        label: format!("{:?}", editor_type),
        path: None,
        editor_type,
        modified: false,
        pinned: false,
    });
    workspace.active_tab = Some(0);
    App::test_new(workspace)
}
