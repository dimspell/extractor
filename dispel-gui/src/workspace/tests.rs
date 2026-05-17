use super::*;
use std::path::Path;

// ═══════════════════════════════════════════════════════════════════════════
// Workspace Creation Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_workspace_creation() {
    let workspace = Workspace::new();
    assert_eq!(workspace.tabs.len(), 0);
    assert_eq!(workspace.active_tab, None);
    assert_eq!(workspace.next_id, 0);
    assert!(workspace.game_path.is_none());
    assert!(workspace.recent_files.is_empty());
    assert!(workspace.recent_game_paths.is_empty());
}

#[test]
fn test_workspace_default() {
    let workspace = Workspace::default();
    assert_eq!(workspace.tabs.len(), 0);
    assert_eq!(workspace.active_tab, None);
    assert_eq!(workspace.next_id, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// WorkspaceTab Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_workspace_tab_creation() {
    let tab = WorkspaceTab {
        id: 1,
        label: "Test Tab".to_string(),
        path: Some(PathBuf::from("/path/to/file")),
        editor_type: EditorType::WeaponEditor,
        modified: true,
        pinned: false,
    };
    assert_eq!(tab.id, 1);
    assert_eq!(tab.label, "Test Tab");
    assert!(tab.path.is_some());
    assert!(tab.modified);
    assert!(!tab.pinned);
}

#[test]
fn test_workspace_tab_clone() {
    let tab = WorkspaceTab {
        id: 1,
        label: "Test".to_string(),
        path: None,
        editor_type: EditorType::Unknown,
        modified: false,
        pinned: true,
    };
    let cloned = tab.clone();
    assert_eq!(tab.id, cloned.id);
    assert_eq!(tab.label, cloned.label);
    assert_eq!(tab.modified, cloned.modified);
    assert_eq!(tab.pinned, cloned.pinned);
}

#[test]
fn test_workspace_tab_debug() {
    let tab = WorkspaceTab {
        id: 1,
        label: "Test".to_string(),
        path: None,
        editor_type: EditorType::Unknown,
        modified: false,
        pinned: false,
    };
    let debug = format!("{:?}", tab);
    assert!(debug.contains("WorkspaceTab"));
    assert!(debug.contains("1"));
    assert!(debug.contains("Test"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Workspace Open/Close Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_workspace_open_new_tab() {
    let mut workspace = Workspace::new();
    let idx = workspace.open("New Tab".to_string(), None);

    assert_eq!(idx, 0);
    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(workspace.active_tab, Some(0));
    assert_eq!(workspace.tabs[0].label, "New Tab");
    assert_eq!(workspace.next_id, 1);
}

#[test]
fn test_workspace_open_with_path() {
    let mut workspace = Workspace::new();
    let path = PathBuf::from("/test/weaponitem.db");

    let idx = workspace.open("WeaponItem".to_string(), Some(path.clone()));

    assert_eq!(idx, 0);
    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(workspace.tabs[0].path, Some(path));
    assert_eq!(workspace.tabs[0].editor_type, EditorType::WeaponEditor);
}

#[test]
fn test_workspace_open_increments_id() {
    let mut workspace = Workspace::new();
    workspace.open("Tab 1".to_string(), Some(PathBuf::from("/path1")));
    workspace.open("Tab 2".to_string(), Some(PathBuf::from("/path2")));
    workspace.open("Tab 3".to_string(), Some(PathBuf::from("/path3")));

    assert_eq!(workspace.next_id, 3);
    assert_eq!(workspace.tabs[0].id, 0);
    assert_eq!(workspace.tabs[1].id, 1);
    assert_eq!(workspace.tabs[2].id, 2);
}

#[test]
fn test_workspace_open_reactivates_existing() {
    let mut workspace = Workspace::new();
    let path = PathBuf::from("/test/file.db");
    workspace.open("Tab 1".to_string(), Some(path.clone()));
    workspace.open("Tab 2".to_string(), None);
    let idx = workspace.open("Tab 1".to_string(), Some(path.clone()));

    assert_eq!(idx, 0);
    assert_eq!(workspace.tabs.len(), 2);
    assert_eq!(workspace.active_tab, Some(0));
}

#[test]
fn test_workspace_close() {
    let mut workspace = Workspace::new();
    workspace.open("Tab 1".to_string(), Some(PathBuf::from("/path1")));
    workspace.open("Tab 2".to_string(), Some(PathBuf::from("/path2")));
    workspace.open("Tab 3".to_string(), Some(PathBuf::from("/path3")));
    workspace.active_tab = Some(1);

    workspace.close(0);

    assert_eq!(workspace.tabs.len(), 2);
    assert_eq!(workspace.tabs[0].label, "Tab 2");
    assert_eq!(workspace.active_tab, Some(0));
}

#[test]
fn test_workspace_close_last_tab() {
    let mut workspace = Workspace::new();
    workspace.open("Tab 1".to_string(), Some(PathBuf::from("/path1")));
    workspace.open("Tab 2".to_string(), Some(PathBuf::from("/path2")));
    workspace.active_tab = Some(1);

    workspace.close(1);

    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(workspace.active_tab, Some(0));
}

#[test]
fn test_workspace_close_out_of_bounds() {
    let mut workspace = Workspace::new();
    workspace.open("Tab 1".to_string(), None);

    workspace.close(10);

    assert_eq!(workspace.tabs.len(), 1);
}

#[test]
fn test_workspace_close_active_tab() {
    let mut workspace = Workspace::new();
    workspace.open("Tab 1".to_string(), Some(PathBuf::from("/path1")));
    workspace.open("Tab 2".to_string(), Some(PathBuf::from("/path2")));
    workspace.active_tab = Some(0);

    workspace.close(0);

    assert_eq!(workspace.tabs.len(), 1);
    assert_eq!(workspace.active_tab, Some(0));
}

#[test]
fn test_workspace_close_only_tab() {
    let mut workspace = Workspace::new();
    workspace.open("Tab 1".to_string(), None);
    workspace.active_tab = Some(0);

    workspace.close(0);

    assert_eq!(workspace.tabs.len(), 0);
    assert_eq!(workspace.active_tab, None);
}

// ═══════════════════════════════════════════════════════════════════════════
// Active Tab Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_workspace_active() {
    let mut workspace = Workspace::new();
    workspace.open("Tab 1".to_string(), Some(PathBuf::from("/path1")));
    workspace.open("Tab 2".to_string(), Some(PathBuf::from("/path2")));
    workspace.active_tab = Some(1);

    let active = workspace.active();
    assert!(active.is_some());
    assert_eq!(active.unwrap().label, "Tab 2");
}

#[test]
fn test_workspace_active_index() {
    let mut workspace = Workspace::new();
    workspace.open("Tab 1".to_string(), None);
    workspace.open("Tab 2".to_string(), None);
    workspace.active_tab = Some(1);

    assert_eq!(workspace.active_index(), Some(1));
}

#[test]
fn test_workspace_active_none_when_empty() {
    let workspace = Workspace::new();
    assert!(workspace.active().is_none());
    assert!(workspace.active_index().is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// Modified Flag Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_workspace_mark_modified() {
    let mut workspace = Workspace::new();
    workspace.open("Tab 1".to_string(), Some(PathBuf::from("/path1")));
    workspace.open("Tab 2".to_string(), Some(PathBuf::from("/path2")));
    workspace.active_tab = Some(0);

    workspace.mark_modified();

    assert!(workspace.tabs[0].modified);
    assert!(!workspace.tabs[1].modified);
}

#[test]
fn test_workspace_clear_modified() {
    let mut workspace = Workspace::new();
    workspace.open("Tab 1".to_string(), None);
    workspace.tabs[0].modified = true;
    workspace.active_tab = Some(0);

    workspace.clear_modified();

    assert!(!workspace.tabs[0].modified);
}

// ═══════════════════════════════════════════════════════════════════════════
// Clear All Tabs Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_clear_all_tabs_clears_tabs() {
    let mut workspace = Workspace::new();

    workspace.tabs.push(WorkspaceTab {
        id: 1,
        label: "Tab 1".to_string(),
        path: None,
        editor_type: EditorType::WeaponEditor,
        modified: false,
        pinned: false,
    });
    workspace.tabs.push(WorkspaceTab {
        id: 2,
        label: "Tab 2".to_string(),
        path: None,
        editor_type: EditorType::MapEditor,
        modified: true,
        pinned: false,
    });
    workspace.active_tab = Some(0);
    workspace.next_id = 10;

    assert_eq!(workspace.tabs.len(), 2);
    assert_eq!(workspace.active_tab, Some(0));
    assert_eq!(workspace.next_id, 10);

    workspace.clear_all_tabs();

    assert_eq!(workspace.tabs.len(), 0);
    assert_eq!(workspace.active_tab, None);
    assert_eq!(workspace.next_id, 10);
}

#[test]
fn test_clear_all_tabs_when_no_tabs() {
    let mut workspace = Workspace::new();
    workspace.clear_all_tabs();
    assert_eq!(workspace.tabs.len(), 0);
    assert_eq!(workspace.active_tab, None);
}

#[test]
fn test_clear_all_tabs_preserves_next_id() {
    let mut workspace = Workspace::new();
    workspace.next_id = 999;
    workspace.clear_all_tabs();
    assert_eq!(workspace.next_id, 999);
}

// ═══════════════════════════════════════════════════════════════════════════
// EditorType Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_editor_type_from_path_db() {
    let path = PathBuf::from("/path/weaponitem.db");
    assert_eq!(EditorType::from_path(&path), EditorType::WeaponEditor);

    let path = PathBuf::from("/path/MONSTER.db");
    assert_eq!(EditorType::from_path(&path), EditorType::MonsterEditor);

    let path = PathBuf::from("/path/magic.db");
    assert_eq!(EditorType::from_path(&path), EditorType::MagicEditor);

    let path = PathBuf::from("/path/store.db");
    assert_eq!(EditorType::from_path(&path), EditorType::StoreEditor);
}

#[test]
fn test_editor_type_from_path_ini() {
    let path = PathBuf::from("/path/allmap.ini");
    assert_eq!(EditorType::from_path(&path), EditorType::AllMapIniEditor);

    let path = PathBuf::from("/path/map.ini");
    assert_eq!(EditorType::from_path(&path), EditorType::MapIniEditor);

    let path = PathBuf::from("/path/extra.ini");
    assert_eq!(EditorType::from_path(&path), EditorType::ExtraIniEditor);

    let path = PathBuf::from("/path/event.ini");
    assert_eq!(EditorType::from_path(&path), EditorType::EventIniEditor);
}

#[test]
fn test_editor_type_from_path_ref() {
    let path = PathBuf::from("/path/partyref.ref");
    assert_eq!(EditorType::from_path(&path), EditorType::PartyRefEditor);

    let path = PathBuf::from("/path/drawitem.ref");
    assert_eq!(EditorType::from_path(&path), EditorType::DrawItemEditor);

    let path = PathBuf::from("/path/eventnpc.ref");
    assert_eq!(EditorType::from_path(&path), EditorType::EventNpcRefEditor);

    let path = PathBuf::from("/path/npcfile.ref");
    assert_eq!(EditorType::from_path(&path), EditorType::NpcRefEditor);

    let path = PathBuf::from("/path/mondun01.ref");
    assert_eq!(EditorType::from_path(&path), EditorType::MonsterRefEditor);

    let path = PathBuf::from("/path/extdun01.ref");
    assert_eq!(EditorType::from_path(&path), EditorType::ExtraRefEditor);
}

#[test]
fn test_editor_type_from_path_scr() {
    let path = PathBuf::from("/path/quest.scr");
    assert_eq!(EditorType::from_path(&path), EditorType::QuestScrEditor);

    let path = PathBuf::from("/path/message.scr");
    assert_eq!(EditorType::from_path(&path), EditorType::MessageScrEditor);
}

#[test]
fn test_editor_type_from_path_dlg_pgp() {
    let path = PathBuf::from("/path/file.dlg");
    assert_eq!(
        EditorType::from_path(&path),
        EditorType::DialogueScriptEditor
    );

    let path = PathBuf::from("/path/file.pgp");
    assert_eq!(EditorType::from_path(&path), EditorType::DialogueTextEditor);
}

#[test]
fn test_editor_type_from_path_spr_map() {
    let path = PathBuf::from("/path/file.spr");
    assert_eq!(EditorType::from_path(&path), EditorType::SpriteViewer);

    let path = PathBuf::from("/path/file.map");
    assert_eq!(EditorType::from_path(&path), EditorType::MapEditor);
}

#[test]
fn test_editor_type_unknown_falls_back_to_hex() {
    let path = PathBuf::from("/path/file.txt");
    assert_eq!(EditorType::from_path(&path), EditorType::HexEditor);

    let path = PathBuf::from("/path/unknown.xyz");
    assert_eq!(EditorType::from_path(&path), EditorType::HexEditor);
}

// ═══════════════════════════════════════════════════════════════════════════
// Timestamp Validation Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_validate_timestamp_valid() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert!(Workspace::validate_timestamp(now));
}

#[test]
fn test_validate_timestamp_future() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let future = now + 120;
    assert!(!Workspace::validate_timestamp(future));
}

#[test]
fn test_validate_timestamp_too_old() {
    let old_timestamp = 0;
    assert!(!Workspace::validate_timestamp(old_timestamp));
}

#[test]
fn test_validate_last_reindexed_some() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let workspace = Workspace {
        last_reindexed_at: Some(now),
        ..Workspace::new()
    };
    assert!(workspace.validate_last_reindexed());
}

#[test]
fn test_validate_last_reindexed_none() {
    let workspace = Workspace::new();
    assert!(workspace.validate_last_reindexed());
}

// ═══════════════════════════════════════════════════════════════════════════
// Debug Info Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_debug_info() {
    let workspace = Workspace::new();
    let info = workspace.debug_info();
    assert!(info.contains("Workspace Debug Info"));
    assert!(info.contains("Tabs: 0"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Serialize/Deserialize Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_workspace_serialize() {
    let workspace = Workspace::new();
    let json = serde_json::to_string(&workspace);
    assert!(json.is_ok());
}

#[test]
fn test_workspace_deserialize() {
    let json =
        r#"{"game_path":null,"recent_files":[],"last_reindexed_at":null,"recent_game_paths":[]}"#;
    let workspace: Workspace = serde_json::from_str(json).unwrap();
    assert_eq!(workspace.recent_game_paths.len(), 0);
    assert_eq!(workspace.next_id, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// Edge Cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_workspace_with_many_tabs() {
    let mut workspace = Workspace::new();
    for i in 0..100 {
        workspace.open(
            format!("Tab {}", i),
            Some(PathBuf::from(format!("/path{}", i))),
        );
    }
    assert_eq!(workspace.tabs.len(), 100);
    assert_eq!(workspace.next_id, 100);
}

#[test]
fn test_close_shifts_active_down() {
    let mut workspace = Workspace::new();
    workspace.open("Tab 1".to_string(), Some(PathBuf::from("/path1")));
    workspace.open("Tab 2".to_string(), Some(PathBuf::from("/path2")));
    workspace.open("Tab 3".to_string(), Some(PathBuf::from("/path3")));
    workspace.active_tab = Some(2);

    workspace.close(0);

    assert_eq!(workspace.active_tab, Some(1));
}

// ═══════════════════════════════════════════════════════════════════════════
// Editor Type Detection Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_monster_db_detection() {
    let path = PathBuf::from("MonsterInGame/Monster.db");
    let editor_type = EditorType::from_path(&path);
    assert_eq!(editor_type, EditorType::MonsterEditor);
}

#[test]
fn test_monster_ini_detection() {
    let path = PathBuf::from("Monster.ini");
    let editor_type = EditorType::from_path(&path);
    assert_eq!(editor_type, EditorType::MonsterIniEditor);
}

#[test]
fn test_monster_files_distinguish_by_extension() {
    let monster_db = EditorType::from_path(&PathBuf::from("Monster.db"));
    let monster_ini = EditorType::from_path(&PathBuf::from("Monster.ini"));

    assert_eq!(monster_db, EditorType::MonsterEditor);
    assert_eq!(monster_ini, EditorType::MonsterIniEditor);
    assert_ne!(monster_db, monster_ini);
}

#[test]
fn test_monster_ini_case_insensitive_detection() {
    assert_eq!(
        EditorType::from_path(&PathBuf::from("MONSTER.INI")),
        EditorType::MonsterIniEditor
    );
    assert_eq!(
        EditorType::from_path(&PathBuf::from("Monster.ini")),
        EditorType::MonsterIniEditor
    );
    assert_eq!(
        EditorType::from_path(&PathBuf::from("monster.INI")),
        EditorType::MonsterIniEditor
    );
}

#[test]
fn test_monster_db_case_insensitive_detection() {
    assert_eq!(
        EditorType::from_path(&PathBuf::from("MONSTER.DB")),
        EditorType::MonsterEditor
    );
    assert_eq!(
        EditorType::from_path(&PathBuf::from("Monster.db")),
        EditorType::MonsterEditor
    );
    assert_eq!(
        EditorType::from_path(&PathBuf::from("monster.DB")),
        EditorType::MonsterEditor
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// editor_type_tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn db_stems_map_to_correct_editor_types() {
    let cases = [
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
        assert_eq!(
            EditorType::from_path(Path::new(filename)),
            expected,
            "failed for {filename}"
        );
    }
}

#[test]
fn ini_stems_map_to_correct_editor_types() {
    let cases = [
        ("AllMap.ini", EditorType::AllMapIniEditor),
        ("Map.ini", EditorType::MapIniEditor),
        ("Extra.ini", EditorType::ExtraIniEditor),
        ("Event.ini", EditorType::EventIniEditor),
        ("Monster.ini", EditorType::MonsterIniEditor),
        ("Npc.ini", EditorType::NpcIniEditor),
        ("Wave.ini", EditorType::WaveIniEditor),
    ];
    for (filename, expected) in cases {
        assert_eq!(
            EditorType::from_path(Path::new(filename)),
            expected,
            "failed for {filename}"
        );
    }
}

#[test]
fn ref_stems_map_to_correct_editor_types() {
    let cases = [
        ("PartyRef.ref", EditorType::PartyRefEditor),
        ("DrawItem.ref", EditorType::DrawItemEditor),
        ("EventNpc.ref", EditorType::EventNpcRefEditor),
        ("Npc01.ref", EditorType::NpcRefEditor),
        ("Mon01.ref", EditorType::MonsterRefEditor),
        ("Ext01.ref", EditorType::ExtraRefEditor),
    ];
    for (filename, expected) in cases {
        assert_eq!(
            EditorType::from_path(Path::new(filename)),
            expected,
            "failed for {filename}"
        );
    }
}

#[test]
fn script_and_special_extensions_map_correctly() {
    assert_eq!(
        EditorType::from_path(Path::new("Quest.scr")),
        EditorType::QuestScrEditor
    );
    assert_eq!(
        EditorType::from_path(Path::new("Message.scr")),
        EditorType::MessageScrEditor
    );
    assert_eq!(
        EditorType::from_path(Path::new("scene.dlg")),
        EditorType::DialogueScriptEditor
    );
    assert_eq!(
        EditorType::from_path(Path::new("text.pgp")),
        EditorType::DialogueTextEditor
    );
    assert_eq!(
        EditorType::from_path(Path::new("sprite.spr")),
        EditorType::SpriteViewer
    );
    assert_eq!(
        EditorType::from_path(Path::new("sound.snf")),
        EditorType::SnfEditor
    );
    assert_eq!(
        EditorType::from_path(Path::new("level.map")),
        EditorType::MapEditor
    );
    assert_eq!(
        EditorType::from_path(Path::new("tiles.btl")),
        EditorType::TilesetEditor
    );
}

#[test]
fn extension_matching_is_case_insensitive() {
    assert_eq!(
        EditorType::from_path(Path::new("MONSTER.DB")),
        EditorType::MonsterEditor
    );
    assert_eq!(
        EditorType::from_path(Path::new("Monster.INI")),
        EditorType::MonsterIniEditor
    );
    assert_eq!(
        EditorType::from_path(Path::new("SPRITE.SPR")),
        EditorType::SpriteViewer
    );
}

#[test]
fn paths_with_directories_are_handled() {
    assert_eq!(
        EditorType::from_path(Path::new("CharacterInGame/weaponItem.db")),
        EditorType::WeaponEditor
    );
    assert_eq!(
        EditorType::from_path(Path::new("MonsterInGame/Monster.db")),
        EditorType::MonsterEditor
    );
}

#[test]
fn unknown_files_fall_back_to_hex_editor() {
    assert_eq!(
        EditorType::from_path(Path::new("Unknown.xyz")),
        EditorType::HexEditor
    );
    assert_eq!(
        EditorType::from_path(Path::new("Random.txt")),
        EditorType::HexEditor
    );
    assert_eq!(
        EditorType::from_path(Path::new("Unknown.db")),
        EditorType::HexEditor
    );
    assert_eq!(
        EditorType::from_path(Path::new("Unknown.ini")),
        EditorType::HexEditor
    );
}

#[test]
fn test_clear_editor_states() {
    use crate::editors::map_editor::MapEditorState;
    use crate::state::AppState;

    let mut state = AppState::default();

    state.map_editors.insert(1, MapEditorState::default());
    state.map_editors.insert(2, MapEditorState::default());

    state.lookups.insert(
        "test_key".to_string(),
        vec![("field".to_string(), "value".to_string())],
    );

    assert_eq!(state.map_editors.len(), 2);
    assert_eq!(state.lookups.len(), 1);

    state.clear_editor_states();

    assert_eq!(state.map_editors.len(), 0);
    assert_eq!(state.dialogue_script_editors.len(), 0);
    assert_eq!(state.sprite_viewers.len(), 0);
    assert_eq!(state.lookups.len(), 0);

    let _ = state.weapon_editor;
    let _ = state.heal_item_editor;
}

#[test]
fn test_clear_editor_states_idempotent() {
    use crate::state::AppState;

    let mut state = AppState::default();

    state.clear_editor_states();

    state.clear_editor_states();

    assert_eq!(state.map_editors.len(), 0);
}
