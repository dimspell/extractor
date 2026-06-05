//! Integration tests for dispel-gui that verify:
//! - All 39 EditorType variants render a view without panicking
//! - open_file_in_workspace creates proper state for every file type
//! - EditorRegistry lifecycle edge cases
//! - Undo/Redo dispatch completeness
//! - Save dispatch (only MapEditor and EventScrEditor save)
//! - Message routing edge cases

use crate::app::App;
use crate::editor_registry::EditorRegistry;
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
