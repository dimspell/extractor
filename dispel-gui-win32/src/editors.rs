//! All spreadsheet editors for DB/INI/ref file types.
// Each editor uses the reusable Spreadsheet component with type-specific
// column definitions and data loading from dispel_core.

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use std::path::PathBuf;

use crate::spreadsheet::{Spreadsheet, ColumnDef, CellValue};

/// Editor type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorTypeId {
    WeaponItem,
    Monster,
    HealItem,
    MiscItem,
    EditItem,
    EventItem,
    MagicSpell,
    Store,
    ChData,
    PartyLevelNpc,
    PartyIniNpc,
    MonsterIni,
    NpcIni,
    EventIni,
    ExtraIni,
    MapIni,
    WaveIni,
    AllMapIni,
    MonsterRef,
    NpcRef,
    ExtraRef,
    EventNpcRef,
    PartyRef,
    DrawItem,
    PartyIni,
    PartyLevelDbLevel,
}

/// Metadata for an editor type.
pub struct EditorTypeInfo {
    pub name: &'static str,
    pub file_pattern: &'static str,
    pub columns: Vec<ColumnDef>,
}

/// Registry of all spreadsheet editor types.
pub fn editor_types() -> Vec<EditorTypeInfo> {
    vec![
        EditorTypeInfo {
            name: "WeaponItem",
            file_pattern: "weaponItem.db",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "Type".to_string(), width: 80, align_right: false },
                ColumnDef { name: "Atk".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Def".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Mat".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Mdf".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Hit".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Avo".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Crit".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Price".to_string(), width: 80, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "Monster",
            file_pattern: "monster.db",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "HP".to_string(), width: 60, align_right: true },
                ColumnDef { name: "MP".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Atk".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Def".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Mat".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Mdf".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Agi".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Luck".to_string(), width: 60, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "HealItem",
            file_pattern: "healItem.db",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "Type".to_string(), width: 80, align_right: false },
                ColumnDef { name: "Power".to_string(), width: 80, align_right: true },
                ColumnDef { name: "Price".to_string(), width: 80, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "MiscItem",
            file_pattern: "miscItem.db",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "Type".to_string(), width: 80, align_right: false },
                ColumnDef { name: "Price".to_string(), width: 80, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "EditItem",
            file_pattern: "editItem.db",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "Type".to_string(), width: 80, align_right: false },
                ColumnDef { name: "Price".to_string(), width: 80, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "EventItem",
            file_pattern: "eventItem.db",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "Type".to_string(), width: 80, align_right: false },
                ColumnDef { name: "Price".to_string(), width: 80, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "MagicSpell",
            file_pattern: "magic.db",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "Type".to_string(), width: 80, align_right: false },
                ColumnDef { name: "Power".to_string(), width: 80, align_right: true },
                ColumnDef { name: "MPCost".to_string(), width: 80, align_right: true },
                ColumnDef { name: "Hit".to_string(), width: 60, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "Store",
            file_pattern: "store.db",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "ItemID".to_string(), width: 80, align_right: true },
                ColumnDef { name: "Price".to_string(), width: 80, align_right: true },
                ColumnDef { name: "Stock".to_string(), width: 80, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "ChData",
            file_pattern: "chdata.db",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "Level".to_string(), width: 60, align_right: true },
                ColumnDef { name: "HP".to_string(), width: 60, align_right: true },
                ColumnDef { name: "MP".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Atk".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Def".to_string(), width: 60, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "PartyLevelNpc",
            file_pattern: "prtlevel.db",
            columns: vec![
                ColumnDef { name: "Level".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Exp".to_string(), width: 100, align_right: true },
                ColumnDef { name: "HP".to_string(), width: 80, align_right: true },
                ColumnDef { name: "MP".to_string(), width: 80, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "PartyIniNpc",
            file_pattern: "prtini.db",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "Level".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Class".to_string(), width: 100, align_right: false },
            ],
        },
        EditorTypeInfo {
            name: "MonsterIni",
            file_pattern: "monster.ini",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "Sprite".to_string(), width: 100, align_right: false },
                ColumnDef { name: "Color".to_string(), width: 80, align_right: false },
            ],
        },
        EditorTypeInfo {
            name: "NpcIni",
            file_pattern: "npc.ini",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "Sprite".to_string(), width: 100, align_right: false },
                ColumnDef { name: "Dialogue".to_string(), width: 150, align_right: false },
            ],
        },
        EditorTypeInfo {
            name: "EventIni",
            file_pattern: "event.ini",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "Type".to_string(), width: 100, align_right: false },
                ColumnDef { name: "Script".to_string(), width: 150, align_right: false },
            ],
        },
        EditorTypeInfo {
            name: "ExtraIni",
            file_pattern: "extra.ini",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "Type".to_string(), width: 100, align_right: false },
            ],
        },
        EditorTypeInfo {
            name: "MapIni",
            file_pattern: "map.ini",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "Width".to_string(), width: 80, align_right: true },
                ColumnDef { name: "Height".to_string(), width: 80, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "WaveIni",
            file_pattern: "wave.ini",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "MonsterID".to_string(), width: 100, align_right: true },
                ColumnDef { name: "Count".to_string(), width: 60, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "AllMapIni",
            file_pattern: "allmap.ini",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "MapFile".to_string(), width: 150, align_right: false },
            ],
        },
        EditorTypeInfo {
            name: "MonsterRef",
            file_pattern: "mon*.ref",
            columns: vec![
                ColumnDef { name: "MapID".to_string(), width: 80, align_right: true },
                ColumnDef { name: "MonsterID".to_string(), width: 100, align_right: true },
                ColumnDef { name: "X".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Y".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Dir".to_string(), width: 60, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "NpcRef",
            file_pattern: "npccat*.ref",
            columns: vec![
                ColumnDef { name: "MapID".to_string(), width: 80, align_right: true },
                ColumnDef { name: "NpcID".to_string(), width: 100, align_right: true },
                ColumnDef { name: "X".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Y".to_string(), width: 60, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "ExtraRef",
            file_pattern: "extdun*.ref",
            columns: vec![
                ColumnDef { name: "MapID".to_string(), width: 80, align_right: true },
                ColumnDef { name: "Type".to_string(), width: 100, align_right: false },
                ColumnDef { name: "X".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Y".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Param".to_string(), width: 80, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "EventNpcRef",
            file_pattern: "eventnpc.ref",
            columns: vec![
                ColumnDef { name: "MapID".to_string(), width: 80, align_right: true },
                ColumnDef { name: "EventID".to_string(), width: 100, align_right: true },
                ColumnDef { name: "NpcID".to_string(), width: 100, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "PartyRef",
            file_pattern: "partyref.ref",
            columns: vec![
                ColumnDef { name: "MapID".to_string(), width: 80, align_right: true },
                ColumnDef { name: "PartyID".to_string(), width: 100, align_right: true },
                ColumnDef { name: "X".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Y".to_string(), width: 60, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "DrawItem",
            file_pattern: "drawitem.ref",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "Type".to_string(), width: 100, align_right: false },
                ColumnDef { name: "Price".to_string(), width: 80, align_right: true },
            ],
        },
        EditorTypeInfo {
            name: "PartyIni",
            file_pattern: "prtini.db",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Name".to_string(), width: 200, align_right: false },
                ColumnDef { name: "Level".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Class".to_string(), width: 100, align_right: false },
            ],
        },
        EditorTypeInfo {
            name: "PartyLevelDbLevel",
            file_pattern: "prtlevel.db",
            columns: vec![
                ColumnDef { name: "Level".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Exp".to_string(), width: 100, align_right: true },
                ColumnDef { name: "HP".to_string(), width: 80, align_right: true },
                ColumnDef { name: "MP".to_string(), width: 80, align_right: true },
                ColumnDef { name: "Atk".to_string(), width: 60, align_right: true },
                ColumnDef { name: "Def".to_string(), width: 60, align_right: true },
            ],
        },
    ]
}

/// Get the editor type ID for a given file path.
pub fn editor_type_for_path(path: &Path) -> Option<EditorTypeId> {
    let file_name = path.file_name()?.to_string_lossy().to_string();
    let editor_types = editor_types();

    for (i, et) in editor_types.iter().enumerate() {
        if matches_pattern(&file_name, et.file_pattern) {
            return Some(EditorTypeId::from_index(i));
        }
    }
    None
}

fn matches_pattern(filename: &str, pattern: &str) -> bool {
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            return filename.starts_with(parts[0]) && filename.ends_with(parts[1]);
        }
    }
    filename == pattern
}

impl EditorTypeId {
    fn from_index(idx: usize) -> Self {
        match idx {
            0 => EditorTypeId::WeaponItem,
            1 => EditorTypeId::Monster,
            2 => EditorTypeId::HealItem,
            3 => EditorTypeId::MiscItem,
            4 => EditorTypeId::EditItem,
            5 => EditorTypeId::EventItem,
            6 => EditorTypeId::MagicSpell,
            7 => EditorTypeId::Store,
            8 => EditorTypeId::ChData,
            9 => EditorTypeId::PartyLevelNpc,
            10 => EditorTypeId::PartyIniNpc,
            11 => EditorTypeId::MonsterIni,
            12 => EditorTypeId::NpcIni,
            13 => EditorTypeId::EventIni,
            14 => EditorTypeId::ExtraIni,
            15 => EditorTypeId::MapIni,
            16 => EditorTypeId::WaveIni,
            17 => EditorTypeId::AllMapIni,
            18 => EditorTypeId::MonsterRef,
            19 => EditorTypeId::NpcRef,
            20 => EditorTypeId::ExtraRef,
            21 => EditorTypeId::EventNpcRef,
            22 => EditorTypeId::PartyRef,
            23 => EditorTypeId::DrawItem,
            24 => EditorTypeId::PartyIni,
            25 => EditorTypeId::PartyLevelDbLevel,
            _ => EditorTypeId::WeaponItem,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            EditorTypeId::WeaponItem => "WeaponItem",
            EditorTypeId::Monster => "Monster",
            EditorTypeId::HealItem => "HealItem",
            EditorTypeId::MiscItem => "MiscItem",
            EditorTypeId::EditItem => "EditItem",
            EditorTypeId::EventItem => "EventItem",
            EditorTypeId::MagicSpell => "MagicSpell",
            EditorTypeId::Store => "Store",
            EditorTypeId::ChData => "ChData",
            EditorTypeId::PartyLevelNpc => "PartyLevelNpc",
            EditorTypeId::PartyIniNpc => "PartyIniNpc",
            EditorTypeId::MonsterIni => "MonsterIni",
            EditorTypeId::NpcIni => "NpcIni",
            EditorTypeId::EventIni => "EventIni",
            EditorTypeId::ExtraIni => "ExtraIni",
            EditorTypeId::MapIni => "MapIni",
            EditorTypeId::WaveIni => "WaveIni",
            EditorTypeId::AllMapIni => "AllMapIni",
            EditorTypeId::MonsterRef => "MonsterRef",
            EditorTypeId::NpcRef => "NpcRef",
            EditorTypeId::ExtraRef => "ExtraRef",
            EditorTypeId::EventNpcRef => "EventNpcRef",
            EditorTypeId::PartyRef => "PartyRef",
            EditorTypeId::DrawItem => "DrawItem",
            EditorTypeId::PartyIni => "PartyIni",
            EditorTypeId::PartyLevelDbLevel => "PartyLevelDbLevel",
        }
    }
}

/// Create a spreadsheet editor for the given editor type.
pub fn create_editor(editor_type: EditorTypeId, parent: HWND) -> Result<Spreadsheet> {
    let info = editor_types();
    let idx = editor_type_index(editor_type);
    let et = &info[idx];
    Spreadsheet::new(parent, et.columns.clone())
}

fn editor_type_index(editor_type: EditorTypeId) -> usize {
    match editor_type {
        EditorTypeId::WeaponItem => 0,
        EditorTypeId::Monster => 1,
        EditorTypeId::HealItem => 2,
        EditorTypeId::MiscItem => 3,
        EditorTypeId::EditItem => 4,
        EditorTypeId::EventItem => 5,
        EditorTypeId::MagicSpell => 6,
        EditorTypeId::Store => 7,
        EditorTypeId::ChData => 8,
        EditorTypeId::PartyLevelNpc => 9,
        EditorTypeId::PartyIniNpc => 10,
        EditorTypeId::MonsterIni => 11,
        EditorTypeId::NpcIni => 12,
        EditorTypeId::EventIni => 13,
        EditorTypeId::ExtraIni => 14,
        EditorTypeId::MapIni => 15,
        EditorTypeId::WaveIni => 16,
        EditorTypeId::AllMapIni => 17,
        EditorTypeId::MonsterRef => 18,
        EditorTypeId::NpcRef => 19,
        EditorTypeId::ExtraRef => 20,
        EditorTypeId::EventNpcRef => 21,
        EditorTypeId::PartyRef => 22,
        EditorTypeId::DrawItem => 23,
        EditorTypeId::PartyIni => 24,
        EditorTypeId::PartyLevelDbLevel => 25,
    }
}
