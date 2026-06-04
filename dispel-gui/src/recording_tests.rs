//! Tests verifying that all editor types record field changes via the mod packager.
//!
//! Coverage:
//! - `observe_field_change` behaves correctly (no-op vs recording task)
//! - Macro-generated editors (22) — tested via `weapon` representative
//! - Custom editors: `wave_ini`, `store`
//! - Tab-based editors (5) — tested via `npc_ref` representative
//! - Known gap: `chest` editor does NOT record
//!
//! The test strategy uses `Task::units()` to distinguish a no-op `Task::none()`
//! (units = 0) from a `Task::done(RecordingObserved(...))` (units = 1).
//! When recording is OFF the returned `units` must be 0; when ON it must be > 0
//! provided the field value actually changed.

use crate::app::App;

use crate::components::generic_editor::{GenericEditorState, MultiFileEditorState};
use crate::editors::chest::{self, ChestEditorMessage};
use crate::editors::mod_packager::recording::observe_field_change;
use crate::editors::npc_ref::{self, NpcRefEditorMessage};
use crate::editors::store::{self, StoreEditorMessage};
use crate::editors::wave_ini::{self, WaveIniEditorMessage};
use crate::editors::weapon::{self, WeaponEditorMessage};
use crate::state::RecordingSession;
use crate::view::editor::SpreadsheetState;
use crate::workspace::Workspace;
use dispel_core::{ExtraRef, Store, WaveIni, WeaponItem, NPC};
use std::path::PathBuf;

// ============================================================================
// Helpers
// ============================================================================

fn app_with_recording() -> App {
    let mut app = App::test_new(Workspace::new());
    app.state.recording = Some(RecordingSession {
        workspace_root: std::env::temp_dir(),
        mod_slug: "test_mod".to_string(),
        mod_name: "Test Mod".to_string(),
        ..Default::default()
    });
    app
}

fn app_without_recording() -> App {
    App::test_new(Workspace::new())
}

// ============================================================================
// observe_field_change  (core recording primitive)
// ============================================================================

#[test]
fn observe_field_change_returns_none_when_no_session() {
    let app = app_without_recording();
    let task = observe_field_change(&app, "test.db", 0, "name", "old".into(), "new".into());
    assert_eq!(task.units(), 0, "no-op when no recording session");
}

#[test]
fn observe_field_change_returns_task_when_session_active() {
    let app = app_with_recording();
    let task = observe_field_change(&app, "test.db", 0, "name", "old".into(), "new".into());
    assert_eq!(task.units(), 1, "produces task when session active");
}

// ============================================================================
// Macro-generated editor: weapon  (representative of all 22)
// ============================================================================

#[test]
fn weapon_editor_records_when_session_active() {
    let mut app = app_with_recording();
    app.state.editors.weapon_editor.catalog = Some(vec![WeaponItem {
        name: "OldName".into(),
        ..Default::default()
    }]);
    let record = app.state.editors.weapon_editor.catalog.as_ref().unwrap()[0].clone();
    app.state.editors.weapon_editor.filtered = vec![(0, record)];

    let task = weapon::handle(
        WeaponEditorMessage::FieldChanged(0, "name".into(), "NewName".into()),
        &mut app,
    );

    assert!(task.units() > 0, "should record when session is active");
    assert_eq!(
        app.state.editors.weapon_editor.catalog.as_ref().unwrap()[0].name,
        "NewName"
    );
}

#[test]
fn weapon_editor_does_not_record_without_session() {
    let mut app = app_without_recording();
    app.state.editors.weapon_editor.catalog = Some(vec![WeaponItem {
        name: "OldName".into(),
        ..Default::default()
    }]);
    let record = app.state.editors.weapon_editor.catalog.as_ref().unwrap()[0].clone();
    app.state.editors.weapon_editor.filtered = vec![(0, record)];

    let task = weapon::handle(
        WeaponEditorMessage::FieldChanged(0, "name".into(), "NewName".into()),
        &mut app,
    );

    assert_eq!(task.units(), 0, "should NOT record without session");
    assert_eq!(
        app.state.editors.weapon_editor.catalog.as_ref().unwrap()[0].name,
        "NewName"
    );
}

#[test]
fn weapon_editor_no_recording_when_value_unchanged() {
    let mut app = app_with_recording();
    app.state.editors.weapon_editor.catalog = Some(vec![WeaponItem {
        name: "SameName".into(),
        ..Default::default()
    }]);
    let record = app.state.editors.weapon_editor.catalog.as_ref().unwrap()[0].clone();
    app.state.editors.weapon_editor.filtered = vec![(0, record)];

    let task = weapon::handle(
        WeaponEditorMessage::FieldChanged(0, "name".into(), "SameName".into()),
        &mut app,
    );

    assert_eq!(task.units(), 0, "should NOT record when value unchanged");
}

// ============================================================================
// wave_ini  (custom wrapper around StandardEditor)
// ============================================================================

#[test]
fn wave_ini_editor_records_when_session_active() {
    let mut app = app_with_recording();
    app.state.editors.wave_ini_editor.catalog = Some(vec![WaveIni {
        id: 1,
        snf_filename: Some("old.wav".into()),
        ..Default::default()
    }]);
    let record = app.state.editors.wave_ini_editor.catalog.as_ref().unwrap()[0].clone();
    app.state.editors.wave_ini_editor.filtered = vec![(0, record)];

    let task = wave_ini::handle(
        WaveIniEditorMessage::FieldChanged(0, "snf_filename".into(), "new.wav".into()),
        &mut app,
    );

    assert!(
        task.units() > 0,
        "wave_ini should record when session active"
    );
}

#[test]
fn wave_ini_editor_does_not_record_without_session() {
    let mut app = app_without_recording();
    app.state.editors.wave_ini_editor.catalog = Some(vec![WaveIni {
        id: 1,
        snf_filename: Some("old.wav".into()),
        ..Default::default()
    }]);
    let record = app.state.editors.wave_ini_editor.catalog.as_ref().unwrap()[0].clone();
    app.state.editors.wave_ini_editor.filtered = vec![(0, record)];

    let task = wave_ini::handle(
        WaveIniEditorMessage::FieldChanged(0, "snf_filename".into(), "new.wav".into()),
        &mut app,
    );

    assert_eq!(
        task.units(),
        0,
        "wave_ini should NOT record without session"
    );
}

// ============================================================================
// store  (fully custom editor, custom FieldChanged handler)
// ============================================================================

#[test]
fn store_editor_records_when_session_active() {
    let mut app = app_with_recording();
    app.state.editors.store_editor.catalog = Some(vec![Store {
        store_name: "OldShop".into(),
        ..Default::default()
    }]);
    app.state.editors.store_editor.filtered_stores = vec![(
        0,
        Store {
            store_name: "OldShop".into(),
            ..Default::default()
        },
    )];

    let task = store::handle(
        StoreEditorMessage::FieldChanged(0, "store_name".into(), "NewShop".into()),
        &mut app,
    );

    assert!(
        task.units() > 0,
        "store editor should record when session active"
    );
    assert_eq!(
        app.state.editors.store_editor.catalog.as_ref().unwrap()[0].store_name,
        "NewShop"
    );
}

#[test]
fn store_editor_does_not_record_without_session() {
    let mut app = app_without_recording();
    app.state.editors.store_editor.catalog = Some(vec![Store {
        store_name: "OldShop".into(),
        ..Default::default()
    }]);
    app.state.editors.store_editor.filtered_stores = vec![(
        0,
        Store {
            store_name: "OldShop".into(),
            ..Default::default()
        },
    )];

    let task = store::handle(
        StoreEditorMessage::FieldChanged(0, "store_name".into(), "NewShop".into()),
        &mut app,
    );

    assert_eq!(
        task.units(),
        0,
        "store editor should NOT record without session"
    );
    assert_eq!(
        app.state.editors.store_editor.catalog.as_ref().unwrap()[0].store_name,
        "NewShop"
    );
}

// ============================================================================
// npc_ref  (tab-based multi-file editor, representative of all 5)
// ============================================================================

#[test]
fn npc_ref_editor_records_when_session_active() {
    let mut app = app_with_recording();
    let tab_id = usize::MAX; // Workspace has no active tab → get_tab_id returns MAX

    app.state.editors.npc_ref_editor.editors.insert(
        tab_id,
        MultiFileEditorState {
            file_list: vec![],
            current_file: Some(PathBuf::from("NpcInGame/Npccat1.ref")),
            editor: GenericEditorState {
                catalog: Some(vec![NPC {
                    name: "OldNPC".into(),
                    ..Default::default()
                }]),
                filtered: vec![(
                    0,
                    NPC {
                        name: "OldNPC".into(),
                        ..Default::default()
                    },
                )],
                ..Default::default()
            },
        },
    );
    app.state
        .editors.npc_ref_editor
        .spreadsheets
        .insert(tab_id, SpreadsheetState::new());
    app.state.shared_game_path = "/game".into();

    let task = npc_ref::handle(
        NpcRefEditorMessage::FieldChanged(0, "name".into(), "NewNPC".into()),
        &mut app,
    );

    assert!(
        task.units() > 0,
        "npc_ref tab editor should record when session active"
    );
}

#[test]
fn npc_ref_editor_does_not_record_without_session() {
    let mut app = app_without_recording();
    let tab_id = usize::MAX;

    app.state.editors.npc_ref_editor.editors.insert(
        tab_id,
        MultiFileEditorState {
            file_list: vec![],
            current_file: Some(PathBuf::from("NpcInGame/Npccat1.ref")),
            editor: GenericEditorState {
                catalog: Some(vec![NPC {
                    name: "OldNPC".into(),
                    ..Default::default()
                }]),
                filtered: vec![(
                    0,
                    NPC {
                        name: "OldNPC".into(),
                        ..Default::default()
                    },
                )],
                ..Default::default()
            },
        },
    );
    app.state
        .editors.npc_ref_editor
        .spreadsheets
        .insert(tab_id, SpreadsheetState::new());
    app.state.shared_game_path = "/game".into();

    let task = npc_ref::handle(
        NpcRefEditorMessage::FieldChanged(0, "name".into(), "NewNPC".into()),
        &mut app,
    );

    assert_eq!(
        task.units(),
        0,
        "npc_ref tab editor should NOT record without session"
    );
}

#[test]
fn npc_ref_editor_no_recording_when_captured_context_missing() {
    let mut app = app_with_recording();
    // Do NOT insert an editor — capture_field_recording_context returns None

    let task = npc_ref::handle(
        NpcRefEditorMessage::FieldChanged(0, "name".into(), "NewNPC".into()),
        &mut app,
    );

    assert_eq!(
        task.units(),
        0,
        "should NOT record when no MultiFileEditorState is present"
    );
}

// ============================================================================
// Chest  (known gap — no recording wired)
// ============================================================================

#[test]
fn chest_editor_does_not_record() {
    let mut app = app_with_recording();
    app.state.editors.chest_editor.all_records = vec![ExtraRef {
        name: "OldChest".into(),
        ..Default::default()
    }];

    let task = chest::handle(
        ChestEditorMessage::FieldChanged(0, "name".into(), "NewChest".into()),
        &mut app,
    );

    assert_eq!(
        task.units(),
        0,
        "chest editor is a known gap — no recording wired"
    );
}
