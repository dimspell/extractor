use crate::components::file_tree::FileTree;
use crate::components::global_search::GlobalSearch;
use crate::components::standard::StandardEditor;
use crate::editors::all_map_ini::AllMapIniEditorState;
use crate::editors::chdata::ChDataEditorState;
use crate::editors::chest::ChestEditorState;
use crate::editors::db_viewer::DbViewerState;
use crate::editors::dialogue_paragraph::DialogueParagraphEditorState;
use crate::editors::dialogue_script::DialogueScriptEditorState;
use crate::editors::draw_item::DrawItemEditorState;
use crate::editors::event_ini::EventIniEditorState;
use crate::editors::event_npc_ref::EventNpcRefEditorState;
use crate::editors::event_scr::EventScriptEditorState;
use crate::editors::extra_ini::ExtraIniEditorState;
use crate::editors::extra_ref::ExtraRefEditorState;
use crate::editors::hex_editor::HexEditorState;
use crate::editors::magic::MagicEditorState;
use crate::editors::map_editor::MapEditorState;
use crate::editors::map_ini::MapIniEditorState;
use crate::editors::message_scr::MessageScrEditorState;
use crate::editors::monster_ini::MonsterIniEditorState;
use crate::editors::monster_ref::MonsterRefEditorState;
use crate::editors::npc_ini::NpcIniEditorState;
use crate::editors::npc_ref::NpcRefEditorState;
use crate::editors::party_ini::PartyIniEditorState;
use crate::editors::party_level_db::PartyLevelDbEditorState;
use crate::editors::quest_scr::QuestScrEditorState;
use crate::editors::snf_editor::SnfEditorState;
use crate::editors::sprite_browser::SpriteViewerState;
use crate::editors::store::StoreEditorState;
use crate::editors::tileset::TilesetEditorState;
use crate::editors::wave_ini::WaveIniEditorState;
use crate::editors::{localization_manager, mod_packager};
use crate::indexation::file_index_cache::{FileIndexCache, FileIndexCacheManager};
use crate::message::{system::SystemMessage, Message};
use crate::view::editor::SpreadsheetState;
use crate::workspace::Workspace;
use dirs;
use dispel_core::Extractor;
use dispel_core::WeaponItem;
use iced::{
    widget::pane_grid::{self, Pane},
    Task,
};
use serde::Serialize;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

/// Application state — all mutable data for the GUI.
pub struct AppState {
    pub status_msg: String,
    pub shared_game_path: String,
    pub log: String,
    pub is_running: bool,
    pub viewer: Box<DbViewerState>,
    pub chest_editor: Box<ChestEditorState>,
    pub weapon_editor: Box<StandardEditor<WeaponItem>>,
    pub heal_item_editor: Box<StandardEditor<dispel_core::HealItem>>,
    pub misc_item_editor: Box<StandardEditor<dispel_core::MiscItem>>,
    pub edit_item_editor: Box<StandardEditor<dispel_core::EditItem>>,
    pub event_item_editor: Box<StandardEditor<dispel_core::EventItem>>,
    pub monster_editor: Box<StandardEditor<dispel_core::Monster>>,
    pub monster_ini_editor: Box<MonsterIniEditorState>,
    pub npc_ini_editor: Box<NpcIniEditorState>,
    pub magic_editor: Box<MagicEditorState>,
    pub store_editor: Box<StoreEditorState>,
    pub party_ref_editor: Box<StandardEditor<dispel_core::PartyRef>>,
    pub party_ini_editor: Box<PartyIniEditorState>,
    pub monster_ref_editors: HashMap<usize, MonsterRefEditorState>,
    pub monster_ref_spreadsheets: HashMap<usize, SpreadsheetState>,
    pub sprite_viewers: HashMap<usize, SpriteViewerState>,
    pub all_map_ini_editor: Box<AllMapIniEditorState>,
    pub dialogue_script_editors: HashMap<usize, DialogueScriptEditorState>,
    pub dialogue_script_spreadsheets: HashMap<usize, SpreadsheetState>,
    pub dialogue_paragraphs_editors: HashMap<usize, DialogueParagraphEditorState>,
    pub dialogue_paragraph_spreadsheets: HashMap<usize, SpreadsheetState>,
    pub draw_item_editor: Box<DrawItemEditorState>,
    pub event_ini_editor: Box<EventIniEditorState>,
    pub event_npc_ref_editor: Box<EventNpcRefEditorState>,
    pub extra_ini_editor: Box<ExtraIniEditorState>,
    pub extra_ref_editors: HashMap<usize, ExtraRefEditorState>,
    pub extra_ref_spreadsheets: HashMap<usize, SpreadsheetState>,
    pub map_ini_editor: Box<MapIniEditorState>,
    pub message_scr_editor: Box<MessageScrEditorState>,
    pub npc_ref_editors: HashMap<usize, NpcRefEditorState>,
    pub npc_ref_spreadsheets: HashMap<usize, SpreadsheetState>,
    pub party_level_db_editor: Box<PartyLevelDbEditorState>,
    pub party_level_db_level_editor: Box<StandardEditor<dispel_core::PartyLevelRecord>>,
    pub quest_scr_editor: Box<QuestScrEditorState>,
    pub event_scr_editor: Box<EventScriptEditorState>,
    pub wave_ini_editor: Box<WaveIniEditorState>,
    pub chdata_editor: Box<ChDataEditorState>,
    pub map_editors: HashMap<usize, MapEditorState>,
    pub tileset_editors: HashMap<usize, TilesetEditorState>,
    pub snf_editors: HashMap<usize, SnfEditorState>,
    pub hex_editors: HashMap<usize, HexEditorState>,
    pub mod_packager_editor: mod_packager::ModPackagerState,
    pub localization_manager: localization_manager::LocalizationManagerState,
    pub lookups: HashMap<String, Vec<(String, String)>>,
    pub workspace: Workspace,
    pub global_search: GlobalSearch,
    pub pane_state: PaneState,
    pub file_index_cache_manager: Option<FileIndexCacheManager>,
    pub file_tree: FileTree,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordingKey {
    pub file_path: String,
    pub record_id: u32,
    pub field: String,
}

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
        self.workspace = Workspace::load(&Self::workspace_path()).unwrap_or_default();
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
        if self.file_index_cache_manager.is_none() {
            if let Ok(cache_manager) = FileIndexCacheManager::new() {
                // Perform initial cache cleanup
                if let Err(e) = cache_manager.perform_periodic_cleanup() {
                    eprintln!("Cache cleanup failed: {}", e);
                }
                self.file_index_cache_manager = Some(cache_manager);
            }
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

    /// Validate file
    pub fn validate_file(&self, path: &Path) {
        // TODO: Implement actual validation logic
        println!("Validating file: {}", path.display());
    }

    /// Show file in OS file manager
    pub fn show_in_file_manager(&self, path: &Path) {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("explorer")
                .arg("/select,")
                .arg(path)
                .spawn()
                .ok();
        }

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("open").arg("-R").arg(path).spawn().ok();
        }

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            if let Some(parent) = path.parent() {
                Command::new("xdg-open").arg(parent).spawn().ok();
            }
        }

        println!("Showing {} in file manager", path.display());
    }

    /// Start background file indexation if needed
    pub fn start_file_indexation_if_needed(&self) -> Option<Task<Message>> {
        let game_path = self.workspace.game_path.as_ref()?.clone();
        let cache_manager = self.file_index_cache_manager.as_ref()?.clone();

        eprintln!(
            "DEBUG: start_file_indexation_if_needed called with game_path: {:?}",
            game_path
        );

        // Check if cache exists and is valid
        if let Ok(Some(cache)) = cache_manager.load_cache() {
            if crate::indexation::indexation_service::IndexationService::validate_sprite_cache(
                &cache, &game_path,
            ) {
                // Cache is valid, no need to reindex
                eprintln!("DEBUG: Cache is valid, skipping reindex");
                return None;
            }
        }

        // Cache is missing or invalid, start background indexation with fallback
        let cache_manager_clone = cache_manager.clone();
        let game_path_clone = game_path.clone();

        Some(Task::perform(
            async move {
                eprintln!("DEBUG: Starting indexation for: {:?}", game_path_clone);
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
                } else {
                    eprintln!("DEBUG: Indexation returned {} files", cache.files.len());
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
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("../.."));
        path.push("..");
        path.push("workspace.json");
        path
    }

    /// Clear all editor states when workspace changes
    /// This prevents stale editor states from referencing old workspace files
    pub fn clear_editor_states(&mut self) {
        // Clear all HashMap-based editor states
        self.sprite_viewers.clear();
        self.tileset_editors.clear();
        self.dialogue_script_editors.clear();
        self.dialogue_script_spreadsheets.clear();
        self.dialogue_paragraphs_editors.clear();
        self.dialogue_paragraph_spreadsheets.clear();
        self.monster_ref_editors.clear();
        self.monster_ref_spreadsheets.clear();
        self.extra_ref_editors.clear();
        self.npc_ref_editors.clear();
        self.npc_ref_spreadsheets.clear();
        self.map_editors.clear();
        self.snf_editors.clear();
        self.hex_editors.clear();

        // Reset boxed editors to default state
        *self.weapon_editor = Default::default();
        *self.heal_item_editor = Default::default();
        *self.misc_item_editor = Default::default();
        *self.edit_item_editor = Default::default();
        *self.event_item_editor = Default::default();
        *self.monster_editor = Default::default();
        *self.npc_ini_editor = Default::default();
        *self.magic_editor = Default::default();
        *self.store_editor = Default::default();
        *self.party_ref_editor = Default::default();
        *self.party_ini_editor = Default::default();
        *self.all_map_ini_editor = Default::default();
        *self.draw_item_editor = Default::default();
        *self.event_ini_editor = Default::default();
        *self.event_npc_ref_editor = Default::default();
        *self.extra_ini_editor = Default::default();
        *self.map_ini_editor = Default::default();
        *self.message_scr_editor = Default::default();
        *self.quest_scr_editor = Default::default();
        *self.wave_ini_editor = Default::default();
        *self.chdata_editor = Default::default();
        *self.event_scr_editor = Default::default();

        // Clear lookups that might reference old workspace data
        self.lookups.clear();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            shared_game_path: String::new(),
            status_msg: String::new(),
            log: String::new(),
            is_running: false,
            viewer: Box::default(),
            chest_editor: Box::default(),
            weapon_editor: Box::default(),
            heal_item_editor: Box::default(),
            misc_item_editor: Box::default(),
            edit_item_editor: Box::default(),
            event_item_editor: Box::default(),
            monster_editor: Box::default(),
            monster_ini_editor: Box::default(),
            npc_ini_editor: Box::default(),
            magic_editor: Box::default(),
            store_editor: Box::default(),
            party_ref_editor: Box::default(),
            party_ini_editor: Box::default(),
            monster_ref_editors: HashMap::new(),
            monster_ref_spreadsheets: HashMap::new(),
            sprite_viewers: HashMap::new(),
            all_map_ini_editor: Box::default(),
            dialogue_script_editors: HashMap::new(),
            dialogue_script_spreadsheets: HashMap::new(),
            dialogue_paragraphs_editors: HashMap::new(),
            dialogue_paragraph_spreadsheets: HashMap::new(),
            draw_item_editor: Box::default(),
            event_ini_editor: Box::default(),
            event_npc_ref_editor: Box::default(),
            extra_ini_editor: Box::default(),
            extra_ref_editors: HashMap::new(),
            map_ini_editor: Box::default(),
            message_scr_editor: Box::default(),
            npc_ref_editors: HashMap::new(),
            npc_ref_spreadsheets: HashMap::new(),
            extra_ref_spreadsheets: HashMap::new(),
            party_level_db_editor: Box::default(),
            party_level_db_level_editor: Box::default(),
            quest_scr_editor: Box::default(),
            event_scr_editor: Box::default(),
            wave_ini_editor: Box::default(),
            chdata_editor: Box::default(),
            map_editors: HashMap::new(),
            tileset_editors: HashMap::new(),
            snf_editors: HashMap::new(),
            hex_editors: HashMap::new(),
            mod_packager_editor: mod_packager::ModPackagerState::default(),
            localization_manager: localization_manager::LocalizationManagerState::default(),
            lookups: HashMap::new(),
            workspace: Workspace::new(),
            global_search: GlobalSearch::new(),
            pane_state: PaneState::default(),
            file_index_cache_manager: None,
            file_tree: FileTree::default(),
            recent_files: Vec::new(),
            recording: None,
        }
    }
}

// ===========================================================================
// JSON extraction helpers for "Extract to JSON" in the file tree
// ===========================================================================

/// Dispatch file extraction to the correct parser based on [`EditorType`].
fn extract_file(path: &Path, editor_type: crate::workspace::EditorType) -> Result<String, String> {
    use crate::workspace::EditorType;

    let value = match editor_type {
        EditorType::WeaponEditor => extract_std::<dispel_core::WeaponItem>(path, "weapons")?,
        EditorType::MonsterEditor => extract_std::<dispel_core::Monster>(path, "monsters")?,
        EditorType::MonsterIniEditor => {
            extract_std::<dispel_core::MonsterIni>(path, "monster_ini")?
        }
        EditorType::HealItemEditor => extract_std::<dispel_core::HealItem>(path, "heal_item")?,
        EditorType::MiscItemEditor => extract_std::<dispel_core::MiscItem>(path, "misc_item")?,
        EditorType::EditItemEditor => extract_std::<dispel_core::EditItem>(path, "edit_item")?,
        EditorType::EventItemEditor => extract_std::<dispel_core::EventItem>(path, "event_item")?,
        EditorType::MagicEditor => extract_std::<dispel_core::MagicSpell>(path, "magic")?,
        EditorType::StoreEditor => extract_std::<dispel_core::Store>(path, "store")?,
        EditorType::ChDataEditor => extract_std::<dispel_core::ChData>(path, "chdata")?,
        EditorType::PartyLevelDbEditor => {
            extract_std::<dispel_core::PartyLevelNpc>(path, "party_level")?
        }
        EditorType::DialogueScriptEditor => {
            extract_std::<dispel_core::DialogueScript>(path, "dialog")?
        }
        EditorType::DialogueTextEditor => {
            extract_std::<dispel_core::DialogueParagraph>(path, "dialog_text")?
        }
        EditorType::DrawItemEditor => extract_std::<dispel_core::DrawItem>(path, "draw_item")?,
        EditorType::EventIniEditor => extract_std::<dispel_core::Event>(path, "event_ini")?,
        EditorType::EventNpcRefEditor => {
            extract_std::<dispel_core::EventNpcRef>(path, "event_npc_ref")?
        }
        EditorType::ExtraIniEditor => extract_std::<dispel_core::Extra>(path, "extra_ini")?,
        EditorType::ExtraRefEditor => extract_std::<dispel_core::ExtraRef>(path, "extra_ref")?,
        EditorType::MapIniEditor => extract_std::<dispel_core::MapIni>(path, "map_ini")?,
        EditorType::MessageScrEditor => extract_std::<dispel_core::Message>(path, "message")?,
        EditorType::MonsterRefEditor => {
            extract_std::<dispel_core::MonsterRef>(path, "monster_ref")?
        }
        EditorType::NpcIniEditor => extract_std::<dispel_core::NpcIni>(path, "npc_ini")?,
        EditorType::NpcRefEditor => extract_std::<dispel_core::NPC>(path, "npc_ref")?,
        EditorType::PartyRefEditor => extract_std::<dispel_core::PartyRef>(path, "party_ref")?,
        EditorType::PartyIniEditor => extract_std::<dispel_core::PartyIniNpc>(path, "party_ini")?,
        EditorType::QuestScrEditor => extract_std::<dispel_core::Quest>(path, "quest")?,
        EditorType::EventScrEditor => extract_std::<dispel_core::EventScript>(path, "event_scr")?,
        EditorType::WaveIniEditor => extract_std::<dispel_core::WaveIni>(path, "wave_ini")?,
        EditorType::AllMapIniEditor => extract_std::<dispel_core::Map>(path, "all_maps")?,
        EditorType::MapEditor => extract_map_file_json(path)?,
        EditorType::TilesetEditor => extract_tileset_file_json(path)?,
        EditorType::SpriteViewer => extract_sprite_file_json(path)?,
        _ => return Err("JSON export not supported for this file type".to_string()),
    };

    serde_json::to_string_pretty(&value).map_err(|e| e.to_string())
}

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
fn extract_map_file_json(path: &Path) -> Result<serde_json::Value, String> {
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
fn extract_tileset_file_json(path: &Path) -> Result<serde_json::Value, String> {
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
fn extract_sprite_file_json(path: &Path) -> Result<serde_json::Value, String> {
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
