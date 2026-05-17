use super::*;
use crate::components::edit_history::EditAction;
use crate::components::editable::EditableRecord;
use dispel_core::{EditItem, EventItem, HealItem, MiscItem, Monster, PartyRef, WeaponItem};

#[test]
fn test_weapon_editor_migration() {
    let mut editor = GenericEditorState::<WeaponItem>::default();

    assert!(editor.catalog.is_none());
    assert!(editor.filtered.is_empty());
    assert!(editor.selected_idx.is_none());

    let mut catalog = Vec::new();
    let mut weapon = WeaponItem::default();
    weapon.name = "Test Sword".to_string();
    weapon.base_price = 100;
    catalog.push(weapon);

    editor.catalog = Some(catalog);
    editor.refresh();

    assert_eq!(editor.filtered.len(), 1);
    assert_eq!(editor.filtered[0].1.name, "Test Sword");
}

#[test]
fn test_monster_editor_migration() {
    let mut editor = GenericEditorState::<Monster>::default();

    assert!(editor.catalog.is_none());
    assert!(editor.filtered.is_empty());
    assert!(editor.selected_idx.is_none());

    let mut catalog = Vec::new();
    let mut monster = Monster::default();
    monster.name = "Test Monster".to_string();
    monster.health_points_max = 50;
    catalog.push(monster);

    editor.catalog = Some(catalog);
    editor.refresh();

    assert_eq!(editor.filtered.len(), 1);
    assert_eq!(editor.filtered[0].1.name, "Test Monster");
}

#[test]
fn test_simple_editor_migrations() {
    let editor1 = GenericEditorState::<EditItem>::default();
    assert!(editor1.catalog.is_none());
    assert!(editor1.filtered.is_empty());

    let editor2 = GenericEditorState::<EventItem>::default();
    assert!(editor2.catalog.is_none());
    assert!(editor2.filtered.is_empty());

    let editor3 = GenericEditorState::<HealItem>::default();
    assert!(editor3.catalog.is_none());
    assert!(editor3.filtered.is_empty());

    let editor4 = GenericEditorState::<MiscItem>::default();
    assert!(editor4.catalog.is_none());
    assert!(editor4.filtered.is_empty());

    let editor5 = GenericEditorState::<PartyRef>::default();
    assert!(editor5.catalog.is_none());
    assert!(editor5.filtered.is_empty());
}

#[test]
fn test_editor_selection_functionality() {
    let mut editor = GenericEditorState::<WeaponItem>::default();

    let mut catalog = Vec::new();
    for i in 0..3 {
        let mut weapon = WeaponItem::default();
        weapon.name = format!("Weapon {}", i);
        weapon.base_price = 100 + i * 10;
        catalog.push(weapon);
    }

    editor.catalog = Some(catalog);
    editor.refresh();

    editor.select(1);

    assert_eq!(editor.selected_idx, Some(1));
    assert_eq!(
        editor.edit_buffers.len(),
        WeaponItem::field_descriptors().len()
    );

    let name_pos = WeaponItem::field_descriptors()
        .iter()
        .position(|d| d.name == "name")
        .expect("name field should exist");

    assert_eq!(editor.edit_buffers[name_pos], "Weapon 1");
}

#[test]
fn test_editor_field_update_functionality() {
    let mut editor = GenericEditorState::<WeaponItem>::default();

    let mut catalog = Vec::new();
    let mut weapon = WeaponItem::default();
    weapon.name = "Original Name".to_string();
    weapon.base_price = 100;
    catalog.push(weapon);

    editor.catalog = Some(catalog);
    editor.refresh();
    editor.select(0);

    let result = editor.update_field(0, "name", "Updated Name".to_string());
    assert!(result, "Field update should succeed");

    if let Some(catalog) = &editor.catalog {
        assert_eq!(catalog[0].name, "Updated Name");
    } else {
        panic!("Catalog should exist");
    }

    assert_eq!(editor.filtered[0].1.name, "Updated Name");

    let name_pos = WeaponItem::field_descriptors()
        .iter()
        .position(|d| d.name == "name")
        .expect("name field should exist");
    assert_eq!(editor.edit_buffers[name_pos], "Updated Name");
}

#[test]
fn test_field_validation_works() {
    let mut editor = GenericEditorState::<WeaponItem>::default();

    let mut catalog = Vec::new();
    let weapon = WeaponItem::default();
    catalog.push(weapon);

    editor.catalog = Some(catalog);
    editor.refresh();
    editor.select(0);

    let result = editor.update_field(0, "base_price", "250".to_string());
    assert!(result, "Valid integer field should be accepted");

    let result = editor.update_field(0, "base_price", "not_a_number".to_string());
    assert!(!result, "Invalid integer field should be rejected");

    let result = editor.update_field(0, "name", "Valid Name".to_string());
    assert!(result, "Valid string field should be accepted");
}

#[test]
fn test_update_field_by_catalog_index() {
    use dispel_core::WeaponItem;
    let mut editor = GenericEditorState::<WeaponItem>::default();
    let mut catalog = Vec::new();
    for i in 0..5 {
        let mut w = WeaponItem::default();
        w.name = format!("Weapon {}", i);
        catalog.push(w);
    }
    editor.catalog = Some(catalog);
    editor.refresh();

    let r = editor.update_field(3, "name", "Updated".to_string());
    assert!(r);
    assert_eq!(editor.catalog.as_ref().unwrap()[3].name, "Updated");
    assert_eq!(
        editor
            .filtered
            .iter()
            .find(|(i, _)| *i == 3)
            .unwrap()
            .1
            .name,
        "Updated"
    );
    assert!(editor.edit_history.can_undo());
}

#[test]
fn test_update_field_works_despite_non_matching_filtered_position() {
    use dispel_core::WeaponItem;
    let mut editor = GenericEditorState::<WeaponItem>::default();
    let mut catalog = Vec::new();
    for i in 0..5 {
        let mut w = WeaponItem::default();
        w.name = format!("Weapon {}", i);
        catalog.push(w);
    }
    editor.catalog = Some(catalog);
    editor.refresh();

    editor.filtered.remove(0);

    let r = editor.update_field(3, "name", "Patched".to_string());
    assert!(r);
    assert_eq!(editor.catalog.as_ref().unwrap()[3].name, "Patched");

    let r2 = editor.update_field(3, "name", "Again".to_string());
    assert!(r2);
    assert_eq!(editor.catalog.as_ref().unwrap()[3].name, "Again");
}

#[test]
fn test_undo_redo_remove_record() {
    use dispel_core::WeaponItem;
    let mut editor = GenericEditorState::<WeaponItem>::default();
    let mut catalog = Vec::new();
    for i in 0..3 {
        let mut w = WeaponItem::default();
        w.name = format!("Weapon {}", i);
        catalog.push(w);
    }
    editor.catalog = Some(catalog);
    editor.refresh();

    let record = editor
        .filtered
        .iter()
        .find(|(i, _)| *i == 1)
        .unwrap()
        .1
        .clone();
    let data = serde_json::to_string(&record).unwrap();
    editor.edit_history.adjust_for_removal(1);
    editor.edit_history.push(EditAction::RecordRemove {
        record_idx: 1,
        data,
    });
    editor.catalog.as_mut().unwrap().remove(1);
    editor.refresh();

    assert_eq!(editor.catalog.as_ref().unwrap().len(), 2);
    assert_eq!(editor.catalog.as_ref().unwrap()[1].name, "Weapon 2");

    let msg = editor.undo();
    let msg = msg.expect("undo should return Some message");
    assert!(msg.starts_with("Undo: restored"));
    assert_eq!(editor.catalog.as_ref().unwrap().len(), 3);
    assert_eq!(editor.catalog.as_ref().unwrap()[1].name, "Weapon 1");

    let msg = editor.redo();
    assert!(msg.unwrap().starts_with("Redo: removed"));
    assert_eq!(editor.catalog.as_ref().unwrap().len(), 2);
}

#[test]
fn test_undo_adjusts_history_after_remove() {
    use dispel_core::WeaponItem;
    let mut editor = GenericEditorState::<WeaponItem>::default();
    let mut catalog = Vec::new();
    for i in 0..4 {
        let mut w = WeaponItem::default();
        w.name = format!("Weapon {}", i);
        catalog.push(w);
    }
    editor.catalog = Some(catalog);
    editor.refresh();

    editor.update_field(3, "name", "Edited".to_string());
    assert!(editor.edit_history.can_undo());

    let record = editor
        .filtered
        .iter()
        .find(|(i, _)| *i == 1)
        .unwrap()
        .1
        .clone();
    let data = serde_json::to_string(&record).unwrap();
    editor.edit_history.adjust_for_removal(1);
    editor.edit_history.push(EditAction::RecordRemove {
        record_idx: 1,
        data,
    });
    editor.catalog.as_mut().unwrap().remove(1);
    editor.refresh();

    let stack = editor.edit_history.undo_stack();
    assert_eq!(stack.len(), 2, "should have RecordRemove + FieldChange");
    match &stack[1] {
        EditAction::FieldChange {
            record_idx, field, ..
        } => {
            assert_eq!(*record_idx, 2, "index should be decremented after removal");
            assert_eq!(field, "name");
        }
        _ => panic!("expected FieldChange at back of stack"),
    }
    match &stack[0] {
        EditAction::RecordRemove { record_idx, .. } => {
            assert_eq!(*record_idx, 1);
        }
        _ => panic!("expected RecordRemove at front of stack"),
    }
}

#[test]
fn test_edit_history_adjust_for_addition() {
    let mut history = EditHistory::new();
    history.push(EditAction::FieldChange {
        record_idx: 0,
        field: "f".into(),
        old_value: "a".into(),
        new_value: "b".into(),
    });
    history.push(EditAction::FieldChange {
        record_idx: 2,
        field: "f".into(),
        old_value: "c".into(),
        new_value: "d".into(),
    });
    history.push(EditAction::FieldChange {
        record_idx: 5,
        field: "f".into(),
        old_value: "e".into(),
        new_value: "f".into(),
    });

    history.adjust_for_addition(2);

    let stack = history.undo_stack();
    assert_eq!(stack[0].record_idx(), 6);
    assert_eq!(stack[1].record_idx(), 3);
    assert_eq!(stack[2].record_idx(), 0);
}

#[test]
fn test_edit_history_adjust_for_removal_drops_matching() {
    let mut history = EditHistory::new();
    history.push(EditAction::FieldChange {
        record_idx: 0,
        field: "f".into(),
        old_value: "a".into(),
        new_value: "b".into(),
    });
    history.push(EditAction::RecordRemove {
        record_idx: 2,
        data: "{}".into(),
    });
    history.push(EditAction::FieldChange {
        record_idx: 5,
        field: "f".into(),
        old_value: "c".into(),
        new_value: "d".into(),
    });

    history.adjust_for_removal(2);

    let stack = history.undo_stack();
    assert_eq!(stack.len(), 2);
    assert_eq!(stack[0].record_idx(), 4);
    assert_eq!(stack[1].record_idx(), 0);
}
