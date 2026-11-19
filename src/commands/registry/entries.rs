//! Declarative catalog entries.
//!
//! Every entry is built through [`Builder`], which sources fields and record
//! metadata from the struct itself. Adding a file type here automatically
//! feeds CLI detection, `schema`, `template`, and the GUI editor mapping.

use serde::{Deserialize, Serialize};

use super::types::{
    DetectKind, ExtractFn, FieldSpec, FileType, PatchFn, ValidateFn, explicit_fields, extract_as,
    extract_map_file, extract_sprite_info, extract_tileset, fields_from_default,
    fields_from_layout, patch_as, patch_not_supported, record_name, validate_as,
};
use crate::references::extractor::Extractor;
use crate::references::layout::RecordLayout;

fn b(
    key: &'static str,
    name: &'static str,
    description: &'static str,
    extensions: &'static [&'static str],
    detect_kind: DetectKind,
    editor: &'static str,
) -> Builder {
    Builder {
        key,
        name,
        description,
        extensions,
        detect_kind,
        editor,
    }
}

struct Builder {
    key: &'static str,
    name: &'static str,
    description: &'static str,
    extensions: &'static [&'static str],
    detect_kind: DetectKind,
    editor: &'static str,
}

impl Builder {
    /// Serde record with validation enabled.
    fn record<T>(self) -> FileType
    where
        T: Default + Serialize + for<'de> Deserialize<'de> + Extractor,
    {
        self.finish(
            fields_from_default::<T>(),
            record_name::<T>(),
            extract_as::<T>,
            patch_as::<T>,
            Some(validate_as::<T>),
        )
    }

    /// Serde record without a validator (complex custom layouts).
    fn record_unchecked<T>(self) -> FileType
    where
        T: Default + Serialize + for<'de> Deserialize<'de> + Extractor,
    {
        self.finish(
            fields_from_default::<T>(),
            record_name::<T>(),
            extract_as::<T>,
            patch_as::<T>,
            None,
        )
    }

    /// Fixed-size binary record: field types come from `RecordLayout`.
    fn layout_record<T>(self) -> FileType
    where
        T: Default + Serialize + for<'de> Deserialize<'de> + Extractor + RecordLayout,
    {
        self.finish(
            fields_from_layout::<T>(),
            T::LAYOUT.type_name,
            extract_as::<T>,
            patch_as::<T>,
            Some(validate_as::<T>),
        )
    }

    fn layout_record_unchecked<T>(self) -> FileType
    where
        T: Default + Serialize + for<'de> Deserialize<'de> + Extractor + RecordLayout,
    {
        self.finish(
            fields_from_layout::<T>(),
            T::LAYOUT.type_name,
            extract_as::<T>,
            patch_as::<T>,
            None,
        )
    }

    /// Raw/non-`Default` format with an explicitly curated field list.
    fn raw(
        self,
        fields: &'static [&'static str],
        record_type: &'static str,
        extract_fn: ExtractFn,
        patch_fn: PatchFn,
        validate_fn: Option<ValidateFn>,
    ) -> FileType {
        self.finish(
            explicit_fields(fields),
            record_type,
            extract_fn,
            patch_fn,
            validate_fn,
        )
    }

    fn finish(
        self,
        fields: Vec<FieldSpec>,
        record_type: &'static str,
        extract_fn: ExtractFn,
        patch_fn: PatchFn,
        validate_fn: Option<ValidateFn>,
    ) -> FileType {
        FileType {
            key: self.key,
            name: self.name,
            description: self.description,
            extensions: self.extensions,
            detect_kind: self.detect_kind,
            extract_fn,
            patch_fn,
            validate_fn,
            fields,
            record_type,
            editor: self.editor,
        }
    }
}

use crate::references::{
    all_map_ini::Map, chdata_db::ChData, chest_spawn::ChestSpawn,
    dialogue_paragraph::DialogueParagraph, dialogue_script::DialogueScript, draw_item::DrawItem,
    edit_item_db::EditItem, event_ini::Event, event_item_db::EventItem, event_npc_ref::EventNpcRef,
    event_scr::EventScript, extra_ini::Extra, extra_ref::ExtraRef, heal_item_db::HealItem,
    magic_db::MagicSpell, map_info_multi::MapInfoMulti, map_ini::MapIni, message_scr::Message,
    misc_item_db::MiscItem, monster_db::Monster, monster_ini::MonsterIni, monster_ref::MonsterRef,
    monster_spawn::MonsterSpawn, npc_ini::NpcIni, npc_ref::NPC, party_ini_db::PartyIniNpc,
    party_level_db::PartyLevelNpc, party_ref::PartyRef, quest_scr::Quest, save_file::SaveFile,
    save_ifo::SaveIfo, store_db::Store, tile_grid::TileGrid, wave_ini::WaveIni,
    weapons_db::WeaponItem,
};

// `MagicSpell`'s patcher owns the canonical `Magic.db` / `MulMagic.db`
// filename list — the same list the mod PatcherRegistry uses.
use crate::references::magic_db::MagicSpellPatcher;

// ── INI types ────────────────────────────────────────────────────────────────

pub(crate) fn make_all_map_ini() -> FileType {
    b(
        "all_maps",
        "AllMap.ini",
        "Master map list",
        &[".ini"],
        DetectKind::Ini("AllMap.ini"),
        "AllMapIniEditor",
    )
    .record::<Map>()
}

pub(crate) fn make_map_ini() -> FileType {
    b(
        "map_ini",
        "Map.ini",
        "Map properties",
        &[".ini"],
        DetectKind::Ini("Map.ini"),
        "MapIniEditor",
    )
    .record::<MapIni>()
}

pub(crate) fn make_extra_ini() -> FileType {
    b(
        "extra_ini",
        "Extra.ini",
        "Interactive object types",
        &[".ini"],
        DetectKind::Ini("Extra.ini"),
        "ExtraIniEditor",
    )
    .record::<Extra>()
}

pub(crate) fn make_event_ini() -> FileType {
    b(
        "event_ini",
        "Event.ini",
        "Script/event mappings",
        &[".ini"],
        DetectKind::Ini("Event.ini"),
        "EventIniEditor",
    )
    .record::<Event>()
}

pub(crate) fn make_monster_ini() -> FileType {
    b(
        "monster_ini",
        "Monster.ini",
        "Monster visual refs",
        &[".ini"],
        DetectKind::Ini("Monster.ini"),
        "MonsterIniEditor",
    )
    .record::<MonsterIni>()
}

pub(crate) fn make_npc_ini() -> FileType {
    b(
        "npc_ini",
        "Npc.ini",
        "NPC visual refs",
        &[".ini"],
        DetectKind::Ini("Npc.ini"),
        "NpcIniEditor",
    )
    .record::<NpcIni>()
}

pub(crate) fn make_wave_ini() -> FileType {
    b(
        "wave_ini",
        "Wave.ini",
        "Audio/SNF references",
        &[".ini"],
        DetectKind::Ini("Wave.ini"),
        "WaveIniEditor",
    )
    .record::<WaveIni>()
}

// ── DB types ─────────────────────────────────────────────────────────────────

pub(crate) fn make_weapons() -> FileType {
    b(
        "weapons",
        "WeaponItem.db",
        "Weapons & armor database",
        &[".db"],
        DetectKind::Db(&["WeaponItem.db"]),
        "WeaponEditor",
    )
    .layout_record::<WeaponItem>()
}

pub(crate) fn make_monsters() -> FileType {
    b(
        "monsters",
        "Monster.db",
        "Monster attributes",
        &[".db"],
        DetectKind::Db(&["Monster.db"]),
        "MonsterEditor",
    )
    .layout_record::<Monster>()
}

pub(crate) fn make_magic() -> FileType {
    b(
        "magic",
        "Magic.db",
        "Magic spell records",
        &[".db"],
        DetectKind::Db(MagicSpellPatcher::FILENAMES),
        "MagicEditor",
    )
    .layout_record_unchecked::<MagicSpell>()
}

pub(crate) fn make_store() -> FileType {
    b(
        "store",
        "Store.db",
        "Shop inventory records",
        &[".db"],
        DetectKind::Db(&["Store.db"]),
        "StoreEditor",
    )
    .record_unchecked::<Store>()
}

pub(crate) fn make_misc_item() -> FileType {
    b(
        "misc_item",
        "MiscItem.db",
        "Generic item records",
        &[".db"],
        DetectKind::Db(&["MiscItem.db"]),
        "MiscItemEditor",
    )
    .layout_record::<MiscItem>()
}

pub(crate) fn make_heal_item() -> FileType {
    b(
        "heal_item",
        "HealItem.db",
        "Consumable records",
        &[".db"],
        DetectKind::Db(&["HealItem.db"]),
        "HealItemEditor",
    )
    .layout_record::<HealItem>()
}

pub(crate) fn make_event_item() -> FileType {
    b(
        "event_item",
        "EventItem.db",
        "Quest item records",
        &[".db"],
        DetectKind::Db(&["EventItem.db"]),
        "EventItemEditor",
    )
    .layout_record::<EventItem>()
}

pub(crate) fn make_edit_item() -> FileType {
    b(
        "edit_item",
        "EditItem.db",
        "Modifiable item records",
        &[".db"],
        DetectKind::Db(&["EditItem.db"]),
        "EditItemEditor",
    )
    .layout_record::<EditItem>()
}

pub(crate) fn make_party_level() -> FileType {
    b(
        "party_level",
        "PrtLevel.db",
        "EXP table records",
        &[".db"],
        DetectKind::Db(&["PrtLevel.db"]),
        "PartyLevelDbEditor",
    )
    .record_unchecked::<PartyLevelNpc>()
}

pub(crate) fn make_party_ini() -> FileType {
    b(
        "party_ini",
        "PrtIni.db",
        "Party NPC metadata",
        &[".db"],
        DetectKind::Db(&["PrtIni.db"]),
        "PartyIniEditor",
    )
    .layout_record_unchecked::<PartyIniNpc>()
}

pub(crate) fn make_chdata() -> FileType {
    b(
        "chdata",
        "ChData.db",
        "Character data records",
        &[".db"],
        DetectKind::Db(&["ChData.db"]),
        "ChDataEditor",
    )
    .layout_record_unchecked::<ChData>()
}

// ── REF types ────────────────────────────────────────────────────────────────

pub(crate) fn make_party_ref() -> FileType {
    b(
        "party_ref",
        "PartyRef.ref",
        "Character definitions",
        &[".ref"],
        DetectKind::RefPrefix("PartyRef"),
        "PartyRefEditor",
    )
    .record::<PartyRef>()
}

pub(crate) fn make_draw_item() -> FileType {
    b(
        "draw_item",
        "DRAWITEM.ref",
        "Map placement records",
        &[".ref"],
        DetectKind::RefPrefix("DRAWITEM"),
        "DrawItemEditor",
    )
    .record::<DrawItem>()
}

pub(crate) fn make_npc_ref() -> FileType {
    b(
        "npc_ref",
        "Npc*.ref",
        "NPC placement records",
        &[".ref"],
        DetectKind::RefPrefix("Npc"),
        "NpcRefEditor",
    )
    .layout_record_unchecked::<NPC>()
}

pub(crate) fn make_monster_ref() -> FileType {
    b(
        "monster_ref",
        "Mon*.ref",
        "Monster placement records",
        &[".ref"],
        DetectKind::RefPrefix("Mon"),
        "MonsterRefEditor",
    )
    .layout_record_unchecked::<MonsterRef>()
}

pub(crate) fn make_extra_ref() -> FileType {
    b(
        "extra_ref",
        "Ext*.ref",
        "Special object placements",
        &[".ref"],
        DetectKind::RefPrefix("Ext"),
        "ExtraRefEditor",
    )
    .layout_record::<ExtraRef>()
}

pub(crate) fn make_event_npc_ref() -> FileType {
    b(
        "event_npc_ref",
        "Eventnpc.ref",
        "Event NPC placements",
        &[".ref"],
        DetectKind::RefPrefix("Eventnpc"),
        "EventNpcRefEditor",
    )
    .record_unchecked::<EventNpcRef>()
}

// ── DLG / PGP types ──────────────────────────────────────────────────────────

pub(crate) fn make_dialog() -> FileType {
    b(
        "dialog",
        "*.dlg",
        "Dialogue script CSV",
        &[".dlg"],
        DetectKind::DlgPrefix("Dlg"),
        "DialogueScriptEditor",
    )
    .record::<DialogueScript>()
}

pub(crate) fn make_dialog_text() -> FileType {
    b(
        "dialog_text",
        "*.pgp",
        "Dialogue text package",
        &[".pgp"],
        DetectKind::PgpPrefix("Pgp"),
        "DialogueTextEditor",
    )
    .record::<DialogueParagraph>()
}

// ── SCR types ────────────────────────────────────────────────────────────────

pub(crate) fn make_quest() -> FileType {
    b(
        "quest",
        "Quest.scr",
        "Quest definitions",
        &[".scr"],
        DetectKind::Scr("Quest.scr"),
        "QuestScrEditor",
    )
    .record_unchecked::<Quest>()
}

pub(crate) fn make_message() -> FileType {
    b(
        "message",
        "Message.scr",
        "Diary game messages",
        &[".scr"],
        DetectKind::Scr("Message.scr"),
        "MessageScrEditor",
    )
    .record_unchecked::<Message>()
}

pub(crate) fn make_event_scr() -> FileType {
    b(
        "event_scr",
        "Event*.scr",
        "Event script files",
        &[".scr"],
        DetectKind::ScrPrefix("Event"),
        "EventScrEditor",
    )
    .raw(
        &[
            "id",
            "header_comments",
            "trailing_newline",
            "variables",
            "map_content",
            "chr_content",
            "npc_content",
            "spr_content",
            "wav_content",
            "actions",
        ],
        "EventScript",
        extract_as::<EventScript>,
        patch_as::<EventScript>,
        None,
    )
}

// ── Raw / extract-only formats ───────────────────────────────────────────────

pub(crate) fn make_map_file() -> FileType {
    b(
        "map_file",
        "*.map",
        "Map geometry, sprites, events, tiles (extract only)",
        &[".map"],
        DetectKind::Extension,
        "MapEditor",
    )
    .raw(&[], "n/a", extract_map_file, patch_not_supported, None)
}

pub(crate) fn make_gtl() -> FileType {
    b(
        "gtl",
        "*.gtl",
        "Ground tile layer (extract only)",
        &[".gtl"],
        DetectKind::Extension,
        "TilesetEditor",
    )
    .raw(&[], "n/a", extract_tileset, patch_not_supported, None)
}

pub(crate) fn make_btl() -> FileType {
    b(
        "btl",
        "*.btl",
        "Building tile layer (extract only)",
        &[".btl"],
        DetectKind::Extension,
        "TilesetEditor",
    )
    .raw(&[], "n/a", extract_tileset, patch_not_supported, None)
}

pub(crate) fn make_sprite() -> FileType {
    b(
        "sprite",
        "*.spr",
        "Sprite/animation file (extract only)",
        &[".spr"],
        DetectKind::Extension,
        "SpriteViewer",
    )
    .raw(&[], "n/a", extract_sprite_info, patch_not_supported, None)
}

pub(crate) fn make_save_ifo() -> FileType {
    b(
        "save_ifo",
        "Save.ifo",
        "Save-slot metadata index (timestamps and occupancy for 0.sav..5.sav)",
        &[".ifo"],
        DetectKind::Db(&["Save.ifo"]),
        "SaveIfoEditor",
    )
    .record_unchecked::<SaveIfo>()
}

// ── Multiplayer file types ───────────────────────────────────────────────────

pub(crate) fn make_chest_spawn() -> FileType {
    b(
        "chest_spawn",
        "*.cst",
        "Multiplayer chest/placeable spawn list (CSV, EUC-KR)",
        &[".cst"],
        DetectKind::Extension,
        "ChestSpawnEditor",
    )
    .record::<ChestSpawn>()
}

pub(crate) fn make_monster_spawn() -> FileType {
    b(
        "monster_spawn",
        "*.mon",
        "Multiplayer monster spawn list (CSV, EUC-KR)",
        &[".mon"],
        DetectKind::Extension,
        "MonsterSpawnEditor",
    )
    .record::<MonsterSpawn>()
}

pub(crate) fn make_map_info_multi() -> FileType {
    b(
        "map_info_multi",
        "*.ifm",
        "Multiplayer map info/linkage struct (binary, 100 B)",
        &[".ifm"],
        DetectKind::Extension,
        "MapInfoMultiEditor",
    )
    .record_unchecked::<MapInfoMulti>()
}

pub(crate) fn make_tile_grid() -> FileType {
    b(
        "tile_grid",
        "*.dat",
        "Multiplayer per-tile u32 grid (binary, 249×249)",
        &[".dat"],
        DetectKind::Extension,
        "TileGridEditor",
    )
    .record_unchecked::<TileGrid>()
}

pub(crate) fn make_save_file() -> FileType {
    b(
        "save_file",
        "*.sav",
        "Game save file with player state, inventory, and world data",
        &[".sav"],
        DetectKind::Db(&["0.sav", "1.sav", "2.sav", "3.sav", "4.sav", "5.sav"]),
        "SaveFileViewer",
    )
    .record_unchecked::<SaveFile>()
}
