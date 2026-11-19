use std::collections::HashMap;

use crate::components::edit_history::EditHistory;
use crate::components::generic_editor::TabbedEditor;
use crate::components::standard::StandardEditor;
use crate::editors::all_map_ini::AllMapIniEditorState;
use crate::editors::chdata::ChDataEditorState;
use crate::editors::db_viewer::DbViewerState;
use crate::editors::draw_item::DrawItemEditorState;
use crate::editors::event_ini::EventIniEditorState;
use crate::editors::event_npc_ref::EventNpcRefEditorState;
use crate::editors::event_scr::EventScriptEditorState;
use crate::editors::extra_ini::ExtraIniEditorState;
use crate::editors::fog_data::FogDataEditorState;
use crate::editors::magic::MagicEditorState;
use crate::editors::map_editor::MapEditorState;
use crate::editors::map_ini::MapIniEditorState;
use crate::editors::message_scr::MessageScrEditorState;
use crate::editors::monster_ini::MonsterIniEditorState;
use crate::editors::npc_ini::NpcIniEditorState;
use crate::editors::party_ini::PartyIniEditorState;
use crate::editors::party_level_db::PartyLevelDbEditorState;
use crate::editors::quest_scr::QuestScrEditorState;
use crate::editors::save_file_viewer::SaveFileViewerState;
use crate::editors::save_ifo::SaveIfoEditorState;
use crate::editors::snf_editor::SnfEditorState;
use crate::editors::sprite_editor::SpriteViewerState;
use crate::editors::store::StoreEditorState;
use crate::editors::tileset::TilesetEditorState;
use crate::editors::wave_ini::WaveIniEditorState;

use crate::editors::{localization_manager, mod_packager};
use crate::workspace::EditorType;
use dispel_core::{
    DialogueParagraph, DialogueScript, EditItem, EventItem, ExtraRef, HealItem, MiscItem, Monster,
    MonsterRef, NPC, PartyLevelRecord, PartyRef, WeaponItem,
};
use hexedit::HexEditorState;

// ===========================================================================
// Descriptor-driven editor lifecycle
// ===========================================================================
//
// Every EditorType declares one `LifecycleFns` value in the authoritative
// editor catalog.  The operations below used to be hand-maintained match/field
// lists in this file; they drifted (see the "boxed editors that were missing
// from reset" block that had to be bolted onto `clear_all`).  Now each editor
// field is named exactly once, in its catalog entry.  Spreadsheet caches are
// refreshed inside the undo/redo operation itself (via the shared
// `refresh_spreadsheet_caches` helper), so no separate refresh pass is needed.

/// Lookup table passed to undo/redo for display-name resolution.
pub type Lookups = HashMap<String, Vec<(String, String)>>;

/// Direction of a lifecycle undo/redo dispatch.
#[derive(Clone, Copy, Debug)]
pub enum UndoRedoOp {
    Undo,
    Redo,
}

/// Homogeneous, field-agnostic lifecycle operations for one editor kind.
///
/// Stored once per editor in [`crate::editor_catalog::EditorDescriptor`].  The
/// closures are non-capturing (they coerce to `fn` pointers) and receive the
/// registry plus the active tab id, so a single catalog entry is enough to
/// drive undo/redo (including spreadsheet cache refresh), tab removal/reset,
/// edit-history lookup and full reset.
#[derive(Clone, Copy)]
pub struct LifecycleFns {
    /// Capabilities, declared once alongside the operations so the catalog's
    /// `supports_undo_redo` / `has_edit_history` predicates cannot drift from
    /// the behavior the lifecycle functions actually implement.
    pub has_undo: bool,
    pub has_history: bool,
    pub has_tab_state: bool,
    pub undo_redo: fn(&mut EditorRegistry, usize, &Lookups, UndoRedoOp) -> Option<String>,
    pub remove_tab: fn(&mut EditorRegistry, usize),
    pub clear_tabs: fn(&mut EditorRegistry),
    pub edit_history: for<'a> fn(&'a EditorRegistry, usize) -> Option<&'a EditHistory>,
    pub clear: fn(&mut EditorRegistry),
}

pub(crate) fn noop_undo_redo(
    _: &mut EditorRegistry,
    _: usize,
    _: &Lookups,
    _: UndoRedoOp,
) -> Option<String> {
    None
}
pub(crate) fn noop_remove_tab(_: &mut EditorRegistry, _: usize) {}
pub(crate) fn noop_clear_tabs(_: &mut EditorRegistry) {}
pub(crate) fn noop_edit_history(_: &EditorRegistry, _: usize) -> Option<&EditHistory> {
    None
}
pub(crate) fn noop_clear(_: &mut EditorRegistry) {}

/// `StandardEditor<T>` stored in a `Box`: undo/redo take lookups and the editor
/// exposes an `EditHistory`. No per-tab state.
macro_rules! lc_standard {
    ($field:ident) => {
        crate::editor_registry::LifecycleFns {
            has_undo: true,
            has_history: true,
            has_tab_state: false,
            undo_redo: |r: &mut crate::editor_registry::EditorRegistry,
                        _tab: usize,
                        lookups: &crate::editor_registry::Lookups,
                        op: crate::editor_registry::UndoRedoOp| {
                match op {
                    crate::editor_registry::UndoRedoOp::Undo => r.$field.undo(lookups),
                    crate::editor_registry::UndoRedoOp::Redo => r.$field.redo(lookups),
                }
            },
            remove_tab: |_: &mut crate::editor_registry::EditorRegistry, _: usize| {},
            clear_tabs: |_: &mut crate::editor_registry::EditorRegistry| {},
            edit_history: |r: &crate::editor_registry::EditorRegistry,
                           _tab: usize|
             -> Option<&crate::components::edit_history::EditHistory> {
                Some(r.$field.edit_history())
            },
            clear: |r: &mut crate::editor_registry::EditorRegistry| {
                *r.$field = Default::default();
            },
        }
    };
}
pub(crate) use lc_standard;

/// Like [`lc_standard`], but the editor intentionally does not expose its
/// `EditHistory` to the history panel (e.g. Monster/MonsterIni/NpcIni).
macro_rules! lc_standard_no_history {
    ($field:ident) => {
        crate::editor_registry::LifecycleFns {
            has_history: false,
            edit_history: |_: &crate::editor_registry::EditorRegistry,
                           _: usize|
             -> Option<&crate::components::edit_history::EditHistory> {
                None
            },
            ..lc_standard!($field)
        }
    };
}
pub(crate) use lc_standard_no_history;

/// `StandardEditor<T>` with an additional owned field that must be reset by
/// `clear_all` (PartyLevelDb's view state, TileGrid's canvas state).
macro_rules! lc_standard_and {
    ($field:ident, $extra:ident) => {
        crate::editor_registry::LifecycleFns {
            clear: |r: &mut crate::editor_registry::EditorRegistry| {
                *r.$field = Default::default();
                r.$extra = Default::default();
            },
            ..lc_standard!($field)
        }
    };
}
pub(crate) use lc_standard_and;

/// Custom boxed editor implementing `UndoRedo` without lookups + `EditHistory`.
macro_rules! lc_box_undo {
    ($field:ident) => {
        crate::editor_registry::LifecycleFns {
            has_undo: true,
            has_history: true,
            has_tab_state: false,
            undo_redo: |r: &mut crate::editor_registry::EditorRegistry,
                        _tab: usize,
                        _lookups: &crate::editor_registry::Lookups,
                        op: crate::editor_registry::UndoRedoOp| {
                use crate::components::generic_editor::UndoRedo;
                match op {
                    crate::editor_registry::UndoRedoOp::Undo => r.$field.undo(),
                    crate::editor_registry::UndoRedoOp::Redo => r.$field.redo(),
                }
            },
            remove_tab: |_: &mut crate::editor_registry::EditorRegistry, _: usize| {},
            clear_tabs: |_: &mut crate::editor_registry::EditorRegistry| {},
            edit_history: |r: &crate::editor_registry::EditorRegistry,
                           _tab: usize|
             -> Option<&crate::components::edit_history::EditHistory> {
                use crate::components::generic_editor::UndoRedo;
                Some(r.$field.edit_history())
            },
            clear: |r: &mut crate::editor_registry::EditorRegistry| {
                *r.$field = Default::default();
            },
        }
    };
}
pub(crate) use lc_box_undo;

/// `TabbedEditor<T>`: per-tab undo/redo, edit history and spreadsheet caches.
///
/// Undo/redo go through `undo_with_cache` / `redo_with_cache`, which refresh
/// that tab's spreadsheet caches via the shared `refresh_spreadsheet_caches`
/// helper in the same operation — no separate refresh pass is needed.
macro_rules! lc_tabbed {
    ($field:ident) => {
        crate::editor_registry::LifecycleFns {
            has_undo: true,
            has_history: true,
            has_tab_state: true,
            undo_redo: |r: &mut crate::editor_registry::EditorRegistry,
                        tab: usize,
                        lookups: &crate::editor_registry::Lookups,
                        op: crate::editor_registry::UndoRedoOp| {
                match op {
                    crate::editor_registry::UndoRedoOp::Undo => {
                        r.$field.undo_with_cache(tab, lookups)
                    }
                    crate::editor_registry::UndoRedoOp::Redo => {
                        r.$field.redo_with_cache(tab, lookups)
                    }
                }
            },
            remove_tab: |r: &mut crate::editor_registry::EditorRegistry, tab: usize| {
                r.$field.remove(&tab);
            },
            clear_tabs: |r: &mut crate::editor_registry::EditorRegistry| {
                r.$field.clear();
            },
            edit_history: |r: &crate::editor_registry::EditorRegistry,
                           tab: usize|
             -> Option<&crate::components::edit_history::EditHistory> {
                r.$field.editors.get(&tab).map(|e| e.edit_history())
            },
            clear: |r: &mut crate::editor_registry::EditorRegistry| {
                r.$field.clear();
            },
        }
    };
}
pub(crate) use lc_tabbed;

/// Plain per-tab `HashMap`: tab lifecycle only, no undo/history.
macro_rules! lc_hash_map {
    ($field:ident) => {
        crate::editor_registry::LifecycleFns {
            has_undo: false,
            has_history: false,
            has_tab_state: true,
            undo_redo: crate::editor_registry::noop_undo_redo,
            remove_tab: |r: &mut crate::editor_registry::EditorRegistry, tab: usize| {
                r.$field.remove(&tab);
            },
            clear_tabs: |r: &mut crate::editor_registry::EditorRegistry| {
                r.$field.clear();
            },
            edit_history: crate::editor_registry::noop_edit_history,
            clear: |r: &mut crate::editor_registry::EditorRegistry| {
                r.$field.clear();
            },
        }
    };
}
pub(crate) use lc_hash_map;

/// Per-tab `HashMap` with snapshot-based undo (SpriteViewer).
macro_rules! lc_sprite_map {
    ($field:ident) => {
        crate::editor_registry::LifecycleFns {
            has_undo: true,
            has_history: false,
            has_tab_state: true,
            undo_redo: |r: &mut crate::editor_registry::EditorRegistry,
                        tab: usize,
                        _lookups: &crate::editor_registry::Lookups,
                        op: crate::editor_registry::UndoRedoOp| {
                r.$field.get_mut(&tab).map(|viewer| {
                    match op {
                        crate::editor_registry::UndoRedoOp::Undo => viewer.undo(),
                        crate::editor_registry::UndoRedoOp::Redo => viewer.redo(),
                    }
                    String::new() // no status message needed
                })
            },
            remove_tab: |r: &mut crate::editor_registry::EditorRegistry, tab: usize| {
                r.$field.remove(&tab);
            },
            clear_tabs: |r: &mut crate::editor_registry::EditorRegistry| {
                r.$field.clear();
            },
            edit_history: crate::editor_registry::noop_edit_history,
            clear: |r: &mut crate::editor_registry::EditorRegistry| {
                r.$field.clear();
            },
        }
    };
}
pub(crate) use lc_sprite_map;

/// Per-tab `HashMap` with boolean snapshot undo (FogDataEditor).
macro_rules! lc_fog_map {
    ($field:ident) => {
        crate::editor_registry::LifecycleFns {
            has_undo: true,
            has_history: false,
            has_tab_state: true,
            undo_redo: |r: &mut crate::editor_registry::EditorRegistry,
                        tab: usize,
                        _lookups: &crate::editor_registry::Lookups,
                        op: crate::editor_registry::UndoRedoOp| {
                r.$field.get_mut(&tab).and_then(|editor| {
                    let changed = match op {
                        crate::editor_registry::UndoRedoOp::Undo => editor.undo(),
                        crate::editor_registry::UndoRedoOp::Redo => editor.redo(),
                    };
                    if changed { Some(String::new()) } else { None }
                })
            },
            remove_tab: |r: &mut crate::editor_registry::EditorRegistry, tab: usize| {
                r.$field.remove(&tab);
            },
            clear_tabs: |r: &mut crate::editor_registry::EditorRegistry| {
                r.$field.clear();
            },
            edit_history: crate::editor_registry::noop_edit_history,
            clear: |r: &mut crate::editor_registry::EditorRegistry| {
                r.$field.clear();
            },
        }
    };
}
pub(crate) use lc_fog_map;

/// Single-instance owned state with no undo/history/tab lifecycle.
macro_rules! lc_owned {
    ($field:ident) => {
        crate::editor_registry::LifecycleFns {
            has_undo: false,
            has_history: false,
            has_tab_state: false,
            undo_redo: crate::editor_registry::noop_undo_redo,
            remove_tab: crate::editor_registry::noop_remove_tab,
            clear_tabs: crate::editor_registry::noop_clear_tabs,
            edit_history: crate::editor_registry::noop_edit_history,
            clear: |r: &mut crate::editor_registry::EditorRegistry| {
                r.$field = Default::default();
            },
        }
    };
}
pub(crate) use lc_owned;

/// Single-instance `Box` state with no undo/history/tab lifecycle.
macro_rules! lc_owned_box {
    ($field:ident) => {
        crate::editor_registry::LifecycleFns {
            has_undo: false,
            has_history: false,
            has_tab_state: false,
            undo_redo: crate::editor_registry::noop_undo_redo,
            remove_tab: crate::editor_registry::noop_remove_tab,
            clear_tabs: crate::editor_registry::noop_clear_tabs,
            edit_history: crate::editor_registry::noop_edit_history,
            clear: |r: &mut crate::editor_registry::EditorRegistry| {
                *r.$field = Default::default();
            },
        }
    };
}
pub(crate) use lc_owned_box;

/// Editors with no registry state at all (`Unknown`).
macro_rules! lc_none {
    () => {
        crate::editor_registry::LifecycleFns {
            has_undo: false,
            has_history: false,
            has_tab_state: false,
            undo_redo: crate::editor_registry::noop_undo_redo,
            remove_tab: crate::editor_registry::noop_remove_tab,
            clear_tabs: crate::editor_registry::noop_clear_tabs,
            edit_history: crate::editor_registry::noop_edit_history,
            clear: crate::editor_registry::noop_clear,
        }
    };
}
pub(crate) use lc_none;

/// Aggregates all editor state fields (35+ types).
///
/// Extracted from `AppState` so that editor lifecycle (clear / reset) is
/// handled in one place and the main state struct stays focused on
/// non-editor concerns (workspace, UI, file tree, …).
#[derive(Default)]
pub struct EditorRegistry {
    pub viewer: Box<DbViewerState>,
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
    pub save_ifo_editor: Box<SaveIfoEditorState>,
    pub map_editors: HashMap<usize, MapEditorState>,
    pub tileset_editors: HashMap<usize, TilesetEditorState>,
    pub snf_editors: HashMap<usize, SnfEditorState>,
    pub hex_editors: HashMap<usize, HexEditorState>,
    pub mod_packager_editor: mod_packager::ModPackagerState,
    pub localization_manager: localization_manager::LocalizationManagerState,
    pub save_file_viewers: HashMap<usize, SaveFileViewerState>,
    pub fog_editors: HashMap<usize, FogDataEditorState>,
    pub chest_spawn_editor: Box<StandardEditor<dispel_core::ChestSpawn>>,
    pub monster_spawn_editor: Box<StandardEditor<dispel_core::MonsterSpawn>>,
    pub map_info_multi_editor: Box<StandardEditor<dispel_core::MapInfoMulti>>,
    pub tile_grid_editor: Box<StandardEditor<dispel_core::TileGrid>>,
    pub tile_grid_view: crate::editors::tile_grid::TileGridViewState,
}

impl EditorRegistry {
    // ─── Tab lifecycle ───────────────────────────────────────────────────────

    /// Remove editor state for a single closed tab.
    ///
    /// Driven by the `remove_tab` function in each catalog descriptor, so a
    /// new HashMap / TabbedEditor only needs one catalog entry.
    pub fn remove_tab(&mut self, tab_id: usize) {
        for et in EditorType::iter() {
            (et.descriptor().lifecycle.remove_tab)(self, tab_id);
        }
    }

    /// Clear editors for every tab.  Use when the workspace is about to lose
    /// all its tabs (CloseAll, workspace reset, …).
    ///
    /// This is **less aggressive** than [`clear_all`](Self::clear_all) in that
    /// it does **not** reset single-instance Box editors (weapon_editor,
    /// monster_editor, …) to their default state — only tab-indexed state is
    /// dropped.
    pub fn close_all_tabs(&mut self) {
        for et in EditorType::iter() {
            (et.descriptor().lifecycle.clear_tabs)(self);
        }
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
        if !editor_type.supports_undo_redo() {
            return None;
        }
        (editor_type.descriptor().lifecycle.undo_redo)(self, tab_id, lookups, UndoRedoOp::Undo)
    }

    /// Perform redo on the editor identified by `editor_type` / `tab_id`.
    /// Returns a status message, or `None` if there's nothing to redo.
    pub fn redo_active(
        &mut self,
        editor_type: EditorType,
        tab_id: usize,
        lookups: &HashMap<String, Vec<(String, String)>>,
    ) -> Option<String> {
        if !editor_type.supports_undo_redo() {
            return None;
        }
        (editor_type.descriptor().lifecycle.undo_redo)(self, tab_id, lookups, UndoRedoOp::Redo)
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
        if !editor_type.has_edit_history() {
            return None;
        }
        (editor_type.descriptor().lifecycle.edit_history)(self, tab_id)
    }

    // ─── Full reset ─────────────────────────────────────────────────────────

    /// Reset **every** editor to its initial state.  Use when the workspace
    /// changes (game path switch, full workspace clear, …).
    ///
    /// The `clear` function is declared once per editor in the catalog, so a
    /// new editor field cannot be forgotten here (the old implementation
    /// needed a follow-up "boxed editors that were missing from reset" patch).
    pub fn clear_all(&mut self) {
        for et in EditorType::iter() {
            (et.descriptor().lifecycle.clear)(self);
        }
    }
}
