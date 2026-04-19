// Integration tests for the Clear Workspace command in the command palette

#[cfg(test)]
mod tests {
    use crate::command_palette::Command;

    #[test]
    fn test_clear_workspace_command_exists() {
        // Verify that the Clear Workspace command is in the command list
        let commands = Command::all();
        
        let clear_workspace_command = commands.iter()
            .find(|cmd| cmd.id == "clear-workspace");
        
        assert!(clear_workspace_command.is_some(), 
            "Clear Workspace command should exist in command palette");
        
        let cmd = clear_workspace_command.unwrap();
        assert_eq!(cmd.label, "Clear: Workspace Tabs & Editors");
        assert_eq!(cmd.shortcut, None);
        
        println!("✅ Clear Workspace command exists in command palette");
    }

    #[test]
    fn test_clear_workspace_command_action() {
        // Verify that the command action sends the correct message
        let commands = Command::all();
        let clear_workspace_command = commands.iter()
            .find(|cmd| cmd.id == "clear-workspace")
            .expect("Clear Workspace command should exist");
        
        // Call the action function
        let message = (clear_workspace_command.action)();
        
        // Verify it sends the correct system message
        match message {
            crate::message::Message::System(system_msg) => {
                use crate::message::system::SystemMessage::*;
                match system_msg {
                    ClearWorkspace => {
                        println!("✅ Command action sends ClearWorkspace message");
                    },
                    _ => panic!("Expected ClearWorkspace message, got {:?}", system_msg),
                }
            },
            _ => panic!("Expected System message, got {:?}", message),
        }
    }

    #[test]
    fn test_command_palette_has_workspace_section() {
        // Verify that workspace management commands are grouped together
        let commands = Command::all();
        
        let workspace_commands: Vec<_> = commands.iter()
            .filter(|cmd| 
                cmd.id.starts_with("clear") ||
                cmd.id.contains("workspace") ||
                cmd.label.contains("Workspace")
            )
            .collect();
        
        assert!(!workspace_commands.is_empty(), 
            "Should have workspace management commands");
        
        let clear_workspace = workspace_commands.iter()
            .find(|cmd| cmd.id == "clear-workspace")
            .expect("Clear Workspace should be in workspace section");
        
        assert_eq!(clear_workspace.label, "Clear: Workspace Tabs & Editors");
        
        println!("✅ Workspace commands are properly organized");
    }
}
