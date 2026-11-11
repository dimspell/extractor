#[cfg(test)]
mod save_dispatch_tests {
    use crate::app::App;
    use crate::message::Message;
    use crate::message::system::SystemMessage;
    use crate::tests::app_with_tab;
    use crate::workspace::{EditorType, Workspace};

    #[test]
    fn save_respects_editor_capability_across_all_types() {
        // Instead of duplicating a hand-rolled list (which drifts out of sync),
        // we verify the CONTRACT: supports_save() == true  ⇔  save is dispatched.
        // Save dispatch uses Task::done() which has units=0, so we check status_msg.
        for et in all_editor_types() {
            let mut app = app_with_tab(et);
            let _task = app.update(Message::System(SystemMessage::Save));
            if et.supports_save() {
                assert_ne!(
                    app.state.status_msg, "This editor does not support saving",
                    "EditorType::{:?} supports_save() but Save rejected it",
                    et
                );
            } else {
                assert_eq!(
                    app.state.status_msg, "This editor does not support saving",
                    "EditorType::{:?} !supports_save() but Save was accepted",
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
            EditorType::SpriteViewer,
            EditorType::SnfEditor,
            EditorType::DbViewer,
            EditorType::TilesetEditor,
            EditorType::MapEditor,
            EditorType::ModPackager,
            EditorType::LocalizationManager,
            EditorType::HexEditor,
            EditorType::FogDataEditor,
            EditorType::Unknown,
        ]
    }

    #[test]
    fn test_save_sprite_viewer_returns_task() {
        // SpriteViewer now supports saving.
        let mut app = app_with_tab(EditorType::SpriteViewer);
        let _task = app.update(Message::System(SystemMessage::Save));
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
