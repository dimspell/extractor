//! Chest editor integration tests.
//!
//! Tests the handle() function for ChestEditorMessage variants,
//! focusing on state mutations: selection, field changes, and
//! edge cases (OOB indices, parse failures, unknown fields).

#[cfg(test)]
mod chest_editor_tests {
    use crate::app::App;
    use crate::editors::chest::ChestEditorMessage;
    use crate::workspace::Workspace;
    use dispel_core::{ExtraObjectType, ExtraRef};

    // ── Helpers ─────────────────────────────────────────────────────────────

    /// Create an App with chest editor all_records pre-populated and
    /// refresh_chests called to build filtered_chests.
    fn app_with_records(records: Vec<ExtraRef>) -> App {
        let mut app = App::test_new(Workspace::new());
        app.state.editors.chest_editor.all_records = records;
        app.refresh_chests();
        app
    }

    // ── SelectChest ────────────────────────────────────────────────────────

    #[test]
    fn test_chest_editor_select_chest() {
        let mut app = app_with_records(vec![
            ExtraRef {
                id: 1,
                name: "Chest A".into(),
                object_type: ExtraObjectType::Chest,
                x_pos: 10,
                y_pos: 20,
                gold_amount: 100,
                item_count: 3,
                item_id: 5,
                ..Default::default()
            },
            ExtraRef {
                id: 2,
                name: "Chest B".into(),
                object_type: ExtraObjectType::Chest,
                x_pos: 30,
                y_pos: 40,
                gold_amount: 200,
                item_count: 1,
                item_id: 9,
                ..Default::default()
            },
            ExtraRef {
                id: 3,
                name: "Chest C".into(),
                object_type: ExtraObjectType::Chest,
                x_pos: 50,
                y_pos: 60,
                gold_amount: 0,
                item_count: 7,
                item_id: 2,
                ..Default::default()
            },
        ]);

        assert_eq!(
            app.state.editors.chest_editor.filtered_chests.len(),
            3,
            "setup: all three records are chests"
        );

        let _task = crate::editors::chest::handle(ChestEditorMessage::SelectChest(1), &mut app);

        assert_eq!(
            app.state.editors.chest_editor.selected_idx,
            Some(1),
            "selected_idx updated"
        );
        assert_eq!(
            app.state.editors.chest_editor.edit_name, "Chest B",
            "edit_name from filtered_chest[1]"
        );
        assert_eq!(app.state.editors.chest_editor.edit_x, "30", "edit_x");
        assert_eq!(app.state.editors.chest_editor.edit_y, "40", "edit_y");
        assert_eq!(app.state.editors.chest_editor.edit_gold, "200", "edit_gold");
        assert_eq!(
            app.state.editors.chest_editor.edit_item_count, "1",
            "edit_item_count"
        );
        assert_eq!(
            app.state.editors.chest_editor.edit_item_id, "9",
            "edit_item_id"
        );
    }

    #[test]
    fn test_chest_editor_select_chest_oob_sets_idx_but_no_fields() {
        let mut app = app_with_records(vec![ExtraRef {
            id: 1,
            name: "Lone Chest".into(),
            object_type: ExtraObjectType::Chest,
            ..Default::default()
        }]);

        assert_eq!(app.state.editors.chest_editor.selected_idx, None);

        let _task = crate::editors::chest::handle(ChestEditorMessage::SelectChest(999), &mut app);

        // selected_idx is set unconditionally
        assert_eq!(app.state.editors.chest_editor.selected_idx, Some(999));
        // But no edit fields are populated because filtered_chests.get(999) fails
        assert_eq!(
            app.state.editors.chest_editor.edit_name, "",
            "edit_name stays empty for OOB index"
        );
    }

    // ── FieldChanged ──────────────────────────────────────────────────────

    #[test]
    fn test_chest_editor_field_changed_name() {
        let mut app = app_with_records(vec![ExtraRef {
            name: "Old Name".into(),
            object_type: ExtraObjectType::Chest,
            ..Default::default()
        }]);

        let _task = crate::editors::chest::handle(
            ChestEditorMessage::FieldChanged(0, "name".into(), "New Name".into()),
            &mut app,
        );

        assert_eq!(
            app.state.editors.chest_editor.edit_name, "New Name",
            "edit_name string buffer updated"
        );
        assert_eq!(
            app.state.editors.chest_editor.all_records[0].name, "New Name",
            "all_records name updated"
        );
    }

    #[test]
    fn test_chest_editor_field_changed_x() {
        let mut app = app_with_records(vec![ExtraRef {
            x_pos: 10,
            object_type: ExtraObjectType::Chest,
            ..Default::default()
        }]);

        let _task = crate::editors::chest::handle(
            ChestEditorMessage::FieldChanged(0, "x".into(), "42".into()),
            &mut app,
        );

        assert_eq!(
            app.state.editors.chest_editor.edit_x, "42",
            "edit_x string buffer updated"
        );
        assert_eq!(
            app.state.editors.chest_editor.all_records[0].x_pos, 42,
            "record x_pos updated"
        );
    }

    #[test]
    fn test_chest_editor_field_changed_y() {
        let mut app = app_with_records(vec![ExtraRef {
            y_pos: 50,
            object_type: ExtraObjectType::Chest,
            ..Default::default()
        }]);

        let _task = crate::editors::chest::handle(
            ChestEditorMessage::FieldChanged(0, "y".into(), "123".into()),
            &mut app,
        );

        assert_eq!(app.state.editors.chest_editor.edit_y, "123");
        assert_eq!(app.state.editors.chest_editor.all_records[0].y_pos, 123);
    }

    #[test]
    fn test_chest_editor_field_changed_gold() {
        let mut app = app_with_records(vec![ExtraRef {
            gold_amount: 0,
            object_type: ExtraObjectType::Chest,
            ..Default::default()
        }]);

        let _task = crate::editors::chest::handle(
            ChestEditorMessage::FieldChanged(0, "gold".into(), "9999".into()),
            &mut app,
        );

        assert_eq!(app.state.editors.chest_editor.edit_gold, "9999");
        assert_eq!(
            app.state.editors.chest_editor.all_records[0].gold_amount,
            9999
        );
    }

    #[test]
    fn test_chest_editor_field_changed_item_count() {
        let mut app = app_with_records(vec![ExtraRef {
            item_count: 0,
            object_type: ExtraObjectType::Chest,
            ..Default::default()
        }]);

        let _task = crate::editors::chest::handle(
            ChestEditorMessage::FieldChanged(0, "item_count".into(), "5".into()),
            &mut app,
        );

        assert_eq!(app.state.editors.chest_editor.edit_item_count, "5");
        assert_eq!(app.state.editors.chest_editor.all_records[0].item_count, 5);
    }

    #[test]
    fn test_chest_editor_field_changed_item_id() {
        let mut app = app_with_records(vec![ExtraRef {
            item_id: 0,
            object_type: ExtraObjectType::Chest,
            ..Default::default()
        }]);

        let _task = crate::editors::chest::handle(
            ChestEditorMessage::FieldChanged(0, "item_id".into(), "77".into()),
            &mut app,
        );

        assert_eq!(app.state.editors.chest_editor.edit_item_id, "77");
        assert_eq!(app.state.editors.chest_editor.all_records[0].item_id, 77);
    }

    #[test]
    fn test_chest_editor_field_changed_invalid_integer_does_not_update_record() {
        let mut app = app_with_records(vec![ExtraRef {
            x_pos: 10,
            object_type: ExtraObjectType::Chest,
            ..Default::default()
        }]);

        let _task = crate::editors::chest::handle(
            ChestEditorMessage::FieldChanged(0, "x".into(), "not-a-number".into()),
            &mut app,
        );

        // The string buffer is updated regardless
        assert_eq!(app.state.editors.chest_editor.edit_x, "not-a-number");
        // The record stays unchanged because parse<i32>() fails
        assert_eq!(
            app.state.editors.chest_editor.all_records[0].x_pos, 10,
            "x_pos unchanged on parse failure"
        );
    }

    #[test]
    fn test_chest_editor_field_change_oob_updates_buffer_only() {
        let mut app = app_with_records(vec![ExtraRef {
            name: "KeepMe".into(),
            object_type: ExtraObjectType::Chest,
            ..Default::default()
        }]);

        let _task = crate::editors::chest::handle(
            ChestEditorMessage::FieldChanged(999, "name".into(), "Changed".into()),
            &mut app,
        );

        // The edit_name string buffer is updated unconditionally
        assert_eq!(app.state.editors.chest_editor.edit_name, "Changed");
        // But all_records is not touched because orig_idx 999 is OOB
        assert_eq!(
            app.state.editors.chest_editor.all_records[0].name, "KeepMe",
            "record unchanged for OOB index"
        );
    }

    #[test]
    fn test_chest_editor_unknown_field_is_silently_ignored() {
        let mut app = app_with_records(vec![ExtraRef {
            id: 42,
            object_type: ExtraObjectType::Chest,
            ..Default::default()
        }]);

        let _task = crate::editors::chest::handle(
            ChestEditorMessage::FieldChanged(0, "nonexistent_field".into(), "value".into()),
            &mut app,
        );

        // Unknown fields are handled by the catch-all `_ => {}` arm — no crash
        assert_eq!(
            app.state.editors.chest_editor.all_records[0].id, 42,
            "record unchanged by unknown field"
        );
    }

    // ── ScanMaps ──────────────────────────────────────────────────────────

    #[test]
    fn test_chest_editor_scan_maps_no_game_path_shows_error() {
        let mut app = App::test_new(Workspace::new());
        // shared_game_path is empty by default

        let task = crate::editors::chest::handle(ChestEditorMessage::ScanMaps, &mut app);

        assert_eq!(
            app.state.editors.chest_editor.status_msg, "Please select game path first.",
            "error message set when game path is empty"
        );
        assert_eq!(task.units(), 0, "no Task produced for missing game path");
    }

    // ── Add / Delete ──────────────────────────────────────────────────────

    #[test]
    fn test_chest_editor_add_is_noop() {
        let mut app = app_with_records(vec![ExtraRef {
            id: 1,
            object_type: ExtraObjectType::Chest,
            ..Default::default()
        }]);

        let count_before = app.state.editors.chest_editor.all_records.len();

        let _task = crate::editors::chest::handle(ChestEditorMessage::Add, &mut app);

        assert_eq!(
            app.state.editors.chest_editor.all_records.len(),
            count_before,
            "Add is currently a no-op"
        );
    }

    #[test]
    fn test_chest_editor_delete_is_noop() {
        let mut app = app_with_records(vec![ExtraRef {
            id: 1,
            object_type: ExtraObjectType::Chest,
            ..Default::default()
        }]);

        let count_before = app.state.editors.chest_editor.all_records.len();

        let _task = crate::editors::chest::handle(ChestEditorMessage::Delete(0), &mut app);

        assert_eq!(
            app.state.editors.chest_editor.all_records.len(),
            count_before,
            "Delete is currently a no-op"
        );
    }

    // ── Save guard: no current_map_file → no-op ──────────────────────────

    #[test]
    fn test_chest_editor_save_without_map_file_is_noop() {
        let mut app = app_with_records(vec![ExtraRef {
            id: 1,
            object_type: ExtraObjectType::Chest,
            ..Default::default()
        }]);

        let task = crate::editors::chest::handle(ChestEditorMessage::Save, &mut app);

        assert_eq!(
            task.units(),
            0,
            "Save without current_map_file produces no task"
        );
    }
}
