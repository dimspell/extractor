mod crosscheck {
    use crate::editor_registry::EditorRegistry;
    use crate::message::system::SystemMessage;
    use crate::message::Message;
    use crate::tests::app_with_tab;
    use crate::workspace::EditorType;
    use std::collections::HashMap;

    /// Every EditorType variant, in the same order as the enum definition.
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

    // ── Save capability cross-check ────────────────────────────────────────

    #[test]
    fn save_contract_holds_for_all_types() {
        for et in all_editor_types() {
            let mut app = app_with_tab(et);
            let _task = app.update(Message::System(SystemMessage::Save));
            if et.supports_save() {
                assert_ne!(
                    app.state.status_msg,
                    "This editor does not support saving",
                    "EditorType::{:?} supports_save() but Save rejected it",
                    et
                );
            } else {
                assert_eq!(
                    app.state.status_msg,
                    "This editor does not support saving",
                    "EditorType::{:?} !supports_save() but Save was accepted",
                    et
                );
            }
        }
    }

    // ── Undo/redo capability cross-check ───────────────────────────────────

    #[test]
    fn undo_active_does_not_panic_for_any_type() {
        let mut registry = EditorRegistry::default();
        let lookups = HashMap::new();
        for et in all_editor_types() {
            let result = registry.undo_active(et, 0, &lookups);
            // All types should return None (empty history) — what matters is
            // that no panic occurs and the gate vs dispatch arm are consistent.
            assert!(
                result.is_none(),
                "EditorType::{:?} undo_active returned Some when history is empty",
                et
            );
        }
    }

    #[test]
    fn redo_active_does_not_panic_for_any_type() {
        let mut registry = EditorRegistry::default();
        let lookups = HashMap::new();
        for et in all_editor_types() {
            let result = registry.redo_active(et, 0, &lookups);
            assert!(
                result.is_none(),
                "EditorType::{:?} redo_active returned Some when history is empty",
                et
            );
        }
    }

    // ── Edit history cross-check ───────────────────────────────────────────

    #[test]
    fn edit_history_does_not_panic_for_any_type() {
        let registry = EditorRegistry::default();
        for et in all_editor_types() {
            let result = registry.get_active_edit_history(et, 0);
            // Tabbed editors have_edit_history() == true but return None when
            // no editor is inserted for the given tab_id — that's correct.
            if !et.has_edit_history() {
                assert!(
                    result.is_none(),
                    "EditorType::{:?} !has_edit_history() but get_active_edit_history returned Some",
                    et
                );
            }
            // If has_edit_history() is true, result may be Some or None
            // depending on whether the editor is initialized — don't assert.
        }
    }

    // ─── Tabbed editor: verify edit_history initialised after insert ──────

    #[test]
    fn tabbed_editor_edit_history_available_after_insert() {
        let mut registry = EditorRegistry::default();
        let tab_id = 42;
        registry
            .npc_ref_editor
            .editors
            .insert(tab_id, Default::default());
        let history = registry.get_active_edit_history(EditorType::NpcRefEditor, tab_id);
        assert!(
            history.is_some(),
            "NpcRefEditor should have edit history after insert"
        );
    }
}
