pub mod allmapini;
pub mod chdata;
pub mod chest;
pub mod dialog;
pub mod dialoguetext;
pub mod drawitem;
pub mod edititem;
pub mod eventini;
pub mod eventitem;
pub mod eventnpcref;
pub mod extraini;
pub mod extraref;
pub mod healitem;
pub mod magic;
pub mod map_editor;
pub mod mapini;
pub mod messagescr;
pub mod miscitem;
pub mod monster;
pub mod monsterini;
pub mod monsterref;
pub mod npcini;
pub mod npcref;
pub mod partyini;
pub mod partyleveldb;
pub mod partyref;
pub mod questscr;
pub mod snf;
pub mod spritebrowser;
pub mod store;
pub mod tileset;
pub mod waveini;
pub mod weapon;

use allmapini::AllMapIniEditorMessage;
use chdata::ChDataEditorMessage;
use chest::ChestEditorMessage;
use dialog::DialogEditorMessage;
use dialoguetext::DialogueTextEditorMessage;
use drawitem::DrawItemEditorMessage;
use edititem::EditItemEditorMessage;
use eventini::EventIniEditorMessage;
use eventitem::EventItemEditorMessage;
use eventnpcref::EventNpcRefEditorMessage;
use extraini::ExtraIniEditorMessage;
use extraref::ExtraRefEditorMessage;
use healitem::HealItemEditorMessage;
use magic::MagicEditorMessage;
pub use map_editor::{MapEditorMessage, MapLayer};
use mapini::MapIniEditorMessage;
use messagescr::MessageScrEditorMessage;
use miscitem::MiscItemEditorMessage;
use monster::MonsterEditorMessage;
use monsterini::MonsterIniEditorMessage;
use monsterref::MonsterRefEditorMessage;
use npcini::NpcIniEditorMessage;
use npcref::NpcRefEditorMessage;
use partyini::PartyIniEditorMessage;
use partyleveldb::PartyLevelDbEditorMessage;
use partyref::PartyRefEditorMessage;
use questscr::QuestScrEditorMessage;
use snf::SnfEditorMessage;
pub use spritebrowser::SpriteViewerMessage;
use store::StoreEditorMessage;
use tileset::TilesetEditorMessage;
use waveini::WaveIniEditorMessage;
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
    Dialog(DialogEditorMessage),
    DialogueText(DialogueTextEditorMessage),
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
}
