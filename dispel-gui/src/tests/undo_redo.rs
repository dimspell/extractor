#[cfg(test)]
mod undo_redo_dispatch_tests {
    use crate::editor_registry::EditorRegistry;
    use crate::workspace::EditorType;
    use crate::workspace::EditorType::*;
    use std::collections::HashMap;

    /// Standard boxed editors that support undo/redo
    fn editors_with_undo() -> Vec<EditorType> {
        vec![
            WeaponEditor,
            HealItemEditor,
            MiscItemEditor,
            EditItemEditor,
            EventItemEditor,
            MonsterEditor,
            MonsterIniEditor,
            NpcIniEditor,
            MagicEditor,
            PartyRefEditor,
            PartyIniEditor,
            AllMapIniEditor,
            DrawItemEditor,
            EventIniEditor,
            EventNpcRefEditor,
            ExtraIniEditor,
            MapIniEditor,
            MessageScrEditor,
            QuestScrEditor,
            WaveIniEditor,
            ChDataEditor,
            PartyLevelDbEditor,
        ]
    }

    /// Editors that have no undo/redo support (should always return None)
    fn editors_without_undo() -> Vec<EditorType> {
        vec![
            StoreEditor,
            SpriteViewer,
            SnfEditor,
            DbViewer,
            TilesetEditor,
            MapEditor,
            ModPackager,
            LocalizationManager,
            HexEditor,
            EventScrEditor,
            Unknown,
        ]
    }

    /// Tab-based editors that support undo/redo (need valid tab_id)
    fn tab_editors_with_undo() -> Vec<EditorType> {
        vec![
            MonsterRefEditor,
            NpcRefEditor,
            ExtraRefEditor,
            DialogueScriptEditor,
            DialogueTextEditor,
        ]
    }

    #[test]
    fn test_undo_active_returns_none_for_editors_without_undo() {
        let mut registry = EditorRegistry::default();
        let lookups = HashMap::new();

        for et in editors_without_undo() {
            let result = registry.undo_active(et, 0, &lookups);
            assert!(
                result.is_none(),
                "EditorType::{:?} should NOT have undo but got Some({:?})",
                et,
                result
            );
        }
    }

    #[test]
    fn test_redo_active_returns_none_for_editors_without_undo() {
        let mut registry = EditorRegistry::default();
        let lookups = HashMap::new();

        for et in editors_without_undo() {
            let result = registry.redo_active(et, 0, &lookups);
            assert!(
                result.is_none(),
                "EditorType::{:?} should NOT have redo but got Some({:?})",
                et,
                result
            );
        }
    }

    #[test]
    fn test_undo_active_empty_history_for_editors_with_undo() {
        let mut registry = EditorRegistry::default();
        let lookups = HashMap::new();

        for et in editors_with_undo() {
            let result = registry.undo_active(et, 0, &lookups);
            assert!(
                result.is_none(),
                "EditorType::{:?} should return None (empty history) but got Some({:?})",
                et,
                result
            );
        }
    }

    #[test]
    fn test_undo_active_tab_editor_without_valid_tab_id() {
        let mut registry = EditorRegistry::default();
        let lookups = HashMap::new();

        for et in tab_editors_with_undo() {
            let result = registry.undo_active(et, 999, &lookups);
            assert!(
                result.is_none(),
                "EditorType::{:?} with unknown tab_id should return None but got Some({:?})",
                et,
                result
            );
        }
    }

    #[test]
    fn test_redo_active_tab_editor_without_valid_tab_id() {
        let mut registry = EditorRegistry::default();
        let lookups = HashMap::new();

        for et in tab_editors_with_undo() {
            let result = registry.redo_active(et, 999, &lookups);
            assert!(
                result.is_none(),
                "EditorType::{:?} with unknown tab_id should return None but got Some({:?})",
                et,
                result
            );
        }
    }
}

#[cfg(test)]
mod edit_history_tests {
    use crate::editor_registry::EditorRegistry;
    use crate::workspace::EditorType::*;

    #[test]
    fn test_all_standard_editors_have_edit_history() {
        let registry = EditorRegistry::default();

        let editors_with_history = vec![
            HealItemEditor,
            MiscItemEditor,
            EditItemEditor,
            EventItemEditor,
            MagicEditor,
            WeaponEditor,
            DrawItemEditor,
            EventIniEditor,
            EventNpcRefEditor,
            ExtraIniEditor,
            MapIniEditor,
            MessageScrEditor,
            PartyLevelDbEditor,
            QuestScrEditor,
            WaveIniEditor,
            AllMapIniEditor,
            ChDataEditor,
            PartyRefEditor,
            PartyIniEditor,
            StoreEditor,
        ];

        for et in editors_with_history {
            let history = registry.get_active_edit_history(et, 0);
            assert!(
                history.is_some(),
                "EditorType::{:?} should have edit history but got None",
                et
            );
        }
    }

    #[test]
    fn test_tab_editors_return_history_only_with_valid_tab_id() {
        let mut registry = EditorRegistry::default();

        assert!(registry
            .get_active_edit_history(MonsterRefEditor, 0)
            .is_none());
        assert!(registry.get_active_edit_history(NpcRefEditor, 0).is_none());
        assert!(registry
            .get_active_edit_history(ExtraRefEditor, 0)
            .is_none());
        assert!(registry
            .get_active_edit_history(DialogueScriptEditor, 0)
            .is_none());
        assert!(registry
            .get_active_edit_history(DialogueTextEditor, 0)
            .is_none());

        registry
            .npc_ref_editor
            .editors
            .insert(42, Default::default());
        assert!(
            registry.get_active_edit_history(NpcRefEditor, 42).is_some(),
            "NpcRefEditor with tab_id=42 should have history after insert"
        );
    }

    #[test]
    fn test_editors_without_history_return_none() {
        let registry = EditorRegistry::default();

        let editors_without = vec![
            EventScrEditor,
            MonsterEditor,
            MonsterIniEditor,
            NpcIniEditor,
            SpriteViewer,
            SnfEditor,
            DbViewer,
            TilesetEditor,
            MapEditor,
            ModPackager,
            LocalizationManager,
            HexEditor,
            Unknown,
        ];

        for et in editors_without {
            let history = registry.get_active_edit_history(et, 0);
            assert!(
                history.is_none(),
                "EditorType::{:?} should NOT have edit history but got Some",
                et
            );
        }
    }
}
