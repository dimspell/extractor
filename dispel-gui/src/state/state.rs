use crate::components::file_tree::FileTree;
use crate::file_index_cache::{FileIndexCache, FileIndexCacheManager};
use crate::generic_editor::GenericEditorState;
use crate::global_search::GlobalSearch;
use crate::message::{system::SystemMessage, Message};
use crate::state::all_map_ini_editor;
use crate::state::chdata_editor;
use crate::state::chest_editor;
use crate::state::db_viewer_state::DbViewerState;
use crate::state::dialog_editor;
use crate::state::dialogue_text_editor;
use crate::state::draw_item_editor;
use crate::state::event_ini_editor;
use crate::state::event_npc_ref_editor;
use crate::state::extra_ini_editor;
use crate::state::extra_ref_editor;
use crate::state::magic_editor;
use crate::state::map_editor;
use crate::state::map_ini_editor;
use crate::state::message_scr_editor;
use crate::state::monster_ini_editor;
use crate::state::monster_ref_editor;
use crate::state::npc_ini_editor;
use crate::state::npc_ref_editor;
use crate::state::party_ini_editor;
use crate::state::party_level_db_editor;
use crate::state::quest_scr_editor;
use crate::state::snf_editor;
use crate::state::sprite_viewer;
use crate::state::store_editor;
use crate::state::tileset_editor;
use crate::state::wave_ini_editor;
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
    pub chest_editor: Box<chest_editor::ChestEditorState>,
    pub weapon_editor: Box<GenericEditorState<WeaponItem>>,
    pub heal_item_editor: Box<GenericEditorState<dispel_core::HealItem>>,
    pub misc_item_editor: Box<GenericEditorState<dispel_core::MiscItem>>,
    pub edit_item_editor: Box<GenericEditorState<dispel_core::EditItem>>,
    pub event_item_editor: Box<GenericEditorState<dispel_core::EventItem>>,
    pub monster_editor: Box<GenericEditorState<dispel_core::Monster>>,
    pub monster_ini_editor: Box<monster_ini_editor::MonsterIniEditorState>,
    pub npc_ini_editor: Box<npc_ini_editor::NpcIniEditorState>,
    pub magic_editor: Box<magic_editor::MagicEditorState>,
    pub store_editor: Box<store_editor::StoreEditorState>,
    pub party_ref_editor: Box<GenericEditorState<dispel_core::PartyRef>>,
    pub party_ini_editor: Box<party_ini_editor::PartyIniEditorState>,
    pub monster_ref_editors: HashMap<usize, monster_ref_editor::MonsterRefEditorState>,
    pub monster_ref_spreadsheets: HashMap<usize, SpreadsheetState>,
    pub sprite_viewers: HashMap<usize, sprite_viewer::SpriteViewerState>,
    pub all_map_ini_editor: Box<all_map_ini_editor::AllMapIniEditorState>,
    pub all_map_ini_spreadsheet: SpreadsheetState,
    pub dialog_editors: HashMap<usize, dialog_editor::DialogEditorState>,
    pub dialog_spreadsheets: HashMap<usize, SpreadsheetState>,
    pub dialogue_text_editors: HashMap<usize, dialogue_text_editor::DialogueTextEditorState>,
    pub dialogue_text_spreadsheets: HashMap<usize, SpreadsheetState>,
    pub draw_item_editor: Box<draw_item_editor::DrawItemEditorState>,
    pub event_ini_editor: Box<event_ini_editor::EventIniEditorState>,
    pub event_npc_ref_editor: Box<event_npc_ref_editor::EventNpcRefEditorState>,
    pub extra_ini_editor: Box<extra_ini_editor::ExtraIniEditorState>,
    pub extra_ref_editors: HashMap<usize, extra_ref_editor::ExtraRefEditorState>,
    pub extra_ref_spreadsheets: HashMap<usize, SpreadsheetState>,
    pub map_ini_editor: Box<map_ini_editor::MapIniEditorState>,
    pub message_scr_editor: Box<message_scr_editor::MessageScrEditorState>,
    pub npc_ref_editors: HashMap<usize, npc_ref_editor::NpcRefEditorState>,
    pub npc_ref_spreadsheets: HashMap<usize, SpreadsheetState>,
    pub party_level_db_editor: Box<party_level_db_editor::PartyLevelDbEditorState>,
    pub quest_scr_editor: Box<quest_scr_editor::QuestScrEditorState>,
    pub wave_ini_editor: Box<wave_ini_editor::WaveIniEditorState>,
    pub chdata_editor: Box<chdata_editor::ChDataEditorState>,
    pub map_editors: HashMap<usize, map_editor::MapEditorState>,
    pub tileset_editors: HashMap<usize, tileset_editor::TilesetEditorState>,
    pub snf_editors: HashMap<usize, snf_editor::SnfEditorState>,
    pub lookups: HashMap<String, Vec<(String, String)>>,
    pub workspace: Workspace,
    pub global_search: GlobalSearch,
    pub heal_item_spreadsheet: SpreadsheetState,
    pub misc_item_spreadsheet: SpreadsheetState,
    pub magic_spreadsheet: SpreadsheetState,
    pub weapon_spreadsheet: SpreadsheetState,
    pub edit_item_spreadsheet: SpreadsheetState,
    pub event_item_spreadsheet: SpreadsheetState,
    pub party_ref_spreadsheet: SpreadsheetState,
    pub party_ini_spreadsheet: SpreadsheetState,
    pub event_ini_spreadsheet: SpreadsheetState,
    pub event_npc_ref_spreadsheet: SpreadsheetState,
    pub extra_ini_spreadsheet: SpreadsheetState,
    pub map_ini_spreadsheet: SpreadsheetState,
    pub message_scr_spreadsheet: SpreadsheetState,
    pub party_level_db_spreadsheet: SpreadsheetState,
    pub quest_scr_spreadsheet: SpreadsheetState,
    pub wave_ini_spreadsheet: SpreadsheetState,
    pub chdata_spreadsheet: SpreadsheetState,
    pub draw_item_spreadsheet: SpreadsheetState,
    pub monster_spreadsheet: SpreadsheetState,
    pub monster_ini_spreadsheet: SpreadsheetState,
    pub npc_ini_spreadsheet: SpreadsheetState,
    pub pane_state: PaneState,
    pub file_index_cache_manager: Option<FileIndexCacheManager>,
    pub file_tree: FileTree,
    /// Recent files tracking for workspace navigation
    pub recent_files: Vec<PathBuf>,
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
            if crate::indexation_service::IndexationService::validate_sprite_cache(
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
                    crate::indexation_service::IndexationService::new(cache_manager_clone.clone());

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
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("dispel-gui");
        path.push("workspace.json");
        path
    }

    /// Clear all editor states when workspace changes
    /// This prevents stale editor states from referencing old workspace files
    pub fn clear_editor_states(&mut self) {
        // Clear all HashMap-based editor states
        self.sprite_viewers.clear();
        self.tileset_editors.clear();
        self.dialog_editors.clear();
        self.dialog_spreadsheets.clear();
        self.dialogue_text_editors.clear();
        self.dialogue_text_spreadsheets.clear();
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
        self.all_map_ini_spreadsheet = SpreadsheetState::new();
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
            all_map_ini_spreadsheet: SpreadsheetState::new(),
            dialog_editors: HashMap::new(),
            dialog_spreadsheets: HashMap::new(),
            dialogue_text_editors: HashMap::new(),
            dialogue_text_spreadsheets: HashMap::new(),
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
            quest_scr_editor: Box::default(),
            wave_ini_editor: Box::default(),
            chdata_editor: Box::default(),
            map_editors: HashMap::new(),
            tileset_editors: HashMap::new(),
            snf_editors: HashMap::new(),
            lookups: HashMap::new(),
            workspace: Workspace::new(),
            global_search: GlobalSearch::new(),
            heal_item_spreadsheet: SpreadsheetState::new(),
            misc_item_spreadsheet: SpreadsheetState::new(),
            magic_spreadsheet: SpreadsheetState::new(),
            weapon_spreadsheet: SpreadsheetState::new(),
            edit_item_spreadsheet: SpreadsheetState::new(),
            event_item_spreadsheet: SpreadsheetState::new(),
            party_ref_spreadsheet: SpreadsheetState::new(),
            party_ini_spreadsheet: SpreadsheetState::new(),
            event_ini_spreadsheet: SpreadsheetState::new(),
            event_npc_ref_spreadsheet: SpreadsheetState::new(),
            extra_ini_spreadsheet: SpreadsheetState::new(),
            map_ini_spreadsheet: SpreadsheetState::new(),
            message_scr_spreadsheet: SpreadsheetState::new(),
            party_level_db_spreadsheet: SpreadsheetState::new(),
            quest_scr_spreadsheet: SpreadsheetState::new(),
            wave_ini_spreadsheet: SpreadsheetState::new(),
            chdata_spreadsheet: SpreadsheetState::new(),
            draw_item_spreadsheet: SpreadsheetState::new(),
            monster_spreadsheet: SpreadsheetState::new(),
            monster_ini_spreadsheet: SpreadsheetState::new(),
            npc_ini_spreadsheet: SpreadsheetState::new(),
            pane_state: PaneState::default(),
            file_index_cache_manager: None,
            file_tree: FileTree::default(),
            recent_files: Vec::new(),
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
}

impl Default for PaneState {
    fn default() -> Self {
        let (mut state, first_pane) = pane_grid::State::new(PaneContent::Sidebar);
        let result = state.split(
            pane_grid::Axis::Vertical,
            first_pane,
            PaneContent::MainContent,
        );
        if let Some((_, split)) = result {
            let ratio: f32 = 232_f32 / 1280_f32;
            state.resize(split, ratio);
        }
        Self {
            state,
            focus: first_pane,
            maximized: None,
        }
    }
}
