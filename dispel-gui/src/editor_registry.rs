use std::collections::HashMap;

use crate::components::edit_history::EditHistory;
use crate::components::generic_editor::UndoRedo;
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
use crate::workspace::EditorType;
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

/// Macro: dispatch `undo` or `redo` to the correct editor field.
/// Arms are defined once and reused for both operations via `$action`.
macro_rules! undo_redo_dispatch {
    ($self:ident, $editor_type:expr, $tab_id:expr, $lookups:expr, $action:ident) => {{
        use $crate::workspace::EditorType;
        match $editor_type {
            // Standard editors (StandardEditor<T> — undo/redo with lookups)
            EditorType::WeaponEditor => $self.weapon_editor.$action($lookups),
            EditorType::HealItemEditor => $self.heal_item_editor.$action($lookups),
            EditorType::MiscItemEditor => $self.misc_item_editor.$action($lookups),
            EditorType::EditItemEditor => $self.edit_item_editor.$action($lookups),
            EditorType::EventItemEditor => $self.event_item_editor.$action($lookups),
            EditorType::MonsterEditor => $self.monster_editor.$action($lookups),
            EditorType::MonsterIniEditor => $self.monster_ini_editor.$action($lookups),
            EditorType::NpcIniEditor => $self.npc_ini_editor.$action($lookups),
            EditorType::MagicEditor => $self.magic_editor.$action($lookups),
            EditorType::PartyRefEditor => $self.party_ref_editor.$action($lookups),
            EditorType::PartyIniEditor => $self.party_ini_editor.$action($lookups),
            EditorType::AllMapIniEditor => $self.all_map_ini_editor.$action($lookups),
            EditorType::DrawItemEditor => $self.draw_item_editor.$action($lookups),
            EditorType::EventIniEditor => $self.event_ini_editor.$action($lookups),
            EditorType::EventNpcRefEditor => $self.event_npc_ref_editor.$action($lookups),
            EditorType::ExtraIniEditor => $self.extra_ini_editor.$action($lookups),
            EditorType::MapIniEditor => $self.map_ini_editor.$action($lookups),
            EditorType::MessageScrEditor => $self.message_scr_editor.$action($lookups),
            EditorType::QuestScrEditor => $self.quest_scr_editor.$action($lookups),
            EditorType::WaveIniEditor => $self.wave_ini_editor.$action($lookups),
            EditorType::ChDataEditor => $self.chdata_editor.$action($lookups),
            EditorType::PartyLevelDbEditor => {
                $self.party_level_db_level_editor.$action($lookups)
            }

            // Custom-layout editor (undo/redo without lookups)
            EditorType::StoreEditor => $self.store_editor.$action(),

            // Tab-based editors (MultiFileEditorState via TabbedEditor)
            EditorType::MonsterRefEditor => $self
                .monster_ref_editor
                .editors
                .get_mut(&$tab_id)
                .and_then(|e| e.$action()),
            EditorType::NpcRefEditor => $self
                .npc_ref_editor
                .editors
                .get_mut(&$tab_id)
                .and_then(|e| e.$action()),
            EditorType::ExtraRefEditor => $self
                .extra_ref_editor
                .editors
                .get_mut(&$tab_id)
                .and_then(|e| e.$action()),
            EditorType::DialogueScriptEditor => $self
                .dialogue_script_editor
                .editors
                .get_mut(&$tab_id)
                .and_then(|e| e.$action()),
            EditorType::DialogueTextEditor => $self
                .dialogue_paragraph_editor
                .editors
                .get_mut(&$tab_id)
                .and_then(|e| e.$action()),

            _ => None,
        }
    }};
}

impl EditorRegistry {
    // ─── Tab lifecycle ───────────────────────────────────────────────────────

    /// Remove editor state for a single closed tab.
    ///
    /// Covers every HashMap / TabbedEditor that maps `tab_id → editor`.
    /// (Previously each close handler duplicated this list by hand, and
    /// `tileset_editors` / `map_editors` / `hex_editors` were forgotten.)
    pub fn remove_tab(&mut self, tab_id: usize) {
        self.sprite_viewers.remove(&tab_id);
        self.extra_ref_editor.remove(&tab_id);
        self.npc_ref_editor.remove(&tab_id);
        self.monster_ref_editor.remove(&tab_id);
        self.dialogue_script_editor.remove(&tab_id);
        self.dialogue_paragraph_editor.remove(&tab_id);
        self.snf_editors.remove(&tab_id);
        self.tileset_editors.remove(&tab_id);
        self.map_editors.remove(&tab_id);
        self.hex_editors.remove(&tab_id);
    }

    /// Clear editors for every tab.  Use when the workspace is about to lose
    /// all its tabs (CloseAll, workspace reset, …).
    ///
    /// This is **less aggressive** than [`clear_all`](Self::clear_all) in that
    /// it does **not** reset single-instance Box editors (weapon_editor,
    /// monster_editor, …) to their default state — only tab-indexed state is
    /// dropped.
    pub fn close_all_tabs(&mut self) {
        self.sprite_viewers.clear();
        self.extra_ref_editor.clear();
        self.npc_ref_editor.clear();
        self.monster_ref_editor.clear();
        self.dialogue_script_editor.clear();
        self.dialogue_paragraph_editor.clear();
        self.snf_editors.clear();
        self.tileset_editors.clear();
        self.map_editors.clear();
        self.hex_editors.clear();
    }

    /// Stop SNF audio playback on every open SNF editor.
    pub fn stop_snf_playback(&mut self) {
        for editor in self.snf_editors.values_mut() {
            editor.playback = None;
        }
    }

    // ─── Undo / Redo ────────────────────────────────────────────────────────

    /// Perform undo on the editor identified by `editor_type` / `tab_id`.
    /// Returns a status message, or `None` if there's nothing to undo.
    pub fn undo_active(
        &mut self,
        editor_type: EditorType,
        tab_id: usize,
        lookups: &HashMap<String, Vec<(String, String)>>,
    ) -> Option<String> {
        undo_redo_dispatch!(self, editor_type, tab_id, lookups, undo)
    }

    /// Perform redo on the editor identified by `editor_type` / `tab_id`.
    /// Returns a status message, or `None` if there's nothing to redo.
    pub fn redo_active(
        &mut self,
        editor_type: EditorType,
        tab_id: usize,
        lookups: &HashMap<String, Vec<(String, String)>>,
    ) -> Option<String> {
        undo_redo_dispatch!(self, editor_type, tab_id, lookups, redo)
    }

    /// Refresh spreadsheet caches after undo/redo for tab-based editors.
    ///
    /// `MultiFileEditorState::undo` / `redo` does not own the
    /// `SpreadsheetState`, so caches go stale — refresh them here.
    pub fn refresh_spreadsheet(
        &mut self,
        editor_type: EditorType,
        tab_id: usize,
        lookups: &HashMap<String, Vec<(String, String)>>,
    ) {
        macro_rules! refresh_tab {
            ($editors:expr, $spreadsheets:expr) => {
                if let (Some(editor), Some(spreadsheet)) =
                    ($editors.get(&tab_id), $spreadsheets.get_mut(&tab_id))
                {
                    if let Some(ref catalog) = editor.editor.catalog {
                        spreadsheet.compute_all_caches(catalog, lookups);
                    }
                }
            };
        }
        match editor_type {
            EditorType::MonsterRefEditor => {
                refresh_tab!(
                    self.monster_ref_editor.editors,
                    self.monster_ref_editor.spreadsheets
                )
            }
            EditorType::NpcRefEditor => {
                refresh_tab!(
                    self.npc_ref_editor.editors,
                    self.npc_ref_editor.spreadsheets
                )
            }
            EditorType::ExtraRefEditor => {
                refresh_tab!(
                    self.extra_ref_editor.editors,
                    self.extra_ref_editor.spreadsheets
                )
            }
            EditorType::DialogueScriptEditor => {
                refresh_tab!(
                    self.dialogue_script_editor.editors,
                    self.dialogue_script_editor.spreadsheets
                )
            }
            EditorType::DialogueTextEditor => {
                refresh_tab!(
                    self.dialogue_paragraph_editor.editors,
                    self.dialogue_paragraph_editor.spreadsheets
                )
            }
            _ => {}
        }
    }

    // ─── Edit-history lookup ────────────────────────────────────────────────

    /// Return the active editor's [`EditHistory`], keyed by `editor_type` /
    /// `tab_id`.  Returns `None` when the editor kind has no history or the
    /// tab lookup fails.
    pub fn get_active_edit_history(
        &self,
        editor_type: EditorType,
        tab_id: usize,
    ) -> Option<&EditHistory> {
        use crate::components::generic_editor::UndoRedo;

        match editor_type {
            // Standard / Box editors — edit_history is always available
            EditorType::HealItemEditor => Some(self.heal_item_editor.edit_history()),
            EditorType::MiscItemEditor => Some(self.misc_item_editor.edit_history()),
            EditorType::EditItemEditor => Some(self.edit_item_editor.edit_history()),
            EditorType::EventItemEditor => Some(self.event_item_editor.edit_history()),
            EditorType::MagicEditor => Some(self.magic_editor.edit_history()),
            EditorType::WeaponEditor => Some(self.weapon_editor.edit_history()),
            EditorType::DrawItemEditor => Some(self.draw_item_editor.edit_history()),
            EditorType::EventIniEditor => Some(self.event_ini_editor.edit_history()),
            EditorType::EventNpcRefEditor => Some(self.event_npc_ref_editor.edit_history()),
            EditorType::ExtraIniEditor => Some(self.extra_ini_editor.edit_history()),
            EditorType::MapIniEditor => Some(self.map_ini_editor.edit_history()),
            EditorType::MessageScrEditor => Some(self.message_scr_editor.edit_history()),
            EditorType::PartyLevelDbEditor => Some(self.party_level_db_editor.edit_history()),
            EditorType::QuestScrEditor => Some(self.quest_scr_editor.edit_history()),
            EditorType::WaveIniEditor => Some(self.wave_ini_editor.edit_history()),
            EditorType::AllMapIniEditor => Some(self.all_map_ini_editor.edit_history()),
            EditorType::ChDataEditor => Some(self.chdata_editor.edit_history()),
            EditorType::PartyRefEditor => Some(self.party_ref_editor.edit_history()),
            EditorType::PartyIniEditor => Some(self.party_ini_editor.edit_history()),
            EditorType::StoreEditor => Some(self.store_editor.edit_history()),

            // Tab-based editors — need a HashMap lookup
            EditorType::MonsterRefEditor => self
                .monster_ref_editor
                .editors
                .get(&tab_id)
                .map(|ed| ed.edit_history()),
            EditorType::ExtraRefEditor => self
                .extra_ref_editor
                .editors
                .get(&tab_id)
                .map(|ed| ed.edit_history()),
            EditorType::NpcRefEditor => self
                .npc_ref_editor
                .editors
                .get(&tab_id)
                .map(|ed| ed.edit_history()),
            EditorType::DialogueScriptEditor => self
                .dialogue_script_editor
                .editors
                .get(&tab_id)
                .map(|ed| ed.edit_history()),
            EditorType::DialogueTextEditor => self
                .dialogue_paragraph_editor
                .editors
                .get(&tab_id)
                .map(|ed| ed.edit_history()),

            // Editors without standard undo/redo
            EditorType::EventScrEditor
            | EditorType::MonsterEditor
            | EditorType::MonsterIniEditor
            | EditorType::NpcIniEditor
            | EditorType::ChestEditor
            | EditorType::SpriteViewer
            | EditorType::SnfEditor
            | EditorType::DbViewer
            | EditorType::TilesetEditor
            | EditorType::MapEditor
            | EditorType::ModPackager
            | EditorType::LocalizationManager
            | EditorType::HexEditor
            | EditorType::Unknown => None,
        }
    }

    // ─── Full reset ─────────────────────────────────────────────────────────

    /// Reset **every** editor to its initial state.  Use when the workspace
    /// changes (game path switch, full workspace clear, …).
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

        // Boxed editors that were missing from reset — bugs found by tests
        *self.monster_ini_editor = Default::default();
        *self.viewer = Default::default();
        *self.chest_editor = Default::default();
        *self.party_level_db_editor = Default::default();
        *self.party_level_db_level_editor = Default::default();

        // Other owned editors
        self.mod_packager_editor = Default::default();
        self.localization_manager = Default::default();
    }
}
