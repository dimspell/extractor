#[cfg(test)]
mod message_routing_tests {
    use crate::app::App;
    use crate::message::Message;
    use crate::tests::app_with_tab;
    use crate::workspace::{EditorType, Workspace};
    use std::path::PathBuf;

    #[test]
    fn test_hex_editor_message_when_no_state_does_not_panic() {
        let mut app = app_with_tab(EditorType::HexEditor);

        use crate::message::editor::EditorMessage;
        use hexedit::selection::NavDir;
        use hexedit::HexEditorMessage;
        // Route through the public update API — should not panic
        let task = app.update(Message::Editor(EditorMessage::HexEditor(
            HexEditorMessage::Nav {
                dir: NavDir::Down,
                extend: false,
            },
        )));
        assert_eq!(
            task.units(),
            0,
            "Should return Task::none() when no hex editor state"
        );
    }

    #[test]
    fn test_open_tool_tab_creates_workspace_tab() {
        let mut app = App::test_new(Workspace::new());

        use crate::message::workspace::WorkspaceMessage;
        let task = app.update(Message::Workspace(WorkspaceMessage::OpenToolTab(
            EditorType::DbViewer,
        )));

        assert_eq!(app.state.workspace.tabs.len(), 1);
        assert_eq!(
            app.state.workspace.tabs[0].editor_type,
            EditorType::DbViewer
        );
        assert_eq!(app.state.workspace.tabs[0].label, "DB Viewer");

        let _ = task;
    }

    #[test]
    fn test_clear_workspace_system_message_is_idempotent() {
        let mut app = App::test_new(Workspace::new());

        app.state.workspace.open("test".into(), None);
        assert_eq!(app.state.workspace.tabs.len(), 1);

        // First clear
        let task1 = app.update(Message::System(
            crate::message::system::SystemMessage::ClearWorkspace,
        ));
        let _ = task1;
        assert_eq!(app.state.workspace.tabs.len(), 0);

        // Second clear — must not panic and must keep state clean
        let task2 = app.update(Message::System(
            crate::message::system::SystemMessage::ClearWorkspace,
        ));
        let _ = task2;
        assert_eq!(app.state.workspace.tabs.len(), 0);
    }

    #[test]
    fn test_track_recent_files_limit() {
        let mut app = App::test_new(Workspace::new());

        // Add 15 files (should be capped to 10)
        for i in 0..15 {
            app.track_recent_file(&PathBuf::from(format!("/game/file{}.db", i)));
        }

        assert_eq!(app.state.recent_files.len(), 10);
        assert_eq!(app.state.recent_files[0], PathBuf::from("/game/file14.db"));
    }

    #[test]
    fn test_track_recent_file_dedup() {
        let mut app = App::test_new(Workspace::new());

        app.track_recent_file(&PathBuf::from("/game/weaponItem.db"));
        app.track_recent_file(&PathBuf::from("/game/monster.db"));
        app.track_recent_file(&PathBuf::from("/game/weaponItem.db"));

        assert_eq!(app.state.recent_files.len(), 2);
        assert_eq!(
            app.state.recent_files[0],
            PathBuf::from("/game/weaponItem.db")
        );
    }
}
