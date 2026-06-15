#[cfg(test)]
mod editor_registry_tests {
    use crate::editor_registry::EditorRegistry;

    #[test]
    fn test_remove_tab_nonexistent_id_does_not_panic() {
        let mut registry = EditorRegistry::default();
        registry.remove_tab(9999);
    }

    #[test]
    fn test_remove_tab_preserves_unrelated_editors() {
        let mut registry = EditorRegistry::default();
        registry.map_editors.insert(1, Default::default());
        registry.sprite_viewers.insert(2, Default::default());

        registry.remove_tab(1);

        assert!(
            registry.sprite_viewers.contains_key(&2),
            "tab 2 sprite viewer should survive removal of tab 1"
        );
        assert!(
            !registry.map_editors.contains_key(&1),
            "tab 1 map editor should be removed"
        );
    }

    #[test]
    fn test_close_all_tabs_preserves_boxed_editors() {
        let mut registry = EditorRegistry::default();

        // Populate HashMap editors
        registry.map_editors.insert(1, Default::default());
        registry.sprite_viewers.insert(3, Default::default());

        // Populate tabbed editor
        registry
            .npc_ref_editor
            .editors
            .insert(1, Default::default());

        registry.close_all_tabs();

        // HashMap and tabbed editors are cleared
        assert!(registry.map_editors.is_empty());
        assert!(registry.sprite_viewers.is_empty());
        assert!(registry.npc_ref_editor.editors.is_empty());

        // Boxed editors like weapon_editor are NOT reset by close_all_tabs
        let _ = &registry.weapon_editor;
    }

    #[test]
    fn test_clear_all_resets_everything() {
        let mut registry = EditorRegistry::default();

        registry.map_editors.insert(1, Default::default());
        registry.sprite_viewers.insert(1, Default::default());
        registry
            .npc_ref_editor
            .editors
            .insert(1, Default::default());
        registry
            .dialogue_script_editor
            .editors
            .insert(1, Default::default());
        registry
            .monster_ref_editor
            .editors
            .insert(1, Default::default());
        registry
            .extra_ref_editor
            .editors
            .insert(1, Default::default());
        registry
            .dialogue_paragraph_editor
            .editors
            .insert(1, Default::default());

        registry.clear_all();

        assert!(registry.map_editors.is_empty());
        assert!(registry.sprite_viewers.is_empty());
        assert!(registry.npc_ref_editor.editors.is_empty());
        assert!(registry.dialogue_script_editor.editors.is_empty());
        assert!(registry.monster_ref_editor.editors.is_empty());
        assert!(registry.extra_ref_editor.editors.is_empty());
        assert!(registry.dialogue_paragraph_editor.editors.is_empty());
    }

    #[test]
    fn test_clear_all_is_idempotent() {
        let mut registry = EditorRegistry::default();
        registry.clear_all();
        registry.clear_all();
        registry.clear_all();
        // No panic = pass
    }
}
