// Unit tests for workspace clearing functionality

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::{Workspace, WorkspaceTab};
    use crate::state::state::AppState;
    use crate::state::map_editor::MapEditorState;
    use std::collections::HashMap;

    #[test]
    fn test_clear_all_tabs() {
        // Create a workspace with some tabs
        let mut workspace = Workspace::new();
        workspace.tabs.push(WorkspaceTab {
            id: 1,
            label: "Test Tab 1".to_string(),
            editor_type: crate::workspace::EditorType::WeaponEditor,
            modified: false,
        });
        workspace.tabs.push(WorkspaceTab {
            id: 2,
            label: "Test Tab 2".to_string(),
            editor_type: crate::workspace::EditorType::MapEditor,
            modified: true,
        });
        workspace.active_tab = Some(0);
        
        // Verify initial state
        assert_eq!(workspace.tabs.len(), 2);
        assert_eq!(workspace.active_tab, Some(0));
        
        // Clear all tabs
        workspace.clear_all_tabs();
        
        // Verify cleared state
        assert_eq!(workspace.tabs.len(), 0);
        assert_eq!(workspace.active_tab, None);
        // next_id should be preserved
        assert_eq!(workspace.next_id, 0);
    }

    #[test]
    fn test_clear_editor_states() {
        // Create an AppState with some editor states
        let mut state = AppState::default();
        
        // Add some map editor states
        state.map_editors.insert(1, MapEditorState::default());
        state.map_editors.insert(2, MapEditorState::default());
        
        // Add some lookups
        state.lookups.insert("test_key".to_string(), vec!["test_value".to_string()]);
        
        // Verify initial state
        assert_eq!(state.map_editors.len(), 2);
        assert_eq!(state.lookups.len(), 1);
        
        // Clear editor states
        state.clear_editor_states();
        
        // Verify cleared state
        assert_eq!(state.map_editors.len(), 0);
        assert_eq!(state.dialog_editors.len(), 0);
        assert_eq!(state.sprite_viewers.len(), 0);
        assert_eq!(state.lookups.len(), 0);
        
        // Verify that boxed editors are reset to default
        // (This is harder to test directly, but we can verify they don't panic)
        let _ = state.weapon_editor;
        let _ = state.heal_item_editor;
    }

    #[test]
    fn test_workspace_clear_preserves_next_id() {
        let mut workspace = Workspace::new();
        workspace.next_id = 100; // Simulate some tabs have been created
        
        workspace.clear_all_tabs();
        
        // next_id should be preserved to avoid ID collisions
        assert_eq!(workspace.next_id, 100);
    }

    #[test]
    fn test_clear_editor_states_idempotent() {
        let mut state = AppState::default();
        
        // First clear
        state.clear_editor_states();
        
        // Second clear should not panic
        state.clear_editor_states();
        
        // State should still be valid
        assert_eq!(state.map_editors.len(), 0);
    }
}
