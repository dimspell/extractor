// Editor message router
use crate::app::App;
use crate::editors::{
    all_map_ini, chdata, chest, dialogue_paragraph, dialogue_script, draw_item, edit_item,
    event_ini, event_item, event_npc_ref, extra_ini, extra_ref, heal_item, hex_editor,
    localization_manager, magic, map_editor, map_ini, message_scr, misc_item, mod_packager,
    monster, monster_ini, monster_ref, npc_ini, npc_ref, party_ini, party_level_db, party_ref,
    quest_scr, snf_editor, sprite_browser, store, tileset, wave_ini, weapon,
};
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
        EditorMessage::ModPackager(msg) => mod_packager::handle(msg, app),
        EditorMessage::Localization(msg) => localization_manager::handle(msg, app),
        EditorMessage::HexEditor(msg) => hex_editor::handle(msg, app),
    }
}

// Common editor framework
mod common;
pub mod tab;
