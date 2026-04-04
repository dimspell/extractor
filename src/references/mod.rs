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
mod all_map_ini_editor;
pub mod chdata_db;
mod chdata_editor;
pub mod dialog;
mod dialog_editor;
pub mod dialogue_text;
mod dialogue_text_editor;
pub mod draw_item;
mod draw_item_editor;
pub mod edit_item_db;
mod edit_item_editor;
pub mod editable;
pub mod enums;
pub mod event_ini;
mod event_ini_editor;
pub mod event_item_db;
mod event_item_editor;
pub mod event_npc_ref;
mod event_npc_ref_editor;
pub mod extra_ini;
mod extra_ini_editor;
pub mod extra_ref;
mod extra_ref_editor;
pub mod extractor;
pub mod heal_item_db;
mod heal_item_editor;
pub mod magic_db;
mod magic_editor;
pub mod map_ini;
mod map_ini_editor;
pub mod message_scr;
mod message_scr_editor;
pub mod misc_item_db;
mod misc_item_editor;
pub mod monster_db;
pub mod monster_ini;
pub mod monster_ref;
mod monster_ref_editor;
pub mod npc_ini;
pub mod npc_ref;
mod npc_ref_editor;
pub mod party_ini_db;
mod party_ini_db_editor;
pub mod party_level_db;
mod party_level_db_editor;
pub mod party_ref;
mod party_ref_editor;
pub mod quest_scr;
mod quest_scr_editor;
pub mod store_db;
mod store_db_editor;
pub mod wave_ini;
mod wave_ini_editor;
mod weapon_editor;
pub mod weapons_db;
