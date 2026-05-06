// Editor modules - vertical slices
// Each directory contains all code for one editor:
// - component.rs: EditableRecord trait impl
// - message.rs: message enum
// - state.rs: state type alias
// - update.rs: message handler
// - view.rs: view function
// - mod.rs: public API

pub mod all_map_ini;
pub mod chdata;
pub mod chest;
pub mod db_viewer;
pub mod dialogue_script;
pub mod dialogue_paragraph;
pub mod draw_item;
pub mod edit_item;
pub mod event_ini;
pub mod event_item;
pub mod event_npc_ref;
pub mod extra_ini;
pub mod extra_ref;
pub mod heal_item;
pub mod localization_manager;
pub mod magic;
pub mod map_editor;
pub mod map_ini;
pub mod message_scr;
pub mod misc_item;
pub mod mod_packager;
pub mod monster;
pub mod monster_ini;
pub mod monster_ref;
pub mod npc_ini;
pub mod npc_ref;
pub mod party_ini;
pub mod party_level_db;
pub mod party_ref;
pub mod quest_scr;
pub mod snf_editor;
pub mod sprite_browser;
pub mod store;
pub mod tileset;
pub mod wave_ini;
pub mod weapon;
