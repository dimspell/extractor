#[cfg(test)]
mod generic_editor_edge_tests {
    use crate::components::generic_editor::GenericEditorState;
    use dispel_core::WeaponItem;

    #[test]
    fn select_out_of_bounds_sets_selected_idx_but_no_buffers() {
        let mut editor = GenericEditorState::<WeaponItem>::default();
        // No catalog — filtered is empty
        editor.select(999);
        assert_eq!(editor.selected_idx, Some(999), "idx recorded even if OOB");
        assert!(editor.edit_buffers.is_empty(), "no buffers loaded for OOB");
    }

    #[test]
    fn select_with_none_catalog_does_not_panic() {
        let mut editor = GenericEditorState::<WeaponItem>::default();
        // select() accesses filtered list; with no catalog, filtered is empty
        editor.select(0); // should not panic
    }

    #[test]
    fn undo_empty_history_returns_none() {
        let mut editor = GenericEditorState::<WeaponItem>::default();
        let result = editor.undo();
        assert!(result.is_none(), "undo on empty history returns None");
    }

    #[test]
    fn redo_empty_history_returns_none() {
        let mut editor = GenericEditorState::<WeaponItem>::default();
        let result = editor.redo();
        assert!(result.is_none(), "redo on empty history returns None");
    }

    #[test]
    fn update_field_with_none_catalog_returns_false() {
        let mut editor = GenericEditorState::<WeaponItem>::default();
        // No catalog, no filtered — update_field should return false
        let result = editor.update_field(0, "name", "Test".into());
        assert!(!result, "update_field with no catalog returns false");
    }

    #[test]
    fn refresh_with_some_catalog_populates_filtered() {
        let mut editor = GenericEditorState::<WeaponItem>::default();
        // refresh with no catalog is a no-op
        editor.refresh();
        assert!(editor.filtered.is_empty(), "filtered stays empty with None catalog");

        // Now set a catalog and refresh
        editor.catalog = Some(vec![WeaponItem {
            name: "Test Sword".into(),
            ..Default::default()
        }]);
        editor.refresh();
        assert_eq!(editor.filtered.len(), 1, "filtered populated from catalog");
        assert_eq!(editor.filtered[0].1.name, "Test Sword");
    }
}
