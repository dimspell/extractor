//! Round-trip integrity tests for dispel-gui
//!
//! Run with: cargo test -p dispel-gui round_trip -- --nocapture

mod round_trip_utils;

#[path = "round_trip/all_map_ini.rs"]
mod all_map_ini;
#[path = "round_trip/chdata_db.rs"]
mod chdata_db;
#[path = "round_trip/dialog.rs"]
mod dialog;
#[path = "round_trip/dialogue_text.rs"]
mod dialogue_text;
#[path = "round_trip/draw_item.rs"]
mod draw_item;
#[path = "round_trip/edit_item_db.rs"]
mod edit_item_db;
#[path = "round_trip/event_ini.rs"]
mod event_ini;
#[path = "round_trip/event_item_db.rs"]
mod event_item_db;
#[path = "round_trip/event_npc_ref.rs"]
mod event_npc_ref;
#[path = "round_trip/extra_ini.rs"]
mod extra_ini;
#[path = "round_trip/extra_ref.rs"]
mod extra_ref;
#[path = "round_trip/heal_item_db.rs"]
mod heal_item_db;
#[path = "round_trip/magic_db.rs"]
mod magic_db;
#[path = "round_trip/map_ini.rs"]
mod map_ini;
#[path = "round_trip/message_scr.rs"]
mod message_scr;
#[path = "round_trip/misc_item_db.rs"]
mod misc_item_db;
#[path = "round_trip/monster_db.rs"]
mod monster_db;
#[path = "round_trip/monster_ini.rs"]
mod monster_ini;
#[path = "round_trip/monster_ref.rs"]
mod monster_ref;
#[path = "round_trip/npc_ini.rs"]
mod npc_ini;
#[path = "round_trip/npc_ref.rs"]
mod npc_ref;
#[path = "round_trip/party_ini_db.rs"]
mod party_ini_db;
#[path = "round_trip/party_level_db.rs"]
mod party_level_db;
#[path = "round_trip/party_ref.rs"]
mod party_ref;
#[path = "round_trip/quest_scr.rs"]
mod quest_scr;
#[path = "round_trip/store_db.rs"]
mod store_db;
#[path = "round_trip/wave_ini.rs"]
mod wave_ini;
#[path = "round_trip/weapons_db.rs"]
mod weapons_db;
