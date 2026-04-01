// References module – Game data file parsers
//
// This module contains parsers for all Dispel game reference files.
// These files store game configuration, databases, and metadata in various
// binary and text formats that define game behavior, assets, and content.

// ===========================================================================
// DISPEL GAME REFERENCE FILE FORMATS
// ===========================================================================
//
// ASCII Overview of Reference File Types:
//
// +--------------------------------------+
// | INI FILES (Configuration)            |
// +--------------------------------------+
// | AllMap.ini    – Master map list      |
// | Map.ini       – Map properties       |
// | Extra.ini     – Interactive objects  |
// | Event.ini     – Script/event mappings|
// | Monster.ini   – Monster visual refs  |
// | Npc.ini       – NPC visual refs      |
// | Wave.ini      – Audio/SNF references |
// +--------------------------------------+
// | DB FILES (Databases)                 |
// +--------------------------------------+
// | weaponItem.db – Armor & weapons      |
// | Monster.db    – Monster stats       |
// | MultiMagic.db – Spells              |
// | STORE.DB      – Shop inventories    |
// | MiscItem.db   – Generic items       |
// | HealItem.db   – Consumables         |
// | EventItem.db  – Quest items         |
// | EditItem.db   – Modifiable items    |
// | PartyLevel.db – EXP tables          |
// | PrtIni.db     – Party NPC metadata  |
// | Magic.db      – Magic spells        |
// | ChData.db     – Character data      |
// +--------------------------------------+
// | REF FILES (References)               |
// +--------------------------------------+
// | PartyRef.ref  – Character definitions|
// | DRAWITEM.ref  – Map placements       |
// | *.pgp  – Dialogue text dialogue      |
// | *.dlg   – Converstaion scripts       |
// | Npccat*.ref  – NPC placements       |
// | Mondun*.ref   – Monster placements  |
// | Extdun*.ref  – Special object plac.  |
// | Eventnpc.ref  – Event-specific NPCs  |
// +--------------------------------------+
// | SCR FILES (Scripts)                  |
// +--------------------------------------+
// | Quest.scr    – Quest definitions    |
// | Message.scr  – Game messages        |
// +--------------------------------------+
//
// FILE ORGANIZATION:
// - INI files: Configuration and mapping data
// - DB files:  Game databases with structured records
// - REF files: Reference data linking IDs to assets
// - SCR files: Script and text content
// - PGP files: Dialogue and text packaging
// - DLG files: Dialogue scripts
//
// ===========================================================================

pub mod all_map_ini;
pub mod chdata_db;
pub mod dialog;
pub mod dialogue_text;
pub mod draw_item;
pub mod edit_item_db;
pub mod enums;
pub mod event_ini;
pub mod event_item_db;
pub mod event_npc_ref;
pub mod extra_ini;
pub mod extra_ref;
pub mod extractor;
pub mod heal_item_db;
pub mod magic_db;
pub mod map_ini;
pub mod message_scr;
pub mod misc_item_db;
pub mod monster_db;
pub mod monster_ini;
pub mod monster_ref;
pub mod npc_ini;
pub mod npc_ref;
pub mod party_ini_db;
pub mod party_level_db;
pub mod party_ref;
pub mod quest_scr;
pub mod store_db;
pub mod wave_ini;
pub mod weapons_db;
