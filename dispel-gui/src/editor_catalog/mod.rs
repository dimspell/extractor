//! Authoritative editor catalog — the single source of truth for editor kinds.
//!
//! Every editor type is declared exactly once in [`define_editor_catalog!`].
//! From that one list this module generates:
//!
//! * the [`EditorType`] enum (re-exported through `crate::workspace`),
//! * [`EditorType::ALL`] / [`EditorType::iter`] (used by exhaustive tests),
//! * the capability predicates ([`EditorType::supports_save`],
//!   [`EditorType::supports_undo_redo`], [`EditorType::has_edit_history`],
//!   [`EditorType::auto_load_catalog`]),
//! * an [`EditorDescriptor`] table carrying view / save / load / spreadsheet /
//!   JSON-export / lifecycle function pointers, and
//! * the corresponding dispatch entry points ([`dispatch_view`],
//!   [`save_task_for_editor`], [`catalog_load_task`],
//!   [`spreadsheet_nav_msg`], [`from_catalog_editor`]).
//!
//! The descriptor's `lifecycle` field is the single owner of the five
//! `EditorRegistry` lifecycle operations (undo/redo — including spreadsheet
//! cache refresh —, `remove_tab`, `close_all_tabs`, `get_active_edit_history`
//! and `clear_all`) that used to be hand-maintained field lists.
//!
//! Adding a new editor therefore requires touching **one** entry here. The
//! compiler then rejects every non-exhaustive `match` over `EditorType`
//! elsewhere in the crate, so the dispatch arms in `editor_registry` and
//! `update/system` cannot silently drift.
//!
//! This is a "deep module": callers only use the small surface above and never
//! need to know that it is backed by a declarative macro.

use crate::editor_registry::LifecycleFns;
use crate::editor_registry::{
    lc_box_undo, lc_fog_map, lc_hash_map, lc_none, lc_owned, lc_owned_box, lc_sprite_map,
    lc_standard, lc_standard_and, lc_standard_no_history, lc_tabbed,
};
use crate::message::{Message, MessageExt};

/// Render an editor for the given immutable app reference.
pub type ViewFn = for<'a> fn(&'a crate::app::App) -> iced::Element<'a, Message>;
/// Build the `Save` message for an editor. Receives the active tab id.
pub type SaveFn = fn(usize) -> Message;
/// Build the raw `LoadCatalog` message for an editor opened without a file path.
pub type LoadFn = fn() -> Message;
/// Build a spreadsheet-navigation message for an editor.
pub type SheetFn = fn(crate::view::editor::SpreadsheetMessage) -> Message;
/// Extract an editor's file to JSON ("Extract to JSON" in the file tree).
pub type ExtractFn = fn(&std::path::Path) -> Result<serde_json::Value, String>;

/// Static description of one editor kind.
#[derive(Clone, Copy)]
pub struct EditorDescriptor {
    pub editor_type: EditorType,
    /// `None` for editors whose view is rendered specially by the caller
    /// (`DbViewer`, `EventScrEditor`) and for `Unknown`.
    pub view: Option<ViewFn>,
    /// `None` when the editor does not support Ctrl+S.
    pub save: Option<SaveFn>,
    /// `None` when the editor does not auto-load a catalog from the game path.
    pub load: Option<LoadFn>,
    /// `None` when the editor has no generic spreadsheet.
    pub sheet: Option<SheetFn>,
    /// `None` when "Extract to JSON" is unsupported for the editor's file.
    pub extract: Option<ExtractFn>,
    /// Homogeneous lifecycle operations (undo/redo — including spreadsheet
    /// cache refresh —, tab removal/reset, edit-history lookup and full reset).
    ///
    /// Declared once per editor here, so adding a HashMap/tabbed/boxed editor
    /// no longer requires editing the seven hand-maintained tables that used
    /// to live in `editor_registry`.
    pub lifecycle: LifecycleFns,
}

macro_rules! define_editor_catalog {
    ($(
        $(#[$attr:meta])*
        $variant:ident {
            view: $view:expr,
            save: $save:expr,
            load: $load:expr,
            sheet: $sheet:expr,
            extract: $extract:expr,
            lifecycle: $lifecycle:expr,
        }
    ),+ $(,)?) => {
        /// Editor type identifier for workspace tabs.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        pub enum EditorType {
            $($(#[$attr])* $variant,)+
        }

        /// Number of declared editor kinds (including `Unknown`).
        pub const EDITOR_COUNT: usize = [$(stringify!($variant)),+].len();

        static DESCRIPTORS: &[EditorDescriptor] = &[
            $(EditorDescriptor {
                editor_type: EditorType::$variant,
                view: $view,
                save: $save,
                load: $load,
                sheet: $sheet,
                extract: $extract,
                lifecycle: $lifecycle,
            },)+
        ];

        impl EditorType {
            /// Every variant in declaration order — the exhaustive list used
            /// by tests and by tooling. Never hand-copy this list again.
            pub const ALL: &'static [EditorType] = &[$(EditorType::$variant),+];

            pub fn iter() -> impl Iterator<Item = EditorType> {
                Self::ALL.iter().copied()
            }

            /// The authoritative descriptor for this editor kind.
            pub fn descriptor(self) -> &'static EditorDescriptor {
                &DESCRIPTORS[self as usize]
            }

            pub fn name(self) -> &'static str {
                match self { $(EditorType::$variant => stringify!($variant),)+ }
            }

            /// `true` when the editor supports Ctrl+S saving. Derived from the
            /// presence of a save message constructor, so it cannot drift.
            pub fn supports_save(self) -> bool {
                self.descriptor().save.is_some()
            }

            /// `true` when the editor auto-loads a catalog from the game path.
            pub fn auto_load_catalog(self) -> bool {
                self.descriptor().load.is_some()
            }

            /// `true` when the editor has undo/redo support. Derived from its
            /// lifecycle declaration so it cannot disagree with the behavior.
            pub fn supports_undo_redo(self) -> bool {
                self.descriptor().lifecycle.has_undo
            }

            /// `true` when the editor exposes an `EditHistory`.
            pub fn has_edit_history(self) -> bool {
                self.descriptor().lifecycle.has_history
            }
        }
    };
}

define_editor_catalog! {
    /// Weapon database (WeaponItem.db).
    WeaponEditor {
        view: Some(crate::editors::weapon::view as ViewFn),
        save: Some(|_t: usize| Message::weapon(crate::editors::weapon::WeaponEditorMessage::Save)),
        load: Some(|| Message::weapon(crate::editors::weapon::WeaponEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::weapon(crate::editors::weapon::WeaponEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_weapon),
        lifecycle: lc_standard!(weapon_editor),
    },
    /// Monster combat database (Monster.db).
    MonsterEditor {
        view: Some(crate::editors::monster::view as ViewFn),
        save: Some(|_t: usize| Message::monster(crate::editors::monster::MonsterEditorMessage::Save)),
        load: Some(|| Message::monster(crate::editors::monster::MonsterEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::monster(crate::editors::monster::MonsterEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_monster),
        lifecycle: lc_standard_no_history!(monster_editor),
    },
    /// Monster visuals (Monster.ini).
    MonsterIniEditor {
        view: Some(crate::editors::monster_ini::view as ViewFn),
        save: Some(|_t: usize| Message::monster_ini(crate::editors::monster_ini::MonsterIniEditorMessage::Save)),
        load: Some(|| Message::monster_ini(crate::editors::monster_ini::MonsterIniEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::monster_ini(crate::editors::monster_ini::MonsterIniEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_monster_ini),
        lifecycle: lc_standard_no_history!(monster_ini_editor),
    },
    /// Healing items (HealItem.db).
    HealItemEditor {
        view: Some(crate::editors::heal_item::view as ViewFn),
        save: Some(|_t: usize| Message::heal_item(crate::editors::heal_item::HealItemEditorMessage::Save)),
        load: Some(|| Message::heal_item(crate::editors::heal_item::HealItemEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::heal_item(crate::editors::heal_item::HealItemEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_heal_item),
        lifecycle: lc_standard!(heal_item_editor),
    },
    /// Miscellaneous items (MiscItem.db).
    MiscItemEditor {
        view: Some(crate::editors::misc_item::view as ViewFn),
        save: Some(|_t: usize| Message::misc_item(crate::editors::misc_item::MiscItemEditorMessage::Save)),
        load: Some(|| Message::misc_item(crate::editors::misc_item::MiscItemEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::misc_item(crate::editors::misc_item::MiscItemEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_misc_item),
        lifecycle: lc_standard!(misc_item_editor),
    },
    /// Edit items (EditItem.db).
    EditItemEditor {
        view: Some(crate::editors::edit_item::view as ViewFn),
        save: Some(|_t: usize| Message::edit_item(crate::editors::edit_item::EditItemEditorMessage::Save)),
        load: Some(|| Message::edit_item(crate::editors::edit_item::EditItemEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::edit_item(crate::editors::edit_item::EditItemEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_edit_item),
        lifecycle: lc_standard!(edit_item_editor),
    },
    /// Event items (EventItem.db).
    EventItemEditor {
        view: Some(crate::editors::event_item::view as ViewFn),
        save: Some(|_t: usize| Message::event_item(crate::editors::event_item::EventItemEditorMessage::Save)),
        load: Some(|| Message::event_item(crate::editors::event_item::EventItemEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::event_item(crate::editors::event_item::EventItemEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_event_item),
        lifecycle: lc_standard!(event_item_editor),
    },
    /// Magic spells (Magic.db).
    MagicEditor {
        view: Some(crate::editors::magic::view as ViewFn),
        save: Some(|_t: usize| Message::magic(crate::editors::magic::MagicEditorMessage::Save)),
        load: Some(|| Message::magic(crate::editors::magic::MagicEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::magic(crate::editors::magic::MagicEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_magic),
        lifecycle: lc_standard!(magic_editor),
    },
    /// Stores (Store.db).
    StoreEditor {
        view: Some(crate::editors::store::view as ViewFn),
        save: Some(|_t: usize| Message::store(crate::editors::store::StoreEditorMessage::Save)),
        load: Some(|| Message::store(crate::editors::store::StoreEditorMessage::LoadCatalog)),
        sheet: None, // custom layout, no generic spreadsheet
        extract: Some(crate::state::extract_store),
        lifecycle: lc_box_undo!(store_editor),
    },
    /// Character data (ChData.db).
    ChDataEditor {
        view: Some(crate::editors::chdata::view as ViewFn),
        save: Some(|_t: usize| Message::chdata(crate::editors::chdata::ChDataEditorMessage::Save)),
        load: Some(|| Message::chdata(crate::editors::chdata::ChDataEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::chdata(crate::editors::chdata::ChDataEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_chdata),
        lifecycle: lc_standard!(chdata_editor),
    },
    /// Party level database (prtlevel.db).
    PartyLevelDbEditor {
        view: Some(crate::editors::party_level_db::view as ViewFn),
        save: Some(|_t: usize| Message::party_level_db(crate::editors::party_level_db::PartyLevelDbEditorMessage::Save)),
        load: Some(|| Message::party_level_db(crate::editors::party_level_db::PartyLevelDbEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::party_level_db(crate::editors::party_level_db::PartyLevelDbEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_party_level),
        lifecycle: lc_standard_and!(party_level_db_level_editor, party_level_db_editor),
    },
    /// Dialogue script (.dlg), one file per tab.
    DialogueScriptEditor {
        view: Some(crate::editors::dialogue_script::view as ViewFn),
        save: Some(|_t: usize| Message::dialogue_script(crate::editors::dialogue_script::DialogueScriptEditorMessage::Save)),
        load: None, // opened by explicit file path
        sheet: Some(|sm| Message::dialogue_script(crate::editors::dialogue_script::DialogueScriptEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_dialog),
        lifecycle: lc_tabbed!(dialogue_script_editor),
    },
    /// Dialogue text (.pgp), one file per tab.
    DialogueTextEditor {
        view: Some(crate::editors::dialogue_paragraph::view as ViewFn),
        save: Some(|_t: usize| Message::dialogue_paragraph(crate::editors::dialogue_paragraph::DialogueParagraphEditorMessage::Save)),
        load: None, // opened by explicit file path
        sheet: Some(|sm| Message::dialogue_paragraph(crate::editors::dialogue_paragraph::DialogueParagraphEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_dialog_text),
        lifecycle: lc_tabbed!(dialogue_paragraph_editor),
    },
    /// Draw items (DrawItem.ref).
    DrawItemEditor {
        view: Some(crate::editors::draw_item::view as ViewFn),
        save: Some(|_t: usize| Message::draw_item(crate::editors::draw_item::DrawItemEditorMessage::Save)),
        load: Some(|| Message::draw_item(crate::editors::draw_item::DrawItemEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::draw_item(crate::editors::draw_item::DrawItemEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_draw_item),
        lifecycle: lc_standard!(draw_item_editor),
    },
    /// Events (Event.ini).
    EventIniEditor {
        view: Some(crate::editors::event_ini::view as ViewFn),
        save: Some(|_t: usize| Message::event_ini(crate::editors::event_ini::EventIniEditorMessage::Save)),
        load: Some(|| Message::event_ini(crate::editors::event_ini::EventIniEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::event_ini(crate::editors::event_ini::EventIniEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_event_ini),
        lifecycle: lc_standard!(event_ini_editor),
    },
    /// Event NPCs (eventnpc.ref).
    EventNpcRefEditor {
        view: Some(crate::editors::event_npc_ref::view as ViewFn),
        save: Some(|_t: usize| Message::event_npc_ref(crate::editors::event_npc_ref::EventNpcRefEditorMessage::Save)),
        load: Some(|| Message::event_npc_ref(crate::editors::event_npc_ref::EventNpcRefEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::event_npc_ref(crate::editors::event_npc_ref::EventNpcRefEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_event_npc_ref),
        lifecycle: lc_standard!(event_npc_ref_editor),
    },
    /// Extra objects (Extra.ini).
    ExtraIniEditor {
        view: Some(crate::editors::extra_ini::view as ViewFn),
        save: Some(|_t: usize| Message::extra_ini(crate::editors::extra_ini::ExtraIniEditorMessage::Save)),
        load: Some(|| Message::extra_ini(crate::editors::extra_ini::ExtraIniEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::extra_ini(crate::editors::extra_ini::ExtraIniEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_extra_ini),
        lifecycle: lc_standard!(extra_ini_editor),
    },
    /// Extra object placements (Ext*.ref), one file per tab.
    ExtraRefEditor {
        view: Some(crate::editors::extra_ref::view as ViewFn),
        save: Some(|_t: usize| Message::extra_ref(crate::editors::extra_ref::ExtraRefEditorMessage::Save)),
        load: None, // opened by explicit file path
        sheet: Some(|sm| Message::extra_ref(crate::editors::extra_ref::ExtraRefEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_extra_ref),
        lifecycle: lc_tabbed!(extra_ref_editor),
    },
    /// Per-map configuration (Map.ini).
    MapIniEditor {
        view: Some(crate::editors::map_ini::view as ViewFn),
        save: Some(|_t: usize| Message::map_ini(crate::editors::map_ini::MapIniEditorMessage::Save)),
        load: Some(|| Message::map_ini(crate::editors::map_ini::MapIniEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::map_ini(crate::editors::map_ini::MapIniEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_map_ini),
        lifecycle: lc_standard!(map_ini_editor),
    },
    /// Text messages (message.scr).
    MessageScrEditor {
        view: Some(crate::editors::message_scr::view as ViewFn),
        save: Some(|_t: usize| Message::message_scr(crate::editors::message_scr::MessageScrEditorMessage::Save)),
        load: Some(|| Message::message_scr(crate::editors::message_scr::MessageScrEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::message_scr(crate::editors::message_scr::MessageScrEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_message),
        lifecycle: lc_standard!(message_scr_editor),
    },
    /// Monster placements (Mondun*.ref), one file per tab.
    MonsterRefEditor {
        view: Some(crate::editors::monster_ref::view as ViewFn),
        save: Some(|_t: usize| Message::monster_ref(crate::editors::monster_ref::MonsterRefEditorMessage::Save)),
        load: None, // opened by explicit file path
        sheet: Some(|sm| Message::monster_ref(crate::editors::monster_ref::MonsterRefEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_monster_ref),
        lifecycle: lc_tabbed!(monster_ref_editor),
    },
    /// NPC visuals (Npc.ini).
    NpcIniEditor {
        view: Some(crate::editors::npc_ini::view as ViewFn),
        save: Some(|_t: usize| Message::npc_ini(crate::editors::npc_ini::NpcIniEditorMessage::Save)),
        load: Some(|| Message::npc_ini(crate::editors::npc_ini::NpcIniEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::npc_ini(crate::editors::npc_ini::NpcIniEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_npc_ini),
        lifecycle: lc_standard_no_history!(npc_ini_editor),
    },
    /// NPC placements (Npc*.ref), one file per tab.
    NpcRefEditor {
        view: Some(crate::editors::npc_ref::view as ViewFn),
        save: Some(|_t: usize| Message::npc_ref(crate::editors::npc_ref::NpcRefEditorMessage::Save)),
        load: None, // opened by explicit file path
        sheet: Some(|sm| Message::npc_ref(crate::editors::npc_ref::NpcRefEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_npc_ref),
        lifecycle: lc_tabbed!(npc_ref_editor),
    },
    /// Party references (partyref.ref).
    PartyRefEditor {
        view: Some(crate::editors::party_ref::view as ViewFn),
        save: Some(|_t: usize| Message::party_ref(crate::editors::party_ref::PartyRefEditorMessage::Save)),
        load: Some(|| Message::party_ref(crate::editors::party_ref::PartyRefEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::party_ref(crate::editors::party_ref::PartyRefEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_party_ref),
        lifecycle: lc_standard!(party_ref_editor),
    },
    /// Party NPCs (prtini.db).
    PartyIniEditor {
        view: Some(crate::editors::party_ini::view as ViewFn),
        save: Some(|_t: usize| Message::party_ini(crate::editors::party_ini::PartyIniEditorMessage::Save)),
        load: Some(|| Message::party_ini(crate::editors::party_ini::PartyIniEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::party_ini(crate::editors::party_ini::PartyIniEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_party_ini),
        lifecycle: lc_standard!(party_ini_editor),
    },
    /// Quests (quest.scr).
    QuestScrEditor {
        view: Some(crate::editors::quest_scr::view as ViewFn),
        save: Some(|_t: usize| Message::quest_scr(crate::editors::quest_scr::QuestScrEditorMessage::Save)),
        load: Some(|| Message::quest_scr(crate::editors::quest_scr::QuestScrEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::quest_scr(crate::editors::quest_scr::QuestScrEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_quest),
        lifecycle: lc_standard!(quest_scr_editor),
    },
    /// Event scripts (event*.scr). View is wrapped by the caller.
    EventScrEditor {
        view: None, // rendered by view/mod.rs with EditorMessage::EventScr wrapping
        save: Some(|_t: usize| Message::event_scr(crate::editors::event_scr::EventScrEditorMessage::SaveScript)),
        load: None, // opened by explicit file path
        sheet: None,
        extract: Some(crate::state::extract_event_scr),
        lifecycle: lc_owned_box!(event_scr_editor),
    },
    /// Wave configuration (wave.ini).
    WaveIniEditor {
        view: Some(crate::editors::wave_ini::view as ViewFn),
        save: Some(|_t: usize| Message::wave_ini(crate::editors::wave_ini::WaveIniEditorMessage::Save)),
        load: Some(|| Message::wave_ini(crate::editors::wave_ini::WaveIniEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::wave_ini(crate::editors::wave_ini::WaveIniEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_wave_ini),
        lifecycle: lc_standard!(wave_ini_editor),
    },
    /// Global map list (AllMap.ini).
    AllMapIniEditor {
        view: Some(crate::editors::all_map_ini::view as ViewFn),
        save: Some(|_t: usize| Message::all_map_ini(crate::editors::all_map_ini::AllMapIniEditorMessage::Save)),
        load: Some(|| Message::all_map_ini(crate::editors::all_map_ini::AllMapIniEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::all_map_ini(crate::editors::all_map_ini::AllMapIniEditorMessage::Spreadsheet(sm))),
        extract: Some(crate::state::extract_all_maps),
        lifecycle: lc_standard!(all_map_ini_editor),
    },
    /// Sprite viewer (.spr), one per tab; snapshot-based undo.
    SpriteViewer {
        view: Some(crate::editors::sprite_editor::view as ViewFn),
        save: Some(|_t: usize| Message::sprite_viewer(crate::editors::sprite_editor::SpriteViewerMessage::Save)),
        load: None, // opened by explicit file path
        sheet: None,
        extract: Some(crate::state::extract_sprite_file_json),
        lifecycle: lc_sprite_map!(sprite_viewers),
    },
    /// Sound effects (.snf), read-only playback, one per tab.
    SnfEditor {
        view: Some(crate::editors::snf_editor::view as ViewFn),
        save: None,
        load: None, // opened by explicit file path
        sheet: None,
        extract: None,
        lifecycle: lc_hash_map!(snf_editors),
    },
    /// SQLite database viewer (tool tab, no EditorMessage variant).
    DbViewer {
        view: None, // rendered by App::view_db_viewer()
        save: None,
        load: None,
        sheet: None,
        extract: None,
        lifecycle: lc_owned_box!(viewer),
    },
    /// Tileset browser (.btl/.gtl), one per tab.
    TilesetEditor {
        view: Some(crate::editors::tileset::view as ViewFn),
        save: None,
        load: None, // opened by explicit file path
        sheet: None,
        extract: Some(crate::state::extract_tileset_file_json),
        lifecycle: lc_hash_map!(tileset_editors),
    },
    /// Visual map editor (.map), one per tab; own undo stack.
    MapEditor {
        view: Some(crate::editors::map_editor::view as ViewFn),
        save: Some(|t: usize| Message::map_editor(crate::editors::map_editor::MapEditorMessage::SaveEntities(t))),
        load: None, // opened by explicit file path
        sheet: None,
        extract: Some(crate::state::extract_map_file_json),
        lifecycle: lc_hash_map!(map_editors),
    },
    /// Mod packaging tool tab.
    ModPackager {
        view: Some(crate::editors::mod_packager::view as ViewFn),
        save: None,
        load: None,
        sheet: None,
        extract: None,
        lifecycle: lc_owned!(mod_packager_editor),
    },
    /// Localization manager tool tab.
    LocalizationManager {
        view: Some(crate::editors::localization_manager::view as ViewFn),
        save: None,
        load: None,
        sheet: None,
        extract: None,
        lifecycle: lc_owned!(localization_manager),
    },
    /// Read-only save-file viewer (.sav), one per tab.
    SaveFileViewer {
        view: Some(crate::editors::save_file_viewer::view as ViewFn),
        save: None,
        load: None, // opened by explicit file path (custom async parse in app.rs)
        sheet: None,
        extract: None,
        lifecycle: lc_hash_map!(save_file_viewers),
    },
    /// Save-slot manager (Save.ifo).
    SaveIfoEditor {
        view: Some(crate::editors::save_ifo::view as ViewFn),
        save: Some(|_t: usize| Message::save_ifo(crate::editors::save_ifo::SaveIfoEditorMessage::Save)),
        load: Some(|| Message::save_ifo(crate::editors::save_ifo::SaveIfoEditorMessage::LoadCatalog)),
        sheet: None, // custom layout, no generic spreadsheet
        extract: None,
        lifecycle: lc_box_undo!(save_ifo_editor),
    },
    /// Map-lighting fade tables (fogdata.dat), one per tab.
    FogDataEditor {
        view: Some(crate::editors::fog_data::view as ViewFn),
        save: Some(|t: usize| Message::fog_data(crate::editors::fog_data::FogDataMessage::Save(t))),
        load: None, // opened by explicit file path
        sheet: None,
        extract: None,
        lifecycle: lc_fog_map!(fog_editors),
    },
    /// Multiplayer chest spawns (.cst).
    ChestSpawnEditor {
        view: Some(crate::editors::chest_spawn::view as ViewFn),
        save: Some(|_t: usize| Message::chest_spawn(crate::editors::chest_spawn::ChestSpawnEditorMessage::Save)),
        load: Some(|| Message::chest_spawn(crate::editors::chest_spawn::ChestSpawnEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::chest_spawn(crate::editors::chest_spawn::ChestSpawnEditorMessage::Spreadsheet(sm))),
        extract: None,
        lifecycle: lc_standard!(chest_spawn_editor),
    },
    /// Multiplayer monster spawns (.mon).
    MonsterSpawnEditor {
        view: Some(crate::editors::monster_spawn::view as ViewFn),
        save: Some(|_t: usize| Message::monster_spawn(crate::editors::monster_spawn::MonsterSpawnEditorMessage::Save)),
        load: Some(|| Message::monster_spawn(crate::editors::monster_spawn::MonsterSpawnEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::monster_spawn(crate::editors::monster_spawn::MonsterSpawnEditorMessage::Spreadsheet(sm))),
        extract: None,
        lifecycle: lc_standard!(monster_spawn_editor),
    },
    /// Multiplayer map metadata (.ifm).
    MapInfoMultiEditor {
        view: Some(crate::editors::map_info_multi::view as ViewFn),
        save: Some(|_t: usize| Message::map_info_multi(crate::editors::map_info_multi::MapInfoMultiEditorMessage::Save)),
        load: Some(|| Message::map_info_multi(crate::editors::map_info_multi::MapInfoMultiEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::map_info_multi(crate::editors::map_info_multi::MapInfoMultiEditorMessage::Spreadsheet(sm))),
        extract: None,
        lifecycle: lc_standard!(map_info_multi_editor),
    },
    /// Multiplayer tile grid (.dat), canvas-based view.
    TileGridEditor {
        view: Some(crate::editors::tile_grid_canvas::view as ViewFn),
        save: Some(|_t: usize| Message::tile_grid(crate::editors::tile_grid::TileGridEditorMessage::Save)),
        load: Some(|| Message::tile_grid(crate::editors::tile_grid::TileGridEditorMessage::LoadCatalog)),
        sheet: Some(|sm| Message::tile_grid(crate::editors::tile_grid::TileGridEditorMessage::Spreadsheet(sm))),
        extract: None,
        lifecycle: lc_standard_and!(tile_grid_editor, tile_grid_view),
    },
    /// Universal fallback binary editor.
    HexEditor {
        view: Some(crate::editors::hex_wrapper::view as ViewFn),
        save: None,
        load: None, // opened by explicit file path
        sheet: None,
        extract: None,
        lifecycle: lc_hash_map!(hex_editors),
    },
    /// Deserialized fallback for unknown editor types (serde `other`).
    #[serde(other)]
    Unknown {
        view: None,
        save: None,
        load: None,
        sheet: None,
        extract: None,
        lifecycle: lc_none!(),
    },
}

/// Look up the descriptor for an editor kind.
pub fn descriptor(et: EditorType) -> &'static EditorDescriptor {
    et.descriptor()
}

/// Map a `dispel-core` catalog `editor` id to its GUI variant.
///
/// Derived from `EditorType::name()` (which is generated from the same macro
/// declaration as the enum), so the id→variant table cannot drift from the
/// catalog. Returns `None` for ids that have no GUI editor.
pub fn from_catalog_editor(id: &str) -> Option<EditorType> {
    EditorType::ALL.iter().copied().find(|et| et.name() == id)
}

/// Route an `EditorType` to its view, or `None` for editors rendered specially
/// by the caller (`None` tab, `DbViewer`, `EventScrEditor`, `Unknown`).
pub fn dispatch_view<'a>(
    editor_type: Option<EditorType>,
    app: &'a crate::app::App,
) -> Option<iced::Element<'a, Message>> {
    descriptor(editor_type?).view.map(|render| render(app))
}

/// Return the `Save` task for an editor, or `None` when saving is unsupported.
pub fn save_task_for_editor(editor_type: EditorType, tab_id: usize) -> Option<iced::Task<Message>> {
    descriptor(editor_type)
        .save
        .map(|build| iced::Task::done(build(tab_id)))
}

/// Return the auto-load catalog task for an editor opened without a file path.
pub fn catalog_load_task(editor_type: EditorType) -> Option<iced::Task<Message>> {
    descriptor(editor_type)
        .load
        .map(|build| iced::Task::done(build()))
}

/// Map a spreadsheet-navigation message to the editor's `Message` variant, or
/// `None` when the editor has no generic spreadsheet.
pub fn spreadsheet_nav_msg(
    editor_type: EditorType,
    sm: crate::view::editor::SpreadsheetMessage,
) -> Option<Message> {
    descriptor(editor_type).sheet.map(|build| build(sm))
}

/// Read an explicit file path into a standard editor's `CatalogLoaded` slot.
///
/// This is the explicit-path counterpart to [`catalog_load_task`] (which
/// derives its path from the configured game directory). Keeping both load
/// paths here avoids duplicating the `Task::perform` boilerplate in
/// `app::App::open_file_in_workspace`.
pub fn load_catalog_from_path<T, W>(path: std::path::PathBuf, wrap: W) -> iced::Task<Message>
where
    T: dispel_core::Extractor + Clone + std::fmt::Debug + Send + 'static,
    W: Fn(crate::components::standard::message::StandardEditorMessage<T>) -> Message
        + Send
        + 'static,
{
    iced::Task::perform(
        async move { T::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
        move |result| {
            wrap(crate::components::standard::message::StandardEditorMessage::CatalogLoaded(result))
        },
    )
}
