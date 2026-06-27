#[cfg(test)]
mod save_dispatch_tests {
    use crate::app::App;
    use crate::message::system::SystemMessage;
    use crate::message::Message;
    use crate::tests::app_with_tab;
    use crate::workspace::{EditorType, Workspace};

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

    #[test]
    fn test_save_sprite_viewer_returns_task() {
        // SpriteViewer now supports saving.
        let mut app = app_with_tab(EditorType::SpriteViewer);
        let task = app.update(Message::System(SystemMessage::Save));
        assert!(task.units() > 0, "SpriteViewer Save produces task");
        // Status message is not the "no save" error.
        assert_ne!(app.state.status_msg, "This editor does not support saving");
    }

    #[test]
    fn test_save_no_active_tab_shows_correct_status() {
        // No tabs open — no active tab to save.
        let mut app = App::test_new(Workspace::new());
        let _ = app.update(Message::System(SystemMessage::Save));
        assert_eq!(app.state.status_msg, "No active tab to save");
    }
}
