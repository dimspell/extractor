#[cfg(test)]
mod save_dispatch_tests {
    use crate::message::Message;
    use crate::message::system::SystemMessage;
    use crate::tests::app_with_tab;
    use crate::workspace::EditorType;

    #[test]
    fn save_respects_editor_capability_across_all_types() {
        // Instead of duplicating a hand-rolled list (which drifts out of sync),
        // we verify the CONTRACT: supports_save() == true  ⇔  save produces a task.
        for et in all_editor_types() {
            let mut app = app_with_tab(et);
            let task = app.update(Message::System(SystemMessage::Save));
            if et.supports_save() {
                assert!(
                    task.units() > 0,
                    "EditorType::{:?} supports_save() but Save returned Task::none()",
                    et
                );
            } else {
                assert_eq!(
                    task.units(),
                    0,
                    "EditorType::{:?} !supports_save() but Save produced a task",
                    et
                );
            }
        }
    }

    /// Returns every `EditorType` variant for exhaustive testing.
    fn all_editor_types() -> Vec<EditorType> {
        vec![
            EditorType::WeaponEditor,
            EditorType::MonsterEditor,
            EditorType::MonsterIniEditor,
            EditorType::HealItemEditor,
            EditorType::MiscItemEditor,
            EditorType::EditItemEditor,
            EditorType::EventItemEditor,
            EditorType::MagicEditor,
            EditorType::StoreEditor,
            EditorType::ChDataEditor,
            EditorType::PartyLevelDbEditor,
            EditorType::DialogueScriptEditor,
            EditorType::DialogueTextEditor,
            EditorType::DrawItemEditor,
            EditorType::EventIniEditor,
            EditorType::EventNpcRefEditor,
            EditorType::ExtraIniEditor,
            EditorType::ExtraRefEditor,
            EditorType::MapIniEditor,
            EditorType::MessageScrEditor,
            EditorType::MonsterRefEditor,
            EditorType::NpcIniEditor,
            EditorType::NpcRefEditor,
            EditorType::PartyRefEditor,
            EditorType::PartyIniEditor,
            EditorType::QuestScrEditor,
            EditorType::EventScrEditor,
            EditorType::WaveIniEditor,
            EditorType::AllMapIniEditor,
            EditorType::ChestEditor,
            EditorType::SpriteViewer,
            EditorType::SnfEditor,
            EditorType::DbViewer,
            EditorType::TilesetEditor,
            EditorType::MapEditor,
            EditorType::ModPackager,
            EditorType::LocalizationManager,
            EditorType::HexEditor,
            EditorType::Unknown,
        ]
    }
}
