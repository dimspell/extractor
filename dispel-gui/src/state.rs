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
use crate::editors::extra_ini::ExtraIniEditorState;
use crate::editors::extra_ref::ExtraRefEditorState;
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
use dispel_core::WeaponItem;
use iced::{
    widget::pane_grid::{self, Pane},
    Task,
};
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
    pub wave_ini_editor: Box<WaveIniEditorState>,
    pub chdata_editor: Box<ChDataEditorState>,
    pub map_editors: HashMap<usize, MapEditorState>,
    pub tileset_editors: HashMap<usize, TilesetEditorState>,
    pub snf_editors: HashMap<usize, SnfEditorState>,
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

    /// Extract file to JSON format
    pub fn extract_file_to_json(&self, path: &Path) {
        // TODO: Implement actual extraction logic
        println!("Extracting {} to JSON", path.display());
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
            wave_ini_editor: Box::default(),
            chdata_editor: Box::default(),
            map_editors: HashMap::new(),
            tileset_editors: HashMap::new(),
            snf_editors: HashMap::new(),
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
