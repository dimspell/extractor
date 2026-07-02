use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

/// Editor type identifier for workspace tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    SpriteViewer,
    SnfEditor,
    DbViewer,
    /// Tileset browser (file-based, opens .btl/.gtl files)
    TilesetEditor,
    /// Visual map editor (file-based, opens .map files)
    MapEditor,
    ModPackager,
    LocalizationManager,
    /// Universal fallback editor for any binary file no dedicated editor
    /// claims. Also reachable via "Open as Hex" from the file tree.
    HexEditor,
    #[serde(other)]
    Unknown,
}

impl EditorType {
    /// Infer editor type from file extension and stem
    pub fn from_path(path: &std::path::Path) -> Self {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        // Try direct (ext, stem) lookup first
        if let Some(&et) = Self::editor_map().get(&(ext.as_str(), stem.as_str())) {
            return et;
        }

        // Fallback: prefix-based patterns (cannot be exact-matched)
        match ext.as_str() {
            "ref" if stem.starts_with("npc") => return EditorType::NpcRefEditor,
            "ref" if stem.starts_with("mon") => return EditorType::MonsterRefEditor,
            "ref" if stem.starts_with("ext") => return EditorType::ExtraRefEditor,
            "scr" if stem.starts_with("event") => return EditorType::EventScrEditor,
            _ => {}
        }

        // Extension-only fallthroughs
        match ext.as_str() {
            "btl" | "gtl" => EditorType::TilesetEditor,
            "dlg" => EditorType::DialogueScriptEditor,
            "pgp" => EditorType::DialogueTextEditor,
            "spr" => EditorType::SpriteViewer,
            "snf" => EditorType::SnfEditor,
            "map" => EditorType::MapEditor,
            _ => EditorType::HexEditor,
        }
    }

    /// Static map of (extension, stem) -> EditorType for exact-match lookups.
    fn editor_map() -> &'static HashMap<(&'static str, &'static str), EditorType> {
        static MAP: OnceLock<HashMap<(&'static str, &'static str), EditorType>> = OnceLock::new();
        MAP.get_or_init(|| {
            let mut m = HashMap::new();
            // .db files
            m.insert(("db", "weaponitem"), EditorType::WeaponEditor);
            m.insert(("db", "monster"), EditorType::MonsterEditor);
            m.insert(("db", "healitem"), EditorType::HealItemEditor);
            m.insert(("db", "miscitem"), EditorType::MiscItemEditor);
            m.insert(("db", "edititem"), EditorType::EditItemEditor);
            m.insert(("db", "eventitem"), EditorType::EventItemEditor);
            m.insert(("db", "store"), EditorType::StoreEditor);
            m.insert(("db", "magic"), EditorType::MagicEditor);
            m.insert(("db", "chdata"), EditorType::ChDataEditor);
            m.insert(("db", "prtlevel"), EditorType::PartyLevelDbEditor);
            m.insert(("db", "prtini"), EditorType::PartyIniEditor);
            // .ini files
            m.insert(("ini", "allmap"), EditorType::AllMapIniEditor);
            m.insert(("ini", "map"), EditorType::MapIniEditor);
            m.insert(("ini", "extra"), EditorType::ExtraIniEditor);
            m.insert(("ini", "event"), EditorType::EventIniEditor);
            m.insert(("ini", "monster"), EditorType::MonsterIniEditor);
            m.insert(("ini", "npc"), EditorType::NpcIniEditor);
            m.insert(("ini", "npcini"), EditorType::NpcIniEditor);
            m.insert(("ini", "wave"), EditorType::WaveIniEditor);
            // .ref files (exact stems only; prefix-based handled separately)
            m.insert(("ref", "partyref"), EditorType::PartyRefEditor);
            m.insert(("ref", "drawitem"), EditorType::DrawItemEditor);
            m.insert(("ref", "eventnpc"), EditorType::EventNpcRefEditor);
            // .scr files (exact stems only; "event*" prefix handled separately)
            m.insert(("scr", "quest"), EditorType::QuestScrEditor);
            m.insert(("scr", "message"), EditorType::MessageScrEditor);
            m
        })
    }

    /// Returns `true` if this editor type supports Ctrl+S saving.
    ///
    /// This is the **single source of truth** for save capability.  Both
    /// `save_task_for_editor()` and the test iterate this method.  When adding
    /// a new saving editor, add its variant here *and* add a dispatch arm in
    /// `save_task_for_editor()` (which will panic at runtime if you forget).
    pub fn supports_save(&self) -> bool {
        !matches!(
            self,
            EditorType::SnfEditor
                | EditorType::DbViewer
                | EditorType::TilesetEditor
                | EditorType::ModPackager
                | EditorType::LocalizationManager
                | EditorType::HexEditor
                | EditorType::Unknown
        )
    }

    /// Returns `true` if this editor type supports undo/redo.
    ///
    /// Single source of truth for `undo_redo_dispatch!` — every type that
    /// returns `true` must also have a match arm in the macro.
    pub fn supports_undo_redo(&self) -> bool {
        !matches!(
            self,
            EditorType::EventScrEditor
                | EditorType::SnfEditor
                | EditorType::DbViewer
                | EditorType::TilesetEditor
                | EditorType::MapEditor
                | EditorType::ModPackager
                | EditorType::LocalizationManager
                | EditorType::HexEditor
                | EditorType::Unknown
        )
    }

    /// Returns `true` if this editor type exposes an edit history panel.
    ///
    /// Single source of truth for `get_active_edit_history()`.
    pub fn has_edit_history(&self) -> bool {
        !matches!(
            self,
            EditorType::EventScrEditor
                | EditorType::MonsterEditor
                | EditorType::MonsterIniEditor
                | EditorType::NpcIniEditor
                | EditorType::SpriteViewer
                | EditorType::SnfEditor
                | EditorType::DbViewer
                | EditorType::TilesetEditor
                | EditorType::MapEditor
                | EditorType::ModPackager
                | EditorType::LocalizationManager
                | EditorType::HexEditor
                | EditorType::Unknown
        )
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
    #[serde(skip)]
    pub tabs: Vec<WorkspaceTab>,
    #[serde(skip)]
    pub active_tab: Option<usize>,
    #[serde(skip)]
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

    /// Open a tab with an explicit editor type, bypassing auto-detection.
    /// Deduplicates on (path, editor_type) tuple so a file already open in
    /// another editor can get a separate tab.
    pub fn open_with_editor_type(
        &mut self,
        label: String,
        path: Option<PathBuf>,
        editor_type: EditorType,
    ) -> usize {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.path == path && t.editor_type == editor_type)
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
