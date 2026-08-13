#[cfg(test)]
mod map_editor_entity_tests {
    use crate::app::App;
    use crate::editors::map_editor;
    use crate::editors::map_editor::{
        MapEditAction, MapEditorMessage, MapEditorState, SelectedEntity,
    };
    use crate::workspace::Workspace;
    use crate::workspace::{EditorType, WorkspaceTab};
    use dispel_core::MonsterRef;
    use std::path::PathBuf;

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
        assert_eq!(redo_action.old_value, "old", "redo old_value preserved");
        assert_eq!(redo_action.new_value, "new", "redo new_value preserved");
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
                field: "map_x".into(),
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
            field: "map_x".into(),
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
            monster_db_id: 1,
            map_x: 100,
            map_y: 200,
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
                "map_x".into(),
                "150".into(),
            ),
            &mut app,
        );

        assert_eq!(task.units(), 0, "EntityFieldChanged produces no task");
        assert_eq!(
            app.state.editors.map_editors[&tab_id].data.monsters[0].map_x, 150,
            "monster map_x updated"
        );
        assert!(
            !app.state.editors.map_editors[&tab_id]
                .data
                .undo_stack
                .is_empty(),
            "undo stack has entry"
        );
        assert!(app.state.workspace.tabs[0].modified, "tab marked modified");
    }

    #[test]
    fn entity_field_changed_nonexistent_tab_id_is_noop() {
        let mut app = app_with_map_editor();

        let task = map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                999,
                SelectedEntity::Monster(0),
                "map_x".into(),
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
                "map_x".into(),
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
                "map_x".into(),
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
        let _ = map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                tab_id,
                SelectedEntity::Monster(0),
                "map_x".into(),
                "150".into(),
            ),
            &mut app,
        );
        assert!(app.state.workspace.tabs[0].modified, "tab dirty after edit");

        // Undo
        let task = map_editor::handle(MapEditorMessage::Undo(tab_id), &mut app);
        assert_eq!(task.units(), 0);
        assert_eq!(
            app.state.editors.map_editors[&tab_id].data.monsters[0].map_x, 100,
            "map_x reverted to original"
        );
        assert!(
            app.state.editors.map_editors[&tab_id]
                .data
                .undo_stack
                .is_empty(),
            "undo stack consumed"
        );
        assert_eq!(
            app.state.editors.map_editors[&tab_id].data.redo_stack.len(),
            1,
            "redo stack has entry"
        );
    }

    #[test]
    fn entity_redo_restores_field() {
        let mut app = app_with_map_editor();
        let tab_id = 0;

        // Edit: map_x 100 → 150
        let _ = map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                tab_id,
                SelectedEntity::Monster(0),
                "map_x".into(),
                "150".into(),
            ),
            &mut app,
        );

        // Undo: 150 → 100
        let _ = map_editor::handle(MapEditorMessage::Undo(tab_id), &mut app);
        assert_eq!(
            app.state.editors.map_editors[&tab_id].data.monsters[0].map_x,
            100
        );

        // Redo: 100 → 150
        let task = map_editor::handle(MapEditorMessage::Redo(tab_id), &mut app);
        assert_eq!(task.units(), 0);
        assert_eq!(
            app.state.editors.map_editors[&tab_id].data.monsters[0].map_x, 150,
            "map_x restored to edited value"
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
            app.state.editors.map_editors[&tab_id].data.npcs[0].name, "Guard Captain",
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
            record_index: 1,
            ..Default::default()
        }];
        app.state.editors.map_editors.insert(tab_id, map_state);

        let task = map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                tab_id,
                SelectedEntity::Extra(0),
                "record_index".into(),
                "99".into(),
            ),
            &mut app,
        );

        assert_eq!(task.units(), 0);
        assert_eq!(
            app.state.editors.map_editors[&tab_id].data.extra_refs[0].record_index, 99,
            "ExtraRef record index updated"
        );
    }

    #[test]
    fn entity_field_changed_collision_tile_does_not_push_undo() {
        let mut app = app_with_map_editor();
        let tab_id = 0;

        let task = map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                tab_id,
                SelectedEntity::CollisionTile(5, 10),
                "collision".into(),
                "true".into(),
            ),
            &mut app,
        );

        assert_eq!(task.units(), 0, "EntityFieldChanged produces no task");
        assert!(
            app.state.editors.map_editors[&tab_id]
                .data
                .undo_stack
                .is_empty(),
            "collision tile field_changed should not push undo"
        );
        assert!(
            app.state.editors.map_editors[&tab_id]
                .data
                .redo_stack
                .is_empty(),
            "redo stack should also be empty"
        );
    }

    #[test]
    fn entity_field_changed_npc_ini_id_updates_value_and_undo_stack() {
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

        // Set game_path so the sprite recompute guard passes
        app.state.workspace.game_path = Some(PathBuf::from("/tmp"));

        let mut map_state = MapEditorState::default();
        map_state.data.npcs = vec![dispel_core::NPC {
            npc_ini_id: 1,
            waypoint1_facing_direction: dispel_core::NpcLookingDirection::Right,
            ..Default::default()
        }];
        app.state.editors.map_editors.insert(tab_id, map_state);

        let task = map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                tab_id,
                SelectedEntity::Npc(0),
                "npc_ini_id".into(),
                "5".into(),
            ),
            &mut app,
        );

        assert_eq!(task.units(), 0, "EntityFieldChanged produces no task");
        assert_eq!(
            app.state.editors.map_editors[&tab_id].data.npcs[0].npc_ini_id, 5,
            "NPC INI ID updated"
        );
        assert_eq!(
            app.state.editors.map_editors[&tab_id].data.undo_stack.len(),
            1,
            "undo stack has one entry"
        );
        assert!(app.state.workspace.tabs[0].modified, "tab marked modified");
    }

    #[test]
    fn entity_multiple_edits_produce_ordered_undo_stack() {
        let mut app = app_with_map_editor();
        let tab_id = 0;

        // Edit 1: map_x 100 → 150
        let _ = map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                tab_id,
                SelectedEntity::Monster(0),
                "map_x".into(),
                "150".into(),
            ),
            &mut app,
        );

        // Edit 2: map_x 150 → 200
        let _ = map_editor::handle(
            MapEditorMessage::EntityFieldChanged(
                tab_id,
                SelectedEntity::Monster(0),
                "map_x".into(),
                "200".into(),
            ),
            &mut app,
        );

        // Undo 1: 200 → 150
        let _ = map_editor::handle(MapEditorMessage::Undo(tab_id), &mut app);
        assert_eq!(
            app.state.editors.map_editors[&tab_id].data.monsters[0].map_x, 150,
            "first undo: 200 → 150"
        );

        // Undo 2: 150 → 100
        let _ = map_editor::handle(MapEditorMessage::Undo(tab_id), &mut app);
        assert_eq!(
            app.state.editors.map_editors[&tab_id].data.monsters[0].map_x, 100,
            "second undo: 150 → 100"
        );

        // Redo 1: 100 → 150
        let _ = map_editor::handle(MapEditorMessage::Redo(tab_id), &mut app);
        assert_eq!(
            app.state.editors.map_editors[&tab_id].data.monsters[0].map_x, 150,
            "first redo: 100 → 150"
        );
    }
}
