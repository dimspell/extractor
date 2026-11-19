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
//! * an [`EditorDescriptor`] table carrying view / save / load / spreadsheet
//!   function pointers, and
//! * the corresponding dispatch entry points ([`dispatch_view`],
//!   [`save_task_for_editor`], [`catalog_load_task`],
//!   [`spreadsheet_nav_msg`]).
//!
//! Adding a new editor therefore requires touching **one** entry here. The
//! compiler then rejects every non-exhaustive `match` over `EditorType`
//! elsewhere in the crate, so the dispatch arms in `editor_registry` and
//! `update/system` cannot silently drift.
//!
//! This is a "deep module": callers only use the small surface above and never
//! need to know that it is backed by a declarative macro.

use crate::message::{Message, MessageExt};

/// Render an editor for the given immutable app reference.
pub type ViewFn = for<'a> fn(&'a crate::app::App) -> iced::Element<'a, Message>;
/// Build the `Save` message for an editor. Receives the active tab id.
pub type SaveFn = fn(usize) -> Message;
/// Build the `LoadCatalog` task for an editor opened without a file path.
pub type LoadFn = fn() -> iced::Task<Message>;
/// Build a spreadsheet-navigation message for an editor.
pub type SheetFn = fn(crate::view::editor::SpreadsheetMessage) -> Message;

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
}

macro_rules! define_editor_catalog {
    ($(
        $(#[$attr:meta])*
        $variant:ident {
            undo: $undo:literal,
            history: $history:literal,
            view: $view:expr,
            save: $save:expr,
            load: $load:expr,
            sheet: $sheet:expr,
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

            pub fn supports_undo_redo(self) -> bool {
                match self { $(EditorType::$variant => $undo,)+ }
            }

            pub fn has_edit_history(self) -> bool {
                match self { $(EditorType::$variant => $history,)+ }
            }
        }
    };
}

define_editor_catalog! {
    /// Weapon database (WeaponItem.db).
    WeaponEditor {
        undo: true, history: true,
        view: Some(crate::editors::weapon::view as ViewFn),
        save: Some(|_t: usize| Message::weapon(crate::editors::weapon::WeaponEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::weapon(crate::editors::weapon::WeaponEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::weapon(crate::editors::weapon::WeaponEditorMessage::Spreadsheet(sm))),
    },
    /// Monster combat database (Monster.db).
    MonsterEditor {
        undo: true, history: false,
        view: Some(crate::editors::monster::view as ViewFn),
        save: Some(|_t: usize| Message::monster(crate::editors::monster::MonsterEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::monster(crate::editors::monster::MonsterEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::monster(crate::editors::monster::MonsterEditorMessage::Spreadsheet(sm))),
    },
    /// Monster visuals (Monster.ini).
    MonsterIniEditor {
        undo: true, history: false,
        view: Some(crate::editors::monster_ini::view as ViewFn),
        save: Some(|_t: usize| Message::monster_ini(crate::editors::monster_ini::MonsterIniEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::monster_ini(crate::editors::monster_ini::MonsterIniEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::monster_ini(crate::editors::monster_ini::MonsterIniEditorMessage::Spreadsheet(sm))),
    },
    /// Healing items (HealItem.db).
    HealItemEditor {
        undo: true, history: true,
        view: Some(crate::editors::heal_item::view as ViewFn),
        save: Some(|_t: usize| Message::heal_item(crate::editors::heal_item::HealItemEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::heal_item(crate::editors::heal_item::HealItemEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::heal_item(crate::editors::heal_item::HealItemEditorMessage::Spreadsheet(sm))),
    },
    /// Miscellaneous items (MiscItem.db).
    MiscItemEditor {
        undo: true, history: true,
        view: Some(crate::editors::misc_item::view as ViewFn),
        save: Some(|_t: usize| Message::misc_item(crate::editors::misc_item::MiscItemEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::misc_item(crate::editors::misc_item::MiscItemEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::misc_item(crate::editors::misc_item::MiscItemEditorMessage::Spreadsheet(sm))),
    },
    /// Edit items (EditItem.db).
    EditItemEditor {
        undo: true, history: true,
        view: Some(crate::editors::edit_item::view as ViewFn),
        save: Some(|_t: usize| Message::edit_item(crate::editors::edit_item::EditItemEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::edit_item(crate::editors::edit_item::EditItemEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::edit_item(crate::editors::edit_item::EditItemEditorMessage::Spreadsheet(sm))),
    },
    /// Event items (EventItem.db).
    EventItemEditor {
        undo: true, history: true,
        view: Some(crate::editors::event_item::view as ViewFn),
        save: Some(|_t: usize| Message::event_item(crate::editors::event_item::EventItemEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::event_item(crate::editors::event_item::EventItemEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::event_item(crate::editors::event_item::EventItemEditorMessage::Spreadsheet(sm))),
    },
    /// Magic spells (Magic.db).
    MagicEditor {
        undo: true, history: true,
        view: Some(crate::editors::magic::view as ViewFn),
        save: Some(|_t: usize| Message::magic(crate::editors::magic::MagicEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::magic(crate::editors::magic::MagicEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::magic(crate::editors::magic::MagicEditorMessage::Spreadsheet(sm))),
    },
    /// Stores (Store.db).
    StoreEditor {
        undo: true, history: true,
        view: Some(crate::editors::store::view as ViewFn),
        save: Some(|_t: usize| Message::store(crate::editors::store::StoreEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::store(crate::editors::store::StoreEditorMessage::LoadCatalog))),
        sheet: None, // custom layout, no generic spreadsheet
    },
    /// Character data (ChData.db).
    ChDataEditor {
        undo: true, history: true,
        view: Some(crate::editors::chdata::view as ViewFn),
        save: Some(|_t: usize| Message::chdata(crate::editors::chdata::ChDataEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::chdata(crate::editors::chdata::ChDataEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::chdata(crate::editors::chdata::ChDataEditorMessage::Spreadsheet(sm))),
    },
    /// Party level database (prtlevel.db).
    PartyLevelDbEditor {
        undo: true, history: true,
        view: Some(crate::editors::party_level_db::view as ViewFn),
        save: Some(|_t: usize| Message::party_level_db(crate::editors::party_level_db::PartyLevelDbEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::party_level_db(crate::editors::party_level_db::PartyLevelDbEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::party_level_db(crate::editors::party_level_db::PartyLevelDbEditorMessage::Spreadsheet(sm))),
    },
    /// Dialogue script (.dlg), one file per tab.
    DialogueScriptEditor {
        undo: true, history: true,
        view: Some(crate::editors::dialogue_script::view as ViewFn),
        save: Some(|_t: usize| Message::dialogue_script(crate::editors::dialogue_script::DialogueScriptEditorMessage::Save)),
        load: None, // opened by explicit file path
        sheet: Some(|sm| Message::dialogue_script(crate::editors::dialogue_script::DialogueScriptEditorMessage::Spreadsheet(sm))),
    },
    /// Dialogue text (.pgp), one file per tab.
    DialogueTextEditor {
        undo: true, history: true,
        view: Some(crate::editors::dialogue_paragraph::view as ViewFn),
        save: Some(|_t: usize| Message::dialogue_paragraph(crate::editors::dialogue_paragraph::DialogueParagraphEditorMessage::Save)),
        load: None, // opened by explicit file path
        sheet: Some(|sm| Message::dialogue_paragraph(crate::editors::dialogue_paragraph::DialogueParagraphEditorMessage::Spreadsheet(sm))),
    },
    /// Draw items (DrawItem.ref).
    DrawItemEditor {
        undo: true, history: true,
        view: Some(crate::editors::draw_item::view as ViewFn),
        save: Some(|_t: usize| Message::draw_item(crate::editors::draw_item::DrawItemEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::draw_item(crate::editors::draw_item::DrawItemEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::draw_item(crate::editors::draw_item::DrawItemEditorMessage::Spreadsheet(sm))),
    },
    /// Events (Event.ini).
    EventIniEditor {
        undo: true, history: true,
        view: Some(crate::editors::event_ini::view as ViewFn),
        save: Some(|_t: usize| Message::event_ini(crate::editors::event_ini::EventIniEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::event_ini(crate::editors::event_ini::EventIniEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::event_ini(crate::editors::event_ini::EventIniEditorMessage::Spreadsheet(sm))),
    },
    /// Event NPCs (eventnpc.ref).
    EventNpcRefEditor {
        undo: true, history: true,
        view: Some(crate::editors::event_npc_ref::view as ViewFn),
        save: Some(|_t: usize| Message::event_npc_ref(crate::editors::event_npc_ref::EventNpcRefEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::event_npc_ref(crate::editors::event_npc_ref::EventNpcRefEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::event_npc_ref(crate::editors::event_npc_ref::EventNpcRefEditorMessage::Spreadsheet(sm))),
    },
    /// Extra objects (Extra.ini).
    ExtraIniEditor {
        undo: true, history: true,
        view: Some(crate::editors::extra_ini::view as ViewFn),
        save: Some(|_t: usize| Message::extra_ini(crate::editors::extra_ini::ExtraIniEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::extra_ini(crate::editors::extra_ini::ExtraIniEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::extra_ini(crate::editors::extra_ini::ExtraIniEditorMessage::Spreadsheet(sm))),
    },
    /// Extra object placements (Ext*.ref), one file per tab.
    ExtraRefEditor {
        undo: true, history: true,
        view: Some(crate::editors::extra_ref::view as ViewFn),
        save: Some(|_t: usize| Message::extra_ref(crate::editors::extra_ref::ExtraRefEditorMessage::Save)),
        load: None, // opened by explicit file path
        sheet: Some(|sm| Message::extra_ref(crate::editors::extra_ref::ExtraRefEditorMessage::Spreadsheet(sm))),
    },
    /// Per-map configuration (Map.ini).
    MapIniEditor {
        undo: true, history: true,
        view: Some(crate::editors::map_ini::view as ViewFn),
        save: Some(|_t: usize| Message::map_ini(crate::editors::map_ini::MapIniEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::map_ini(crate::editors::map_ini::MapIniEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::map_ini(crate::editors::map_ini::MapIniEditorMessage::Spreadsheet(sm))),
    },
    /// Text messages (message.scr).
    MessageScrEditor {
        undo: true, history: true,
        view: Some(crate::editors::message_scr::view as ViewFn),
        save: Some(|_t: usize| Message::message_scr(crate::editors::message_scr::MessageScrEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::message_scr(crate::editors::message_scr::MessageScrEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::message_scr(crate::editors::message_scr::MessageScrEditorMessage::Spreadsheet(sm))),
    },
    /// Monster placements (Mondun*.ref), one file per tab.
    MonsterRefEditor {
        undo: true, history: true,
        view: Some(crate::editors::monster_ref::view as ViewFn),
        save: Some(|_t: usize| Message::monster_ref(crate::editors::monster_ref::MonsterRefEditorMessage::Save)),
        load: None, // opened by explicit file path
        sheet: Some(|sm| Message::monster_ref(crate::editors::monster_ref::MonsterRefEditorMessage::Spreadsheet(sm))),
    },
    /// NPC visuals (Npc.ini).
    NpcIniEditor {
        undo: true, history: false,
        view: Some(crate::editors::npc_ini::view as ViewFn),
        save: Some(|_t: usize| Message::npc_ini(crate::editors::npc_ini::NpcIniEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::npc_ini(crate::editors::npc_ini::NpcIniEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::npc_ini(crate::editors::npc_ini::NpcIniEditorMessage::Spreadsheet(sm))),
    },
    /// NPC placements (Npc*.ref), one file per tab.
    NpcRefEditor {
        undo: true, history: true,
        view: Some(crate::editors::npc_ref::view as ViewFn),
        save: Some(|_t: usize| Message::npc_ref(crate::editors::npc_ref::NpcRefEditorMessage::Save)),
        load: None, // opened by explicit file path
        sheet: Some(|sm| Message::npc_ref(crate::editors::npc_ref::NpcRefEditorMessage::Spreadsheet(sm))),
    },
    /// Party references (partyref.ref).
    PartyRefEditor {
        undo: true, history: true,
        view: Some(crate::editors::party_ref::view as ViewFn),
        save: Some(|_t: usize| Message::party_ref(crate::editors::party_ref::PartyRefEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::party_ref(crate::editors::party_ref::PartyRefEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::party_ref(crate::editors::party_ref::PartyRefEditorMessage::Spreadsheet(sm))),
    },
    /// Party NPCs (prtini.db).
    PartyIniEditor {
        undo: true, history: true,
        view: Some(crate::editors::party_ini::view as ViewFn),
        save: Some(|_t: usize| Message::party_ini(crate::editors::party_ini::PartyIniEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::party_ini(crate::editors::party_ini::PartyIniEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::party_ini(crate::editors::party_ini::PartyIniEditorMessage::Spreadsheet(sm))),
    },
    /// Quests (quest.scr).
    QuestScrEditor {
        undo: true, history: true,
        view: Some(crate::editors::quest_scr::view as ViewFn),
        save: Some(|_t: usize| Message::quest_scr(crate::editors::quest_scr::QuestScrEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::quest_scr(crate::editors::quest_scr::QuestScrEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::quest_scr(crate::editors::quest_scr::QuestScrEditorMessage::Spreadsheet(sm))),
    },
    /// Event scripts (event*.scr). View is wrapped by the caller.
    EventScrEditor {
        undo: false, history: false,
        view: None, // rendered by view/mod.rs with EditorMessage::EventScr wrapping
        save: Some(|_t: usize| Message::event_scr(crate::editors::event_scr::EventScrEditorMessage::SaveScript)),
        load: None, // opened by explicit file path
        sheet: None,
    },
    /// Wave configuration (wave.ini).
    WaveIniEditor {
        undo: true, history: true,
        view: Some(crate::editors::wave_ini::view as ViewFn),
        save: Some(|_t: usize| Message::wave_ini(crate::editors::wave_ini::WaveIniEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::wave_ini(crate::editors::wave_ini::WaveIniEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::wave_ini(crate::editors::wave_ini::WaveIniEditorMessage::Spreadsheet(sm))),
    },
    /// Global map list (AllMap.ini).
    AllMapIniEditor {
        undo: true, history: true,
        view: Some(crate::editors::all_map_ini::view as ViewFn),
        save: Some(|_t: usize| Message::all_map_ini(crate::editors::all_map_ini::AllMapIniEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::all_map_ini(crate::editors::all_map_ini::AllMapIniEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::all_map_ini(crate::editors::all_map_ini::AllMapIniEditorMessage::Spreadsheet(sm))),
    },
    /// Sprite viewer (.spr), one per tab; snapshot-based undo.
    SpriteViewer {
        undo: true, history: false,
        view: Some(crate::editors::sprite_editor::view as ViewFn),
        save: Some(|_t: usize| Message::sprite_viewer(crate::editors::sprite_editor::SpriteViewerMessage::Save)),
        load: None, // opened by explicit file path
        sheet: None,
    },
    /// Sound effects (.snf), read-only playback, one per tab.
    SnfEditor {
        undo: false, history: false,
        view: Some(crate::editors::snf_editor::view as ViewFn),
        save: None,
        load: None, // opened by explicit file path
        sheet: None,
    },
    /// SQLite database viewer (tool tab, no EditorMessage variant).
    DbViewer {
        undo: false, history: false,
        view: None, // rendered by App::view_db_viewer()
        save: None,
        load: None,
        sheet: None,
    },
    /// Tileset browser (.btl/.gtl), one per tab.
    TilesetEditor {
        undo: false, history: false,
        view: Some(crate::editors::tileset::view as ViewFn),
        save: None,
        load: None, // opened by explicit file path
        sheet: None,
    },
    /// Visual map editor (.map), one per tab; own undo stack.
    MapEditor {
        undo: false, history: false,
        view: Some(crate::editors::map_editor::view as ViewFn),
        save: Some(|t: usize| Message::map_editor(crate::editors::map_editor::MapEditorMessage::SaveEntities(t))),
        load: None, // opened by explicit file path
        sheet: None,
    },
    /// Mod packaging tool tab.
    ModPackager {
        undo: false, history: false,
        view: Some(crate::editors::mod_packager::view as ViewFn),
        save: None,
        load: None,
        sheet: None,
    },
    /// Localization manager tool tab.
    LocalizationManager {
        undo: false, history: false,
        view: Some(crate::editors::localization_manager::view as ViewFn),
        save: None,
        load: None,
        sheet: None,
    },
    /// Read-only save-file viewer (.sav), one per tab.
    SaveFileViewer {
        undo: false, history: false,
        view: Some(crate::editors::save_file_viewer::view as ViewFn),
        save: None,
        load: None, // opened by explicit file path (custom async parse in app.rs)
        sheet: None,
    },
    /// Save-slot manager (Save.ifo).
    SaveIfoEditor {
        undo: true, history: true,
        view: Some(crate::editors::save_ifo::view as ViewFn),
        save: Some(|_t: usize| Message::save_ifo(crate::editors::save_ifo::SaveIfoEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::save_ifo(crate::editors::save_ifo::SaveIfoEditorMessage::LoadCatalog))),
        sheet: None, // custom layout, no generic spreadsheet
    },
    /// Map-lighting fade tables (fogdata.dat), one per tab.
    FogDataEditor {
        undo: true, history: false,
        view: Some(crate::editors::fog_data::view as ViewFn),
        save: Some(|t: usize| Message::fog_data(crate::editors::fog_data::FogDataMessage::Save(t))),
        load: None, // opened by explicit file path
        sheet: None,
    },
    /// Multiplayer chest spawns (.cst).
    ChestSpawnEditor {
        undo: true, history: true,
        view: Some(crate::editors::chest_spawn::view as ViewFn),
        save: Some(|_t: usize| Message::chest_spawn(crate::editors::chest_spawn::ChestSpawnEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::chest_spawn(crate::editors::chest_spawn::ChestSpawnEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::chest_spawn(crate::editors::chest_spawn::ChestSpawnEditorMessage::Spreadsheet(sm))),
    },
    /// Multiplayer monster spawns (.mon).
    MonsterSpawnEditor {
        undo: true, history: true,
        view: Some(crate::editors::monster_spawn::view as ViewFn),
        save: Some(|_t: usize| Message::monster_spawn(crate::editors::monster_spawn::MonsterSpawnEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::monster_spawn(crate::editors::monster_spawn::MonsterSpawnEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::monster_spawn(crate::editors::monster_spawn::MonsterSpawnEditorMessage::Spreadsheet(sm))),
    },
    /// Multiplayer map metadata (.ifm).
    MapInfoMultiEditor {
        undo: true, history: true,
        view: Some(crate::editors::map_info_multi::view as ViewFn),
        save: Some(|_t: usize| Message::map_info_multi(crate::editors::map_info_multi::MapInfoMultiEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::map_info_multi(crate::editors::map_info_multi::MapInfoMultiEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::map_info_multi(crate::editors::map_info_multi::MapInfoMultiEditorMessage::Spreadsheet(sm))),
    },
    /// Multiplayer tile grid (.dat), canvas-based view.
    TileGridEditor {
        undo: true, history: true,
        view: Some(crate::editors::tile_grid_canvas::view as ViewFn),
        save: Some(|_t: usize| Message::tile_grid(crate::editors::tile_grid::TileGridEditorMessage::Save)),
        load: Some(|| iced::Task::done(Message::tile_grid(crate::editors::tile_grid::TileGridEditorMessage::LoadCatalog))),
        sheet: Some(|sm| Message::tile_grid(crate::editors::tile_grid::TileGridEditorMessage::Spreadsheet(sm))),
    },
    /// Universal fallback binary editor.
    HexEditor {
        undo: false, history: false,
        view: Some(crate::editors::hex_wrapper::view as ViewFn),
        save: None,
        load: None, // opened by explicit file path
        sheet: None,
    },
    /// Deserialized fallback for unknown editor types (serde `other`).
    #[serde(other)]
    Unknown {
        undo: false, history: false,
        view: None,
        save: None,
        load: None,
        sheet: None,
    },
}

/// Look up the descriptor for an editor kind.
pub fn descriptor(et: EditorType) -> &'static EditorDescriptor {
    et.descriptor()
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
    descriptor(editor_type).load.map(|build| build())
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
