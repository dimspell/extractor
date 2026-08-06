#[cfg(test)]
mod workspace_tab_tests {
    use crate::app::App;
    use crate::message::Message;
    use crate::message::workspace::WorkspaceMessage;
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
