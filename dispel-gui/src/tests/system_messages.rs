//! Integration tests for system-message handlers.
//!
//! Coverage gap identified in the staging-branch review: 10 of ~20 system
//! messages had zero test coverage.  This file adds tests for the most
//! impactful missing handlers.
//!
//! Tests send messages through `app.update(Message::System(...))` rather
//! than calling system::handle directly (which is private).

use crate::app::App;
use crate::message::Message;
use crate::message::system::SystemMessage;
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
        let _task = app.update(Message::System(SystemMessage::Save));
        assert_ne!(app.state.status_msg, "This editor does not support saving");
    }

    #[test]
    fn save_event_scr_returns_task() {
        let mut app = app_with_tab(EditorType::EventScrEditor);
        let _task = app.update(Message::System(SystemMessage::Save));
        assert_ne!(app.state.status_msg, "This editor does not support saving");
    }

    #[test]
    fn save_map_editor_returns_task() {
        let mut app = app_with_tab(EditorType::MapEditor);
        let _task = app.update(Message::System(SystemMessage::Save));
        assert_ne!(app.state.status_msg, "This editor does not support saving");
    }

    #[test]
    fn save_weapon_editor_returns_task() {
        let mut app = app_with_tab(EditorType::WeaponEditor);
        let _task = app.update(Message::System(SystemMessage::Save));
        assert_ne!(app.state.status_msg, "This editor does not support saving");
    }

    #[test]
    fn save_snf_and_hex_dispatch_save_messages() {
        // Ctrl+S calls the descriptor's save closure through
        // `save_task_for_editor`; both editors have their own save plumbing.
        let snf = crate::editor_catalog::descriptor(EditorType::SnfEditor)
            .save
            .expect("SnfEditor must declare a save action")(1);
        assert!(matches!(
            snf,
            Message::Editor(crate::message::EditorMessage::Snf(
                crate::editors::snf_editor::SnfEditorMessage::Save(1)
            ))
        ));

        let hex = crate::editor_catalog::descriptor(EditorType::HexEditor)
            .save
            .expect("HexEditor must declare a save action")(1);
        assert!(matches!(
            hex,
            Message::Editor(crate::message::EditorMessage::HexEditorSave(1))
        ));

        for et in [EditorType::SnfEditor, EditorType::HexEditor] {
            let mut app = app_with_tab(et);
            let _ = app.update(Message::System(SystemMessage::Save));
            assert_ne!(
                app.state.status_msg, "This editor does not support saving",
                "{et:?} must not reject Ctrl+S"
            );
        }
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
        let _task = app.update(Message::System(SystemMessage::IndexLoaded(Ok(idx))));

        assert_eq!(app.state.status_msg, "Search index loaded");
        assert_eq!(
            app.search_index.game_path,
            Some("/fake/path".to_string()),
            "index game_path should be set"
        );
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

// ============================================================================
// Save messages carry the tab id from the descriptor
// ============================================================================

#[cfg(test)]
mod save_tab_routing {
    use super::*;

    fn app_with_two_tabs(editor_type: EditorType) -> (App, usize, usize) {
        let mut app = App::test_new(Workspace::new());
        let a = app
            .state
            .workspace
            .open("a.bin".into(), Some(std::path::PathBuf::from("a.bin")));
        let b = app
            .state
            .workspace
            .open("b.bin".into(), Some(std::path::PathBuf::from("b.bin")));
        for idx in [a, b] {
            app.state.workspace.tabs[idx].editor_type = editor_type;
        }
        let a_id = app.state.workspace.tabs[a].id;
        let b_id = app.state.workspace.tabs[b].id;
        (app, a_id, b_id)
    }

    #[test]
    fn test_snf_save_result_targets_carried_tab_not_active() {
        let (mut app, saved_tab, other_tab) = app_with_two_tabs(EditorType::SnfEditor);
        let mut a = crate::editors::snf_editor::SnfEditorState::load_from_path(
            std::path::Path::new("a.bin"),
        );
        a.modified = true;
        let mut b = crate::editors::snf_editor::SnfEditorState::load_from_path(
            std::path::Path::new("b.bin"),
        );
        b.modified = true;
        app.state.editors.snf_editors.insert(saved_tab, a);
        app.state.editors.snf_editors.insert(other_tab, b);
        assert_eq!(app.state.workspace.active_tab_id().as_usize(), other_tab);

        let _ = app.update(Message::Editor(crate::message::EditorMessage::Snf(
            crate::editors::snf_editor::SnfEditorMessage::SaveDone(
                saved_tab,
                Ok("a.bin".to_string()),
            ),
        )));

        assert!(
            !app.state.editors.snf_editors[&saved_tab].modified,
            "the tab named in the save message must be cleared"
        );
        assert!(
            app.state.editors.snf_editors[&other_tab].modified,
            "the active but unrelated tab must keep its unsaved state"
        );
    }

    #[test]
    fn test_hex_save_targets_carried_tab_not_active() {
        let (mut app, saved_tab, other_tab) = app_with_two_tabs(EditorType::HexEditor);
        app.state.editors.hex_editors.insert(
            saved_tab,
            hexedit::HexEditorState::load_from_path(std::path::Path::new("a.bin")),
        );
        app.state.editors.hex_editors.insert(
            other_tab,
            hexedit::HexEditorState::load_from_path(std::path::Path::new("b.bin")),
        );
        assert_eq!(app.state.workspace.active_tab_id().as_usize(), other_tab);

        let _ = app.update(Message::Editor(
            crate::message::EditorMessage::HexEditorSave(saved_tab),
        ));

        assert_eq!(
            app.state.editors.hex_editors[&saved_tab].status_msg, "Save not available.",
            "the save message must reach the tab it names"
        );
        assert_ne!(
            app.state.editors.hex_editors[&other_tab].status_msg, "Save not available.",
            "the active tab must not be used as the save target"
        );
    }
}
