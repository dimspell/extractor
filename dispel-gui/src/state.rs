use crate::editor_registry::EditorRegistry;
use crate::indexation::file_index_cache::{FileIndexCache, FileIndexCacheManager};
use crate::message::{Message, system::SystemMessage};
use crate::workspace::EditorType;
use crate::workspace::Workspace;
use dirs;
use dispel_core::Extractor;
use iced::{
    Task,
    widget::pane_grid::{self, Pane},
};
use serde::Serialize;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

/// Application state — all mutable data for the GUI.
pub struct AppState {
    pub status_msg: String,
    pub shared_game_path: String,
    pub editors: EditorRegistry,
    pub lookups: HashMap<String, Vec<(String, String)>>,
    pub workspace: Workspace,
    pub pane_state: PaneState,
    pub file_index_cache_manager: Option<FileIndexCacheManager>,
    /// Recent files tracking for workspace navigation
    pub recent_files: Vec<PathBuf>,
    /// Active mod-recording session, if any. While set, every successful
    /// catalog edit is appended to that mod's `ChangeLog`.
    pub recording: Option<RecordingSession>,
}

/// In-flight recording state. Lives on [`AppState`] while the user has
/// "Record into …" turned on in the Mod Manager.
///
/// Edits are debounced per `(file, record_id, field)` key: incoming changes
/// land in [`pending`](Self::pending) instead of being persisted directly,
/// and a delayed `RecordingDebounceFired` message flushes them after the
/// idle interval has elapsed without a superseding edit.
#[derive(Debug, Clone, Default)]
pub struct RecordingSession {
    pub workspace_root: PathBuf,
    pub mod_slug: String,
    pub mod_name: String,
    /// Count of committed (persisted) actions, not pending ones.
    pub recorded_count: usize,
    /// Per-key debounce buffer.
    pub pending: std::collections::HashMap<RecordingKey, PendingEdit>,
    /// Monotonic counter; bumped on every observed edit so stale debounce
    /// timers can recognise and drop themselves.
    pub next_generation: u64,
}

/// Field identity is owned by `dispel-core`. Recording, conflict detection,
/// resolution pins and apply all use the very same key type now.
pub type RecordingKey = dispel_core::modding::FieldKey;

#[derive(Debug, Clone)]
pub struct PendingEdit {
    /// The value present BEFORE the user started editing this field — kept
    /// stable across keystrokes so the persisted `FieldDelta.old` is the
    /// true pristine value rather than the previous keystroke.
    pub original_old: dispel_core::modding::Value,
    /// Most recent value seen for this key.
    pub latest_new: dispel_core::modding::Value,
    /// Generation of the most recent edit — only the timer carrying the
    /// matching generation is allowed to flush.
    pub generation: u64,
}

impl AppState {
    pub fn save_workspace(&self) -> io::Result<()> {
        let path = Self::workspace_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Sync recent files from app state to workspace before saving
        let mut workspace = self.workspace.clone();
        workspace.recent_files = self.recent_files.clone();
        workspace.save(&path)
    }

    pub fn load_workspace(&mut self) -> io::Result<()> {
        self.load_workspace_from(&Self::workspace_path())
    }

    fn load_workspace_from(&mut self, path: &PathBuf) -> io::Result<()> {
        self.workspace = Workspace::load(path)?;
        // Sync game_path from workspace to shared_game_path
        if let Some(ref path) = self.workspace.game_path {
            self.shared_game_path = path.to_string_lossy().to_string();
        }

        // Sync recent files from workspace to app state
        self.recent_files = self.workspace.recent_files.clone();
        Ok(())
    }

    /// Initialize file index cache manager
    pub fn initialize_cache_manager(&mut self) {
        if self.file_index_cache_manager.is_none()
            && let Ok(cache_manager) = FileIndexCacheManager::new()
        {
            // Perform initial cache cleanup
            if let Err(e) = cache_manager.perform_periodic_cleanup() {
                eprintln!("Cache cleanup failed: {}", e);
            }
            self.file_index_cache_manager = Some(cache_manager);
        }
    }

    /// Extract file to JSON format with a save dialog
    pub fn extract_file_to_json(&mut self, path: &Path) {
        let suggested_name = path
            .file_stem()
            .map(|s| format!("{}.json", s.to_string_lossy()))
            .unwrap_or_else(|| "export.json".to_string());

        let output_path = match rfd::FileDialog::new()
            .set_file_name(&suggested_name)
            .add_filter("JSON", &["json"])
            .save_file()
        {
            Some(p) => p,
            None => return,
        };

        let editor_type = crate::workspace::EditorType::from_path(path);
        let result = extract_file(path, editor_type);

        match result {
            Ok(json_str) => match std::fs::write(&output_path, json_str.as_bytes()) {
                Ok(_) => {
                    self.status_msg = format!("Exported JSON to {}", output_path.display());
                }
                Err(e) => {
                    self.status_msg = format!("Failed to write JSON: {}", e);
                }
            },
            Err(e) => {
                self.status_msg = format!("JSON export failed: {}", e);
            }
        }
    }

    /// Validate a file by parsing it with the extractor declared in the editor
    /// catalog. A successful parse/serialize means the file is structurally
    /// readable; a failure reports the parser's error in the status bar.
    pub fn validate_file(&mut self, path: &Path) {
        let editor_type = crate::workspace::EditorType::from_path(path);
        if editor_type.descriptor().extract.is_none() {
            self.status_msg = format!(
                "Validation not available for this file type: {}",
                path.display()
            );
            return;
        }

        match extract_file(path, editor_type) {
            Ok(_) => {
                self.status_msg = format!(
                    "Validated {}: file is structurally readable",
                    path.display()
                );
            }
            Err(e) => {
                self.status_msg = format!("Validation failed for {}: {}", path.display(), e);
            }
        }
    }

    /// Start background file indexation if needed
    pub fn start_file_indexation_if_needed(&self) -> Option<Task<Message>> {
        let game_path = self.workspace.game_path.as_ref()?.clone();
        let cache_manager = self.file_index_cache_manager.as_ref()?.clone();

        // Check if cache exists and is valid
        if let Ok(Some(cache)) = cache_manager.load_cache()
            && crate::indexation::indexation_service::IndexationService::validate_sprite_cache(
                &cache, &game_path,
            )
        {
            // Cache is valid, no need to reindex
            return None;
        }

        // Cache is missing or invalid, start background indexation with fallback
        let cache_manager_clone = cache_manager.clone();
        let game_path_clone = game_path.clone();

        Some(Task::perform(
            async move {
                let indexation_service =
                    crate::indexation::indexation_service::IndexationService::new(
                        cache_manager_clone.clone(),
                    );

                // Start indexation - it will collect whatever files it can even on error
                let handle =
                    indexation_service.start_indexation_with_fallback(game_path_clone.clone());
                let cache: FileIndexCache = match handle.await {
                    Ok(cache) => cache,
                    Err(e) => {
                        eprintln!("Error during file indexation: {}", e);
                        FileIndexCache {
                            game_path: game_path_clone,
                            last_indexed: 0,
                            files: Vec::new(),
                        }
                    }
                };

                // Log any issues but still return what we got
                if cache.files.is_empty() {
                    eprintln!(
                        "Warning: File indexation returned no files - possible error during scan"
                    );
                }

                cache
            },
            move |cache: FileIndexCache| {
                // Even with partial results, we use what we got
                Message::System(SystemMessage::CacheIndexationComplete(cache))
            },
        ))
    }

    fn workspace_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("dispel-gui").join("workspace.json")
    }

    /// Clear all editor states when workspace changes
    /// This prevents stale editor states from referencing old workspace files
    pub fn clear_editor_states(&mut self) {
        self.editors.clear_all();
        self.lookups.clear();
    }

    /// Perform undo on the active editor.
    /// Returns a status message, or `None` if there's nothing to undo.
    pub fn undo_active(&mut self, editor_type: EditorType, tab_id: usize) -> Option<String> {
        self.editors.undo_active(editor_type, tab_id, &self.lookups)
    }

    /// Perform redo on the active editor.
    /// Returns a status message, or `None` if there's nothing to redo.
    pub fn redo_active(&mut self, editor_type: EditorType, tab_id: usize) -> Option<String> {
        self.editors.redo_active(editor_type, tab_id, &self.lookups)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            shared_game_path: String::new(),
            status_msg: String::new(),
            editors: EditorRegistry::default(),
            lookups: HashMap::new(),
            workspace: Workspace::new(),
            pane_state: PaneState::default(),
            file_index_cache_manager: None,
            recent_files: Vec::new(),
            recording: None,
        }
    }
}

// ===========================================================================
// JSON extraction helpers for "Extract to JSON" in the file tree
// ===========================================================================

/// Dispatch file extraction to the parser declared in the editor catalog.
fn extract_file(path: &Path, editor_type: crate::workspace::EditorType) -> Result<String, String> {
    let extract = editor_type
        .descriptor()
        .extract
        .ok_or_else(|| "JSON export not supported for this file type".to_string())?;
    let value = extract(path)?;
    serde_json::to_string_pretty(&value).map_err(|e| e.to_string())
}

/// Generate a named catalog extraction wrapper for a standard `Extractor` type.
macro_rules! extract_std_fn {
    ($name:ident, $ty:ty, $key:literal) => {
        pub(crate) fn $name(path: &Path) -> Result<serde_json::Value, String> {
            extract_std::<$ty>(path, $key)
        }
    };
}

extract_std_fn!(extract_weapon, dispel_core::WeaponItem, "weapons");
extract_std_fn!(extract_monster, dispel_core::Monster, "monsters");
extract_std_fn!(extract_monster_ini, dispel_core::MonsterIni, "monster_ini");
extract_std_fn!(extract_heal_item, dispel_core::HealItem, "heal_item");
extract_std_fn!(extract_misc_item, dispel_core::MiscItem, "misc_item");
extract_std_fn!(extract_edit_item, dispel_core::EditItem, "edit_item");
extract_std_fn!(extract_event_item, dispel_core::EventItem, "event_item");
extract_std_fn!(extract_magic, dispel_core::MagicSpell, "magic");
extract_std_fn!(extract_store, dispel_core::Store, "store");
extract_std_fn!(extract_chdata, dispel_core::ChData, "chdata");
extract_std_fn!(
    extract_party_level,
    dispel_core::PartyLevelNpc,
    "party_level"
);
extract_std_fn!(extract_dialog, dispel_core::DialogueScript, "dialog");
extract_std_fn!(
    extract_dialog_text,
    dispel_core::DialogueParagraph,
    "dialog_text"
);
extract_std_fn!(extract_draw_item, dispel_core::DrawItem, "draw_item");
extract_std_fn!(extract_event_ini, dispel_core::Event, "event_ini");
extract_std_fn!(
    extract_event_npc_ref,
    dispel_core::EventNpcRef,
    "event_npc_ref"
);
extract_std_fn!(extract_extra_ini, dispel_core::Extra, "extra_ini");
extract_std_fn!(extract_extra_ref, dispel_core::ExtraRef, "extra_ref");
extract_std_fn!(extract_map_ini, dispel_core::MapIni, "map_ini");
extract_std_fn!(extract_message, dispel_core::Message, "message");
extract_std_fn!(extract_monster_ref, dispel_core::MonsterRef, "monster_ref");
extract_std_fn!(extract_npc_ini, dispel_core::NpcIni, "npc_ini");
extract_std_fn!(extract_npc_ref, dispel_core::NPC, "npc_ref");
extract_std_fn!(extract_party_ref, dispel_core::PartyRef, "party_ref");
extract_std_fn!(extract_party_ini, dispel_core::PartyIniNpc, "party_ini");
extract_std_fn!(extract_quest, dispel_core::Quest, "quest");
extract_std_fn!(extract_event_scr, dispel_core::EventScript, "event_scr");
extract_std_fn!(extract_wave_ini, dispel_core::WaveIni, "wave_ini");
extract_std_fn!(extract_all_maps, dispel_core::Map, "all_maps");

/// Generic extraction for types that implement [`Extractor`] + [`Serialize`].
fn extract_std<T: Extractor + Serialize>(
    path: &Path,
    type_key: &str,
) -> Result<serde_json::Value, String> {
    let records = T::read_file(path).map_err(|e| format!("Failed to read {}: {}", type_key, e))?;
    let data =
        serde_json::to_value(&records).map_err(|e| format!("Serialization failed: {}", e))?;
    Ok(serde_json::json!({
        "_meta": { "file_type": type_key, "record_count": records.len() },
        "data": data,
    }))
}

/// Extract a `.map` file to JSON.
pub(crate) fn extract_map_file_json(path: &Path) -> Result<serde_json::Value, String> {
    use std::fs::File;
    use std::io::BufReader;

    let file = File::open(path).map_err(|e| format!("Failed to open map file: {}", e))?;
    let mut reader = BufReader::new(file);
    let map_data = dispel_core::map::read_map_data(&mut reader)
        .map_err(|e| format!("Failed to parse map: {}", e))?;
    let json = map_data.to_json();
    let data = serde_json::to_value(&json).map_err(|e| format!("Serialization failed: {}", e))?;
    Ok(serde_json::json!({
        "_meta": { "file_type": "map_file", "record_count": 1 },
        "data": data,
    }))
}

/// Extract a `.gtl`/`.btl` tileset file to JSON (metadata only, no pixel data).
pub(crate) fn extract_tileset_file_json(path: &Path) -> Result<serde_json::Value, String> {
    let tiles = dispel_core::map::tileset::extract(path)
        .map_err(|e| format!("Failed to read tileset: {}", e))?;
    let tile_entries: Vec<serde_json::Value> = tiles
        .iter()
        .enumerate()
        .map(|(i, _)| serde_json::json!({ "index": i, "pixels": null }))
        .collect();
    Ok(serde_json::json!({
        "_meta": { "file_type": "tileset", "record_count": tiles.len() },
        "data": {
            "tile_count": tiles.len(),
            "tile_width": 32,
            "tile_height": 32,
            "color_format": "RGB565",
            "tiles": tile_entries,
            "note": "Pixel data omitted. Use 'map tiles' command to extract individual tile images.",
        }
    }))
}

/// Extract a `.spr` sprite file info to JSON.
pub(crate) fn extract_sprite_file_json(path: &Path) -> Result<serde_json::Value, String> {
    let info = dispel_core::sprite::get_sprite_info(path)
        .map_err(|e| format!("Failed to read sprite info: {}", e))?;
    let data = serde_json::to_value(&info).map_err(|e| format!("Serialization failed: {}", e))?;
    Ok(serde_json::json!({
        "_meta": { "file_type": "sprite", "record_count": 1 },
        "data": data,
    }))
}

#[derive(Debug, Clone)]
pub enum PaneContent {
    Sidebar,
    MainContent,
    HistoryPanel,
}

#[derive(Debug, Clone)]
pub struct PaneState {
    pub state: pane_grid::State<PaneContent>,
    pub focus: Pane,
    pub maximized: Option<Pane>,
    pub sidebar_split: Option<pane_grid::Split>,
}

impl Default for PaneState {
    fn default() -> Self {
        let (mut state, first_pane) = pane_grid::State::new(PaneContent::Sidebar);
        let result = state.split(
            pane_grid::Axis::Vertical,
            first_pane,
            PaneContent::MainContent,
        );
        let sidebar_split = if let Some((_, split)) = result {
            let ratio: f32 = 232_f32 / 1280_f32;
            state.resize(split, ratio);
            Some(split)
        } else {
            None
        };
        Self {
            state,
            focus: first_pane,
            maximized: None,
            sidebar_split,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_workspace_propagates_corrupt_json() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("workspace.json");
        std::fs::write(&path, b"{ this is not json").expect("write corrupt workspace");
        let mut state = AppState::default();
        state
            .load_workspace_from(&path)
            .expect_err("corrupt workspace must fail to load");
    }

    #[test]
    fn test_load_workspace_missing_file_yields_fresh_workspace() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("workspace.json");
        let mut state = AppState::default();
        state
            .load_workspace_from(&path)
            .expect("missing workspace is not an error");
        assert!(state.workspace.game_path.is_none());
        assert!(state.recent_files.is_empty());
    }

    #[test]
    fn test_validate_unsupported_type_reports_not_available() {
        let mut state = AppState::default();
        state.validate_file(Path::new("effects.snf"));
        assert!(
            state
                .status_msg
                .contains("not available for this file type"),
            "unexpected status: {}",
            state.status_msg
        );
    }

    #[test]
    fn test_validate_missing_file_reports_parse_error() {
        let mut state = AppState::default();
        state.validate_file(Path::new("/nonexistent/WeaponItem.db"));
        assert!(
            state.status_msg.contains("Validation failed"),
            "unexpected status: {}",
            state.status_msg
        );
        assert!(
            state.status_msg.contains("Failed to read weapons"),
            "parse error missing from status: {}",
            state.status_msg
        );
        assert!(
            !state
                .status_msg
                .contains("not available for this file type")
        );
    }

    #[test]
    fn test_validate_parseable_file_reports_success() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("WeaponItem.db");
        // One count word followed by a single zeroed 284-byte WeaponItem record.
        let mut bytes = 1i32.to_le_bytes().to_vec();
        bytes.extend(std::iter::repeat_n(0u8, 284));
        std::fs::write(&path, bytes).expect("write synthetic weapon database");

        let mut state = AppState::default();
        state.validate_file(&path);
        assert!(
            state.status_msg.contains("structurally readable"),
            "unexpected status: {}",
            state.status_msg
        );
        assert!(state.status_msg.contains("WeaponItem.db"));
    }
}
