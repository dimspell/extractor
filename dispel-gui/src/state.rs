use crate::all_map_ini_editor;
use crate::chdata_editor;
use crate::chest_editor;
use crate::db_viewer_state::DbViewerState;
use crate::dialog_editor;
use crate::dialogue_text_editor;
use crate::draw_item_editor;
use crate::edit_item_editor;
use crate::event_ini_editor;
use crate::event_item_editor;
use crate::event_npc_ref_editor;
use crate::extra_ini_editor;
use crate::extra_ref_editor;
use crate::heal_item_editor;
use crate::magic_editor;
use crate::map_ini_editor;
use crate::message_scr_editor;
use crate::misc_item_editor;
use crate::monster_editor;
use crate::monster_ref_editor;
use crate::npc_ini_editor;
use crate::npc_ref_editor;
use crate::party_ini_editor;
use crate::party_level_db_editor;
use crate::party_ref_editor;
use crate::quest_scr_editor;
use crate::sprite_browser;
use crate::store_editor;
use crate::types::{DbOp, MapOp, RefOp, Tab};
use crate::wave_ini_editor;
use crate::weapon_editor;
use std::collections::HashMap;

/// Application state — all mutable data for the GUI.
///
/// This struct was extracted from the monolithic `App` struct in `app.rs`
/// as part of the GUI rewrite (Phase 1, Step 1.1).
pub struct AppState {
    // Navigation
    pub active_tab: Tab,

    // Shared Game Path (set once, used by all editor tabs)
    pub shared_game_path: String,

    // Map fields
    pub map_op: Option<MapOp>,
    pub map_input: String,
    pub map_output: String,
    pub map_map_path: String,
    pub map_btl_path: String,
    pub map_gtl_path: String,
    pub map_save_sprites: bool,
    pub map_database: String,
    pub map_map_id: String,
    pub map_gtl_atlas: String,
    pub map_btl_atlas: String,
    pub map_atlas_columns: String,
    pub map_game_path: String,

    // Ref fields
    pub ref_op: Option<RefOp>,
    pub ref_input: String,

    // Database fields
    pub db_op: Option<DbOp>,

    // Global
    pub extractor_path: String,
    pub log: String,
    pub is_running: bool,

    // DB Viewer
    pub viewer: Box<DbViewerState>,

    // Editor states — one per file type
    pub chest_editor: Box<chest_editor::ChestEditorState>,
    pub weapon_editor: Box<weapon_editor::WeaponEditorState>,
    pub heal_item_editor: Box<heal_item_editor::HealItemEditorState>,
    pub misc_item_editor: Box<misc_item_editor::MiscItemEditorState>,
    pub edit_item_editor: Box<edit_item_editor::EditItemEditorState>,
    pub event_item_editor: Box<event_item_editor::EventItemEditorState>,
    pub monster_editor: Box<monster_editor::MonsterEditorState>,
    pub npc_ini_editor: Box<npc_ini_editor::NpcIniEditorState>,
    pub magic_editor: Box<magic_editor::MagicEditorState>,
    pub store_editor: Box<store_editor::StoreEditorState>,
    pub party_ref_editor: Box<party_ref_editor::PartyRefEditorState>,
    pub party_ini_editor: Box<party_ini_editor::PartyIniEditorState>,
    pub monster_ref_editor: Box<monster_ref_editor::MonsterRefEditorState>,
    pub sprite_browser: Box<sprite_browser::SpriteBrowserState>,
    pub all_map_ini_editor: Box<all_map_ini_editor::AllMapIniEditorState>,
    pub dialog_editor: Box<dialog_editor::DialogEditorState>,
    pub dialogue_text_editor: Box<dialogue_text_editor::DialogueTextEditorState>,
    pub draw_item_editor: Box<draw_item_editor::DrawItemEditorState>,
    pub event_ini_editor: Box<event_ini_editor::EventIniEditorState>,
    pub event_npc_ref_editor: Box<event_npc_ref_editor::EventNpcRefEditorState>,
    pub extra_ini_editor: Box<extra_ini_editor::ExtraIniEditorState>,
    pub extra_ref_editor: Box<extra_ref_editor::ExtraRefEditorState>,
    pub map_ini_editor: Box<map_ini_editor::MapIniEditorState>,
    pub message_scr_editor: Box<message_scr_editor::MessageScrEditorState>,
    pub npc_ref_editor: Box<npc_ref_editor::NpcRefEditorState>,
    pub party_level_db_editor: Box<party_level_db_editor::PartyLevelDbEditorState>,
    pub quest_scr_editor: Box<quest_scr_editor::QuestScrEditorState>,
    pub wave_ini_editor: Box<wave_ini_editor::WaveIniEditorState>,
    pub chdata_editor: Box<chdata_editor::ChDataEditorState>,

    /// Lookup data for dropdown fields: lookup_key -> Vec<(id, display_name)>
    pub lookups: HashMap<String, Vec<(String, String)>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_tab: Tab::Map,
            shared_game_path: String::new(),
            map_op: Some(MapOp::Render),
            map_input: String::new(),
            map_output: String::from("map.png"),
            map_map_path: String::new(),
            map_btl_path: String::new(),
            map_gtl_path: String::new(),
            map_save_sprites: false,
            map_database: String::from("database.sqlite"),
            map_map_id: String::new(),
            map_gtl_atlas: String::new(),
            map_btl_atlas: String::new(),
            map_atlas_columns: String::from("48"),
            map_game_path: String::new(),
            ref_op: Some(RefOp::AllMaps),
            ref_input: String::new(),
            db_op: Some(DbOp::Import),
            extractor_path: String::from("dispel-extractor"),
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
            npc_ini_editor: Box::default(),
            magic_editor: Box::default(),
            store_editor: Box::default(),
            party_ref_editor: Box::default(),
            party_ini_editor: Box::default(),
            monster_ref_editor: Box::default(),
            lookups: HashMap::new(),
            sprite_browser: Box::default(),
            all_map_ini_editor: Box::default(),
            dialog_editor: Box::default(),
            dialogue_text_editor: Box::default(),
            draw_item_editor: Box::default(),
            event_ini_editor: Box::default(),
            event_npc_ref_editor: Box::default(),
            extra_ini_editor: Box::default(),
            extra_ref_editor: Box::default(),
            map_ini_editor: Box::default(),
            message_scr_editor: Box::default(),
            npc_ref_editor: Box::default(),
            party_level_db_editor: Box::default(),
            quest_scr_editor: Box::default(),
            wave_ini_editor: Box::default(),
            chdata_editor: Box::default(),
        }
    }
}
