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
use crate::editors::mod_packager;
use crate::editors::mod_packager::recording::{observe_field_change, ObservedAction};
use crate::editors::mod_packager::ModPackagerMessage;
use crate::editors::npc_ref::{self, NpcRefEditorMessage};
use crate::editors::store::{self, StoreEditorMessage};
use crate::editors::wave_ini::{self, WaveIniEditorMessage};
use crate::editors::weapon::{self, WeaponEditorMessage};
use crate::state::{RecordingKey, RecordingSession};
use crate::view::editor::SpreadsheetState;
use crate::workspace::Workspace;
use dispel_core::modding::Value;
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
// RecordingObserved — debounce generation mechanics
// ============================================================================

fn make_key(file_path: &str, record_id: u32, field: &str) -> RecordingKey {
    RecordingKey {
        file_path: file_path.into(),
        record_id,
        field: field.into(),
    }
}

fn make_observed(key: RecordingKey, old: &str, new: &str) -> ObservedAction {
    ObservedAction {
        key,
        old: Value::String(old.into()),
        new: Value::String(new.into()),
    }
}

#[test]
fn recording_observed_bumps_generation_one_edit() {
    let mut app = app_with_recording();
    assert_eq!(app.state.recording.as_ref().unwrap().next_generation, 0);

    let task = mod_packager::handle(
        ModPackagerMessage::RecordingObserved(make_observed(
            make_key("test.db", 0, "name"),
            "old",
            "new",
        )),
        &mut app,
    );

    assert_eq!(app.state.recording.as_ref().unwrap().next_generation, 1);
    assert_eq!(app.state.recording.as_ref().unwrap().pending.len(), 1);
    assert!(
        task.units() > 0,
        "debounce timer task produced when session active"
    );
}

#[test]
fn recording_observed_accumulates_distinct_keys() {
    let mut app = app_with_recording();
    let keys = ["name", "desc", "price"];

    for (i, field) in keys.iter().enumerate() {
        let task = mod_packager::handle(
            ModPackagerMessage::RecordingObserved(make_observed(
                make_key("test.db", 0, field),
                "old",
                "new",
            )),
            &mut app,
        );
        assert!(task.units() > 0, "timer produced for key {}", i);
    }

    assert_eq!(
        app.state.recording.as_ref().unwrap().pending.len(),
        3,
        "three distinct keys accumulate"
    );
    assert_eq!(
        app.state.recording.as_ref().unwrap().next_generation,
        3,
        "generation bumped per observed edit"
    );
}

#[test]
fn recording_observed_distinct_record_ids_are_distinct_keys() {
    let mut app = app_with_recording();

    for id in 0..5 {
        let task = mod_packager::handle(
            ModPackagerMessage::RecordingObserved(make_observed(
                make_key("test.db", id, "name"),
                "old",
                "new",
            )),
            &mut app,
        );
        assert!(task.units() > 0);
    }

    assert_eq!(
        app.state.recording.as_ref().unwrap().pending.len(),
        5,
        "different record_ids produce different keys"
    );
}

#[test]
fn recording_observed_distinct_file_paths_are_distinct_keys() {
    let mut app = app_with_recording();

    let task = mod_packager::handle(
        ModPackagerMessage::RecordingObserved(make_observed(
            make_key("weaponItem.db", 0, "name"),
            "old",
            "new",
        )),
        &mut app,
    );
    assert!(task.units() > 0);
    let task = mod_packager::handle(
        ModPackagerMessage::RecordingObserved(make_observed(
            make_key("Monster.db", 0, "name"),
            "old",
            "new",
        )),
        &mut app,
    );
    assert!(task.units() > 0);

    assert_eq!(
        app.state.recording.as_ref().unwrap().pending.len(),
        2,
        "different file paths produce different keys"
    );
}

#[test]
fn recording_observed_same_key_updates_in_place() {
    let mut app = app_with_recording();
    let key = make_key("test.db", 0, "name");

    // First edit: old→new1
    let task = mod_packager::handle(
        ModPackagerMessage::RecordingObserved(make_observed(key.clone(), "original", "new1")),
        &mut app,
    );
    assert!(task.units() > 0);
    assert_eq!(app.state.recording.as_ref().unwrap().next_generation, 1);

    // Second edit: same key, new2 (supersedes)
    let task = mod_packager::handle(
        ModPackagerMessage::RecordingObserved(make_observed(key.clone(), "original", "new2")),
        &mut app,
    );
    assert!(task.units() > 0);
    assert_eq!(app.state.recording.as_ref().unwrap().next_generation, 2);

    // original_old is preserved from the first insert; latest_new is the latest value
    let pending = app.state.recording.as_ref().unwrap().pending.get(&key);
    assert!(pending.is_some(), "entry still in pending");
    let pending = pending.unwrap();
    assert_eq!(
        pending.original_old,
        Value::String("original".into()),
        "original_old preserved across rapid edits"
    );
    assert_eq!(
        pending.latest_new,
        Value::String("new2".into()),
        "latest_new is the most recent value"
    );
    assert_eq!(
        pending.generation, 2,
        "generation matches latest edit"
    );
    assert_eq!(
        app.state.recording.as_ref().unwrap().pending.len(),
        1,
        "only one entry for the key"
    );
}

// ============================================================================
// RecordingDebounceFired — stale timer detection & flush
// ============================================================================

#[test]
fn recording_debounce_fired_stale_generation_dropped() {
    let mut app = app_with_recording();
    let key = make_key("test.db", 0, "name");

    // Insert pending entry with gen=2 (simulate rapid edits)
    let task = mod_packager::handle(
        ModPackagerMessage::RecordingObserved(make_observed(key.clone(), "old", "new")),
        &mut app,
    );
    assert!(task.units() > 0);
    // Generation is now 1

    // Fire with stale generation 0 (which is < current gen 1, so stale)
    let stale_task = mod_packager::handle(
        ModPackagerMessage::RecordingDebounceFired {
            key: key.clone(),
            generation: 0,
        },
        &mut app,
    );

    assert_eq!(
        stale_task.units(),
        0,
        "stale timer dropped — no task produced"
    );
    // Entry still in pending (not removed by stale timer)
    assert!(
        app.state.recording.as_ref().unwrap().pending.contains_key(&key),
        "stale timer does not remove pending entry"
    );
}

#[test]
fn recording_debounce_fired_matching_generation_produces_task() {
    let mut app = app_with_recording();
    let key = make_key("test.db", 0, "name");

    // Insert a pending entry
    let task = mod_packager::handle(
        ModPackagerMessage::RecordingObserved(make_observed(key.clone(), "old", "new")),
        &mut app,
    );
    assert!(task.units() > 0);
    let gen = app.state.recording.as_ref().unwrap().next_generation; // 1

    // Fire with matching generation — should produce a flush task
    let flush_task = mod_packager::handle(
        ModPackagerMessage::RecordingDebounceFired {
            key: key.clone(),
            generation: gen,
        },
        &mut app,
    );

    // The flush task will attempt disk I/O (Workspace::open) but in a sync
    // test it just sits as an Iced Task — we verify it was produced.
    assert!(
        flush_task.units() > 0,
        "matching generation produces flush task"
    );
    // Pending entry removed before the async task runs
    assert!(
        !app.state.recording.as_ref().unwrap().pending.contains_key(&key),
        "pending entry removed during flush"
    );
}

#[test]
fn recording_debounce_fired_nonexistent_key_is_noop() {
    let mut app = app_with_recording();

    let task = mod_packager::handle(
        ModPackagerMessage::RecordingDebounceFired {
            key: make_key("nonexistent.db", 99, "never_set"),
            generation: 0,
        },
        &mut app,
    );

    assert_eq!(
        task.units(),
        0,
        "nonexistent key — no task produced"
    );
}

#[test]
fn recording_debounce_fired_multiple_distinct_keys() {
    let mut app = app_with_recording();
    let key_a = make_key("a.db", 0, "name");
    let key_b = make_key("b.db", 0, "name");

    let _ = mod_packager::handle(
        ModPackagerMessage::RecordingObserved(make_observed(key_a.clone(), "old", "new")),
        &mut app,
    );
    let _ = mod_packager::handle(
        ModPackagerMessage::RecordingObserved(make_observed(key_b.clone(), "old", "new")),
        &mut app,
    );

    // Flush key_a — only key_a removed, key_b stays
    let task = mod_packager::handle(
        ModPackagerMessage::RecordingDebounceFired {
            key: key_a.clone(),
            generation: 1,
        },
        &mut app,
    );
    assert!(task.units() > 0, "flush task for key_a");

    assert!(
        !app.state.recording.as_ref().unwrap().pending.contains_key(&key_a),
        "key_a removed from pending"
    );
    assert!(
        app.state.recording.as_ref().unwrap().pending.contains_key(&key_b),
        "key_b still pending"
    );
    assert_eq!(
        app.state.recording.as_ref().unwrap().pending.len(),
        1,
        "only key_b remains"
    );
}

#[test]
fn recording_debounce_fired_without_session_is_noop() {
    let mut app = app_without_recording();

    let task = mod_packager::handle(
        ModPackagerMessage::RecordingDebounceFired {
            key: make_key("test.db", 0, "name"),
            generation: 0,
        },
        &mut app,
    );

    assert_eq!(
        task.units(),
        0,
        "no-op when no recording session"
    );
}

// ============================================================================
// RecordingObserved — no-session guard
// ============================================================================

#[test]
fn recording_observed_without_session_is_noop() {
    let mut app = app_without_recording();

    let task = mod_packager::handle(
        ModPackagerMessage::RecordingObserved(make_observed(
            make_key("test.db", 0, "name"),
            "old",
            "new",
        )),
        &mut app,
    );

    assert_eq!(
        task.units(),
        0,
        "no-op when no recording session"
    );
}

// ============================================================================
// Flush no-op suppression (pending.original_old == pending.latest_new)
// ============================================================================

#[test]
fn recording_debounce_fired_noop_edit_discarded() {
    let mut app = app_with_recording();
    let key = make_key("test.db", 0, "name");

    // Insert pending where old == new (user typed something then reverted)
    let _ = mod_packager::handle(
        ModPackagerMessage::RecordingObserved(make_observed(key.clone(), "same", "same")),
        &mut app,
    );
    let gen = app.state.recording.as_ref().unwrap().next_generation;

    // Flush with matching generation — should detect old==new and discard
    let task = mod_packager::handle(
        ModPackagerMessage::RecordingDebounceFired {
            key: key.clone(),
            generation: gen,
        },
        &mut app,
    );

    assert_eq!(
        task.units(),
        0,
        "no-op edit discarded — no flush task"
    );
    assert!(
        !app.state.recording.as_ref().unwrap().pending.contains_key(&key),
        "no-op entry removed from pending"
    );
}

// ============================================================================
// StopRecording — lifecycle
// ============================================================================

#[test]
fn recording_stop_recording_clears_session() {
    let mut app = app_with_recording();

    let task = mod_packager::handle(ModPackagerMessage::StopRecording, &mut app);

    assert!(
        app.state.recording.is_none(),
        "session cleared after stop"
    );
    assert!(
        task.units() > 0,
        "stop recording produces a task (Refresh after flush)"
    );
}

#[test]
fn recording_stop_recording_tracks_committed_count() {
    let mut app = app_with_recording();
    // Simulate previous committed persist
    app.state.recording.as_mut().unwrap().recorded_count = 5;

    let task = mod_packager::handle(ModPackagerMessage::StopRecording, &mut app);

    assert!(app.state.recording.is_none());
    assert!(task.units() > 0);
    assert!(
        app.state
            .editors
            .mod_packager_editor
            .status_msg
            .contains("5"),
        "status message includes committed count"
    );
}

#[test]
fn recording_stop_recording_no_session_does_not_set_status() {
    let mut app = app_without_recording();

    let task = mod_packager::handle(ModPackagerMessage::StopRecording, &mut app);

    // Even without a session, StopRecording chains a Refresh task to update UI.
    // The real assertion is that the status message is NOT set (stopped_name empty).
    assert!(task.units() > 0, "Refresh task produced");
    assert!(
        app.state.recording.is_none(),
        "recording remains None"
    );
    assert!(
        app.state.editors.mod_packager_editor.status_msg.is_empty(),
        "no status message when no session"
    );
}

#[test]
fn recording_stop_recording_clears_session_with_pending_entries() {
    let mut app = app_with_recording();

    // Add a pending entry
    let _ = mod_packager::handle(
        ModPackagerMessage::RecordingObserved(make_observed(
            make_key("test.db", 0, "name"),
            "old",
            "new",
        )),
        &mut app,
    );
    assert_eq!(app.state.recording.as_ref().unwrap().pending.len(), 1);

    let task = mod_packager::handle(ModPackagerMessage::StopRecording, &mut app);
    assert!(task.units() > 0);

    // Session is cleared
    assert!(app.state.recording.is_none(), "session cleared");
    // Status message mentions the mod name
    let msg = &app.state.editors.mod_packager_editor.status_msg;
    assert!(
        msg.contains("Test Mod"),
        "status mentions mod name: {msg}"
    );
}

// ============================================================================
// StartRecording — lifecycle
// ============================================================================

#[test]
fn recording_start_recording_without_workspace_shows_error() {
    let mut app = app_with_recording();
    // Ensure workspace_root is None
    app.state.editors.mod_packager_editor.workspace_root = None;

    let task = mod_packager::handle(
        ModPackagerMessage::StartRecording("test_mod".into()),
        &mut app,
    );

    assert_eq!(
        task.units(),
        0,
        "no task when workspace is missing"
    );
    assert!(
        app.state
            .editors
            .mod_packager_editor
            .status_msg
            .contains("Open a workspace first"),
        "shows error message"
    );
}

#[test]
fn recording_start_recording_initializes_session() {
    let mut app = App::test_new(Workspace::new());
    app.state.editors.mod_packager_editor.workspace_root = Some("/tmp/ws".into());
    // Add a mod so the name can be resolved
    use dispel_core::modding::{InstalledMod, ModManifest};
    app.state.editors.mod_packager_editor.mods = vec![InstalledMod {
        slug: "test_mod".into(),
        manifest: ModManifest {
            manifest_version: 1,
            name: "Test Mod".into(),
            version: String::new(),
            author: String::new(),
            description: String::new(),
            dependencies: vec![],
            load_order_hint: None,
        },
        change_count: 0,
        enabled: true,
    }];

    let _task = mod_packager::handle(
        ModPackagerMessage::StartRecording("test_mod".into()),
        &mut app,
    );

    // StartRecording returns Task::none() when there is no prior recording
    // session that needs flushing — the new session captures edits going forward.

    let session = app.state.recording.as_ref().expect("session initialized");
    assert_eq!(session.mod_slug, "test_mod");
    assert_eq!(session.mod_name, "Test Mod");
    assert_eq!(session.recorded_count, 0);
    assert!(session.pending.is_empty());
    assert_eq!(session.next_generation, 0);
    assert_eq!(
        session.workspace_root,
        std::path::PathBuf::from("/tmp/ws")
    );
    assert!(
        app.state
            .editors
            .mod_packager_editor
            .status_msg
            .contains("Recording into"),
        "status confirms recording"
    );
}

// ============================================================================
// RecordingPersisted — result handling
// ============================================================================

#[test]
fn recording_persisted_ok_increments_count() {
    let mut app = app_with_recording();
    assert_eq!(app.state.recording.as_ref().unwrap().recorded_count, 0);

    let task = mod_packager::handle(
        ModPackagerMessage::RecordingPersisted(Ok(())),
        &mut app,
    );

    assert_eq!(task.units(), 0, "RecordingPersisted returns no task");
    assert_eq!(
        app.state.recording.as_ref().unwrap().recorded_count,
        1,
        "recorded_count incremented"
    );
}

#[test]
fn recording_persisted_error_sets_status() {
    let mut app = app_with_recording();

    let task = mod_packager::handle(
        ModPackagerMessage::RecordingPersisted(Err("disk full".into())),
        &mut app,
    );

    assert_eq!(task.units(), 0);
    assert!(
        app.state
            .editors
            .mod_packager_editor
            .status_msg
            .contains("disk full"),
        "status shows error"
    );
}

// ============================================================================
// chdata  (standard macro-generated editor)
// ============================================================================

#[test]
fn chdata_editor_records_when_session_active() {
    let mut app = app_with_recording();
    app.state.editors.chdata_editor.catalog = Some(vec![dispel_core::ChData {
        warrior_strength: 10,
        ..Default::default()
    }]);
    let record = app.state.editors.chdata_editor.catalog.as_ref().unwrap()[0].clone();
    app.state.editors.chdata_editor.filtered = vec![(0, record)];

    let task = crate::editors::chdata::handle(
        crate::editors::chdata::ChDataEditorMessage::FieldChanged(
            0,
            "warrior_strength".into(),
            "15".into(),
        ),
        &mut app,
    );

    assert!(
        task.units() > 0,
        "chdata should record when session active"
    );
    assert_eq!(
        app.state.editors.chdata_editor.catalog.as_ref().unwrap()[0].warrior_strength,
        15,
        "field value updated"
    );
}

#[test]
fn chdata_editor_does_not_record_without_session() {
    let mut app = app_without_recording();
    app.state.editors.chdata_editor.catalog = Some(vec![dispel_core::ChData {
        warrior_strength: 10,
        ..Default::default()
    }]);
    let record = app.state.editors.chdata_editor.catalog.as_ref().unwrap()[0].clone();
    app.state.editors.chdata_editor.filtered = vec![(0, record)];

    let task = crate::editors::chdata::handle(
        crate::editors::chdata::ChDataEditorMessage::FieldChanged(
            0,
            "warrior_strength".into(),
            "15".into(),
        ),
        &mut app,
    );

    assert_eq!(
        task.units(),
        0,
        "chdata should NOT record without session"
    );
}

// ============================================================================
// party_level_db  (custom two-tier editor)
// ============================================================================

#[test]
fn party_level_db_editor_records_when_session_active() {
    let mut app = app_with_recording();
    // Set up the outer NPC catalog with a selected NPC
    app.state.editors.party_level_db_editor.catalog = Some(vec![dispel_core::PartyLevelNpc {
        npc_index: 0,
        records: vec![dispel_core::PartyLevelRecord {
            level: 1,
            strength: 10,
            ..Default::default()
        }],
    }]);
    app.state.editors.party_level_db_editor.selected_npc_idx = Some(0);

    // Set up the inner level editor with a matching record
    app.state.editors.party_level_db_level_editor.catalog = Some(vec![
        dispel_core::PartyLevelRecord {
            level: 1,
            strength: 10,
            ..Default::default()
        },
    ]);
    let record = app.state.editors.party_level_db_level_editor
        .catalog
        .as_ref()
        .unwrap()[0]
        .clone();
    app.state.editors.party_level_db_level_editor.state.filtered = vec![(0, record)];

    let task = crate::editors::party_level_db::handle(
        crate::editors::party_level_db::PartyLevelDbEditorMessage::FieldChanged(
            0,
            "strength".into(),
            "20".into(),
        ),
        &mut app,
    );

    assert!(
        task.units() > 0,
        "party_level_db should record when session active"
    );
}

#[test]
fn party_level_db_editor_does_not_record_without_session() {
    let mut app = app_without_recording();
    app.state.editors.party_level_db_editor.catalog = Some(vec![dispel_core::PartyLevelNpc {
        npc_index: 0,
        records: vec![dispel_core::PartyLevelRecord {
            level: 1,
            strength: 10,
            ..Default::default()
        }],
    }]);
    app.state.editors.party_level_db_editor.selected_npc_idx = Some(0);

    app.state.editors.party_level_db_level_editor.catalog = Some(vec![
        dispel_core::PartyLevelRecord {
            level: 1,
            strength: 10,
            ..Default::default()
        },
    ]);
    let record = app.state.editors.party_level_db_level_editor
        .catalog
        .as_ref()
        .unwrap()[0]
        .clone();
    app.state.editors.party_level_db_level_editor.state.filtered = vec![(0, record)];

    let task = crate::editors::party_level_db::handle(
        crate::editors::party_level_db::PartyLevelDbEditorMessage::FieldChanged(
            0,
            "strength".into(),
            "20".into(),
        ),
        &mut app,
    );

    assert_eq!(
        task.units(),
        0,
        "party_level_db should NOT record without session"
    );
}

#[test]
fn party_level_db_no_recording_when_value_unchanged() {
    let mut app = app_with_recording();
    app.state.editors.party_level_db_editor.catalog = Some(vec![dispel_core::PartyLevelNpc {
        npc_index: 0,
        records: vec![dispel_core::PartyLevelRecord {
            level: 1,
            strength: 10,
            ..Default::default()
        }],
    }]);
    app.state.editors.party_level_db_editor.selected_npc_idx = Some(0);

    app.state.editors.party_level_db_level_editor.catalog = Some(vec![
        dispel_core::PartyLevelRecord {
            level: 1,
            strength: 10,
            ..Default::default()
        },
    ]);
    let record = app.state.editors.party_level_db_level_editor
        .catalog
        .as_ref()
        .unwrap()[0]
        .clone();
    app.state.editors.party_level_db_level_editor.state.filtered = vec![(0, record)];

    let task = crate::editors::party_level_db::handle(
        crate::editors::party_level_db::PartyLevelDbEditorMessage::FieldChanged(
            0,
            "strength".into(),
            "10".into(),
        ),
        &mut app,
    );

    assert_eq!(
        task.units(),
        0,
        "party_level_db should NOT record when value unchanged"
    );
}

#[test]
fn party_level_db_no_recording_when_no_npc_selected() {
    let mut app = app_with_recording();
    // Do NOT set selected_npc_idx — capture returns None

    app.state.editors.party_level_db_level_editor.catalog = Some(vec![
        dispel_core::PartyLevelRecord {
            level: 1,
            strength: 10,
            ..Default::default()
        },
    ]);
    let record = app.state.editors.party_level_db_level_editor
        .catalog
        .as_ref()
        .unwrap()[0]
        .clone();
    app.state.editors.party_level_db_level_editor.state.filtered = vec![(0, record)];

    let task = crate::editors::party_level_db::handle(
        crate::editors::party_level_db::PartyLevelDbEditorMessage::FieldChanged(
            0,
            "strength".into(),
            "20".into(),
        ),
        &mut app,
    );

    assert_eq!(
        task.units(),
        0,
        "party_level_db should NOT record when no NPC selected"
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
