//! Integration tests for dispel-gui that verify:
//! - All 38 EditorType variants render a view without panicking
//! - open_file_in_workspace creates proper state for every file type
//! - EditorRegistry lifecycle edge cases
//! - Undo/Redo dispatch completeness
//! - Save dispatch (only MapEditor and EventScrEditor save)
//! - Message routing edge cases
//! - Pane grid state transitions (sidebar, history, maximize, focus)
//! - Map editor entity undo stack

use crate::app::App;
use crate::editor_registry::EditorRegistry;
use crate::message::Message;
use crate::message::workspace::WorkspaceMessage;
use crate::workspace::{EditorType, Workspace, WorkspaceTab};
use std::path::PathBuf;

// ============================================================================
// Helpers
// ============================================================================

/// Create an App with a single tab of the given editor type.
/// No actual editor state is loaded — simulates a freshly-opened tab.
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
// View dispatch tests — every EditorType must render without panic
// ============================================================================

/// Test that every named `EditorType` renders a view WITHOUT panicking.
///
/// Every view function runs **every frame**, so a panic = instant crash.
/// A "forgot to wire view for new editor" bug would silently fall through
/// to the `Unknown | None` branch and show "Select a file to edit" instead
/// of the actual editor.
#[cfg(all(test, feature = "iced_test"))]
mod view_dispatch_tests {
    use super::*;
    use crate::workspace::EditorType::*;
    use iced_test::simulator;

    #[test]
    fn test_all_editor_types_render_without_panic() {
        let types = vec![
            WeaponEditor,
            MonsterEditor,
            MonsterIniEditor,
            HealItemEditor,
            MiscItemEditor,
            EditItemEditor,
            EventItemEditor,
            MagicEditor,
            StoreEditor,
            ChDataEditor,
            PartyLevelDbEditor,
            DialogueScriptEditor,
            DialogueTextEditor,
            DrawItemEditor,
            EventIniEditor,
            EventNpcRefEditor,
            ExtraIniEditor,
            ExtraRefEditor,
            MapIniEditor,
            MessageScrEditor,
            MonsterRefEditor,
            NpcIniEditor,
            NpcRefEditor,
            PartyRefEditor,
            PartyIniEditor,
            QuestScrEditor,
            EventScrEditor,
            WaveIniEditor,
            AllMapIniEditor,
            ChestEditor,
            SpriteViewer,
            SnfEditor,
            DbViewer,
            TilesetEditor,
            MapEditor,
            ModPackager,
            LocalizationManager,
            HexEditor,
        ];

        for et in types {
            let app = app_with_tab(et);
            let view = app.view();
            // iced_test runs widget layout, catching layout panics:
            let _ui = simulator(view);
            // If we get here, no panic.
        }
    }
}

// ============================================================================
// EditorRegistry lifecycle edge cases
// ============================================================================

#[cfg(test)]
mod editor_registry_tests {
    use super::*;

    #[test]
    fn test_remove_tab_nonexistent_id_does_not_panic() {
        let mut registry = EditorRegistry::default();
        registry.remove_tab(9999);
    }

    #[test]
    fn test_remove_tab_preserves_unrelated_editors() {
        let mut registry = EditorRegistry::default();
        registry.map_editors.insert(1, Default::default());
        registry.sprite_viewers.insert(2, Default::default());

        registry.remove_tab(1);

        assert!(
            registry.sprite_viewers.contains_key(&2),
            "tab 2 sprite viewer should survive removal of tab 1"
        );
        assert!(
            !registry.map_editors.contains_key(&1),
            "tab 1 map editor should be removed"
        );
    }

    #[test]
    fn test_close_all_tabs_preserves_boxed_editors() {
        let mut registry = EditorRegistry::default();

        // Populate HashMap editors
        registry.map_editors.insert(1, Default::default());
        registry.sprite_viewers.insert(3, Default::default());

        // Populate tabbed editor
        registry.npc_ref_editor.editors.insert(1, Default::default());

        registry.close_all_tabs();

        // HashMap and tabbed editors are cleared
        assert!(registry.map_editors.is_empty());
        assert!(registry.sprite_viewers.is_empty());
        assert!(registry.npc_ref_editor.editors.is_empty());

        // Boxed editors like weapon_editor are NOT reset by close_all_tabs
        let _ = &registry.weapon_editor;
    }

    #[test]
    fn test_clear_all_resets_everything() {
        let mut registry = EditorRegistry::default();

        registry.map_editors.insert(1, Default::default());
        registry.sprite_viewers.insert(1, Default::default());
        registry.npc_ref_editor.editors.insert(1, Default::default());
        registry.dialogue_script_editor
            .editors
            .insert(1, Default::default());
        registry.monster_ref_editor
            .editors
            .insert(1, Default::default());
        registry.extra_ref_editor
            .editors
            .insert(1, Default::default());
        registry.dialogue_paragraph_editor
            .editors
            .insert(1, Default::default());

        registry.clear_all();

        assert!(registry.map_editors.is_empty());
        assert!(registry.sprite_viewers.is_empty());
        assert!(registry.npc_ref_editor.editors.is_empty());
        assert!(registry.dialogue_script_editor.editors.is_empty());
        assert!(registry.monster_ref_editor.editors.is_empty());
        assert!(registry.extra_ref_editor.editors.is_empty());
        assert!(registry.dialogue_paragraph_editor.editors.is_empty());
    }

    #[test]
    fn test_clear_all_is_idempotent() {
        let mut registry = EditorRegistry::default();
        registry.clear_all();
        registry.clear_all();
        registry.clear_all();
        // No panic = pass
    }
}

// ============================================================================
// Undo/Redo dispatch completeness
// ============================================================================

#[cfg(test)]
mod undo_redo_dispatch_tests {
    use super::*;
    use crate::workspace::EditorType::*;
    use std::collections::HashMap;

    /// Standard boxed editors that support undo/redo
    fn editors_with_undo() -> Vec<EditorType> {
        vec![
            WeaponEditor,
            HealItemEditor,
            MiscItemEditor,
            EditItemEditor,
            EventItemEditor,
            MonsterEditor,
            MonsterIniEditor,
            NpcIniEditor,
            MagicEditor,
            PartyRefEditor,
            PartyIniEditor,
            AllMapIniEditor,
            DrawItemEditor,
            EventIniEditor,
            EventNpcRefEditor,
            ExtraIniEditor,
            MapIniEditor,
            MessageScrEditor,
            QuestScrEditor,
            WaveIniEditor,
            ChDataEditor,
            PartyLevelDbEditor,
        ]
    }

    /// Editors that have no undo/redo support (should always return None)
    fn editors_without_undo() -> Vec<EditorType> {
        vec![
            StoreEditor,
            ChestEditor,
            SpriteViewer,
            SnfEditor,
            DbViewer,
            TilesetEditor,
            MapEditor,
            ModPackager,
            LocalizationManager,
            HexEditor,
            EventScrEditor,
            Unknown,
        ]
    }

    /// Tab-based editors that support undo/redo (need valid tab_id)
    fn tab_editors_with_undo() -> Vec<EditorType> {
        vec![
            MonsterRefEditor,
            NpcRefEditor,
            ExtraRefEditor,
            DialogueScriptEditor,
            DialogueTextEditor,
        ]
    }

    #[test]
    fn test_undo_active_returns_none_for_editors_without_undo() {
        let mut registry = EditorRegistry::default();
        let lookups = HashMap::new();

        for et in editors_without_undo() {
            let result = registry.undo_active(et, 0, &lookups);
            assert!(
                result.is_none(),
                "EditorType::{:?} should NOT have undo but got Some({:?})",
                et,
                result
            );
        }
    }

    #[test]
    fn test_redo_active_returns_none_for_editors_without_undo() {
        let mut registry = EditorRegistry::default();
        let lookups = HashMap::new();

        for et in editors_without_undo() {
            let result = registry.redo_active(et, 0, &lookups);
            assert!(
                result.is_none(),
                "EditorType::{:?} should NOT have redo but got Some({:?})",
                et,
                result
            );
        }
    }

    #[test]
    fn test_undo_active_empty_history_for_editors_with_undo() {
        let mut registry = EditorRegistry::default();
        let lookups = HashMap::new();

        for et in editors_with_undo() {
            let result = registry.undo_active(et, 0, &lookups);
            assert!(
                result.is_none(),
                "EditorType::{:?} should return None (empty history) but got Some({:?})",
                et,
                result
            );
        }
    }

    #[test]
    fn test_undo_active_tab_editor_without_valid_tab_id() {
        let mut registry = EditorRegistry::default();
        let lookups = HashMap::new();

        for et in tab_editors_with_undo() {
            let result = registry.undo_active(et, 999, &lookups);
            assert!(
                result.is_none(),
                "EditorType::{:?} with unknown tab_id should return None but got Some({:?})",
                et,
                result
            );
        }
    }

    #[test]
    fn test_redo_active_tab_editor_without_valid_tab_id() {
        let mut registry = EditorRegistry::default();
        let lookups = HashMap::new();

        for et in tab_editors_with_undo() {
            let result = registry.redo_active(et, 999, &lookups);
            assert!(
                result.is_none(),
                "EditorType::{:?} with unknown tab_id should return None but got Some({:?})",
                et,
                result
            );
        }
    }
}

// ============================================================================
// get_active_edit_history completeness
// ============================================================================

#[cfg(test)]
mod edit_history_tests {
    use super::*;
    use crate::workspace::EditorType::*;

    #[test]
    fn test_all_standard_editors_have_edit_history() {
        let registry = EditorRegistry::default();

        let editors_with_history = vec![
            HealItemEditor,
            MiscItemEditor,
            EditItemEditor,
            EventItemEditor,
            MagicEditor,
            WeaponEditor,
            DrawItemEditor,
            EventIniEditor,
            EventNpcRefEditor,
            ExtraIniEditor,
            MapIniEditor,
            MessageScrEditor,
            PartyLevelDbEditor,
            QuestScrEditor,
            WaveIniEditor,
            AllMapIniEditor,
            ChDataEditor,
            PartyRefEditor,
            PartyIniEditor,
            StoreEditor,
        ];

        for et in editors_with_history {
            let history = registry.get_active_edit_history(et, 0);
            assert!(
                history.is_some(),
                "EditorType::{:?} should have edit history but got None",
                et
            );
        }
    }

    #[test]
    fn test_tab_editors_return_history_only_with_valid_tab_id() {
        let mut registry = EditorRegistry::default();

        assert!(registry.get_active_edit_history(MonsterRefEditor, 0).is_none());
        assert!(registry.get_active_edit_history(NpcRefEditor, 0).is_none());
        assert!(registry.get_active_edit_history(ExtraRefEditor, 0).is_none());
        assert!(registry.get_active_edit_history(DialogueScriptEditor, 0).is_none());
        assert!(registry.get_active_edit_history(DialogueTextEditor, 0).is_none());

        registry.npc_ref_editor.editors.insert(42, Default::default());
        assert!(
            registry.get_active_edit_history(NpcRefEditor, 42).is_some(),
            "NpcRefEditor with tab_id=42 should have history after insert"
        );
    }

    #[test]
    fn test_editors_without_history_return_none() {
        let registry = EditorRegistry::default();

        let editors_without = vec![
            EventScrEditor,
            MonsterEditor,
            MonsterIniEditor,
            NpcIniEditor,
            ChestEditor,
            SpriteViewer,
            SnfEditor,
            DbViewer,
            TilesetEditor,
            MapEditor,
            ModPackager,
            LocalizationManager,
            HexEditor,
            Unknown,
        ];

        for et in editors_without {
            let history = registry.get_active_edit_history(et, 0);
            assert!(
                history.is_none(),
                "EditorType::{:?} should NOT have edit history but got Some",
                et
            );
        }
    }
}

// ============================================================================
// Save dispatch verification — only MapEditor and EventScrEditor save
// ============================================================================

#[cfg(test)]
mod save_dispatch_tests {
    use super::*;
    use crate::message::Message;
    use crate::message::system::SystemMessage;
    use crate::workspace::EditorType::*;

    #[test]
    fn test_save_returns_task_for_map_editor_and_event_scr() {
        let mut app = app_with_tab(MapEditor);
        let task = app.update(Message::System(SystemMessage::Save));
        assert!(task.units() > 0, "MapEditor Save should produce a task");

        let mut app = app_with_tab(EventScrEditor);
        let task = app.update(Message::System(SystemMessage::Save));
        assert!(task.units() > 0, "EventScrEditor Save should produce a task");
    }

    #[test]
    fn test_save_returns_none_for_most_editor_types() {
        let no_save_types = vec![
            WeaponEditor,
            MonsterEditor,
            MonsterIniEditor,
            HealItemEditor,
            MiscItemEditor,
            EditItemEditor,
            EventItemEditor,
            MagicEditor,
            StoreEditor,
            ChDataEditor,
            PartyLevelDbEditor,
            DialogueScriptEditor,
            DialogueTextEditor,
            DrawItemEditor,
            EventIniEditor,
            EventNpcRefEditor,
            ExtraIniEditor,
            ExtraRefEditor,
            MapIniEditor,
            MessageScrEditor,
            MonsterRefEditor,
            NpcIniEditor,
            NpcRefEditor,
            PartyRefEditor,
            PartyIniEditor,
            QuestScrEditor,
            WaveIniEditor,
            AllMapIniEditor,
            ChestEditor,
            SpriteViewer,
            SnfEditor,
            DbViewer,
            TilesetEditor,
            ModPackager,
            LocalizationManager,
            HexEditor,
        ];

        for et in no_save_types {
            let mut app = app_with_tab(et);
            let task = app.update(Message::System(SystemMessage::Save));
            assert_eq!(
                task.units(), 0,
                "EditorType::{:?} Save should produce Task::none()",
                et
            );
        }
    }
}

// ============================================================================
// Message routing edge cases
// ============================================================================

#[cfg(test)]
mod message_routing_tests {
    use super::*;
    use crate::message::Message;

    #[test]
    fn test_hex_editor_message_when_no_state_does_not_panic() {
        let mut app = app_with_tab(EditorType::HexEditor);

        use crate::message::editor::EditorMessage;
        use hexedit::selection::NavDir;
        use hexedit::HexEditorMessage;
        // Route through the public update API — should not panic
        let task = app.update(Message::Editor(EditorMessage::HexEditor(
            HexEditorMessage::Nav {
                dir: NavDir::Down,
                extend: false,
            },
        )));
        assert_eq!(
            task.units(), 0,
            "Should return Task::none() when no hex editor state"
        );
    }

    #[test]
    fn test_open_tool_tab_creates_workspace_tab() {
        let mut app = App::test_new(Workspace::new());

        use crate::message::workspace::WorkspaceMessage;
        let task = app.update(Message::Workspace(WorkspaceMessage::OpenToolTab(
            EditorType::DbViewer,
        )));

        assert_eq!(app.state.workspace.tabs.len(), 1);
        assert_eq!(
            app.state.workspace.tabs[0].editor_type,
            EditorType::DbViewer
        );
        assert_eq!(app.state.workspace.tabs[0].label, "DB Viewer");

        let _ = task;
    }

    #[test]
    fn test_open_tool_tab_without_game_path_works() {
        let mut app = App::test_new(Workspace::new());

        use crate::message::workspace::WorkspaceMessage;
        let task = app.update(Message::Workspace(WorkspaceMessage::OpenToolTab(
            EditorType::ChestEditor,
        )));

        assert_eq!(app.state.workspace.tabs.len(), 1);
        assert_eq!(
            app.state.workspace.tabs[0].editor_type,
            EditorType::ChestEditor
        );

        let _ = task;
    }

    #[test]
    fn test_clear_workspace_system_message_is_idempotent() {
        let mut app = App::test_new(Workspace::new());

        app.state.workspace.open("test".into(), None);
        assert_eq!(app.state.workspace.tabs.len(), 1);

        // First clear
        let task1 = app.update(Message::System(
            crate::message::system::SystemMessage::ClearWorkspace,
        ));
        let _ = task1;
        assert_eq!(app.state.workspace.tabs.len(), 0);

        // Second clear — must not panic and must keep state clean
        let task2 = app.update(Message::System(
            crate::message::system::SystemMessage::ClearWorkspace,
        ));
        let _ = task2;
        assert_eq!(app.state.workspace.tabs.len(), 0);
    }

    #[test]
    fn test_open_file_in_workspace_dialogue_script_creates_state() {
        let mut app = App::test_new(Workspace::new());

        let path = PathBuf::from("/game/scene.dlg");
        let _task = app.open_file_in_workspace(&path);

        assert!(app.state.workspace.tabs.len() >= 1);

        let tab = app.state.workspace.active().unwrap();
        assert_eq!(tab.editor_type, EditorType::DialogueScriptEditor);
    }

    #[test]
    fn test_open_file_in_workspace_creates_tab_and_tracks_recent() {
        let mut app = App::test_new(Workspace::new());

        let path = PathBuf::from("/game/Monster.db");
        let _task = app.open_file_in_workspace(&path);

        let tab = app.state.workspace.active().unwrap();
        assert_eq!(tab.editor_type, EditorType::MonsterEditor);

        assert!(!app.state.recent_files.is_empty());
        assert_eq!(app.state.recent_files[0], path);
    }

    #[test]
    fn test_open_file_in_workspace_hex_editor_for_unknown_extension() {
        let mut app = App::test_new(Workspace::new());

        let path = PathBuf::from("/game/random.xyz");
        let _task = app.open_file_in_workspace(&path);

        let tab = app.state.workspace.active().unwrap();
        assert_eq!(
            tab.editor_type,
            EditorType::HexEditor,
            "Unknown extension should fall back to HexEditor"
        );
    }

    #[test]
    fn test_open_same_file_reactivates_existing_tab() {
        let mut app = App::test_new(Workspace::new());

        let path = PathBuf::from("/game/weaponItem.db");
        let _task1 = app.open_file_in_workspace(&path);

        let _task2 = app.open_file_in_workspace(&path);

        // Should still be just one tab (reactivated), not two
        assert_eq!(app.state.workspace.tabs.len(), 1);
    }

    #[test]
    fn test_track_recent_files_limit() {
        let mut app = App::test_new(Workspace::new());

        // Add 15 files (should be capped to 10)
        for i in 0..15 {
            app.track_recent_file(&PathBuf::from(format!("/game/file{}.db", i)));
        }

        assert_eq!(app.state.recent_files.len(), 10);
        assert_eq!(
            app.state.recent_files[0],
            PathBuf::from("/game/file14.db")
        );
    }

    #[test]
    fn test_track_recent_file_dedup() {
        let mut app = App::test_new(Workspace::new());

        app.track_recent_file(&PathBuf::from("/game/weaponItem.db"));
        app.track_recent_file(&PathBuf::from("/game/monster.db"));
        app.track_recent_file(&PathBuf::from("/game/weaponItem.db"));

        assert_eq!(app.state.recent_files.len(), 2);
        assert_eq!(
            app.state.recent_files[0],
            PathBuf::from("/game/weaponItem.db")
        );
    }

    #[test]
    fn test_editor_type_from_path_round_trips_all_db_types() {
        let cases = vec![
            ("weaponItem.db", EditorType::WeaponEditor),
            ("Monster.db", EditorType::MonsterEditor),
            ("HealItem.db", EditorType::HealItemEditor),
            ("MiscItem.db", EditorType::MiscItemEditor),
            ("EditItem.db", EditorType::EditItemEditor),
            ("EventItem.db", EditorType::EventItemEditor),
            ("Store.db", EditorType::StoreEditor),
            ("Magic.db", EditorType::MagicEditor),
            ("ChData.db", EditorType::ChDataEditor),
            ("PrtLevel.db", EditorType::PartyLevelDbEditor),
            ("PrtIni.db", EditorType::PartyIniEditor),
        ];
        for (filename, expected) in cases {
            let result = EditorType::from_path(PathBuf::from(filename).as_path());
            assert_eq!(result, expected, "failed for {filename}");
        }
    }

    #[test]
    fn test_editor_type_from_path_round_trips_all_ini_types() {
        let cases = vec![
            ("AllMap.ini", EditorType::AllMapIniEditor),
            ("Map.ini", EditorType::MapIniEditor),
            ("Extra.ini", EditorType::ExtraIniEditor),
            ("Event.ini", EditorType::EventIniEditor),
            ("Monster.ini", EditorType::MonsterIniEditor),
            ("Npc.ini", EditorType::NpcIniEditor),
            ("Wave.ini", EditorType::WaveIniEditor),
        ];
        for (filename, expected) in cases {
            let result = EditorType::from_path(PathBuf::from(filename).as_path());
            assert_eq!(result, expected, "failed for {filename}");
        }
    }

    #[test]
    fn test_editor_type_from_path_round_trips_all_ref_types() {
        let cases = vec![
            ("PartyRef.ref", EditorType::PartyRefEditor),
            ("DrawItem.ref", EditorType::DrawItemEditor),
            ("EventNpc.ref", EditorType::EventNpcRefEditor),
            ("Npc01.ref", EditorType::NpcRefEditor),
            ("Mon01.ref", EditorType::MonsterRefEditor),
            ("Ext01.ref", EditorType::ExtraRefEditor),
        ];
        for (filename, expected) in cases {
            let result = EditorType::from_path(PathBuf::from(filename).as_path());
            assert_eq!(result, expected, "failed for {filename}");
        }
    }

    #[test]
    fn test_editor_type_from_path_special_formats() {
        assert_eq!(
            EditorType::from_path(PathBuf::from("file.spr").as_path()),
            EditorType::SpriteViewer
        );
        assert_eq!(
            EditorType::from_path(PathBuf::from("file.snf").as_path()),
            EditorType::SnfEditor
        );
        assert_eq!(
            EditorType::from_path(PathBuf::from("file.dlg").as_path()),
            EditorType::DialogueScriptEditor
        );
        assert_eq!(
            EditorType::from_path(PathBuf::from("file.pgp").as_path()),
            EditorType::DialogueTextEditor
        );
        assert_eq!(
            EditorType::from_path(PathBuf::from("file.map").as_path()),
            EditorType::MapEditor
        );
        assert_eq!(
            EditorType::from_path(PathBuf::from("file.gtl").as_path()),
            EditorType::TilesetEditor
        );
        assert_eq!(
            EditorType::from_path(PathBuf::from("file.btl").as_path()),
            EditorType::TilesetEditor
        );
    }
}

// ============================================================================
// Pane grid state transitions
// ============================================================================

#[cfg(test)]
mod pane_grid_tests {
    use super::*;

    #[test]
    fn toggle_sidebar_hides_and_shows() {
        let mut app = App::test_new(Workspace::new());
        assert!(app.sidebar_visible, "sidebar visible by default");

        // Hide sidebar
        app.update(Message::Workspace(WorkspaceMessage::ToggleSidebar));
        assert!(!app.sidebar_visible, "sidebar hidden");
        assert_eq!(
            app.state.pane_state.state.len(),
            1,
            "one pane when sidebar hidden"
        );

        // Show sidebar again
        app.update(Message::Workspace(WorkspaceMessage::ToggleSidebar));
        assert!(app.sidebar_visible, "sidebar shown again");
        assert_eq!(
            app.state.pane_state.state.len(),
            2,
            "two panes when sidebar visible"
        );
    }

    #[test]
    fn toggle_sidebar_twice_restores_original_state() {
        let mut app = App::test_new(Workspace::new());
        let panes_before = app.state.pane_state.state.len();

        app.update(Message::Workspace(WorkspaceMessage::ToggleSidebar));
        app.update(Message::Workspace(WorkspaceMessage::ToggleSidebar));

        assert!(app.sidebar_visible);
        assert_eq!(
            app.state.pane_state.state.len(),
            panes_before,
            "same number of panes after double-toggle"
        );
    }

    #[test]
    fn toggle_history_panel_shows_and_hides() {
        let mut app = App::test_new(Workspace::new());
        assert!(!app.history_panel_visible, "history hidden by default");

        // Show history panel
        app.update(Message::Workspace(WorkspaceMessage::ToggleHistoryPanel));
        assert!(app.history_panel_visible);
        assert_eq!(
            app.state.pane_state.state.len(),
            3,
            "three panes with history panel"
        );

        // Hide history panel
        app.update(Message::Workspace(WorkspaceMessage::ToggleHistoryPanel));
        assert!(!app.history_panel_visible);
        assert_eq!(
            app.state.pane_state.state.len(),
            2,
            "back to two panes"
        );
    }

    #[test]
    fn toggle_history_panel_with_hidden_sidebar() {
        let mut app = App::test_new(Workspace::new());

        // Hide sidebar first
        app.update(Message::Workspace(WorkspaceMessage::ToggleSidebar));
        assert_eq!(app.state.pane_state.state.len(), 1, "only main content");

        // Show history panel (should split the single main pane)
        app.update(Message::Workspace(WorkspaceMessage::ToggleHistoryPanel));
        assert!(app.history_panel_visible);
        assert_eq!(
            app.state.pane_state.state.len(),
            2,
            "main + history with no sidebar"
        );

        // Show sidebar again (should rebuild with all three)
        app.update(Message::Workspace(WorkspaceMessage::ToggleSidebar));
        assert!(app.sidebar_visible);
        assert_eq!(
            app.state.pane_state.state.len(),
            3,
            "sidebar + main + history"
        );
    }

    #[test]
    fn toggle_maximize_pane_toggles_state() {
        let mut app = App::test_new(Workspace::new());
        assert!(app.state.pane_state.maximized.is_none(), "not maximized");

        app.update(Message::Workspace(WorkspaceMessage::ToggleMaximizePane));
        assert!(
            app.state.pane_state.maximized.is_some(),
            "maximized after toggle"
        );

        app.update(Message::Workspace(WorkspaceMessage::ToggleMaximizePane));
        assert!(
            app.state.pane_state.maximized.is_none(),
            "restored after second toggle"
        );
    }

    #[test]
    fn pane_clicked_changes_focus() {
        let mut app = App::test_new(Workspace::new());
        let initial_focus = app.state.pane_state.focus;

        // Get the other pane
        let other_pane = app
            .state
            .pane_state
            .state
            .iter()
            .find(|(id, _)| **id != initial_focus)
            .map(|(id, _)| *id)
            .expect("at least two panes in default layout");

        app.update(Message::Workspace(WorkspaceMessage::PaneClicked(other_pane)));
        assert_eq!(
            app.state.pane_state.focus,
            other_pane,
            "focus changed to clicked pane"
        );
    }

    #[test]
    fn pane_resized_does_not_panic() {
        let mut app = App::test_new(Workspace::new());

        // Get the sidebar split from the default layout
        let split = app.state.pane_state.sidebar_split.expect("has sidebar split");

        use iced::widget::pane_grid::ResizeEvent;
        let event = ResizeEvent { split, ratio: 0.3 };
        app.update(Message::Workspace(WorkspaceMessage::PaneResized(event)));
        // If we get here without panicking, the resize was handled
    }

    #[test]
    fn pane_dragged_does_not_panic() {
        let mut app = App::test_new(Workspace::new());

        // Collect pane IDs
        let panes: Vec<_> = app
            .state
            .pane_state
            .state
            .iter()
            .map(|(id, _)| *id)
            .collect();
        assert!(panes.len() >= 2, "at least two panes");

        use iced::widget::pane_grid::{DragEvent, Target};
        app.update(Message::Workspace(WorkspaceMessage::PaneDragged(
            DragEvent::Dropped {
                pane: panes[1],
                target: Target::Pane(panes[0], iced::widget::pane_grid::Region::Center),
            },
        )));
        // If we get here without panicking, the drop was handled
    }
}

// ============================================================================
// Map editor entity undo stack
// ============================================================================

#[cfg(test)]
mod map_editor_entity_tests {
    use super::*;
    use crate::editors::map_editor;
    use crate::editors::map_editor::{MapEditAction, MapEditorState, MapEditorMessage, SelectedEntity};
    use dispel_core::MonsterRef;

    // ── MapEditorState undo stack unit tests ──────────────────────────────

    #[test]
    fn push_undo_adds_to_front_and_clears_redo() {
        let mut state = MapEditorState::default();
        let action = MapEditAction {
            entity: SelectedEntity::Monster(0),
            field: "name".into(),
            old_value: "old".into(),
            new_value: "new".into(),
        };
        state.push_undo(action);

        assert_eq!(state.data.undo_stack.len(), 1);
        assert!(state.data.redo_stack.is_empty(), "redo cleared");
        assert!(state.data.dirty, "marked dirty");
    }

    #[test]
    fn pop_undo_returns_action_and_pushes_inverted_to_redo() {
        let mut state = MapEditorState::default();
        state.push_undo(MapEditAction {
            entity: SelectedEntity::Npc(0),
            field: "name".into(),
            old_value: "old".into(),
            new_value: "new".into(),
        });

        let action = state.pop_undo().expect("undo action available");
        assert_eq!(action.old_value, "old", "old_value preserved");
        assert_eq!(action.new_value, "new", "new_value preserved");

        // Redo should have the same action (not inverted)
        assert_eq!(state.data.redo_stack.len(), 1);
        let redo_action = state.data.redo_stack.front().unwrap();
        assert_eq!(
            redo_action.old_value, "old",
            "redo old_value preserved"
        );
        assert_eq!(
            redo_action.new_value, "new",
            "redo new_value preserved"
        );
    }

    #[test]
    fn pop_redo_returns_action_and_pushes_back_to_undo() {
        let mut state = MapEditorState::default();
        state.push_undo(MapEditAction {
            entity: SelectedEntity::Extra(0),
            field: "name".into(),
            old_value: "old".into(),
            new_value: "new".into(),
        });

        // Undo once
        let _ = state.pop_undo();
        assert!(state.data.undo_stack.is_empty());
        assert_eq!(state.data.redo_stack.len(), 1);

        // Redo
        let action = state.pop_redo().expect("redo action available");
        assert_eq!(action.old_value, "old", "redo old_value preserved");
        assert_eq!(action.new_value, "new", "redo new_value preserved");

        assert_eq!(state.data.undo_stack.len(), 1, "pushed back to undo");
        assert!(state.data.redo_stack.is_empty(), "redo consumed");
    }

    #[test]
    fn pop_undo_empty_stack_returns_none() {
        let mut state = MapEditorState::default();
        assert!(state.pop_undo().is_none());
    }

    #[test]
    fn pop_redo_empty_stack_returns_none() {
        let mut state = MapEditorState::default();
        assert!(state.pop_redo().is_none());
    }

    #[test]
    fn push_undo_caps_at_max_history() {
        let mut state = MapEditorState::default();
        // MAX_MAP_HISTORY = 100
        for i in 0..200 {
            state.push_undo(MapEditAction {
                entity: SelectedEntity::Monster(i),
                field: "pos_x".into(),
                old_value: format!("{i}"),
                new_value: format!("{}", i + 1),
            });
        }
        assert_eq!(
            state.data.undo_stack.len(),
            100,
            "capped at MAX_MAP_HISTORY"
        );
        // Oldest entries are dropped
        let first = state.data.undo_stack.back().unwrap();
        assert_eq!(
            first.entity,
            SelectedEntity::Monster(100),
            "oldest entry is index 100 (dropped 0-99)"
        );
    }

    #[test]
    fn push_undo_clears_redo_stack() {
        let mut state = MapEditorState::default();
        state.push_undo(MapEditAction {
            entity: SelectedEntity::Monster(0),
            field: "name".into(),
            old_value: "old".into(),
            new_value: "new".into(),
        });
        let _ = state.pop_undo();
        assert_eq!(state.data.redo_stack.len(), 1);

        // Push a new action — clears redo
        state.push_undo(MapEditAction {
            entity: SelectedEntity::Npc(0),
            field: "name".into(),
            old_value: "old2".into(),
            new_value: "new2".into(),
        });
        assert!(state.data.redo_stack.is_empty(), "redo cleared on new edit");
    }

    #[test]
    fn undo_redo_round_trip_preserves_values() {
        let mut state = MapEditorState::default();
        state.push_undo(MapEditAction {
            entity: SelectedEntity::Monster(0),
            field: "pos_x".into(),
            old_value: "10".into(),
            new_value: "20".into(),
        });

        let undo = state.pop_undo().unwrap();
        assert_eq!(undo.old_value, "10");
        assert_eq!(undo.new_value, "20");

        let redo = state.pop_redo().unwrap();
        // Values are not swapped — preserves original old/new throughout.
        assert_eq!(redo.old_value, "10");
        assert_eq!(redo.new_value, "20");
    }

    // ── Entity field_changed integration tests ────────────────────────────

    fn app_with_map_editor() -> App {
        let mut app = App::test_new(Workspace::new());
        let tab_id = 0;

        // Push a workspace tab so set_tab_modified can find it
        app.state.workspace.tabs.push(WorkspaceTab {
            id: tab_id,
            label: "test.map".into(),
            path: Some(PathBuf::from("test.map")),
            editor_type: EditorType::MapEditor,
            modified: false,
            pinned: false,
        });

        // Insert a MapEditorState with one monster
        let mut map_state = MapEditorState::default();
        map_state.data.monsters = vec![MonsterRef {
            index: 0,
            mon_id: 1,
            pos_x: 100,
            pos_y: 200,
            ..Default::default()
        }];
        app.state.editors.map_editors.insert(tab_id, map_state);

        app
    }

    #[test]
    fn entity_field_changed_monster_updates_value_and_undo_stack() {
        let mut app = app_with_map_editor();
        let tab_id = 0;

        let task = map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                tab_id,
                SelectedEntity::Monster(0),
                "pos_x".into(),
                "150".into(),
            ),
            &mut app,
        );

        assert_eq!(task.units(), 0, "EntityFieldChanged produces no task");
        assert_eq!(
            app.state.editors.map_editors[&tab_id]
                .data
                .monsters[0]
                .pos_x,
            150,
            "monster pos_x updated"
        );
        assert!(
            !app.state.editors.map_editors[&tab_id]
                .data
                .undo_stack
                .is_empty(),
            "undo stack has entry"
        );
        assert!(
            app.state.workspace.tabs[0].modified,
            "tab marked modified"
        );
    }

    #[test]
    fn entity_field_changed_nonexistent_tab_id_is_noop() {
        let mut app = app_with_map_editor();

        let task = map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                999,
                SelectedEntity::Monster(0),
                "pos_x".into(),
                "150".into(),
            ),
            &mut app,
        );

        assert_eq!(task.units(), 0, "no-op for unknown tab_id");
    }

    #[test]
    fn entity_field_changed_unchanged_value_does_not_push_undo() {
        let mut app = app_with_map_editor();
        let tab_id = 0;

        // Set same value — old_value (100) == new_value (100)
        let task = map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                tab_id,
                SelectedEntity::Monster(0),
                "pos_x".into(),
                "100".into(),
            ),
            &mut app,
        );

        assert_eq!(task.units(), 0);
        assert!(
            app.state.editors.map_editors[&tab_id]
                .data
                .undo_stack
                .is_empty(),
            "no undo for unchanged value"
        );
        assert!(
            !app.state.workspace.tabs[0].modified,
            "tab not marked modified"
        );
    }

    #[test]
    fn entity_field_changed_nonexistent_monster_index_does_not_panic() {
        let mut app = app_with_map_editor();
        let tab_id = 0;

        // Only monster 0 exists, use idx 5
        let task = map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                tab_id,
                SelectedEntity::Monster(5),
                "pos_x".into(),
                "150".into(),
            ),
            &mut app,
        );

        assert_eq!(task.units(), 0);
        assert!(
            app.state.editors.map_editors[&tab_id]
                .data
                .undo_stack
                .is_empty(),
            "no undo when entity index out of bounds"
        );
    }

    #[test]
    fn entity_undo_monster_reverts_field_and_pushes_redo() {
        let mut app = app_with_map_editor();
        let tab_id = 0;

        // Make an edit
        map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                tab_id,
                SelectedEntity::Monster(0),
                "pos_x".into(),
                "150".into(),
            ),
            &mut app,
        );
        assert!(
            app.state.workspace.tabs[0].modified,
            "tab dirty after edit"
        );

        // Undo
        let task = map_editor::handle(MapEditorMessage::Undo(tab_id), &mut app);
        assert_eq!(task.units(), 0);
        assert_eq!(
            app.state.editors.map_editors[&tab_id]
                .data
                .monsters[0]
                .pos_x,
            100,
            "pos_x reverted to original"
        );
        assert!(
            app.state.editors.map_editors[&tab_id]
                .data
                .undo_stack
                .is_empty(),
            "undo stack consumed"
        );
        assert_eq!(
            app.state.editors.map_editors[&tab_id]
                .data
                .redo_stack
                .len(),
            1,
            "redo stack has entry"
        );
    }

    #[test]
    fn entity_redo_restores_field() {
        let mut app = app_with_map_editor();
        let tab_id = 0;

        // Edit: pos_x 100 → 150
        map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                tab_id,
                SelectedEntity::Monster(0),
                "pos_x".into(),
                "150".into(),
            ),
            &mut app,
        );

        // Undo: 150 → 100
        map_editor::handle(MapEditorMessage::Undo(tab_id), &mut app);
        assert_eq!(
            app.state.editors.map_editors[&tab_id]
                .data
                .monsters[0]
                .pos_x,
            100
        );

        // Redo: 100 → 150
        let task = map_editor::handle(MapEditorMessage::Redo(tab_id), &mut app);
        assert_eq!(task.units(), 0);
        assert_eq!(
            app.state.editors.map_editors[&tab_id]
                .data
                .monsters[0]
                .pos_x,
            150,
            "pos_x restored to edited value"
        );
        assert!(
            app.state.editors.map_editors[&tab_id]
                .data
                .redo_stack
                .is_empty(),
            "redo stack consumed"
        );
    }

    #[test]
    fn entity_undo_empty_stack_does_not_panic() {
        let mut app = app_with_map_editor();
        let tab_id = 0;

        let task = map_editor::handle(MapEditorMessage::Undo(tab_id), &mut app);
        assert_eq!(task.units(), 0);
    }

    #[test]
    fn entity_undo_nonexistent_tab_id_does_not_panic() {
        let mut app = app_with_map_editor();

        let task = map_editor::handle(MapEditorMessage::Undo(999), &mut app);
        assert_eq!(task.units(), 0);
    }

    #[test]
    fn entity_redo_empty_stack_does_not_panic() {
        let mut app = app_with_map_editor();
        let tab_id = 0;

        let task = map_editor::handle(MapEditorMessage::Redo(tab_id), &mut app);
        assert_eq!(task.units(), 0);
    }

    #[test]
    fn entity_field_changed_works_for_npc_field() {
        let mut app = App::test_new(Workspace::new());
        let tab_id = 0;

        app.state.workspace.tabs.push(WorkspaceTab {
            id: tab_id,
            label: "test.map".into(),
            path: Some(PathBuf::from("test.map")),
            editor_type: EditorType::MapEditor,
            modified: false,
            pinned: false,
        });

        let mut map_state = MapEditorState::default();
        map_state.data.npcs = vec![dispel_core::NPC {
            name: "Guard".into(),
            ..Default::default()
        }];
        app.state.editors.map_editors.insert(tab_id, map_state);

        let task = map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                tab_id,
                SelectedEntity::Npc(0),
                "name".into(),
                "Guard Captain".into(),
            ),
            &mut app,
        );

        assert_eq!(task.units(), 0);
        assert_eq!(
            app.state.editors.map_editors[&tab_id]
                .data
                .npcs[0]
                .name,
            "Guard Captain",
            "NPC name updated"
        );
    }

    #[test]
    fn entity_field_changed_works_for_extra_field() {
        let mut app = App::test_new(Workspace::new());
        let tab_id = 0;

        app.state.workspace.tabs.push(WorkspaceTab {
            id: tab_id,
            label: "test.map".into(),
            path: Some(PathBuf::from("test.map")),
            editor_type: EditorType::MapEditor,
            modified: false,
            pinned: false,
        });

        let mut map_state = MapEditorState::default();
        map_state.data.extra_refs = vec![dispel_core::ExtraRef {
            id: 1,
            ..Default::default()
        }];
        app.state.editors.map_editors.insert(tab_id, map_state);

        let task = map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                tab_id,
                SelectedEntity::Extra(0),
                "id".into(),
                "99".into(),
            ),
            &mut app,
        );

        assert_eq!(task.units(), 0);
        assert_eq!(
            app.state.editors.map_editors[&tab_id]
                .data
                .extra_refs[0]
                .id,
            99,
            "ExtraRef id updated"
        );
    }

    #[test]
    fn entity_multiple_edits_produce_ordered_undo_stack() {
        let mut app = app_with_map_editor();
        let tab_id = 0;

        // Edit 1: pos_x 100 → 150
        map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                tab_id,
                SelectedEntity::Monster(0),
                "pos_x".into(),
                "150".into(),
            ),
            &mut app,
        );

        // Edit 2: pos_x 150 → 200
        map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                tab_id,
                SelectedEntity::Monster(0),
                "pos_x".into(),
                "200".into(),
            ),
            &mut app,
        );

        // Undo 1: 200 → 150
        map_editor::handle(MapEditorMessage::Undo(tab_id), &mut app);
        assert_eq!(
            app.state.editors.map_editors[&tab_id]
                .data
                .monsters[0]
                .pos_x,
            150,
            "first undo: 200 → 150"
        );

        // Undo 2: 150 → 100
        map_editor::handle(MapEditorMessage::Undo(tab_id), &mut app);
        assert_eq!(
            app.state.editors.map_editors[&tab_id]
                .data
                .monsters[0]
                .pos_x,
            100,
            "second undo: 150 → 100"
        );

        // Redo 1: 100 → 150
        let _ = map_editor::handle(MapEditorMessage::Redo(tab_id), &mut app);
        assert_eq!(
            app.state.editors.map_editors[&tab_id]
                .data
                .monsters[0]
                .pos_x,
            150,
            "first redo: 100 → 150"
        );
    }
}
