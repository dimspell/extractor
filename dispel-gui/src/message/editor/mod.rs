//! Aggregation point for all editor messages.
//!
//! The actual message types live in their editor slices under `crate::editors::*`.
//! This module only assembles the top-level `EditorMessage` enum.

use crate::editors::all_map_ini::AllMapIniEditorMessage;
use crate::editors::chdata::ChDataEditorMessage;
use crate::editors::chest::ChestEditorMessage;
use crate::editors::dialogue_paragraph::DialogueParagraphEditorMessage;
use crate::editors::dialogue_script::DialogueScriptEditorMessage;
use crate::editors::draw_item::DrawItemEditorMessage;
use crate::editors::edit_item::EditItemEditorMessage;
use crate::editors::event_ini::EventIniEditorMessage;
use crate::editors::event_item::EventItemEditorMessage;
use crate::editors::event_npc_ref::EventNpcRefEditorMessage;
use crate::editors::extra_ini::ExtraIniEditorMessage;
use crate::editors::extra_ref::ExtraRefEditorMessage;
use crate::editors::heal_item::HealItemEditorMessage;
use crate::editors::localization_manager::LocalizationMessage;
use crate::editors::magic::MagicEditorMessage;
use crate::editors::map_editor::MapEditorMessage;
use crate::editors::map_ini::MapIniEditorMessage;
use crate::editors::message_scr::MessageScrEditorMessage;
use crate::editors::misc_item::MiscItemEditorMessage;
use crate::editors::mod_packager::ModPackagerMessage;
use crate::editors::monster::MonsterEditorMessage;
use crate::editors::monster_ini::MonsterIniEditorMessage;
use crate::editors::monster_ref::MonsterRefEditorMessage;
use crate::editors::npc_ini::NpcIniEditorMessage;
use crate::editors::npc_ref::NpcRefEditorMessage;
use crate::editors::party_ini::PartyIniEditorMessage;
use crate::editors::party_level_db::PartyLevelDbEditorMessage;
use crate::editors::party_ref::PartyRefEditorMessage;
use crate::editors::quest_scr::QuestScrEditorMessage;
use crate::editors::snf_editor::SnfEditorMessage;
use crate::editors::sprite_browser::SpriteViewerMessage;
use crate::editors::store::StoreEditorMessage;
use crate::editors::tileset::TilesetEditorMessage;
use crate::editors::wave_ini::WaveIniEditorMessage;
use crate::editors::weapon::WeaponEditorMessage;

#[derive(Debug, Clone)]
pub enum EditorMessage {
    Weapon(WeaponEditorMessage),
    Monster(MonsterEditorMessage),
    Chest(ChestEditorMessage),
    HealItem(HealItemEditorMessage),
    MiscItem(MiscItemEditorMessage),
    EditItem(EditItemEditorMessage),
    EventItem(EventItemEditorMessage),
    NpcIni(NpcIniEditorMessage),
    MonsterIni(MonsterIniEditorMessage),
    Magic(MagicEditorMessage),
    Store(StoreEditorMessage),
    PartyRef(PartyRefEditorMessage),
    PartyIni(PartyIniEditorMessage),
    SpriteViewer(SpriteViewerMessage),
    MonsterRef(MonsterRefEditorMessage),
    AllMapIni(AllMapIniEditorMessage),
    DialogueScript(DialogueScriptEditorMessage),
    DialogueParagraph(DialogueParagraphEditorMessage),
    DrawItem(DrawItemEditorMessage),
    EventIni(EventIniEditorMessage),
    EventNpcRef(EventNpcRefEditorMessage),
    ExtraIni(ExtraIniEditorMessage),
    ExtraRef(ExtraRefEditorMessage),
    MapIni(MapIniEditorMessage),
    MessageScr(MessageScrEditorMessage),
    NpcRef(NpcRefEditorMessage),
    PartyLevelDb(PartyLevelDbEditorMessage),
    QuestScr(QuestScrEditorMessage),
    WaveIni(WaveIniEditorMessage),
    ChData(ChDataEditorMessage),
    MapEditor(MapEditorMessage),
    Tileset(TilesetEditorMessage),
    Snf(SnfEditorMessage),
    ModPackager(ModPackagerMessage),
    Localization(LocalizationMessage),
}
