use std::collections::HashMap;

use crate::components::generic_editor::TabbedEditor;
use crate::components::standard::StandardEditor;
use crate::editors::all_map_ini::AllMapIniEditorState;
use crate::editors::chdata::ChDataEditorState;
use crate::editors::chest::ChestEditorState;
use crate::editors::db_viewer::DbViewerState;
use crate::editors::draw_item::DrawItemEditorState;
use crate::editors::event_ini::EventIniEditorState;
use crate::editors::event_npc_ref::EventNpcRefEditorState;
use crate::editors::event_scr::EventScriptEditorState;
use crate::editors::extra_ini::ExtraIniEditorState;
use crate::editors::magic::MagicEditorState;
use crate::editors::map_editor::MapEditorState;
use crate::editors::map_ini::MapIniEditorState;
use crate::editors::message_scr::MessageScrEditorState;
use crate::editors::monster_ini::MonsterIniEditorState;
use crate::editors::npc_ini::NpcIniEditorState;
use crate::editors::party_ini::PartyIniEditorState;
use crate::editors::party_level_db::PartyLevelDbEditorState;
use crate::editors::quest_scr::QuestScrEditorState;
use crate::editors::snf_editor::SnfEditorState;
use crate::editors::sprite_browser::SpriteViewerState;
use crate::editors::store::StoreEditorState;
use crate::editors::tileset::TilesetEditorState;
use crate::editors::wave_ini::WaveIniEditorState;
use crate::editors::{localization_manager, mod_packager};
use dispel_core::{
    DialogueParagraph, DialogueScript, EditItem, EventItem, ExtraRef, HealItem, MiscItem,
    Monster, MonsterRef, NPC, PartyLevelRecord, PartyRef, WeaponItem,
};
use hexedit::HexEditorState;

/// Aggregates all editor state fields (35+ types).
///
/// Extracted from `AppState` so that editor lifecycle (clear / reset) is
/// handled in one place and the main state struct stays focused on
/// non-editor concerns (workspace, UI, file tree, …).
#[derive(Default)]
pub struct EditorRegistry {
    pub viewer: Box<DbViewerState>,
    pub chest_editor: Box<ChestEditorState>,
    pub weapon_editor: Box<StandardEditor<WeaponItem>>,
    pub heal_item_editor: Box<StandardEditor<HealItem>>,
    pub misc_item_editor: Box<StandardEditor<MiscItem>>,
    pub edit_item_editor: Box<StandardEditor<EditItem>>,
    pub event_item_editor: Box<StandardEditor<EventItem>>,
    pub monster_editor: Box<StandardEditor<Monster>>,
    pub monster_ini_editor: Box<MonsterIniEditorState>,
    pub npc_ini_editor: Box<NpcIniEditorState>,
    pub magic_editor: Box<MagicEditorState>,
    pub store_editor: Box<StoreEditorState>,
    pub party_ref_editor: Box<StandardEditor<PartyRef>>,
    pub party_ini_editor: Box<PartyIniEditorState>,
    pub monster_ref_editor: TabbedEditor<MonsterRef>,
    pub sprite_viewers: HashMap<usize, SpriteViewerState>,
    pub all_map_ini_editor: Box<AllMapIniEditorState>,
    pub dialogue_script_editor: TabbedEditor<DialogueScript>,
    pub dialogue_paragraph_editor: TabbedEditor<DialogueParagraph>,
    pub draw_item_editor: Box<DrawItemEditorState>,
    pub event_ini_editor: Box<EventIniEditorState>,
    pub event_npc_ref_editor: Box<EventNpcRefEditorState>,
    pub extra_ini_editor: Box<ExtraIniEditorState>,
    pub extra_ref_editor: TabbedEditor<ExtraRef>,
    pub map_ini_editor: Box<MapIniEditorState>,
    pub message_scr_editor: Box<MessageScrEditorState>,
    pub npc_ref_editor: TabbedEditor<NPC>,
    pub party_level_db_editor: Box<PartyLevelDbEditorState>,
    pub party_level_db_level_editor: Box<StandardEditor<PartyLevelRecord>>,
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
}

impl EditorRegistry {
    /// Reset every editor to its initial state.
    pub fn clear_all(&mut self) {
        // HashMap-based editors
        self.sprite_viewers.clear();
        self.tileset_editors.clear();
        self.dialogue_script_editor.clear();
        self.dialogue_paragraph_editor.clear();
        self.monster_ref_editor.clear();
        self.extra_ref_editor.clear();
        self.npc_ref_editor.clear();
        self.map_editors.clear();
        self.snf_editors.clear();
        self.hex_editors.clear();

        // Boxed editors — reset to default
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

        // Other owned editors
        self.mod_packager_editor = Default::default();
        self.localization_manager = Default::default();
    }
}
