use crate::message::editor::weapon::WeaponEditorMessage;
use crate::message::EditorMessage;

define_message_ext! {
    weapon:         Weapon(crate::message::editor::weapon::WeaponEditorMessage),
    monster:        Monster(crate::message::editor::monster::MonsterEditorMessage),
    monster_ini:    MonsterIni(crate::message::editor::monsterini::MonsterIniEditorMessage),
    chest:          Chest(crate::message::editor::chest::ChestEditorMessage),
    heal_item:      HealItem(crate::message::editor::healitem::HealItemEditorMessage),
    misc_item:      MiscItem(crate::message::editor::miscitem::MiscItemEditorMessage),
    edit_item:      EditItem(crate::message::editor::edititem::EditItemEditorMessage),
    event_item:     EventItem(crate::message::editor::eventitem::EventItemEditorMessage),
    npc_ini:        NpcIni(crate::message::editor::npcini::NpcIniEditorMessage),
    magic:          Magic(crate::message::editor::magic::MagicEditorMessage),
    store:          Store(crate::message::editor::store::StoreEditorMessage),
    party_ref:      PartyRef(crate::message::editor::partyref::PartyRefEditorMessage),
    party_ini:      PartyIni(crate::message::editor::partyini::PartyIniEditorMessage),
    sprite_viewer:  SpriteViewer(crate::message::editor::spritebrowser::SpriteViewerMessage),
    monster_ref:    MonsterRef(crate::message::editor::monsterref::MonsterRefEditorMessage),
    all_map_ini:    AllMapIni(crate::message::editor::allmapini::AllMapIniEditorMessage),
    dialog:         Dialog(crate::message::editor::dialog::DialogEditorMessage),
    dialogue_text:  DialogueText(crate::message::editor::dialoguetext::DialogueTextEditorMessage),
    draw_item:      DrawItem(crate::message::editor::drawitem::DrawItemEditorMessage),
    event_ini:      EventIni(crate::message::editor::eventini::EventIniEditorMessage),
    event_npc_ref:  EventNpcRef(crate::message::editor::eventnpcref::EventNpcRefEditorMessage),
    extra_ini:      ExtraIni(crate::message::editor::extraini::ExtraIniEditorMessage),
    extra_ref:      ExtraRef(crate::message::editor::extraref::ExtraRefEditorMessage),
    map_ini:        MapIni(crate::message::editor::mapini::MapIniEditorMessage),
    message_scr:    MessageScr(crate::message::editor::messagescr::MessageScrEditorMessage),
    npc_ref:        NpcRef(crate::message::editor::npcref::NpcRefEditorMessage),
    party_level_db: PartyLevelDb(crate::message::editor::partyleveldb::PartyLevelDbEditorMessage),
    quest_scr:      QuestScr(crate::message::editor::questscr::QuestScrEditorMessage),
    wave_ini:       WaveIni(crate::message::editor::waveini::WaveIniEditorMessage),
    ch_data:        ChData(crate::message::editor::chdata::ChDataEditorMessage),
    map_editor:     MapEditor(crate::message::editor::map_editor::MapEditorMessage),
    tileset_editor: Tileset(crate::message::editor::tileset::TilesetEditorMessage),
    snf_editor:     Snf(crate::message::editor::snf::SnfEditorMessage),
}

/// Extension trait for building nested editor messages.
pub trait EditorMessageExt {
    fn nested<E, M>(message: M) -> EditorMessage
    where
        E: Into<EditorMessage>,
        M: Into<E>;
}

impl EditorMessageExt for EditorMessage {
    fn nested<E, M>(message: M) -> EditorMessage
    where
        E: Into<EditorMessage>,
        M: Into<E>,
    {
        message.into().into()
    }
}

/// Convenience extension for constructing Weapon editor messages.
pub trait WeaponEditorMessageExt {
    fn weapon(self) -> EditorMessage;
}

impl WeaponEditorMessageExt for WeaponEditorMessage {
    fn weapon(self) -> EditorMessage {
        EditorMessage::Weapon(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::components::file_tree::message::FileTreeMessage;
    use crate::message::editor::weapon::WeaponEditorMessage;
    use crate::message::editor::EditorMessage;
    use crate::message::{Message, MessageExt};
    use std::path::PathBuf;

    #[test]
    fn test_weapon_message_constructor() {
        let msg = Message::weapon(WeaponEditorMessage::LoadCatalog);
        match msg {
            Message::Editor(EditorMessage::Weapon(WeaponEditorMessage::LoadCatalog)) => (),
            _ => panic!("Unexpected message variant"),
        }
    }

    #[test]
    fn test_file_tree_message_constructor() {
        let path = PathBuf::from("test/path");
        let msg = Message::file_tree(FileTreeMessage::OpenFile(path.clone()));
        match msg {
            Message::Workspace(crate::message::WorkspaceMessage::FileTree(
                FileTreeMessage::OpenFile(p),
            )) if p == path => (),
            _ => panic!("Unexpected message variant"),
        }
    }

    #[test]
    fn test_store_message_constructor() {
        let msg = Message::store(crate::message::editor::store::StoreEditorMessage::LoadCatalog);
        match msg {
            Message::Editor(EditorMessage::Store(
                                crate::message::editor::store::StoreEditorMessage::LoadCatalog,
            )) => (),
            _ => panic!("Unexpected message variant"),
        }
    }
}
