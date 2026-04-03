use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::LazyLock;

use crate::references::extractor::Extractor;

/// Detection strategy for a file type.
pub(crate) enum DetectKind {
    /// Match by known INI filename (case-insensitive).
    IniFilename(&'static str),
    /// Match by known DB filename (case-insensitive).
    DbFilename(&'static [&'static str]),
    /// Match by known REF filename prefix (case-insensitive).
    RefFilename(&'static str),
    /// Match by known SCR filename (case-insensitive).
    ScrFilename(&'static str),
    /// Match by known DLG filename prefix (case-insensitive).
    DlgFilename(&'static str),
    /// Match by known PGP filename prefix (case-insensitive).
    PgpFilename(&'static str),
}

/// A registered file type with detection, extraction, and patching capabilities.
pub struct FileType {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub extensions: &'static [&'static str],
    detect_kind: DetectKind,
    pub extract_fn: fn(&Path) -> Result<serde_json::Value, Box<dyn std::error::Error>>,
    pub patch_fn: fn(&serde_json::Value, &Path) -> Result<(), Box<dyn std::error::Error>>,
    pub validate_fn: Option<fn(&serde_json::Value) -> Result<(), Vec<String>>>,
}

impl FileType {
    pub fn matches(&self, path: &Path) -> bool {
        match self.detect_kind {
            DetectKind::IniFilename(name) => detect_filename(path, name),
            DetectKind::DbFilename(names) => detect_db_filename(path, names),
            DetectKind::RefFilename(prefix) => detect_filename_prefix(path, prefix),
            DetectKind::ScrFilename(name) => detect_filename(path, name),
            DetectKind::DlgFilename(prefix) => detect_filename_prefix(path, prefix),
            DetectKind::PgpFilename(prefix) => detect_filename_prefix(path, prefix),
        }
    }
}

/// Result of file type detection.
pub enum DetectResult {
    /// Exactly one type matched.
    Single(&'static FileType),
    /// No type matched.
    None,
}

// impl DetectResult {
//     pub fn into_single(self) -> Option<&'static FileType> {
//         match self {
//             DetectResult::Single(ft) => Some(ft),
//             DetectResult::None => None,
//         }
//     }
// }

/// The static registry of all supported file types.
pub static FILE_TYPES: LazyLock<Vec<FileType>> = LazyLock::new(|| {
    vec![
        // INI types
        make_all_map_ini(),
        make_map_ini(),
        make_extra_ini(),
        make_event_ini(),
        make_monster_ini(),
        make_npc_ini(),
        make_wave_ini(),
        // DB types
        make_weapons(),
        make_monsters(),
        make_magic(),
        make_store(),
        make_misc_item(),
        make_heal_item(),
        make_event_item(),
        make_edit_item(),
        make_party_level(),
        make_party_ini(),
        make_chdata(),
        // REF types
        make_party_ref(),
        make_draw_item(),
        make_npc_ref(),
        make_monster_ref(),
        make_extra_ref(),
        make_event_npc_ref(),
        // DLG/PGP types
        make_dialog(),
        make_dialog_text(),
        // SCR types
        make_quest(),
        make_message(),
        // Map types (extract only, no patch)
        make_map_file(),
        make_gtl(),
        make_btl(),
        // Sprite files (extract only, no patch)
        make_sprite(),
    ]
});

/// Look up a file type by its key.
pub fn get_by_key(key: &str) -> Option<&'static FileType> {
    FILE_TYPES.iter().find(|ft| ft.key == key)
}

/// Detect the file type for a given path.
pub fn detect(path: &Path) -> DetectResult {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    let ext = match ext {
        Some(e) => e,
        None => return DetectResult::None,
    };

    let candidates: Vec<&FileType> = FILE_TYPES
        .iter()
        .filter(|ft| {
            ft.extensions.iter().any(|&ext_match| {
                let normalized = ext_match.trim_start_matches('.');
                ext == normalized
            })
        })
        .collect();

    if candidates.is_empty() {
        return DetectResult::None;
    }

    if candidates.len() == 1 {
        return DetectResult::Single(candidates[0]);
    }

    // Try each candidate's detection function; first match wins
    for ft in &candidates {
        if ft.matches(path) {
            return DetectResult::Single(ft);
        }
    }

    // Fall back to the first candidate in registry order
    DetectResult::Single(candidates[0])
}

/// Get a human-friendly list of all file types for error messages.
pub fn format_type_list() -> String {
    FILE_TYPES
        .iter()
        .map(|ft| {
            format!(
                "  {} ({}) — {}",
                ft.key,
                ft.extensions.join(", "),
                ft.description
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ===========================================================================
// Registry entry builders
// ===========================================================================

fn make_all_map_ini() -> FileType {
    FileType {
        key: "all_maps",
        name: "AllMap.ini",
        description: "Master map list",
        extensions: &[".ini"],
        detect_kind: DetectKind::IniFilename("AllMap.ini"),
        extract_fn: extract_as::<crate::references::all_map_ini::Map>,
        patch_fn: patch_as::<crate::references::all_map_ini::Map>,
        validate_fn: Some(validate_as::<crate::references::all_map_ini::Map>),
    }
}

fn make_map_ini() -> FileType {
    FileType {
        key: "map_ini",
        name: "Map.ini",
        description: "Map properties",
        extensions: &[".ini"],
        detect_kind: DetectKind::IniFilename("Map.ini"),
        extract_fn: extract_as::<crate::references::map_ini::MapIni>,
        patch_fn: patch_as::<crate::references::map_ini::MapIni>,
        validate_fn: Some(validate_as::<crate::references::map_ini::MapIni>),
    }
}

fn make_extra_ini() -> FileType {
    FileType {
        key: "extra_ini",
        name: "Extra.ini",
        description: "Interactive object types",
        extensions: &[".ini"],
        detect_kind: DetectKind::IniFilename("Extra.ini"),
        extract_fn: extract_as::<crate::references::extra_ini::Extra>,
        patch_fn: patch_as::<crate::references::extra_ini::Extra>,
        validate_fn: Some(validate_as::<crate::references::extra_ini::Extra>),
    }
}

fn make_event_ini() -> FileType {
    FileType {
        key: "event_ini",
        name: "Event.ini",
        description: "Script/event mappings",
        extensions: &[".ini"],
        detect_kind: DetectKind::IniFilename("Event.ini"),
        extract_fn: extract_as::<crate::references::event_ini::Event>,
        patch_fn: patch_as::<crate::references::event_ini::Event>,
        validate_fn: Some(validate_as::<crate::references::event_ini::Event>),
    }
}

fn make_monster_ini() -> FileType {
    FileType {
        key: "monster_ini",
        name: "Monster.ini",
        description: "Monster visual refs",
        extensions: &[".ini"],
        detect_kind: DetectKind::IniFilename("Monster.ini"),
        extract_fn: extract_as::<crate::references::monster_ini::MonsterIni>,
        patch_fn: patch_as::<crate::references::monster_ini::MonsterIni>,
        validate_fn: Some(validate_as::<crate::references::monster_ini::MonsterIni>),
    }
}

fn make_npc_ini() -> FileType {
    FileType {
        key: "npc_ini",
        name: "Npc.ini",
        description: "NPC visual refs",
        extensions: &[".ini"],
        detect_kind: DetectKind::IniFilename("Npc.ini"),
        extract_fn: extract_as::<crate::references::npc_ini::NpcIni>,
        patch_fn: patch_as::<crate::references::npc_ini::NpcIni>,
        validate_fn: Some(validate_as::<crate::references::npc_ini::NpcIni>),
    }
}

fn make_wave_ini() -> FileType {
    FileType {
        key: "wave_ini",
        name: "Wave.ini",
        description: "Audio/SNF references",
        extensions: &[".ini"],
        detect_kind: DetectKind::IniFilename("Wave.ini"),
        extract_fn: extract_as::<crate::references::wave_ini::WaveIni>,
        patch_fn: patch_as::<crate::references::wave_ini::WaveIni>,
        validate_fn: Some(validate_as::<crate::references::wave_ini::WaveIni>),
    }
}

fn make_weapons() -> FileType {
    FileType {
        key: "weapons",
        name: "WeaponItem.db",
        description: "Weapons & armor database",
        extensions: &[".db"],
        detect_kind: DetectKind::DbFilename(&["WeaponItem.db", "weaponItem.db"]),
        extract_fn: extract_as::<crate::references::weapons_db::WeaponItem>,
        patch_fn: patch_as::<crate::references::weapons_db::WeaponItem>,
        validate_fn: Some(validate_as::<crate::references::weapons_db::WeaponItem>),
    }
}

fn make_monsters() -> FileType {
    FileType {
        key: "monsters",
        name: "Monster.db",
        description: "Monster attributes",
        extensions: &[".db"],
        detect_kind: DetectKind::DbFilename(&["Monster.db", "monster.db"]),
        extract_fn: extract_as::<crate::references::monster_db::Monster>,
        patch_fn: patch_as::<crate::references::monster_db::Monster>,
        validate_fn: Some(validate_as::<crate::references::monster_db::Monster>),
    }
}

fn make_magic() -> FileType {
    FileType {
        key: "magic",
        name: "Magic.db",
        description: "Magic spell records",
        extensions: &[".db"],
        detect_kind: DetectKind::DbFilename(&["Magic.db", "magic.db", "MulMagic.db"]),
        extract_fn: extract_as::<crate::references::magic_db::MagicSpell>,
        patch_fn: patch_as::<crate::references::magic_db::MagicSpell>,
        validate_fn: None,
    }
}

fn make_store() -> FileType {
    FileType {
        key: "store",
        name: "Store.db",
        description: "Shop inventory records",
        extensions: &[".db"],
        detect_kind: DetectKind::DbFilename(&["Store.db", "STORE.DB", "store.db"]),
        extract_fn: extract_as::<crate::references::store_db::Store>,
        patch_fn: patch_as::<crate::references::store_db::Store>,
        validate_fn: None,
    }
}

fn make_misc_item() -> FileType {
    FileType {
        key: "misc_item",
        name: "MiscItem.db",
        description: "Generic item records",
        extensions: &[".db"],
        detect_kind: DetectKind::DbFilename(&["MiscItem.db", "miscitem.db"]),
        extract_fn: extract_as::<crate::references::misc_item_db::MiscItem>,
        patch_fn: patch_as::<crate::references::misc_item_db::MiscItem>,
        validate_fn: Some(validate_as::<crate::references::misc_item_db::MiscItem>),
    }
}

fn make_heal_item() -> FileType {
    FileType {
        key: "heal_item",
        name: "HealItem.db",
        description: "Consumable records",
        extensions: &[".db"],
        detect_kind: DetectKind::DbFilename(&["HealItem.db", "healitem.db"]),
        extract_fn: extract_as::<crate::references::heal_item_db::HealItem>,
        patch_fn: patch_as::<crate::references::heal_item_db::HealItem>,
        validate_fn: Some(validate_as::<crate::references::heal_item_db::HealItem>),
    }
}

fn make_event_item() -> FileType {
    FileType {
        key: "event_item",
        name: "EventItem.db",
        description: "Quest item records",
        extensions: &[".db"],
        detect_kind: DetectKind::DbFilename(&["EventItem.db", "eventitem.db"]),
        extract_fn: extract_as::<crate::references::event_item_db::EventItem>,
        patch_fn: patch_as::<crate::references::event_item_db::EventItem>,
        validate_fn: Some(validate_as::<crate::references::event_item_db::EventItem>),
    }
}

fn make_edit_item() -> FileType {
    FileType {
        key: "edit_item",
        name: "EditItem.db",
        description: "Modifiable item records",
        extensions: &[".db"],
        detect_kind: DetectKind::DbFilename(&["EditItem.db", "edititem.db"]),
        extract_fn: extract_as::<crate::references::edit_item_db::EditItem>,
        patch_fn: patch_as::<crate::references::edit_item_db::EditItem>,
        validate_fn: Some(validate_as::<crate::references::edit_item_db::EditItem>),
    }
}

fn make_party_level() -> FileType {
    FileType {
        key: "party_level",
        name: "PrtLevel.db",
        description: "EXP table records",
        extensions: &[".db"],
        detect_kind: DetectKind::DbFilename(&["PrtLevel.db", "prtlevel.db"]),
        extract_fn: extract_as::<crate::references::party_level_db::PartyLevelNpc>,
        patch_fn: patch_as::<crate::references::party_level_db::PartyLevelNpc>,
        validate_fn: None,
    }
}

fn make_party_ini() -> FileType {
    FileType {
        key: "party_ini",
        name: "PrtIni.db",
        description: "Party NPC metadata",
        extensions: &[".db"],
        detect_kind: DetectKind::DbFilename(&["PrtIni.db", "prtini.db"]),
        extract_fn: extract_as::<crate::references::party_ini_db::PartyIniNpc>,
        patch_fn: patch_as::<crate::references::party_ini_db::PartyIniNpc>,
        validate_fn: None,
    }
}

fn make_chdata() -> FileType {
    FileType {
        key: "chdata",
        name: "ChData.db",
        description: "Character data records",
        extensions: &[".db"],
        detect_kind: DetectKind::DbFilename(&["ChData.db", "chdata.db"]),
        extract_fn: extract_as::<crate::references::chdata_db::ChData>,
        patch_fn: patch_as::<crate::references::chdata_db::ChData>,
        validate_fn: None,
    }
}

fn make_party_ref() -> FileType {
    FileType {
        key: "party_ref",
        name: "PartyRef.ref",
        description: "Character definitions",
        extensions: &[".ref"],
        detect_kind: DetectKind::RefFilename("PartyRef"),
        extract_fn: extract_as::<crate::references::party_ref::PartyRef>,
        patch_fn: patch_as::<crate::references::party_ref::PartyRef>,
        validate_fn: Some(validate_as::<crate::references::party_ref::PartyRef>),
    }
}

fn make_draw_item() -> FileType {
    FileType {
        key: "draw_item",
        name: "DRAWITEM.ref",
        description: "Map placement records",
        extensions: &[".ref"],
        detect_kind: DetectKind::RefFilename("DRAWITEM"),
        extract_fn: extract_as::<crate::references::draw_item::DrawItem>,
        patch_fn: patch_as::<crate::references::draw_item::DrawItem>,
        validate_fn: Some(validate_as::<crate::references::draw_item::DrawItem>),
    }
}

fn make_npc_ref() -> FileType {
    FileType {
        key: "npc_ref",
        name: "Npc*.ref",
        description: "NPC placement records",
        extensions: &[".ref"],
        detect_kind: DetectKind::RefFilename("Npc"),
        extract_fn: extract_as::<crate::references::npc_ref::NPC>,
        patch_fn: patch_as::<crate::references::npc_ref::NPC>,
        validate_fn: None,
    }
}

fn make_monster_ref() -> FileType {
    FileType {
        key: "monster_ref",
        name: "Mon*.ref",
        description: "Monster placement records",
        extensions: &[".ref"],
        detect_kind: DetectKind::RefFilename("Mon"),
        extract_fn: extract_as::<crate::references::monster_ref::MonsterRef>,
        patch_fn: patch_as::<crate::references::monster_ref::MonsterRef>,
        validate_fn: None,
    }
}

fn make_extra_ref() -> FileType {
    FileType {
        key: "extra_ref",
        name: "Ext*.ref",
        description: "Special object placements",
        extensions: &[".ref"],
        detect_kind: DetectKind::RefFilename("Ext"),
        extract_fn: extract_as::<crate::references::extra_ref::ExtraRef>,
        patch_fn: patch_as::<crate::references::extra_ref::ExtraRef>,
        validate_fn: Some(validate_as::<crate::references::extra_ref::ExtraRef>),
    }
}

fn make_event_npc_ref() -> FileType {
    FileType {
        key: "event_npc_ref",
        name: "Eventnpc.ref",
        description: "Event NPC placements",
        extensions: &[".ref"],
        detect_kind: DetectKind::RefFilename("Eventnpc"),
        extract_fn: extract_as::<crate::references::event_npc_ref::EventNpcRef>,
        patch_fn: patch_as::<crate::references::event_npc_ref::EventNpcRef>,
        validate_fn: None,
    }
}

fn make_dialog() -> FileType {
    FileType {
        key: "dialog",
        name: "*.dlg",
        description: "Dialogue script CSV",
        extensions: &[".dlg"],
        detect_kind: DetectKind::DlgFilename("Dlg"),
        extract_fn: extract_as::<crate::references::dialog::Dialog>,
        patch_fn: patch_as::<crate::references::dialog::Dialog>,
        validate_fn: Some(validate_as::<crate::references::dialog::Dialog>),
    }
}

fn make_dialog_text() -> FileType {
    FileType {
        key: "dialog_text",
        name: "*.pgp",
        description: "Dialogue text package",
        extensions: &[".pgp"],
        detect_kind: DetectKind::PgpFilename("Pgp"),
        extract_fn: extract_as::<crate::references::dialogue_text::DialogueText>,
        patch_fn: patch_as::<crate::references::dialogue_text::DialogueText>,
        validate_fn: Some(validate_as::<crate::references::dialogue_text::DialogueText>),
    }
}

fn make_quest() -> FileType {
    FileType {
        key: "quest",
        name: "Quest.scr",
        description: "Quest definitions",
        extensions: &[".scr"],
        detect_kind: DetectKind::ScrFilename("Quest.scr"),
        extract_fn: extract_as::<crate::references::quest_scr::Quest>,
        patch_fn: patch_as::<crate::references::quest_scr::Quest>,
        validate_fn: None,
    }
}

fn make_message() -> FileType {
    FileType {
        key: "message",
        name: "Message.scr",
        description: "Diary game messages",
        extensions: &[".scr"],
        detect_kind: DetectKind::ScrFilename("Message.scr"),
        extract_fn: extract_as::<crate::references::message_scr::Message>,
        patch_fn: patch_as::<crate::references::message_scr::Message>,
        validate_fn: None,
    }
}

fn make_map_file() -> FileType {
    FileType {
        key: "map_file",
        name: "*.map",
        description: "Map geometry, sprites, events, tiles (extract only)",
        extensions: &[".map"],
        detect_kind: DetectKind::DbFilename(&[]),
        extract_fn: extract_map_file,
        patch_fn: patch_not_supported,
        validate_fn: None,
    }
}

fn make_gtl() -> FileType {
    FileType {
        key: "gtl",
        name: "*.gtl",
        description: "Ground tile layer (extract only)",
        extensions: &[".gtl"],
        detect_kind: DetectKind::DbFilename(&[]),
        extract_fn: extract_tileset,
        patch_fn: patch_not_supported,
        validate_fn: None,
    }
}

fn make_btl() -> FileType {
    FileType {
        key: "btl",
        name: "*.btl",
        description: "Building tile layer (extract only)",
        extensions: &[".btl"],
        detect_kind: DetectKind::DbFilename(&[]),
        extract_fn: extract_tileset,
        patch_fn: patch_not_supported,
        validate_fn: None,
    }
}

fn make_sprite() -> FileType {
    FileType {
        key: "sprite",
        name: "*.spr",
        description: "Sprite/animation file (extract only)",
        extensions: &[".spr"],
        detect_kind: DetectKind::DbFilename(&[]),
        extract_fn: extract_sprite_info,
        patch_fn: patch_not_supported,
        validate_fn: None,
    }
}

// ===========================================================================
// Generic extract/patch/validate helpers
// ===========================================================================

fn extract_as<T>(path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>>
where
    T: Extractor + Serialize,
{
    let records = T::read_file(path)?;
    let value = serde_json::to_value(&records)?;
    Ok(value)
}

fn patch_as<T>(data: &serde_json::Value, path: &Path) -> Result<(), Box<dyn std::error::Error>>
where
    T: Extractor + for<'de> Deserialize<'de>,
{
    let records: Vec<T> = serde_json::from_value(data.clone())?;
    T::save_file(&records, path)?;
    Ok(())
}

fn validate_as<T>(data: &serde_json::Value) -> Result<(), Vec<String>>
where
    T: for<'de> Deserialize<'de>,
{
    let result: Result<Vec<T>, serde_json::Error> = serde_json::from_value(data.clone());
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(vec![format!("Deserialization error: {}", e)]),
    }
}

fn patch_not_supported(
    _data: &serde_json::Value,
    _path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    Err("Patch operation not supported for this file type".into())
}

fn extract_map_file(path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use crate::map;
    use std::fs::File;
    use std::io::BufReader;

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let map_data = map::read_map_data(&mut reader)?;
    let json = map_data.to_json();
    Ok(serde_json::to_value(&json)?)
}

fn extract_tileset(path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use crate::map::tileset;

    let tiles = tileset::extract(path)?;
    let tile_entries: Vec<serde_json::Value> = tiles
        .iter()
        .enumerate()
        .map(|(i, _tile)| {
            serde_json::json!({
                "index": i,
                "pixels": null,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "tile_count": tiles.len(),
        "tile_width": 32,
        "tile_height": 32,
        "rendered_width": 62,
        "rendered_height": 32,
        "color_format": "RGB565",
        "tiles": tile_entries,
        "note": "Pixel data omitted. Use 'map tiles' command to extract individual tile images.",
    }))
}

fn extract_sprite_info(path: &Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use crate::sprite;
    let info = sprite::get_sprite_info(path)?;
    Ok(serde_json::to_value(&info)?)
}

// ===========================================================================
// Detection helpers
// ===========================================================================

/// Check if the filename matches one of the known names (case-insensitive).
fn detect_db_filename(path: &Path, names: &[&str]) -> bool {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n.to_lowercase(),
        None => return false,
    };
    names.iter().any(|&name| file_name == name.to_lowercase())
}

/// Check if the filename starts with the given prefix (case-insensitive).
fn detect_filename_prefix(path: &Path, prefix: &str) -> bool {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n.to_lowercase(),
        None => return false,
    };
    file_name.starts_with(&prefix.to_lowercase())
}

/// Check if the filename matches exactly (case-insensitive).
fn detect_filename(path: &Path, name: &str) -> bool {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n.to_lowercase(),
        None => return false,
    };
    file_name == name.to_lowercase()
}
