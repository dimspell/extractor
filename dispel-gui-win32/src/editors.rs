//! All spreadsheet editors for DB/INI/ref file types.
// Each editor uses the reusable Spreadsheet component with type-specific
// column definitions and data loading from dispel_core.

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use std::path::PathBuf;

use crate::spreadsheet::{Spreadsheet, ColumnDef, CellValue, Row};
use dispel_core::WeaponItem;
use dispel_core::Monster;
use dispel_core::HealItem;
use dispel_core::MiscItem;
use dispel_core::EditItem;
use dispel_core::EventItem;
use dispel_core::MagicSpell;
use dispel_core::ChData;
use dispel_core::PartyIniNpc;
use dispel_core::PartyLevelNpc;
use dispel_core::MonsterRef;
use dispel_core::ExtraRef;
use dispel_core::NPC;
use dispel_core::MonsterIni;
use dispel_core::NpcIni;
use dispel_core::Extra;
use dispel_core::Event;
use dispel_core::EventNpcRef;
use dispel_core::MapIni;
use dispel_core::WaveIni;
use dispel_core::Map;
use dispel_core::Extractor;
use dispel_core::Store;
use dispel_core::PartyRef;
use dispel_core::DrawItem;
use dispel_core::DialogueParagraph;
use dispel_core::DialogueScript;
use dispel_core::Message;
use dispel_core::Quest;

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
    DialogueScript,
    DialogueParagraph,
    MessageScr,
    QuestScr,
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
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Name".to_string(), width: 160, align_right: false, numeric: false },
                ColumnDef { name: "Desc".to_string(), width: 200, align_right: false, numeric: false },
                ColumnDef { name: "Price".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "P1".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "HP".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "MP".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Str".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Agi".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Wis".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Con".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Avo".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Hit".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Atk".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Def".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Mat".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Dur".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "P2".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "P3".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "ReqStr".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "P4".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "ReqAgi".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "P5".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "ReqWis".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "P6".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "P7".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "P8".to_string(), width: 60, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "Monster",
            file_pattern: "monster.db",
            columns: vec![
                ColumnDef { name: "Name".to_string(), width: 160, align_right: false, numeric: false },
                ColumnDef { name: "HPmax".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "HPmin".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "MPmax".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "MPmin".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "AtkMax".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "AtkMin".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "DefMax".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "DefMin".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "MatMax".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "MatMin".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "ExpMax".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "ExpMin".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "GoldMax".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "GoldMin".to_string(), width: 80, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "HealItem",
            file_pattern: "healItem.db",
            columns: vec![
                ColumnDef { name: "Name".to_string(), width: 160, align_right: false, numeric: false },
                ColumnDef { name: "Price".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "HPheal".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "MPheal".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "FullHP".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "FullMP".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Poison".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Petrif".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Polymorph".to_string(), width: 70, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "MiscItem",
            file_pattern: "miscItem.db",
            columns: vec![
                ColumnDef { name: "Name".to_string(), width: 160, align_right: false, numeric: false },
                ColumnDef { name: "Price".to_string(), width: 80, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "EditItem",
            file_pattern: "editItem.db",
            columns: vec![
                ColumnDef { name: "Name".to_string(), width: 160, align_right: false, numeric: false },
                ColumnDef { name: "Price".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "HP".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "MP".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Str".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Agi".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Wis".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Con".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Atk".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Def".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Mat".to_string(), width: 60, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "EventItem",
            file_pattern: "eventItem.db",
            columns: vec![
                ColumnDef { name: "Name".to_string(), width: 160, align_right: false, numeric: false },
                ColumnDef { name: "Price".to_string(), width: 80, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "MagicSpell",
            file_pattern: "magic.db",
            columns: vec![
                ColumnDef { name: "Enabled".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "ManaCost".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "Success".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "BaseDmg".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "Range".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "LevelReq".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "EffectVal".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "EffectType".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "School".to_string(), width: 70, align_right: false, numeric: false },
                ColumnDef { name: "AnimID".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "VisID".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "IconID".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "Target".to_string(), width: 80, align_right: false, numeric: false },
            ],
        },
        EditorTypeInfo {
            name: "Store",
            file_pattern: "store.db",
            columns: vec![
                ColumnDef { name: "Name".to_string(), width: 160, align_right: false, numeric: false },
                ColumnDef { name: "InnCost".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "Unknown".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "Products".to_string(), width: 80, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "ChData",
            file_pattern: "chdata.db",
            columns: vec![
                ColumnDef { name: "W_Str".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "W_Con".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "K_Str".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "K_Con".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "A_Str".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "A_Con".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "M_Str".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "M_Con".to_string(), width: 60, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "PartyLevelNpc",
            file_pattern: "prtlevel.db",
            columns: vec![
                ColumnDef { name: "NpcIdx".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Records".to_string(), width: 80, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "PartyIniNpc",
            file_pattern: "prtini.db",
            columns: vec![
                ColumnDef { name: "Name".to_string(), width: 160, align_right: false, numeric: false },
                ColumnDef { name: "U1".to_string(), width: 40, align_right: true, numeric: true },
                ColumnDef { name: "U2".to_string(), width: 40, align_right: true, numeric: true },
                ColumnDef { name: "U3".to_string(), width: 40, align_right: true, numeric: true },
                ColumnDef { name: "U4".to_string(), width: 40, align_right: true, numeric: true },
                ColumnDef { name: "U5".to_string(), width: 50, align_right: true, numeric: true },
                ColumnDef { name: "U6".to_string(), width: 50, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "MonsterIni",
            file_pattern: "monster.ini",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Name".to_string(), width: 160, align_right: false, numeric: false },
                ColumnDef { name: "Sprite".to_string(), width: 100, align_right: false, numeric: false },
                ColumnDef { name: "AtkAnim".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "HitAnim".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "DeathAnim".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "WalkAnim".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "CastAnim".to_string(), width: 70, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "NpcIni",
            file_pattern: "npc.ini",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Sprite".to_string(), width: 100, align_right: false, numeric: false },
                ColumnDef { name: "Desc".to_string(), width: 200, align_right: false, numeric: false },
            ],
        },
        EditorTypeInfo {
            name: "EventIni",
            file_pattern: "event.ini",
            columns: vec![
                ColumnDef { name: "EventID".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "ReqEvent".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "Counter".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "Filename".to_string(), width: 150, align_right: false, numeric: false },
            ],
        },
        EditorTypeInfo {
            name: "ExtraIni",
            file_pattern: "extra.ini",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Sprite".to_string(), width: 100, align_right: false, numeric: false },
                ColumnDef { name: "Flag".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Desc".to_string(), width: 150, align_right: false, numeric: false },
            ],
        },
        EditorTypeInfo {
            name: "MapIni",
            file_pattern: "map.ini",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "EventCam".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "StartX".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "StartY".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "MapID".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "MonsterFile".to_string(), width: 120, align_right: false, numeric: false },
                ColumnDef { name: "NpcFile".to_string(), width: 120, align_right: false, numeric: false },
                ColumnDef { name: "ExtraFile".to_string(), width: 120, align_right: false, numeric: false },
                ColumnDef { name: "Music".to_string(), width: 60, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "WaveIni",
            file_pattern: "wave.ini",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "SNF".to_string(), width: 120, align_right: false, numeric: false },
                ColumnDef { name: "Flag".to_string(), width: 60, align_right: false, numeric: false },
            ],
        },
        EditorTypeInfo {
            name: "AllMapIni",
            file_pattern: "allmap.ini",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "MapFile".to_string(), width: 120, align_right: false, numeric: false },
                ColumnDef { name: "MapName".to_string(), width: 160, align_right: false, numeric: false },
                ColumnDef { name: "PGP".to_string(), width: 120, align_right: false, numeric: false },
                ColumnDef { name: "DLG".to_string(), width: 120, align_right: false, numeric: false },
            ],
        },
        EditorTypeInfo {
            name: "MonsterRef",
            file_pattern: "mon*.ref",
            columns: vec![
                ColumnDef { name: "FileID".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "MonsterID".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "X".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Y".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Flag1".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Flag2".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Pad3".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Flag3".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "EventID".to_string(), width: 80, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "NpcRef",
            file_pattern: "npccat*.ref",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "NpcID".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "Name".to_string(), width: 160, align_right: false, numeric: false },
                ColumnDef { name: "Desc".to_string(), width: 200, align_right: false, numeric: false },
            ],
        },
        EditorTypeInfo {
            name: "ExtraRef",
            file_pattern: "extdun*.ref",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Name".to_string(), width: 120, align_right: false, numeric: false },
                ColumnDef { name: "Type".to_string(), width: 80, align_right: false, numeric: false },
                ColumnDef { name: "X".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Y".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Gold".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "EventID".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "MsgID".to_string(), width: 80, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "EventNpcRef",
            file_pattern: "eventnpc.ref",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "EventID".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "Name".to_string(), width: 160, align_right: false, numeric: false },
            ],
        },
        EditorTypeInfo {
            name: "PartyRef",
            file_pattern: "partyref.ref",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "FullName".to_string(), width: 160, align_right: false, numeric: false },
                ColumnDef { name: "JobName".to_string(), width: 100, align_right: false, numeric: false },
                ColumnDef { name: "MapID".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "NpcID".to_string(), width: 70, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "DrawItem",
            file_pattern: "drawitem.ref",
            columns: vec![
                ColumnDef { name: "MapID".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "X".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Y".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Item".to_string(), width: 80, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "PartyIni",
            file_pattern: "prtini.db",
            columns: vec![
                ColumnDef { name: "Name".to_string(), width: 160, align_right: false, numeric: false },
                ColumnDef { name: "U1".to_string(), width: 40, align_right: true, numeric: true },
                ColumnDef { name: "U2".to_string(), width: 40, align_right: true, numeric: true },
                ColumnDef { name: "U3".to_string(), width: 40, align_right: true, numeric: true },
                ColumnDef { name: "U4".to_string(), width: 40, align_right: true, numeric: true },
                ColumnDef { name: "U5".to_string(), width: 50, align_right: true, numeric: true },
                ColumnDef { name: "U6".to_string(), width: 50, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "PartyLevelDbLevel",
            file_pattern: "prtlevel.db",
            columns: vec![
                ColumnDef { name: "NpcIdx".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Level".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Str".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Con".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Wis".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "HP".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "MP".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Agi".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Atk".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "MRechg".to_string(), width: 70, align_right: true, numeric: true },
                ColumnDef { name: "Def".to_string(), width: 60, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "DialogueScript",
            file_pattern: "*.dlg",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "PrevEvent".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "NextDlg".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "Type".to_string(), width: 60, align_right: false, numeric: false },
                ColumnDef { name: "Owner".to_string(), width: 60, align_right: false, numeric: false },
                ColumnDef { name: "DlgID".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "Next1".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Next2".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Next3".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "EventID".to_string(), width: 80, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "DialogueParagraph",
            file_pattern: "*.pgp",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Text".to_string(), width: 300, align_right: false, numeric: false },
                ColumnDef { name: "Comment".to_string(), width: 200, align_right: false, numeric: false },
                ColumnDef { name: "Param1".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "WaveID".to_string(), width: 80, align_right: true, numeric: true },
            ],
        },
        EditorTypeInfo {
            name: "MessageScr",
            file_pattern: "Message.scr",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Line1".to_string(), width: 200, align_right: false, numeric: false },
                ColumnDef { name: "Line2".to_string(), width: 200, align_right: false, numeric: false },
                ColumnDef { name: "Line3".to_string(), width: 200, align_right: false, numeric: false },
            ],
        },
        EditorTypeInfo {
            name: "QuestScr",
            file_pattern: "Quest.scr",
            columns: vec![
                ColumnDef { name: "ID".to_string(), width: 60, align_right: true, numeric: true },
                ColumnDef { name: "Type".to_string(), width: 80, align_right: true, numeric: true },
                ColumnDef { name: "Title".to_string(), width: 200, align_right: false, numeric: false },
                ColumnDef { name: "Description".to_string(), width: 300, align_right: false, numeric: false },
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
            26 => EditorTypeId::DialogueScript,
            27 => EditorTypeId::DialogueParagraph,
            28 => EditorTypeId::MessageScr,
            29 => EditorTypeId::QuestScr,
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
            EditorTypeId::DialogueScript => "DialogueScript",
            EditorTypeId::DialogueParagraph => "DialogueParagraph",
            EditorTypeId::MessageScr => "MessageScr",
            EditorTypeId::QuestScr => "QuestScr",
        }
    }
}

/// Create a spreadsheet editor for the given editor type.
pub fn create_editor(editor_type: EditorTypeId, parent: HWND, hwnd_main: HWND, tab_id: usize) -> Result<Spreadsheet> {
    let info = editor_types();
    let idx = editor_type_index(editor_type);
    let et = &info[idx];
    Spreadsheet::new(parent, hwnd_main, et.columns.clone(), tab_id)
}

// ── Per-type row conversion ───────────────────────────────────────────────

/// Convert a WeaponItem record to a Row of CellValue.
/// All 27 struct fields are emitted in display-friendly order
/// (matches column definitions and save function indices).
pub fn weapon_item_to_row(item: &WeaponItem) -> Row {
    vec![
        CellValue::Integer(item.id as i64),                    // 0: ID
        CellValue::String(item.name.clone()),                  // 1: Name
        CellValue::String(item.description.clone()),           // 2: Desc
        CellValue::Integer(item.base_price as i64),            // 3: Price
        CellValue::Integer(item.padding1 as i64),              // 4: P1
        CellValue::Integer(item.health_points as i64),         // 5: HP
        CellValue::Integer(item.mana_points as i64),            // 6: MP
        CellValue::Integer(item.strength as i64),              // 7: Str
        CellValue::Integer(item.agility as i64),               // 8: Agi
        CellValue::Integer(item.wisdom as i64),                 // 9: Wis
        CellValue::Integer(item.constitution as i64),          // 10: Con
        CellValue::Integer(item.to_dodge as i64),              // 11: Avo
        CellValue::Integer(item.to_hit as i64),                // 12: Hit
        CellValue::Integer(item.attack as i64),                // 13: Atk
        CellValue::Integer(item.defense as i64),               // 14: Def
        CellValue::Integer(item.magical_strength as i64),      // 15: Mat
        CellValue::Integer(item.durability as i64),            // 16: Dur
        CellValue::Integer(item.padding2 as i64),              // 17: P2
        CellValue::Integer(item.padding3 as i64),              // 18: P3
        CellValue::Integer(item.req_strength as i64),          // 19: ReqStr
        CellValue::Integer(item.padding4 as i64),              // 20: P4
        CellValue::Integer(item.req_agility as i64),           // 21: ReqAgi
        CellValue::Integer(item.padding5 as i64),              // 22: P5
        CellValue::Integer(item.req_wisdom as i64),            // 23: ReqWis
        CellValue::Integer(item.padding6 as i64),              // 24: P6
        CellValue::Integer(item.padding7 as i64),              // 25: P7
        CellValue::Integer(item.padding8 as i64),              // 26: P8
    ]
}

pub fn save_weapon_items(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let originals = WeaponItem::parse(&mut Cursor::new(original_data), original_data.len() as u64)
        .unwrap_or_default();
    let mut items = originals;
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i];
        if row.len() < 27 { continue; }
        let item = &mut items[i];
        // 0:  ID
        if let CellValue::Integer(v) = &row[0] { item.id = *v as i32; }
        // 1:  Name
        if let CellValue::String(s) = &row[1] { item.name = s.clone(); }
        // 2:  Desc
        if let CellValue::String(s) = &row[2] { item.description = s.clone(); }
        // 3:  Price
        if let CellValue::Integer(v) = &row[3] { item.base_price = *v as i32; }
        // 4:  P1 (i32)
        if let CellValue::Integer(v) = &row[4] { item.padding1 = *v as i32; }
        // 5:  HP
        if let CellValue::Integer(v) = &row[5] { item.health_points = *v as i16; }
        // 6:  MP
        if let CellValue::Integer(v) = &row[6] { item.mana_points = *v as i16; }
        // 7:  Str
        if let CellValue::Integer(v) = &row[7] { item.strength = *v as i16; }
        // 8:  Agi
        if let CellValue::Integer(v) = &row[8] { item.agility = *v as i16; }
        // 9:  Wis
        if let CellValue::Integer(v) = &row[9] { item.wisdom = *v as i16; }
        // 10: Con
        if let CellValue::Integer(v) = &row[10] { item.constitution = *v as i16; }
        // 11: Avo (to_dodge)
        if let CellValue::Integer(v) = &row[11] { item.to_dodge = *v as i16; }
        // 12: Hit (to_hit)
        if let CellValue::Integer(v) = &row[12] { item.to_hit = *v as i16; }
        // 13: Atk (attack)
        if let CellValue::Integer(v) = &row[13] { item.attack = *v as i16; }
        // 14: Def (defense)
        if let CellValue::Integer(v) = &row[14] { item.defense = *v as i16; }
        // 15: Mat (magical_strength)
        if let CellValue::Integer(v) = &row[15] { item.magical_strength = *v as i16; }
        // 16: Dur (durability)
        if let CellValue::Integer(v) = &row[16] { item.durability = *v as i16; }
        // 17: P2
        if let CellValue::Integer(v) = &row[17] { item.padding2 = *v as i16; }
        // 18: P3
        if let CellValue::Integer(v) = &row[18] { item.padding3 = *v as i16; }
        // 19: ReqStr
        if let CellValue::Integer(v) = &row[19] { item.req_strength = *v as i16; }
        // 20: P4
        if let CellValue::Integer(v) = &row[20] { item.padding4 = *v as i16; }
        // 21: ReqAgi
        if let CellValue::Integer(v) = &row[21] { item.req_agility = *v as i16; }
        // 22: P5
        if let CellValue::Integer(v) = &row[22] { item.padding5 = *v as i16; }
        // 23: ReqWis
        if let CellValue::Integer(v) = &row[23] { item.req_wisdom = *v as i16; }
        // 24: P6
        if let CellValue::Integer(v) = &row[24] { item.padding6 = *v as i16; }
        // 25: P7
        if let CellValue::Integer(v) = &row[25] { item.padding7 = *v as i16; }
        // 26: P8
        if let CellValue::Integer(v) = &row[26] { item.padding8 = *v as i16; }
    }
    WeaponItem::save_file(&items, path)
}

/// Convert a Monster record to a Row of CellValue.
pub fn monster_to_row(item: &Monster) -> Row {
    vec![
        CellValue::String(item.name.clone()),
        CellValue::Integer(item.health_points_max as i64),
        CellValue::Integer(item.health_points_min as i64),
        CellValue::Integer(item.mana_points_max as i64),
        CellValue::Integer(item.mana_points_min as i64),
        CellValue::Integer(item.offense_max as i64),
        CellValue::Integer(item.offense_min as i64),
        CellValue::Integer(item.defense_max as i64),
        CellValue::Integer(item.defense_min as i64),
        CellValue::Integer(item.magic_attack_max as i64),
        CellValue::Integer(item.magic_attack_min as i64),
        CellValue::Integer(item.exp_gain_max as i64),
        CellValue::Integer(item.exp_gain_min as i64),
        CellValue::Integer(item.gold_drop_max as i64),
        CellValue::Integer(item.gold_drop_min as i64),
    ]
}

pub fn save_monsters(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let originals = Monster::parse(&mut Cursor::new(original_data), original_data.len() as u64)
        .unwrap_or_default();
    let mut items = originals;
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i];
        if row.len() < 15 { continue; }
        let item = &mut items[i];
        if let CellValue::String(s) = &row[0] { item.name = s.clone(); }
        if let CellValue::Integer(v) = &row[1] { item.health_points_max = *v as i32; }
        if let CellValue::Integer(v) = &row[2] { item.health_points_min = *v as i32; }
        if let CellValue::Integer(v) = &row[3] { item.mana_points_max = *v as i32; }
        if let CellValue::Integer(v) = &row[4] { item.mana_points_min = *v as i32; }
        if let CellValue::Integer(v) = &row[5] { item.offense_max = *v as i32; }
        if let CellValue::Integer(v) = &row[6] { item.offense_min = *v as i32; }
        if let CellValue::Integer(v) = &row[7] { item.defense_max = *v as i32; }
        if let CellValue::Integer(v) = &row[8] { item.defense_min = *v as i32; }
        if let CellValue::Integer(v) = &row[9] { item.magic_attack_max = *v as i32; }
        if let CellValue::Integer(v) = &row[10] { item.magic_attack_min = *v as i32; }
        if let CellValue::Integer(v) = &row[11] { item.exp_gain_max = *v as i32; }
        if let CellValue::Integer(v) = &row[12] { item.exp_gain_min = *v as i32; }
        if let CellValue::Integer(v) = &row[13] { item.gold_drop_max = *v as i32; }
        if let CellValue::Integer(v) = &row[14] { item.gold_drop_min = *v as i32; }
    }
    Monster::save_file(&items, path)
}

// ── HealItem ──────────────────────────────────────────────────────────────

pub fn heal_item_to_row(item: &HealItem) -> Row {
    vec![
        CellValue::String(item.name.clone()),
        CellValue::Integer(item.base_price as i64),
        CellValue::Integer(item.health_points as i64),
        CellValue::Integer(item.mana_points as i64),
        CellValue::Integer(item.restore_full_health as i64),
        CellValue::Integer(item.restore_full_mana as i64),
        CellValue::Integer(item.poison_heal as i64),
        CellValue::Integer(item.petrif_heal as i64),
        CellValue::Integer(item.polimorph_heal as i64),
    ]
}

pub fn save_heal_items(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = HealItem::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 9 { continue; }
        if let CellValue::String(s) = &row[0] { items[i].name = s.clone(); }
        if let CellValue::Integer(v) = &row[1] { items[i].base_price = *v as i32; }
        if let CellValue::Integer(v) = &row[2] { items[i].health_points = *v as i16; }
        if let CellValue::Integer(v) = &row[3] { items[i].mana_points = *v as i16; }
        if let CellValue::Integer(v) = &row[4] { items[i].restore_full_health = HealItemFlag::from(*v as u8); }
        if let CellValue::Integer(v) = &row[5] { items[i].restore_full_mana = HealItemFlag::from(*v as u8); }
        if let CellValue::Integer(v) = &row[6] { items[i].poison_heal = HealItemFlag::from(*v as u8); }
        if let CellValue::Integer(v) = &row[7] { items[i].petrif_heal = HealItemFlag::from(*v as u8); }
        if let CellValue::Integer(v) = &row[8] { items[i].polimorph_heal = HealItemFlag::from(*v as u8); }
    }
    HealItem::save_file(&items, path)
}

// ── MiscItem ──────────────────────────────────────────────────────────────

pub fn misc_item_to_row(item: &MiscItem) -> Row {
    vec![
        CellValue::String(item.name.clone()),
        CellValue::Integer(item.base_price as i64),
    ]
}

pub fn save_misc_items(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = MiscItem::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 2 { continue; }
        if let CellValue::String(s) = &row[0] { items[i].name = s.clone(); }
        if let CellValue::Integer(v) = &row[1] { items[i].base_price = *v as i32; }
    }
    MiscItem::save_file(&items, path)
}

// ── EditItem ──────────────────────────────────────────────────────────────

pub fn edit_item_to_row(item: &EditItem) -> Row {
    vec![
        CellValue::String(item.name.clone()),
        CellValue::Integer(item.base_price as i64),
        CellValue::Integer(item.health_points as i64),
        CellValue::Integer(item.mana_points as i64),
        CellValue::Integer(item.strength as i64),
        CellValue::Integer(item.agility as i64),
        CellValue::Integer(item.wisdom as i64),
        CellValue::Integer(item.constitution as i64),
        CellValue::Integer(item.offense as i64),
        CellValue::Integer(item.defense as i64),
        CellValue::Integer(item.magical_power as i64),
    ]
}

pub fn save_edit_items(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = EditItem::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 11 { continue; }
        if let CellValue::String(s) = &row[0] { items[i].name = s.clone(); }
        if let CellValue::Integer(v) = &row[1] { items[i].base_price = *v as i32; }
        if let CellValue::Integer(v) = &row[2] { items[i].health_points = *v as i16; }
        if let CellValue::Integer(v) = &row[3] { items[i].mana_points = *v as i16; }
        if let CellValue::Integer(v) = &row[4] { items[i].strength = *v as i16; }
        if let CellValue::Integer(v) = &row[5] { items[i].agility = *v as i16; }
        if let CellValue::Integer(v) = &row[6] { items[i].wisdom = *v as i16; }
        if let CellValue::Integer(v) = &row[7] { items[i].constitution = *v as i16; }
        if let CellValue::Integer(v) = &row[8] { items[i].offense = *v as i16; }
        if let CellValue::Integer(v) = &row[9] { items[i].defense = *v as i16; }
        if let CellValue::Integer(v) = &row[10] { items[i].magical_power = *v as i16; }
    }
    EditItem::save_file(&items, path)
}

// ── EventItem ─────────────────────────────────────────────────────────────

pub fn event_item_to_row(item: &EventItem) -> Row {
    vec![
        CellValue::String(item.name.clone()),
        CellValue::Integer(item.base_price as i64),
    ]
}

pub fn save_event_items(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = EventItem::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 2 { continue; }
        if let CellValue::String(s) = &row[0] { items[i].name = s.clone(); }
        if let CellValue::Integer(v) = &row[1] { items[i].base_price = *v as i32; }
    }
    EventItem::save_file(&items, path)
}

// ── MagicSpell ────────────────────────────────────────────────────────────

pub fn magic_spell_to_row(item: &MagicSpell) -> Row {
    vec![
        CellValue::Integer(item.enabled as i64),
        CellValue::Integer(item.mana_cost as i64),
        CellValue::Integer(item.success_rate as i64),
        CellValue::Integer(item.base_damage as i64),
        CellValue::Integer(item.range as i64),
        CellValue::Integer(item.level_required as i64),
        CellValue::Integer(item.effect_value as i64),
        CellValue::Integer(item.effect_type as i64),
        CellValue::Integer(item.magic_school as i64),
        CellValue::Integer(item.animation_id as i64),
        CellValue::Integer(item.visual_id as i64),
        CellValue::Integer(item.icon_id as i64),
        CellValue::Integer(item.target_type as i64),
    ]
}

pub fn save_magic_spells(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = MagicSpell::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 13 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].enabled = MagicSpellFlag::from(*v as u32); }
        if let CellValue::Integer(v) = &row[1] { items[i].mana_cost = *v as u32; }
        if let CellValue::Integer(v) = &row[2] { items[i].success_rate = *v as u32; }
        if let CellValue::Integer(v) = &row[3] { items[i].base_damage = *v as u32; }
        if let CellValue::Integer(v) = &row[4] { items[i].range = *v as u32; }
        if let CellValue::Integer(v) = &row[5] { items[i].level_required = *v as u32; }
        if let CellValue::Integer(v) = &row[6] { items[i].effect_value = *v as u32; }
        if let CellValue::Integer(v) = &row[7] { items[i].effect_type = *v as u32; }
        if let CellValue::Integer(v) = &row[8] { items[i].magic_school = MagicSchool::from(*v as u32); }
        if let CellValue::Integer(v) = &row[9] { items[i].animation_id = *v as u32; }
        if let CellValue::Integer(v) = &row[10] { items[i].visual_id = *v as u32; }
        if let CellValue::Integer(v) = &row[11] { items[i].icon_id = *v as u32; }
        if let CellValue::Integer(v) = &row[12] { items[i].target_type = SpellTargetType::from(*v as u32); }
    }
    MagicSpell::save_file(&items, path)
}

// ── ChData ────────────────────────────────────────────────────────────────

pub fn chdata_to_row(item: &ChData) -> Row {
    vec![
        CellValue::Integer(item.warrior_strength as i64),
        CellValue::Integer(item.warrior_constitution as i64),
        CellValue::Integer(item.knight_strength as i64),
        CellValue::Integer(item.knight_constitution as i64),
        CellValue::Integer(item.archer_strength as i64),
        CellValue::Integer(item.archer_constitution as i64),
        CellValue::Integer(item.mage_strength as i64),
        CellValue::Integer(item.mage_constitution as i64),
    ]
}

pub fn save_chdata(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = ChData::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 8 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].warrior_strength = *v as i16; }
        if let CellValue::Integer(v) = &row[1] { items[i].warrior_constitution = *v as i16; }
        if let CellValue::Integer(v) = &row[2] { items[i].knight_strength = *v as i16; }
        if let CellValue::Integer(v) = &row[3] { items[i].knight_constitution = *v as i16; }
        if let CellValue::Integer(v) = &row[4] { items[i].archer_strength = *v as i16; }
        if let CellValue::Integer(v) = &row[5] { items[i].archer_constitution = *v as i16; }
        if let CellValue::Integer(v) = &row[6] { items[i].mage_strength = *v as i16; }
        if let CellValue::Integer(v) = &row[7] { items[i].mage_constitution = *v as i16; }
    }
    ChData::save_file(&items, path)
}

// ── PartyIniNpc ───────────────────────────────────────────────────────────

pub fn party_ini_npc_to_row(item: &PartyIniNpc) -> Row {
    vec![
        CellValue::String(item.name.clone()),
        CellValue::Integer(item.unknown1 as i64),
        CellValue::Integer(item.unknown2 as i64),
        CellValue::Integer(item.unknown3 as i64),
        CellValue::Integer(item.unknown4 as i64),
        CellValue::Integer(item.unknown5 as i64),
        CellValue::Integer(item.unknown6 as i64),
    ]
}

pub fn save_party_ini_npcs(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = PartyIniNpc::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 7 { continue; }
        if let CellValue::String(s) = &row[0] { items[i].name = s.clone(); }
        if let CellValue::Integer(v) = &row[1] { items[i].unknown1 = *v as u8; }
        if let CellValue::Integer(v) = &row[2] { items[i].unknown2 = *v as u8; }
        if let CellValue::Integer(v) = &row[3] { items[i].unknown3 = *v as u8; }
        if let CellValue::Integer(v) = &row[4] { items[i].unknown4 = *v as u8; }
        if let CellValue::Integer(v) = &row[5] { items[i].unknown5 = *v as u16; }
        if let CellValue::Integer(v) = &row[6] { items[i].unknown6 = *v as u16; }
    }
    PartyIniNpc::save_file(&items, path)
}

// ── MonsterRef ────────────────────────────────────────────────────────────

pub fn monster_ref_to_row(item: &MonsterRef) -> Row {
    vec![
        CellValue::Integer(item.file_id as i64),
        CellValue::Integer(item.mon_id as i64),
        CellValue::Integer(item.pos_x as i64),
        CellValue::Integer(item.pos_y as i64),
        CellValue::Integer(item.padding1 as i64),
        CellValue::Integer(item.padding2 as i64),
        CellValue::Integer(item.padding3 as i64),
        CellValue::Integer(item.padding4 as i64),
        CellValue::Integer(item.event_id as i64),
    ]
}

pub fn save_monster_refs(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = MonsterRef::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 9 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].file_id = *v as i32; }
        if let CellValue::Integer(v) = &row[1] { items[i].mon_id = *v as i32; }
        if let CellValue::Integer(v) = &row[2] { items[i].pos_x = *v as i32; }
        if let CellValue::Integer(v) = &row[3] { items[i].pos_y = *v as i32; }
        if let CellValue::Integer(v) = &row[4] { items[i].padding1 = BooleanFlag::from(*v as i32); }
        if let CellValue::Integer(v) = &row[5] { items[i].padding2 = BooleanFlag::from(*v as i32); }
        if let CellValue::Integer(v) = &row[6] { items[i].padding3 = *v as i32; }
        if let CellValue::Integer(v) = &row[7] { items[i].padding4 = TriStateFlag::from(*v as i32); }
        if let CellValue::Integer(v) = &row[8] { items[i].event_id = *v as i32; }
    }
    MonsterRef::save_file(&items, path)
}

// ── ExtraRef ──────────────────────────────────────────────────────────────

pub fn extra_ref_to_row(item: &ExtraRef) -> Row {
    vec![
        CellValue::Integer(item.id as i64),
        CellValue::String(item.name.clone()),
        CellValue::Integer(item.object_type as i64),
        CellValue::Integer(item.x_pos as i64),
        CellValue::Integer(item.y_pos as i64),
        CellValue::Integer(item.gold_amount as i64),
        CellValue::Integer(item.event_id as i64),
        CellValue::Integer(item.message_id as i64),
    ]
}

pub fn save_extra_refs(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = ExtraRef::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 8 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].id = *v as i32; }
        if let CellValue::String(s) = &row[1] { items[i].name = s.clone(); }
        if let CellValue::Integer(v) = &row[2] { items[i].object_type = ExtraObjectType::from(*v as u32); }
        if let CellValue::Integer(v) = &row[3] { items[i].x_pos = *v as i32; }
        if let CellValue::Integer(v) = &row[4] { items[i].y_pos = *v as i32; }
        if let CellValue::Integer(v) = &row[5] { items[i].gold_amount = *v as i32; }
        if let CellValue::Integer(v) = &row[6] { items[i].event_id = *v as i32; }
        if let CellValue::Integer(v) = &row[7] { items[i].message_id = *v as i32; }
    }
    ExtraRef::save_file(&items, path)
}

// ── INI/REF editors ──────────────────────────────────────────────────────

pub fn monster_ini_to_row(item: &MonsterIni) -> Row {
    vec![
        CellValue::Integer(item.id as i64),
        CellValue::String(item.name.clone().unwrap_or_default()),
        CellValue::String(item.sprite_filename.clone().unwrap_or_default()),
        CellValue::Integer(item.attack as i64),
        CellValue::Integer(item.hit as i64),
        CellValue::Integer(item.death as i64),
        CellValue::Integer(item.walking as i64),
        CellValue::Integer(item.casting_magic as i64),
    ]
}
pub fn save_monster_ini(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = MonsterIni::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 8 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].id = *v as i32; }
        if let CellValue::String(s) = &row[1] { items[i].name = Some(s.clone()); }
        if let CellValue::String(s) = &row[2] { items[i].sprite_filename = Some(s.clone()); }
        if let CellValue::Integer(v) = &row[3] { items[i].attack = *v as i32; }
        if let CellValue::Integer(v) = &row[4] { items[i].hit = *v as i32; }
        if let CellValue::Integer(v) = &row[5] { items[i].death = *v as i32; }
        if let CellValue::Integer(v) = &row[6] { items[i].walking = *v as i32; }
        if let CellValue::Integer(v) = &row[7] { items[i].casting_magic = *v as i32; }
    }
    MonsterIni::save_file(&items, path)
}

pub fn npc_ini_to_row(item: &NpcIni) -> Row {
    vec![
        CellValue::Integer(item.id as i64),
        CellValue::String(item.sprite_filename.clone().unwrap_or_default()),
        CellValue::String(item.description.clone()),
    ]
}
pub fn save_npc_ini(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = NpcIni::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 3 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].id = *v as i32; }
        if let CellValue::String(s) = &row[1] { items[i].sprite_filename = Some(s.clone()); }
        if let CellValue::String(s) = &row[2] { items[i].description = s.clone(); }
    }
    NpcIni::save_file(&items, path)
}

pub fn event_ini_to_row(item: &Event) -> Row {
    vec![
        CellValue::Integer(item.event_id as i64),
        CellValue::Integer(item.required_event_id as i64),
        CellValue::Integer(item.counter as i64),
        CellValue::String(item.event_filename.clone().unwrap_or_default()),
    ]
}
pub fn save_event_ini(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = Event::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 4 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].event_id = *v as i32; }
        if let CellValue::Integer(v) = &row[1] { items[i].required_event_id = *v as i32; }
        if let CellValue::Integer(v) = &row[2] { items[i].counter = *v as i32; }
        if let CellValue::String(s) = &row[3] { items[i].event_filename = Some(s.clone()); }
    }
    Event::save_file(&items, path)
}

pub fn extra_ini_to_row(item: &Extra) -> Row {
    vec![
        CellValue::Integer(item.id as i64),
        CellValue::String(item.sprite_filename.clone().unwrap_or_default()),
        CellValue::Integer(item.unknown as i64),
        CellValue::String(item.description.clone().unwrap_or_default()),
    ]
}
pub fn save_extra_ini(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = Extra::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 4 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].id = *v as i32; }
        if let CellValue::String(s) = &row[1] { items[i].sprite_filename = Some(s.clone()); }
        if let CellValue::Integer(v) = &row[2] { items[i].unknown = *v as i32; }
        if let CellValue::String(s) = &row[3] { items[i].description = Some(s.clone()); }
    }
    Extra::save_file(&items, path)
}

pub fn map_ini_to_row(item: &MapIni) -> Row {
    vec![
        CellValue::Integer(item.id as i64),
        CellValue::Integer(item.event_id_on_camera_move as i64),
        CellValue::Integer(item.start_pos_x as i64),
        CellValue::Integer(item.start_pos_y as i64),
        CellValue::Integer(item.map_id as i64),
        CellValue::String(item.monsters_filename.clone().unwrap_or_default()),
        CellValue::String(item.npc_filename.clone().unwrap_or_default()),
        CellValue::String(item.extra_filename.clone().unwrap_or_default()),
        CellValue::Integer(item.cd_music_track_number as i64),
    ]
}
pub fn save_map_ini(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = MapIni::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 9 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].id = *v as i32; }
        if let CellValue::Integer(v) = &row[1] { items[i].event_id_on_camera_move = *v as i32; }
        if let CellValue::Integer(v) = &row[2] { items[i].start_pos_x = *v as i32; }
        if let CellValue::Integer(v) = &row[3] { items[i].start_pos_y = *v as i32; }
        if let CellValue::Integer(v) = &row[4] { items[i].map_id = *v as i32; }
        if let CellValue::String(s) = &row[5] { items[i].monsters_filename = Some(s.clone()); }
        if let CellValue::String(s) = &row[6] { items[i].npc_filename = Some(s.clone()); }
        if let CellValue::String(s) = &row[7] { items[i].extra_filename = Some(s.clone()); }
        if let CellValue::Integer(v) = &row[8] { items[i].cd_music_track_number = *v as i32; }
    }
    MapIni::save_file(&items, path)
}

pub fn wave_ini_to_row(item: &WaveIni) -> Row {
    vec![
        CellValue::Integer(item.id as i64),
        CellValue::String(item.snf_filename.clone().unwrap_or_default()),
        CellValue::String(item.unknown_flag.clone().unwrap_or_default()),
    ]
}
pub fn save_wave_ini(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = WaveIni::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 3 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].id = *v as i32; }
        if let CellValue::String(s) = &row[1] { items[i].snf_filename = Some(s.clone()); }
        if let CellValue::String(s) = &row[2] { items[i].unknown_flag = Some(s.clone()); }
    }
    WaveIni::save_file(&items, path)
}

pub fn all_map_ini_to_row(item: &Map) -> Row {
    vec![
        CellValue::Integer(item.id as i64),
        CellValue::String(item.map_filename.clone()),
        CellValue::String(item.map_name.clone()),
        CellValue::String(item.pgp_filename.clone().unwrap_or_default()),
        CellValue::String(item.dlg_filename.clone().unwrap_or_default()),
    ]
}
pub fn save_all_map_ini(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = Map::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 5 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].id = *v as i32; }
        if let CellValue::String(s) = &row[1] { items[i].map_filename = s.clone(); }
        if let CellValue::String(s) = &row[2] { items[i].map_name = s.clone(); }
        if let CellValue::String(s) = &row[3] { items[i].pgp_filename = Some(s.clone()); }
        if let CellValue::String(s) = &row[4] { items[i].dlg_filename = Some(s.clone()); }
    }
    Map::save_file(&items, path)
}

pub fn npc_ref_to_row(item: &NPC) -> Row {
    vec![
        CellValue::Integer(item.id as i64),
        CellValue::Integer(item.npc_id as i64),
        CellValue::String(item.name.clone()),
        CellValue::String(item.description.clone()),
    ]
}
pub fn save_npc_refs(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = NPC::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 4 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].id = *v as i32; }
        if let CellValue::Integer(v) = &row[1] { items[i].npc_id = *v as i32; }
        if let CellValue::String(s) = &row[2] { items[i].name = s.clone(); }
        if let CellValue::String(s) = &row[3] { items[i].description = s.clone(); }
    }
    NPC::save_file(&items, path)
}

pub fn party_ref_to_row(item: &PartyRef) -> Row {
    vec![
        CellValue::Integer(item.id as i64),
        CellValue::String(item.full_name.clone().unwrap_or_default()),
        CellValue::String(item.job_name.clone().unwrap_or_default()),
        CellValue::Integer(item.root_map_id as i64),
        CellValue::Integer(item.npc_id as i64),
    ]
}
pub fn save_party_refs(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = PartyRef::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 5 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].id = *v as i32; }
        if let CellValue::String(s) = &row[1] { items[i].full_name = Some(s.clone()); }
        if let CellValue::String(s) = &row[2] { items[i].job_name = Some(s.clone()); }
        if let CellValue::Integer(v) = &row[3] { items[i].root_map_id = *v as i32; }
        if let CellValue::Integer(v) = &row[4] { items[i].npc_id = *v as i32; }
    }
    PartyRef::save_file(&items, path)
}

pub fn draw_item_to_row(item: &DrawItem) -> Row {
    vec![
        CellValue::Integer(item.map_id as i64),
        CellValue::Integer(item.x_coord as i64),
        CellValue::Integer(item.y_coord as i64),
        CellValue::Integer(item.item.raw() as i64),
    ]
}
pub fn save_draw_items(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = DrawItem::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 4 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].map_id = *v as i32; }
        if let CellValue::Integer(v) = &row[1] { items[i].x_coord = *v as i32; }
        if let CellValue::Integer(v) = &row[2] { items[i].y_coord = *v as i32; }
        if let CellValue::Integer(v) = &row[3] { items[i].item = (*v as i32).into(); }
    }
    DrawItem::save_file(&items, path)
}

pub fn store_to_row(item: &Store) -> Row {
    vec![
        CellValue::String(item.store_name.clone()),
        CellValue::Integer(item.inn_night_cost as i64),
        CellValue::Integer(item.some_unknown_number as i64),
        CellValue::Integer(item.products.len() as i64),
    ]
}
pub fn save_stores(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = Store::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 4 { continue; }
        if let CellValue::String(s) = &row[0] { items[i].store_name = s.clone(); }
        if let CellValue::Integer(v) = &row[1] { items[i].inn_night_cost = *v as i32; }
        if let CellValue::Integer(v) = &row[2] { items[i].some_unknown_number = *v as i16; }
    }
    Store::save_file(&items, path)
}

// ── Dialogue editors (pipe-delimited flat lists, spreadsheet-edit able) ───

pub fn dialogue_script_to_row(item: &DialogueScript) -> Row {
    vec![
        CellValue::Integer(item.id as i64),
        CellValue::Integer(item.required_event_id.unwrap_or(0) as i64),
        CellValue::Integer(item.next_dialog_to_check.unwrap_or(0) as i64),
        CellValue::Integer(item.dialog_type.map(|v| v as i64).unwrap_or(0)),
        CellValue::Integer(item.dialog_owner.map(|v| v as i64).unwrap_or(0)),
        CellValue::Integer(item.dialog_id.unwrap_or(0) as i64),
        CellValue::Integer(item.next_dialog_id1.unwrap_or(0) as i64),
        CellValue::Integer(item.next_dialog_id2.unwrap_or(0) as i64),
        CellValue::Integer(item.next_dialog_id3.unwrap_or(0) as i64),
        CellValue::Integer(item.triggered_event_id.unwrap_or(0) as i64),
    ]
}
pub fn save_dialogue_scripts(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = DialogueScript::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 10 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].id = *v as i32; }
        if let CellValue::Integer(v) = &row[1] { items[i].required_event_id = Some(*v as i32); }
        if let CellValue::Integer(v) = &row[2] { items[i].next_dialog_to_check = Some(*v as i32); }
        if let CellValue::Integer(v) = &row[3] { items[i].dialog_type = Some(DialogType::from(*v as i32)); }
        if let CellValue::Integer(v) = &row[4] { items[i].dialog_owner = Some(DialogOwner::from(*v as i32)); }
        if let CellValue::Integer(v) = &row[5] { items[i].dialog_id = Some(*v as i32); }
        if let CellValue::Integer(v) = &row[6] { items[i].next_dialog_id1 = Some(*v as i32); }
        if let CellValue::Integer(v) = &row[7] { items[i].next_dialog_id2 = Some(*v as i32); }
        if let CellValue::Integer(v) = &row[8] { items[i].next_dialog_id3 = Some(*v as i32); }
        if let CellValue::Integer(v) = &row[9] { items[i].triggered_event_id = Some(*v as i32); }
    }
    DialogueScript::save_file(&items, path)
}

pub fn dialogue_paragraph_to_row(item: &DialogueParagraph) -> Row {
    vec![
        CellValue::Integer(item.id as i64),
        CellValue::String(item.text.clone()),
        CellValue::String(item.comment.clone()),
        CellValue::Integer(item.param1 as i64),
        CellValue::Integer(item.wave_ini_entry_id as i64),
    ]
}
pub fn save_dialogue_paragraphs(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = DialogueParagraph::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 5 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].id = *v as i32; }
        if let CellValue::String(s) = &row[1] { items[i].text = s.clone(); }
        if let CellValue::String(s) = &row[2] { items[i].comment = s.clone(); }
        if let CellValue::Integer(v) = &row[3] { items[i].param1 = *v as i32; }
        if let CellValue::Integer(v) = &row[4] { items[i].wave_ini_entry_id = *v as i32; }
    }
    DialogueParagraph::save_file(&items, path)
}

// ── MessageScr ─────────────────────────────────────────────────────────

pub fn message_scr_to_row(item: &Message) -> Row {
    vec![
        CellValue::Integer(item.id as i64),
        CellValue::String(item.line1.clone().unwrap_or_default()),
        CellValue::String(item.line2.clone().unwrap_or_default()),
        CellValue::String(item.line3.clone().unwrap_or_default()),
    ]
}

pub fn save_message_scr(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = Message::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 4 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].id = *v as i32; }
        if let CellValue::String(s) = &row[1] { items[i].line1 = if s.is_empty() || s == "null" { None } else { Some(s.clone()) }; }
        if let CellValue::String(s) = &row[2] { items[i].line2 = if s.is_empty() || s == "null" { None } else { Some(s.clone()) }; }
        if let CellValue::String(s) = &row[3] { items[i].line3 = if s.is_empty() || s == "null" { None } else { Some(s.clone()) }; }
    }
    Message::save_file(&items, path)
}

// ── QuestScr ──────────────────────────────────────────────────────────

pub fn quest_scr_to_row(item: &Quest) -> Row {
    vec![
        CellValue::Integer(item.id as i64),
        CellValue::Integer(item.type_id as i64),
        CellValue::String(item.title.clone()),
        CellValue::String(item.description.clone()),
    ]
}

pub fn save_quest_scr(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = Quest::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 4 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].id = *v as i32; }
        if let CellValue::Integer(v) = &row[1] { items[i].type_id = *v as i32; }
        if let CellValue::String(s) = &row[2] { items[i].title = s.clone(); }
        if let CellValue::String(s) = &row[3] { items[i].description = s.clone(); }
    }
    Quest::save_file(&items, path)
}

// ── EventNpcRef ────────────────────────────────────────────────────────

pub fn event_npc_ref_to_row(item: &EventNpcRef) -> Row {
    vec![
        CellValue::Integer(item.id as i64),
        CellValue::Integer(item.event_id as i64),
        CellValue::String(item.name.clone()),
    ]
}

pub fn save_event_npc_refs(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = EventNpcRef::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    for i in 0..items.len().min(rows.len()) {
        let row = &rows[i]; if row.len() < 3 { continue; }
        if let CellValue::Integer(v) = &row[0] { items[i].id = *v as i32; }
        if let CellValue::Integer(v) = &row[1] { items[i].event_id = *v as i32; }
        if let CellValue::String(s) = &row[2] { items[i].name = s.clone(); }
    }
    EventNpcRef::save_file(&items, path)
}

// ── PartyLevelNpc ──────────────────────────────────────────────────────

pub fn party_level_npc_to_row(item: &PartyLevelNpc) -> Row {
    vec![
        CellValue::Integer(item.npc_index as i64),
        CellValue::Integer(item.records.len() as i64),
    ]
}

pub fn save_party_level_npcs(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = PartyLevelNpc::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    // PartyLevelNpc save only supports NPC index tracking; records are complex sub-structures
    // For now, just save the original data back (preserving unsupported edits to record count won't work)
    PartyLevelNpc::save_file(&items, path)
}

/// Flatten PartyLevelRecord entries into individual rows for the DbLevel sub-view.
pub fn party_level_db_level_to_rows(items: &[PartyLevelNpc]) -> Vec<Row> {
    let mut rows = Vec::new();
    for npc in items {
        for rec in &npc.records {
            rows.push(vec![
                CellValue::Integer(npc.npc_index as i64),
                CellValue::Integer(rec.level as i64),
                CellValue::Integer(rec.strength as i64),
                CellValue::Integer(rec.constitution as i64),
                CellValue::Integer(rec.wisdom as i64),
                CellValue::Integer(rec.health_points as i64),
                CellValue::Integer(rec.mana_points as i64),
                CellValue::Integer(rec.agility as i64),
                CellValue::Integer(rec.attack as i64),
                CellValue::Integer(rec.mana_recharge as i64),
                CellValue::Integer(rec.defense as i64),
            ])
        }
    }
    rows
}

pub fn save_party_level_db_levels(rows: &[Row], original_data: &[u8], path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Cursor;
    let mut items = PartyLevelNpc::parse(&mut Cursor::new(original_data), original_data.len() as u64).unwrap_or_default();
    // Apply flattened edits back to the nested records
    let mut row_idx = 0;
    for npc in &mut items {
        for rec in &mut npc.records {
            if row_idx >= rows.len() { break; }
            let row = &rows[row_idx];
            row_idx += 1;
            if row.len() < 11 { continue; }
            // Skip npc_index (0) and level (1) — these are structural identifiers
            if let CellValue::Integer(v) = &row[2] { rec.strength = *v as u32; }
            if let CellValue::Integer(v) = &row[3] { rec.constitution = *v as u32; }
            if let CellValue::Integer(v) = &row[4] { rec.wisdom = *v as u32; }
            if let CellValue::Integer(v) = &row[5] { rec.health_points = *v as u16; }
            if let CellValue::Integer(v) = &row[6] { rec.mana_points = *v as u16; }
            if let CellValue::Integer(v) = &row[7] { rec.agility = *v as u32; }
            if let CellValue::Integer(v) = &row[8] { rec.attack = *v as u32; }
            if let CellValue::Integer(v) = &row[9] { rec.mana_recharge = *v as u32; }
            if let CellValue::Integer(v) = &row[10] { rec.defense = *v as u16; }
        }
    }
    PartyLevelNpc::save_file(&items, path)
}

pub fn editor_type_index(editor_type: EditorTypeId) -> usize {
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
        EditorTypeId::DialogueScript => 26,
        EditorTypeId::DialogueParagraph => 27,
        EditorTypeId::MessageScr => 28,
        EditorTypeId::QuestScr => 29,
    }
}
