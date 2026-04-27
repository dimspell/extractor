//! Dispel Extractor Core Library
//!
//! This library provides parsers and data structures for Dispel game files.
//! It's used by both the CLI extractor and the GUI editor.

pub mod database;
pub mod localization;
pub mod map;
pub mod references;
pub mod snf;
pub mod sprite;

// Re-export key types for easy access
pub use references::{
    all_map_ini::Map,
    chdata_db::ChData,
    dialogue_paragraph::DialogueParagraph,
    dialogue_script::DialogueScript,
    draw_item::DrawItem,
    edit_item_db::EditItem,
    enums::*,
    event_ini::Event,
    event_item_db::EventItem,
    event_npc_ref::EventNpcRef,
    event_scr::save_event_scripts,
    event_scr::EventScript,
    extra_ini::Extra,
    extra_ref::ExtraRef,
    extractor::Extractor,
    heal_item_db::HealItem,
    magic_db::MagicSpell,
    map_ini::MapIni,
    message_scr::Message,
    misc_item_db::MiscItem,
    monster_db::Monster,
    monster_ini::MonsterIni,
    monster_ref::MonsterRef,
    npc_ini::NpcIni,
    npc_ref::NPC,
    party_ini_db::PartyIniNpc,
    party_level_db::{PartyLevelNpc, PartyLevelRecord},
    party_ref::PartyRef,
    quest_scr::Quest,
    store_db::Store,
    wave_ini::WaveIni,
    weapons_db::WeaponItem,
};
pub use localization::{
    export_csv, export_po, import_csv, import_po, truncate_to_fit, Localizable, TextEncoding,
    TextEntry, TruncationStatus,
};
