// Editor message router
use crate::app::App;
use crate::message::editor::EditorMessage;
use iced::Task;

pub fn handle(message: EditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        EditorMessage::Weapon(msg) => weapon::handle(msg, app),
        EditorMessage::Monster(msg) => monster::handle(msg, app),
        EditorMessage::HealItem(msg) => heal_item::handle(msg, app),
        EditorMessage::MiscItem(msg) => misc_item::handle(msg, app),
        EditorMessage::EditItem(msg) => edit_item::handle(msg, app),
        EditorMessage::EventItem(msg) => event_item::handle(msg, app),
        EditorMessage::NpcIni(msg) => npc_ini::handle(msg, app),
        EditorMessage::MonsterIni(msg) => monster_ini::handle(msg, app),
        EditorMessage::Magic(msg) => magic::handle(msg, app),
        EditorMessage::Store(msg) => store::handle(msg, app),
        EditorMessage::PartyRef(msg) => party_ref::handle(msg, app),
        EditorMessage::PartyIni(msg) => party_ini::handle(msg, app),
        EditorMessage::SpriteViewer(msg) => sprite_browser::handle(msg, app),
        EditorMessage::MonsterRef(msg) => monster_ref::handle(msg, app),
        EditorMessage::AllMapIni(msg) => all_map_ini::handle(msg, app),
        EditorMessage::DialogueScript(msg) => dialogue_script::handle(msg, app),
        EditorMessage::DialogueParagraph(msg) => dialogue_paragraph::handle(msg, app),
        EditorMessage::DrawItem(msg) => draw_item::handle(msg, app),
        EditorMessage::EventIni(msg) => event_ini::handle(msg, app),
        EditorMessage::EventNpcRef(msg) => event_npc_ref::handle(msg, app),
        EditorMessage::ExtraIni(msg) => extra_ini::handle(msg, app),
        EditorMessage::ExtraRef(msg) => extra_ref::handle(msg, app),
        EditorMessage::MapIni(msg) => map_ini::handle(msg, app),
        EditorMessage::MessageScr(msg) => message_scr::handle(msg, app),
        EditorMessage::NpcRef(msg) => npc_ref::handle(msg, app),
        EditorMessage::PartyLevelDb(msg) => party_level_db::handle(msg, app),
        EditorMessage::QuestScr(msg) => quest_scr::handle(msg, app),
        EditorMessage::WaveIni(msg) => wave_ini::handle(msg, app),
        EditorMessage::ChData(msg) => chdata::handle(msg, app),
        EditorMessage::Chest(msg) => chest::handle(msg, app),
        EditorMessage::MapEditor(msg) => map_editor::handle(msg, app),
        EditorMessage::Tileset(msg) => tileset::handle(msg, app),
        EditorMessage::Snf(msg) => snf_editor::handle(msg, app),
    }
}

// Common editor framework
mod common;

// Editor-specific handler modules
mod all_map_ini;
mod chdata;
pub mod chest;
mod dialogue_paragraph;
mod dialogue_script;
mod draw_item;
mod edit_item;
mod event_ini;
mod event_item;
mod event_npc_ref;
mod extra_ini;
mod extra_ref;
mod heal_item;
mod magic;
mod map_editor;
mod map_ini;
mod message_scr;
mod misc_item;
mod monster;
mod monster_ini;
mod monster_ref;
mod npc_ini;
mod npc_ref;
mod party_ini;
mod party_level_db;
mod party_ref;
mod quest_scr;
mod snf_editor;
mod sprite_browser;
mod store;
mod tileset;
mod wave_ini;
mod weapon;

// Re-export common framework for use in other modules
// Macros are automatically available at crate root when using #[macro_export]
