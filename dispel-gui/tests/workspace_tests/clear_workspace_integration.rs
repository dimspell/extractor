// Integration tests for workspace clearing functionality
// These tests verify that the workspace clearing works end-to-end

#[cfg(test)]
mod tests {
    use crate::app::App;
    use crate::message::system::SystemMessage;
    use crate::message::Message;

    #[test]
    fn test_clear_workspace_command_integration() {
        // Create a new app instance
        let mut app = App::new().0;
        
        // Simulate opening some files to create editor states
        // (In a real scenario, this would be done through the file tree)
        
        // Verify initial state - app should have some default state
        let initial_tab_count = app.state.workspace.tabs.len();
        let initial_editor_count = app.state.map_editors.len() +
                                   app.state.dialog_editors.len() +
                                   app.state.sprite_viewers.len();
        
        println!("Initial state: {} tabs, {} editors", initial_tab_count, initial_editor_count);
        
        // Handle the ClearWorkspace message
        let task = crate::update::system::handle(SystemMessage::ClearWorkspace, &mut app);
        
        // Verify the task was handled (should be Task::none)
        match task {
            iced::Task::none() => {},
            _ => panic!("Expected Task::none, got something else"),
        }
        
        // Verify workspace was cleared
        assert_eq!(app.state.workspace.tabs.len(), 0, "All tabs should be cleared");
        assert_eq!(app.state.workspace.active_tab, None, "Active tab should be cleared");
        assert_eq!(app.state.map_editors.len(), 0, "Map editors should be cleared");
        assert_eq!(app.state.dialog_editors.len(), 0, "Dialog editors should be cleared");
        assert_eq!(app.state.sprite_viewers.len(), 0, "Sprite viewers should be cleared");
        
        // Verify status message was set
        assert!(app.state.status_msg.contains("Workspace cleared"), 
            "Status message should indicate workspace was cleared");
        
        println!("✅ Workspace clearing integration test passed");
    }

    #[test]
    fn test_clear_workspace_preserves_game_path() {
        // Create a new app instance
        let mut app = App::new().0;
        
        // Set a game path (simulating user action)
        app.state.workspace.game_path = Some(std::path::PathBuf::from("/test/game"));
        app.state.shared_game_path = "/test/game".to_string();
        
        // Clear workspace
        let task = crate::update::system::handle(SystemMessage::ClearWorkspace, &mut app);
        match task {
            iced::Task::none() => {},
            _ => panic!("Expected Task::none"),
        }
        
        // Verify game path is preserved (only tabs and editors should be cleared)
        assert_eq!(app.state.workspace.game_path, Some(std::path::PathBuf::from("/test/game")), 
            "Game path should be preserved after clearing workspace");
        assert_eq!(app.state.shared_game_path, "/test/game", 
            "Shared game path should be preserved after clearing workspace");
        
        println!("✅ Game path preservation test passed");
    }

    #[test]
    fn test_clear_workspace_idempotent() {
        // Create a new app instance
        let mut app = App::new().0;
        
        // Clear workspace twice
        let task1 = crate::update::system::handle(SystemMessage::ClearWorkspace, &mut app);
        let task2 = crate::update::system::handle(SystemMessage::ClearWorkspace, &mut app);
        
        // Both should handle successfully
        match task1 {
            iced::Task::none() => {},
            _ => panic!("First clear should return Task::none"),
        }
        match task2 {
            iced::Task::none() => {},
            _ => panic!("Second clear should return Task::none"),
        }
        
        // Should still be in valid state
        assert_eq!(app.state.workspace.tabs.len(), 0);
        assert_eq!(app.state.map_editors.len(), 0);
        
        println!("✅ Idempotent clearing test passed");
    }
}
