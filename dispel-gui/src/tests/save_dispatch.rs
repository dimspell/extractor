#[cfg(test)]
mod save_dispatch_tests {
    use crate::message::Message;
    use crate::message::system::SystemMessage;
    use crate::tests::app_with_tab;
    use crate::workspace::EditorType;
    use crate::workspace::EditorType::*;

    #[test]
    fn test_save_returns_task_for_map_editor_and_event_scr() {
        let mut app = app_with_tab(MapEditor);
        let task = app.update(Message::System(SystemMessage::Save));
        assert!(task.units() > 0, "MapEditor Save should produce a task");

        let mut app = app_with_tab(EventScrEditor);
        let task = app.update(Message::System(SystemMessage::Save));
        assert!(task.units() > 0, "EventScrEditor Save should produce a task");
    }

    #[test]
    fn test_save_returns_none_for_editor_types_without_save() {
        // Only editor types that genuinely lack Ctrl+S.
        // Most standard editors now support Save via the helper.
        let no_save_types = vec![
            EditorType::SpriteViewer,
            EditorType::SnfEditor,
            EditorType::DbViewer,
            EditorType::TilesetEditor,
            EditorType::ModPackager,
            EditorType::LocalizationManager,
            EditorType::HexEditor,
            EditorType::Unknown,
        ];

        for et in no_save_types {
            let mut app = app_with_tab(et);
            let task = app.update(Message::System(SystemMessage::Save));
            assert_eq!(
                task.units(), 0,
                "EditorType::{:?} Save should produce Task::none()",
                et
            );
        }
    }
}

#[cfg(test)]
mod system_save_tests {
    use crate::message::Message;
    use crate::message::system::SystemMessage;
    use crate::tests::app_with_tab;
    use crate::workspace::EditorType;

    #[test]
    fn save_returns_task_for_all_saving_editor_types() {
        // All editor types that should return a Save task via Ctrl+S.
        // Keep in sync with save_task_for_editor() in update/system.rs.
        let saving_types: Vec<EditorType> = vec![
            EditorType::WeaponEditor,
            EditorType::MonsterEditor,
            EditorType::MonsterIniEditor,
            EditorType::HealItemEditor,
            EditorType::MiscItemEditor,
            EditorType::EditItemEditor,
            EditorType::EventItemEditor,
            EditorType::NpcIniEditor,
            EditorType::MagicEditor,
            EditorType::PartyRefEditor,
            EditorType::PartyIniEditor,
            EditorType::AllMapIniEditor,
            EditorType::DrawItemEditor,
            EditorType::EventIniEditor,
            EditorType::EventNpcRefEditor,
            EditorType::ExtraIniEditor,
            EditorType::MapIniEditor,
            EditorType::MessageScrEditor,
            EditorType::QuestScrEditor,
            EditorType::WaveIniEditor,
            EditorType::ChDataEditor,
            EditorType::StoreEditor,
            EditorType::ChestEditor,
            EditorType::PartyLevelDbEditor,
            EditorType::MapEditor,
            EditorType::EventScrEditor,
            // Tabbed editors (now wired for Ctrl+S)
            EditorType::DialogueScriptEditor,
            EditorType::DialogueTextEditor,
            EditorType::ExtraRefEditor,
            EditorType::MonsterRefEditor,
            EditorType::NpcRefEditor,
        ];
        for et in saving_types {
            let mut app = app_with_tab(et);
            let task = app.update(Message::System(SystemMessage::Save));
            assert!(
                task.units() > 0,
                "EditorType::{:?} Save should produce a task (not Task::none())",
                et
            );
        }
    }

    #[test]
    fn save_returns_task_for_map_editor() {
        let mut app = app_with_tab(EditorType::MapEditor);
        let task = app.update(Message::System(SystemMessage::Save));
        assert!(task.units() > 0, "MapEditor Save should produce a task");
    }

    #[test]
    fn save_returns_task_for_event_scr_editor() {
        let mut app = app_with_tab(EditorType::EventScrEditor);
        let task = app.update(Message::System(SystemMessage::Save));
        assert!(task.units() > 0, "EventScrEditor Save should produce a task");
    }
}
