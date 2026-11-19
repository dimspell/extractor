use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

/// Editor type identifier for workspace tabs.
///
/// The enum itself is generated from the authoritative editor catalog; all
/// capability predicates and descriptor-driven dispatch live in
/// `crate::editor_catalog`. It is re-exported here to preserve the historical
/// `crate::workspace::EditorType` path used throughout the crate.
pub use crate::editor_catalog::EditorType;

/// A typed workspace-tab identifier.
///
/// The old code used the raw `usize::MAX` sentinel for "no active tab" in
/// fifteen-plus places. `TabId` contains that sentinel in one place (see
/// [`TabId::NONE`]) and gives callers an explicit opt-in via [`TabId::NONE`] or
/// [`Workspace::active_tab_id`]. Handlers that only use the id as a `HashMap`
/// key can convert with [`TabId::as_usize`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(pub usize);

impl TabId {
    /// Sentinel for "no active tab". Previously spelled `usize::MAX`.
    pub const NONE: TabId = TabId(usize::MAX);

    pub fn as_usize(self) -> usize {
        self.0
    }

    pub fn is_none(self) -> bool {
        self == TabId::NONE
    }
}

impl EditorType {
    /// Infer editor type from a path.
    ///
    /// The shared file-type catalog in `dispel-core` is the single source of
    /// truth for filename/extension detection. Only GUI-specific names that
    /// have no catalog entry are handled locally.
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

        // GUI-only names (no corresponding catalog key).
        if let Some(&et) = Self::gui_only_map().get(&(ext.as_str(), stem.as_str())) {
            return et;
        }

        // Shared catalog detection.
        if let dispel_core::commands::registry::DetectResult::Single(ft) =
            dispel_core::commands::registry::detect(path)
            && let Some(et) = Self::from_catalog_editor(ft.editor)
        {
            return et;
        }

        // GUI-only formats the catalog doesn't describe.
        match ext.as_str() {
            "snf" => EditorType::SnfEditor,
            _ => EditorType::HexEditor,
        }
    }

    /// Map a catalog `editor` id to its GUI variant.
    ///
    /// The string→variant table used to be hand-copied here and validated only
    /// against itself; it is now derived from the authoritative editor catalog.
    pub(crate) fn from_catalog_editor(id: &str) -> Option<Self> {
        crate::editor_catalog::from_catalog_editor(id)
    }

    /// Exact (extension, stem) pairs that have no catalog entry.
    fn gui_only_map() -> &'static HashMap<(&'static str, &'static str), EditorType> {
        static MAP: OnceLock<HashMap<(&'static str, &'static str), EditorType>> = OnceLock::new();
        MAP.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert(("ini", "npcini"), EditorType::NpcIniEditor);
            m.insert(("dat", "fogdata"), EditorType::FogDataEditor);
            m
        })
    }

    // Capability predicates (supports_save / supports_undo_redo /
    // has_edit_history / auto_load_catalog) are generated from the authoritative
    // editor catalog in `crate::editor_catalog`.
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
        } else if let Some(active) = self.active_tab
            && active > idx
        {
            self.active_tab = Some(active - 1);
        }
    }

    pub fn active(&self) -> Option<&WorkspaceTab> {
        self.active_tab.and_then(|idx| self.tabs.get(idx))
    }

    pub fn active_index(&self) -> Option<usize> {
        self.active_tab
    }

    /// Typed id of the active tab, or [`TabId::NONE`] when no tab is active.
    /// This is the single, typed replacement for the old `usize::MAX` sentinel.
    pub fn active_tab_id(&self) -> TabId {
        self.active().map(|t| TabId(t.id)).unwrap_or(TabId::NONE)
    }

    pub fn mark_modified(&mut self) {
        if let Some(idx) = self.active_tab
            && let Some(tab) = self.tabs.get_mut(idx)
        {
            tab.modified = true;
        }
    }

    pub fn clear_modified(&mut self) {
        if let Some(idx) = self.active_tab
            && let Some(tab) = self.tabs.get_mut(idx)
        {
            tab.modified = false;
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
