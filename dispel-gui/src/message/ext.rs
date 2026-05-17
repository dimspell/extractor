use crate::editors::weapon::WeaponEditorMessage;
use crate::message::EditorMessage;

define_message_ext! {
    weapon:         Weapon(crate::editors::weapon::WeaponEditorMessage),
    monster:        Monster(crate::editors::monster::MonsterEditorMessage),
    monster_ini:    MonsterIni(crate::editors::monster_ini::MonsterIniEditorMessage),
    chest:          Chest(crate::editors::chest::ChestEditorMessage),
    heal_item:      HealItem(crate::editors::heal_item::HealItemEditorMessage),
    misc_item:      MiscItem(crate::editors::misc_item::MiscItemEditorMessage),
    edit_item:      EditItem(crate::editors::edit_item::EditItemEditorMessage),
    event_item:     EventItem(crate::editors::event_item::EventItemEditorMessage),
    npc_ini:        NpcIni(crate::editors::npc_ini::NpcIniEditorMessage),
    magic:          Magic(crate::editors::magic::MagicEditorMessage),
    store:          Store(crate::editors::store::StoreEditorMessage),
    party_ref:      PartyRef(crate::editors::party_ref::PartyRefEditorMessage),
    party_ini:      PartyIni(crate::editors::party_ini::PartyIniEditorMessage),
    sprite_viewer:  SpriteViewer(crate::editors::sprite_browser::SpriteViewerMessage),
    monster_ref:    MonsterRef(crate::editors::monster_ref::MonsterRefEditorMessage),
    all_map_ini:    AllMapIni(crate::editors::all_map_ini::AllMapIniEditorMessage),
    dialogue_script: DialogueScript(crate::editors::dialogue_script::DialogueScriptEditorMessage),
    dialogue_paragraph:  DialogueParagraph(crate::editors::dialogue_paragraph::DialogueParagraphEditorMessage),
    draw_item:      DrawItem(crate::editors::draw_item::DrawItemEditorMessage),
    event_ini:      EventIni(crate::editors::event_ini::EventIniEditorMessage),
    event_npc_ref:  EventNpcRef(crate::editors::event_npc_ref::EventNpcRefEditorMessage),
    extra_ini:      ExtraIni(crate::editors::extra_ini::ExtraIniEditorMessage),
    extra_ref:      ExtraRef(crate::editors::extra_ref::ExtraRefEditorMessage),
    map_ini:        MapIni(crate::editors::map_ini::MapIniEditorMessage),
    message_scr:    MessageScr(crate::editors::message_scr::MessageScrEditorMessage),
    npc_ref:        NpcRef(crate::editors::npc_ref::NpcRefEditorMessage),
    party_level_db: PartyLevelDb(crate::editors::party_level_db::PartyLevelDbEditorMessage),
    quest_scr:      QuestScr(crate::editors::quest_scr::QuestScrEditorMessage),
    event_scr:      EventScr(crate::editors::event_scr::EventScrEditorMessage),
    wave_ini:       WaveIni(crate::editors::wave_ini::WaveIniEditorMessage),
    chdata:         ChData(crate::editors::chdata::ChDataEditorMessage),
    map_editor:     MapEditor(crate::editors::map_editor::MapEditorMessage),
    tileset_editor: Tileset(crate::editors::tileset::TilesetEditorMessage),
    snf_editor:     Snf(crate::editors::snf_editor::SnfEditorMessage),
    mod_packager:    ModPackager(crate::editors::mod_packager::ModPackagerMessage),
    localization:    Localization(crate::editors::localization_manager::LocalizationMessage),
    hex_editor:      HexEditor(crate::editors::hex_editor::HexEditorMessage),
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
    use crate::editors::weapon::WeaponEditorMessage;
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
        let msg = Message::store(crate::editors::store::StoreEditorMessage::LoadCatalog);
        match msg {
            Message::Editor(EditorMessage::Store(
                crate::editors::store::StoreEditorMessage::LoadCatalog,
            )) => (),
            _ => panic!("Unexpected message variant"),
        }
    }
}
