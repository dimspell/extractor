//! Undo/redo and edit-history coverage for every editor.
//!
//! The old version of this file hand-copied five editor lists. They went stale
//! (StoreEditor was classified as "no undo", new multiplayer editors were
//! missing) and silently stopped testing real behavior. All lists are now
//! derived from the authoritative editor catalog via `EditorType::iter()` and
//! its lifecycle capability flags.

#[cfg(test)]
mod undo_redo_dispatch_tests {
    use crate::editor_registry::EditorRegistry;
    use crate::workspace::EditorType;
    use std::collections::HashMap;

    fn editors_with_undo() -> impl Iterator<Item = EditorType> {
        EditorType::iter().filter(|et| et.supports_undo_redo())
    }

    fn editors_without_undo() -> impl Iterator<Item = EditorType> {
        EditorType::iter().filter(|et| !et.supports_undo_redo())
    }

    /// Tab-indexed editors with undo need a real tab entry to exercise them;
    /// here we verify that an unknown tab id is handled gracefully.
    fn tab_editors_with_undo() -> impl Iterator<Item = EditorType> {
        EditorType::iter()
            .filter(|et| et.supports_undo_redo() && et.descriptor().lifecycle.has_tab_state)
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
    use crate::workspace::EditorType;

    #[test]
    fn test_all_non_tab_editors_with_history_have_edit_history() {
        let registry = EditorRegistry::default();
        let mut checked = 0;

        for et in EditorType::iter()
            .filter(|et| et.has_edit_history() && !et.descriptor().lifecycle.has_tab_state)
        {
            let history = registry.get_active_edit_history(et, 0);
            assert!(
                history.is_some(),
                "EditorType::{:?} declares edit history but get_active_edit_history returned None",
                et
            );
            checked += 1;
        }
        assert!(
            checked > 0,
            "catalog should declare some non-tab history editors"
        );
    }

    #[test]
    fn test_tab_editors_return_history_only_with_valid_tab_id() {
        let mut registry = EditorRegistry::default();

        let tabbed: Vec<EditorType> = EditorType::iter()
            .filter(|et| et.has_edit_history() && et.descriptor().lifecycle.has_tab_state)
            .collect();
        assert!(
            !tabbed.is_empty(),
            "catalog should declare tabbed history editors"
        );

        for &et in &tabbed {
            assert!(
                registry.get_active_edit_history(et, 0).is_none(),
                "EditorType::{:?} without tab state should have no history",
                et
            );
        }

        // Insert per-tab state through the generic lifecycle for each tabbed
        // editor and verify the history becomes visible.
        registry
            .npc_ref_editor
            .editors
            .insert(42, Default::default());
        assert!(
            registry
                .get_active_edit_history(EditorType::NpcRefEditor, 42)
                .is_some(),
            "NpcRefEditor with tab_id=42 should have history after insert"
        );
    }

    #[test]
    fn test_editors_without_history_return_none() {
        let registry = EditorRegistry::default();

        for et in EditorType::iter().filter(|et| !et.has_edit_history()) {
            let history = registry.get_active_edit_history(et, 0);
            assert!(
                history.is_none(),
                "EditorType::{:?} should NOT have edit history but got Some",
                et
            );
        }
    }
}
