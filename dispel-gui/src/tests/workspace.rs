#[cfg(test)]
mod workspace_tab_tests {
    use crate::app::App;
    use crate::message::workspace::WorkspaceMessage;
    use crate::message::Message;
    use crate::workspace::{EditorType, Workspace, WorkspaceTab};

    #[test]
    fn reopen_active_tab_as_hex_no_active_tab_does_not_panic() {
        let mut app = App::test_new(Workspace::new());
        let task = app.update(Message::Workspace(WorkspaceMessage::ReopenActiveTabAsHex));
        assert_eq!(task.units(), 0);
    }

    #[test]
    fn reopen_active_tab_as_hex_without_path_does_not_panic() {
        let mut app = App::test_new(Workspace::new());
        app.state.workspace.tabs.push(WorkspaceTab {
            id: 1,
            label: "test.map".into(),
            path: None,
            editor_type: EditorType::MapEditor,
            modified: false,
            pinned: false,
        });
        app.state.workspace.active_tab = Some(0);
        let task = app.update(Message::Workspace(WorkspaceMessage::ReopenActiveTabAsHex));
        assert_eq!(task.units(), 0, "no path → no task");
    }
}

#[cfg(test)]
mod workspace_reopen_tests {
    use crate::app::App;
    use crate::workspace::{EditorType, Workspace};
    use std::path::PathBuf;

    #[test]
    fn open_file_already_open_reactivates_tab() {
        let mut app = App::test_new(Workspace::new());
        let path = PathBuf::from("test.map");

        // Open once
        let _task1 = app.open_file_in_workspace(&path);
        assert_eq!(app.state.workspace.tabs.len(), 1, "one tab after first open");

        // Open again — should reactivate, not create new tab
        let _task2 = app.open_file_in_workspace(&path);
        assert_eq!(
            app.state.workspace.tabs.len(),
            1,
            "same tab reactivated, no duplicate"
        );
    }

    #[test]
    fn open_same_path_different_type_creates_separate_tab() {
        let mut app = App::test_new(Workspace::new());
        let path = PathBuf::from("test.map");

        // Open normally (MapEditor)
        let _task1 = app.open_file_in_workspace(&path);
        assert_eq!(app.state.workspace.tabs.len(), 1);

        // Open as hex (different editor type) — should create new tab
        let _task2 = app.open_file_in_workspace_as_hex(&path);
        assert_eq!(
            app.state.workspace.tabs.len(),
            2,
            "second tab for different editor type"
        );
    }

    #[test]
    fn open_unknown_path_does_not_panic() {
        let mut app = App::test_new(Workspace::new());
        let path = PathBuf::from("nonexistent.xyz");
        let _task = app.open_file_in_workspace(&path);
        // Should create a hex editor tab (fallback for unknown extensions)
        assert_eq!(app.state.workspace.tabs.len(), 1, "tab created for unknown extension");
        assert_eq!(
            app.state.workspace.active().unwrap().editor_type,
            EditorType::HexEditor,
            "unknown extension defaults to HexEditor"
        );
    }
}
