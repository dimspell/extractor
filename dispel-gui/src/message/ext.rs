use crate::message::editor::weapon::WeaponEditorMessage;
use crate::message::EditorMessage;

define_message_ext! {
    weapon:         Weapon(crate::message::editor::weapon::WeaponEditorMessage),
    monster_db:     Monster(crate::message::editor::monster_db::MonsterEditorMessage),
    monster_ini:    MonsterIni(crate::message::editor::monster_ini::MonsterIniEditorMessage),
    chest:          Chest(crate::message::editor::chest::ChestEditorMessage),
    heal_item:      HealItem(crate::message::editor::heal_item::HealItemEditorMessage),
    misc_item:      MiscItem(crate::message::editor::misc_item::MiscItemEditorMessage),
    edit_item:      EditItem(crate::message::editor::edit_item::EditItemEditorMessage),
    event_item:     EventItem(crate::message::editor::event_item::EventItemEditorMessage),
    npc_ini:        NpcIni(crate::message::editor::npc_ini::NpcIniEditorMessage),
    magic:          Magic(crate::message::editor::magic::MagicEditorMessage),
    store:          Store(crate::message::editor::store::StoreEditorMessage),
    party_ref:      PartyRef(crate::message::editor::party_ref::PartyRefEditorMessage),
    party_ini:      PartyIni(crate::message::editor::party_ini::PartyIniEditorMessage),
    sprite_viewer:  SpriteViewer(crate::message::editor::spritebrowser::SpriteViewerMessage),
    monster_ref:    MonsterRef(crate::message::editor::monster_ref::MonsterRefEditorMessage),
    all_map_ini:    AllMapIni(crate::message::editor::all_map_ini::AllMapIniEditorMessage),
    dialogue_script: DialogueScript(crate::message::editor::dialogue_script::DialogueScriptEditorMessage),
    dialogue_paragraph:  DialogueParagraph(crate::message::editor::dialogue_paragraph::DialogueParagraphEditorMessage),
    draw_item:      DrawItem(crate::message::editor::draw_item::DrawItemEditorMessage),
    event_ini:      EventIni(crate::message::editor::event_ini::EventIniEditorMessage),
    event_npc_ref:  EventNpcRef(crate::message::editor::event_npc_ref::EventNpcRefEditorMessage),
    extra_ini:      ExtraIni(crate::message::editor::extra_ini::ExtraIniEditorMessage),
    extra_ref:      ExtraRef(crate::message::editor::extra_ref::ExtraRefEditorMessage),
    map_ini:        MapIni(crate::message::editor::map_ini::MapIniEditorMessage),
    message_scr:    MessageScr(crate::message::editor::message_scr::MessageScrEditorMessage),
    npc_ref:        NpcRef(crate::message::editor::npc_ref::NpcRefEditorMessage),
    party_level_db: PartyLevelDb(crate::message::editor::party_level_db::PartyLevelDbEditorMessage),
    quest_scr:      QuestScr(crate::message::editor::quest_scr::QuestScrEditorMessage),
    wave_ini:       WaveIni(crate::message::editor::wave_ini::WaveIniEditorMessage),
    ch_data:        ChData(crate::message::editor::chdata::ChDataEditorMessage),
    map_editor:     MapEditor(crate::message::editor::map_editor::MapEditorMessage),
    tileset_editor: Tileset(crate::message::editor::tileset::TilesetEditorMessage),
    snf_editor:     Snf(crate::message::editor::snf::SnfEditorMessage),
    mod_packager:    ModPackager(crate::message::editor::mod_packager::ModPackagerMessage),
    localization:    Localization(crate::message::editor::localization::LocalizationMessage),
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
