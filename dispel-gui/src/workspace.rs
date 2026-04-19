use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

/// Editor type identifier for workspace tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorType {
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
    DialogEditor,
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
    /// Tileset browser (file-based, opens .btl/.gtl files)
    TilesetEditor,
    /// Visual map editor (file-based, opens .map files)
    MapEditor,
    #[serde(other)]
    Unknown,
}

impl EditorType {
    /// Infer editor type from file extension and stem
    pub fn from_path(path: &std::path::Path) -> Self {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        match ext.to_lowercase().as_str() {
            "db" => match stem.as_str() {
                "weaponitem" => EditorType::WeaponEditor,
                "monster" => EditorType::MonsterEditor,
                "healitem" => EditorType::HealItemEditor,
                "miscitem" => EditorType::MiscItemEditor,
                "edititem" => EditorType::EditItemEditor,
                "eventitem" => EditorType::EventItemEditor,
                "store" => EditorType::StoreEditor,
                "magic" => EditorType::MagicEditor,
                "chdata" => EditorType::ChDataEditor,
                "prtlevel" => EditorType::PartyLevelDbEditor,
                "prtini" => EditorType::PartyIniEditor,
                _ => EditorType::Unknown,
            },
            "ini" => match stem.as_str() {
                "allmap" => EditorType::AllMapIniEditor,
                "map" => EditorType::MapIniEditor,
                "extra" => EditorType::ExtraIniEditor,
                "event" => EditorType::EventIniEditor,
                "monster" => EditorType::MonsterIniEditor,
                "npc" => EditorType::NpcIniEditor,
                "npcini" => EditorType::NpcIniEditor,
                "wave" => EditorType::WaveIniEditor,
                _ => EditorType::Unknown,
            },
            "ref" => match stem.as_str() {
                "partyref" => EditorType::PartyRefEditor,
                "drawitem" => EditorType::DrawItemEditor,
                "eventnpc" => EditorType::EventNpcRefEditor,
                _ => {
                    if stem.starts_with("npc") {
                        EditorType::NpcRefEditor
                    } else if stem.starts_with("mon") {
                        EditorType::MonsterRefEditor
                    } else if stem.starts_with("ext") {
                        EditorType::ExtraRefEditor
                    } else {
                        EditorType::Unknown
                    }
                }
            },
            "scr" => match stem.as_str() {
                "quest" => EditorType::QuestScrEditor,
                "message" => EditorType::MessageScrEditor,
                _ => EditorType::Unknown,
            },
            "btl" | "gtl" => EditorType::TilesetEditor,
            "dlg" => EditorType::DialogEditor,
            "pgp" => EditorType::DialogueTextEditor,
            "spr" => EditorType::SpriteViewer,
            "snf" => EditorType::SnfEditor,
            "map" => EditorType::MapEditor,
            _ => EditorType::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(EditorType::from_path(&path), EditorType::DialogEditor);

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
    fn test_editor_type_unknown() {
        let path = PathBuf::from("/path/file.txt");
        assert_eq!(EditorType::from_path(&path), EditorType::Unknown);

        let path = PathBuf::from("/path/unknown.xyz");
        assert_eq!(EditorType::from_path(&path), EditorType::Unknown);
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
        let json = r#"{"tabs":[],"active_tab":null,"next_id":0,"game_path":null,"recent_files":[],"last_reindexed_at":null,"recent_game_paths":[]}"#;
        let workspace: Workspace = serde_json::from_str(json).unwrap();
        assert_eq!(workspace.tabs.len(), 0);
        assert_eq!(workspace.next_id, 0);
    }

    #[test]
    fn test_workspace_roundtrip() {
        let mut workspace = Workspace::new();
        workspace.open("Tab 1".to_string(), Some(PathBuf::from("/path/file.db")));

        let json = serde_json::to_string(&workspace).unwrap();
        let restored: Workspace = serde_json::from_str(&json).unwrap();

        assert_eq!(workspace.tabs.len(), restored.tabs.len());
        assert_eq!(workspace.tabs[0].label, restored.tabs[0].label);
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
}

/// A workspace tab that can hold any editor or view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceTab {
    pub id: usize,
    pub label: String,
    pub path: Option<PathBuf>,
    pub editor_type: EditorType,
    pub modified: bool,
    pub pinned: bool,
}

/// The workspace manages dynamic tabs instead of a fixed Tab enum.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Workspace {
    pub tabs: Vec<WorkspaceTab>,
    pub active_tab: Option<usize>,
    pub next_id: usize,
    pub game_path: Option<PathBuf>,
    /// Recent files tracking for workspace navigation
    pub recent_files: Vec<PathBuf>,
    /// Timestamp of last file index reindexation (Unix timestamp)
    pub last_reindexed_at: Option<u64>,
    /// Recently used game paths (max 5)
    #[serde(default)]
    pub recent_game_paths: Vec<PathBuf>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: None,
            next_id: 0,
            game_path: None,
            recent_files: Vec::new(),
            last_reindexed_at: None,
            recent_game_paths: Vec::new(),
        }
    }

    pub fn save(&self, path: &PathBuf) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &PathBuf) -> io::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let json = fs::read_to_string(path)?;
        let workspace: Workspace = serde_json::from_str(&json)?;
        Ok(workspace)
    }

    /// Validate a timestamp to ensure it's reasonable
    /// Returns true if the timestamp is valid (not in future, not too old)
    pub fn validate_timestamp(timestamp: u64) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Check if timestamp is in the future (allow 1 minute clock skew)
        if timestamp > now + 60 {
            return false;
        }

        // Check if timestamp is too old (older than 5 years)
        if timestamp < now.saturating_sub(5 * 365 * 24 * 60 * 60) {
            return false;
        }

        true
    }

    /// Validate the last_reindexed_at timestamp in this workspace
    pub fn validate_last_reindexed(&self) -> bool {
        if let Some(timestamp) = self.last_reindexed_at {
            Self::validate_timestamp(timestamp)
        } else {
            // No timestamp set yet, which is valid
            true
        }
    }

    /// Get debug information about this workspace including timestamp
    pub fn debug_info(&self) -> String {
        let mut info = format!(
            "Workspace Debug Info:\n  Tabs: {}\n  Active Tab: {:?}\n  Game Path: {:?}\n  Recent Files: {}\n",
            self.tabs.len(),
            self.active_tab,
            self.game_path,
            self.recent_files.len()
        );

        if let Some(timestamp) = self.last_reindexed_at {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let age_seconds = now.saturating_sub(timestamp);

            // Format age as human-readable string
            let (age_str, unit) = if age_seconds < 60 {
                (age_seconds.to_string(), "seconds")
            } else if age_seconds < 3600 {
                ((age_seconds / 60).to_string(), "minutes")
            } else if age_seconds < 86400 {
                ((age_seconds / 3600).to_string(), "hours")
            } else {
                ((age_seconds / 86400).to_string(), "days")
            };

            // Format timestamp as readable date/time
            let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0);
            let date_str = datetime.map_or_else(
                || "Invalid timestamp".to_string(),
                |dt| dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            );

            info.push_str(&format!(
                "  Last Reindexed: {} ({} {} ago)\n  Timestamp Valid: {}\n",
                date_str,
                age_str,
                unit,
                self.validate_last_reindexed()
            ));
        } else {
            info.push_str("  Last Reindexed: Never\n");
        }

        info
    }

    pub fn open(&mut self, label: String, path: Option<PathBuf>) -> usize {
        if let Some(idx) = self.tabs.iter().position(|t| t.path == path) {
            self.active_tab = Some(idx);
            return idx;
        }
        let id = self.next_id;
        self.next_id += 1;
        let idx = self.tabs.len();
        let editor_type = path
            .as_ref()
            .map(|p| EditorType::from_path(p.as_path()))
            .unwrap_or(EditorType::Unknown);
        self.tabs.push(WorkspaceTab {
            id,
            label,
            path,
            editor_type,
            modified: false,
            pinned: false,
        });
        self.active_tab = Some(idx);
        idx
    }

    /// Open a tool tab that is not backed by a file path.
    /// Re-activates an existing tab of the same type rather than opening a duplicate.
    pub fn open_tool(&mut self, label: String, editor_type: EditorType) -> usize {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.path.is_none() && t.editor_type == editor_type)
        {
            self.active_tab = Some(idx);
            return idx;
        }
        let id = self.next_id;
        self.next_id += 1;
        let idx = self.tabs.len();
        self.tabs.push(WorkspaceTab {
            id,
            label,
            path: None,
            editor_type,
            modified: false,
            pinned: false,
        });
        self.active_tab = Some(idx);
        idx
    }

    pub fn close(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }
        let was_active = self.active_tab == Some(idx);
        self.tabs.remove(idx);
        if was_active {
            self.active_tab = if self.tabs.is_empty() {
                None
            } else {
                Some(idx.min(self.tabs.len() - 1))
            };
        } else if let Some(active) = self.active_tab {
            if active > idx {
                self.active_tab = Some(active - 1);
            }
        }
    }

    pub fn active(&self) -> Option<&WorkspaceTab> {
        self.active_tab.and_then(|idx| self.tabs.get(idx))
    }

    pub fn active_index(&self) -> Option<usize> {
        self.active_tab
    }

    pub fn mark_modified(&mut self) {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                tab.modified = true;
            }
        }
    }

    pub fn clear_modified(&mut self) {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                tab.modified = false;
            }
        }
    }

    /// Clear all tabs and reset workspace state
    /// This should be called when the game path changes to avoid stale editor states
    pub fn clear_all_tabs(&mut self) {
        self.tabs.clear();
        self.active_tab = None;
        // Don't reset next_id to preserve ID uniqueness across workspace sessions
    }
}
