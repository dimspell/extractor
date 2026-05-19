// Editor message router
use crate::app::App;
use crate::editors::{
    all_map_ini, chdata, chest, dialogue_paragraph, dialogue_script, draw_item, edit_item,
    event_ini, event_item, event_npc_ref, event_scr, extra_ini, extra_ref, heal_item,
    localization_manager, magic, map_editor, map_ini, message_scr, misc_item, mod_packager,
    monster, monster_ini, monster_ref, npc_ini, npc_ref, party_ini, party_level_db, party_ref,
    quest_scr, snf_editor, sprite_browser, store, tileset, wave_ini, weapon,
};
use crate::message::editor::EditorMessage;
use crate::message::{Message, MessageExt};
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
        EditorMessage::EventScr(msg) => event_scr::handle(msg, app),
        EditorMessage::WaveIni(msg) => wave_ini::handle(msg, app),
        EditorMessage::ChData(msg) => chdata::handle(msg, app),
        EditorMessage::Chest(msg) => chest::handle(msg, app),
        EditorMessage::MapEditor(msg) => map_editor::handle(msg, app),
        EditorMessage::Tileset(msg) => tileset::handle(msg, app),
        EditorMessage::Snf(msg) => snf_editor::handle(msg, app),
        EditorMessage::ModPackager(msg) => mod_packager::handle(msg, app),
        EditorMessage::Localization(msg) => localization_manager::handle(msg, app),
        EditorMessage::HexEditor(msg) => {
            let tab_id = app
                .state
                .workspace
                .active()
                .map(|t| t.id)
                .unwrap_or(usize::MAX);
            let Some(state) = app.state.hex_editors.get_mut(&tab_id) else {
                return Task::none();
            };
            let has_dirty = state.provider.dirty_count() > 0;
            let has_session = app.state.recording.is_some();
            let has_game = app.state.workspace.game_path.is_some();
            let in_game_dir = app
                .state
                .workspace
                .game_path
                .as_ref()
                .map(|gp| state.path.starts_with(gp))
                .unwrap_or(false);
            let can_save = has_dirty && has_session && has_game && in_game_dir;
            let save_label = match &app.state.recording {
                Some(s) => format!("Save into `{}`", s.mod_slug),
                None => "Save into recording".to_string(),
            };
            let save_hint = if !has_session {
                "  ·  no recording active".to_string()
            } else if !has_game {
                "  ·  set a game directory".to_string()
            } else if !in_game_dir {
                "  ·  file is outside the game directory".to_string()
            } else if !has_dirty {
                "  ·  no edits to save".to_string()
            } else {
                String::new()
            };
            let config = hexedit::HexEditorConfig {
                on_save: crate::editors::mod_packager::hex_save::build_save_callback(
                    &app.state.recording,
                    &app.state.workspace.game_path,
                ),
                save_label,
                can_save,
                save_hint,
                extra_entries: Vec::new(),
            };
            hexedit::update(state, &config, msg).map(Message::hex_editor)
        }
    }
}

// Common editor framework
mod common;
pub mod tab;
