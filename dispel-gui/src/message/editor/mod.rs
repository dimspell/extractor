pub mod all_map_ini;
pub mod localization;
pub mod mod_packager;
pub mod chdata;
pub mod chest;
pub mod dialogue_paragraph;
pub mod dialogue_script;
pub mod draw_item;
pub mod edit_item;
pub mod event_ini;
pub mod event_item;
pub mod event_npc_ref;
pub mod extra_ini;
pub mod extra_ref;
pub mod heal_item;
pub mod magic;
pub mod map_editor;
pub mod map_ini;
pub mod message_scr;
pub mod misc_item;
pub mod monster_db;
pub mod monster_ini;
pub mod monster_ref;
pub mod npc_ini;
pub mod npc_ref;
pub mod party_ini;
pub mod party_level_db;
pub mod party_ref;
pub mod quest_scr;
pub mod snf;
pub mod spritebrowser;
pub mod store;
pub mod tileset;
pub mod wave_ini;
pub mod weapon;

use all_map_ini::AllMapIniEditorMessage;
use localization::LocalizationMessage;
use mod_packager::ModPackagerMessage;
use chdata::ChDataEditorMessage;
use chest::ChestEditorMessage;
use dialogue_paragraph::DialogueParagraphEditorMessage;
use dialogue_script::DialogueScriptEditorMessage;
use draw_item::DrawItemEditorMessage;
use edit_item::EditItemEditorMessage;
use event_ini::EventIniEditorMessage;
use event_item::EventItemEditorMessage;
use event_npc_ref::EventNpcRefEditorMessage;
use extra_ini::ExtraIniEditorMessage;
use extra_ref::ExtraRefEditorMessage;
use heal_item::HealItemEditorMessage;
use magic::MagicEditorMessage;
pub use map_editor::{MapEditorMessage, MapLayer};
use map_ini::MapIniEditorMessage;
use message_scr::MessageScrEditorMessage;
use misc_item::MiscItemEditorMessage;
use monster_db::MonsterEditorMessage;
use monster_ini::MonsterIniEditorMessage;
use monster_ref::MonsterRefEditorMessage;
use npc_ini::NpcIniEditorMessage;
use npc_ref::NpcRefEditorMessage;
use party_ini::PartyIniEditorMessage;
use party_level_db::PartyLevelDbEditorMessage;
use party_ref::PartyRefEditorMessage;
use quest_scr::QuestScrEditorMessage;
use snf::SnfEditorMessage;
pub use spritebrowser::SpriteViewerMessage;
use store::StoreEditorMessage;
use tileset::TilesetEditorMessage;
use wave_ini::WaveIniEditorMessage;
use weapon::WeaponEditorMessage;

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
