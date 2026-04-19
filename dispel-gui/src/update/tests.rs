// Unit tests for system message handlers

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::message::system::SystemMessage;
    use crate::state::state::AppState;
    use crate::workspace::Workspace;

    #[test]
    fn test_clear_workspace_message() {
        // Create a test app with some state
        let mut app = App::new().0;
        
        // Add some workspace tabs
        app.state.workspace.tabs.push(crate::workspace::WorkspaceTab {
            id: 1,
            label: "Test Tab".to_string(),
            editor_type: crate::workspace::EditorType::WeaponEditor,
            modified: false,
        });
        app.state.workspace.active_tab = Some(0);
        
        // Add some editor states
        app.state.map_editors.insert(1, crate::state::map_editor::MapEditorState::default());
        app.state.lookups.insert("test".to_string(), vec!["value".to_string()]);
        
        // Verify initial state
        assert_eq!(app.state.workspace.tabs.len(), 1);
        assert_eq!(app.state.map_editors.len(), 1);
        assert_eq!(app.state.lookups.len(), 1);
        
        // Handle the ClearWorkspace message
        let task = crate::update::system::handle(SystemMessage::ClearWorkspace, &mut app);
        
        // Verify the message was handled (task should be Task::none())
        assert!(matches!(task, iced::Task::none()));
        
        // Verify workspace was cleared
        assert_eq!(app.state.workspace.tabs.len(), 0);
        assert_eq!(app.state.workspace.active_tab, None);
        assert_eq!(app.state.map_editors.len(), 0);
        assert_eq!(app.state.lookups.len(), 0);
        
        // Verify status message was set
        assert!(app.state.status_msg.contains("Workspace cleared"));
    }

    #[test]
    fn test_clear_workspace_with_no_tabs() {
        // Create a test app with no tabs
        let mut app = App::new().0;
        
        // Verify initial state
        assert_eq!(app.state.workspace.tabs.len(), 0);
        assert_eq!(app.state.map_editors.len(), 0);
        
        // Handle the ClearWorkspace message (should not panic)
        let task = crate::update::system::handle(SystemMessage::ClearWorkspace, &mut app);
        
        // Verify the message was handled
        assert!(matches!(task, iced::Task::none()));
        
        // Verify status message was set
        assert!(app.state.status_msg.contains("Workspace cleared"));
    }
}
