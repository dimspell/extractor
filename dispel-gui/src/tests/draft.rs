#[cfg(test)]
mod draft_tests {
    use crate::app::App;
    use crate::message::system::SystemMessage;
    use crate::message::Message;
    use crate::workspace::Workspace;

    #[test]
    fn toggle_auto_save_flips_flag() {
        let mut app = App::test_new(Workspace::new());
        let initially_enabled = app.draft_manager.is_auto_save_enabled();

        let task = app.update(Message::System(SystemMessage::ToggleAutoSave));
        assert_eq!(
            app.draft_manager.is_auto_save_enabled(),
            !initially_enabled,
            "flag toggled"
        );
        assert!(app.state.status_msg.contains("Auto-save"), "status updated");
        assert_eq!(task.units(), 0);
    }

    #[test]
    fn toggle_auto_save_twice_returns_to_original() {
        let mut app = App::test_new(Workspace::new());
        let initially_enabled = app.draft_manager.is_auto_save_enabled();

        let _ = app.update(Message::System(SystemMessage::ToggleAutoSave));
        let _ = app.update(Message::System(SystemMessage::ToggleAutoSave));
        assert_eq!(
            app.draft_manager.is_auto_save_enabled(),
            initially_enabled,
            "back to original after double toggle"
        );
    }

    #[test]
    fn discard_draft_nonexistent_path_does_not_panic() {
        let mut app = App::test_new(Workspace::new());
        let task = app.update(Message::System(SystemMessage::DiscardDraft(
            "/nonexistent/file.ini".into(),
        )));
        assert!(
            app.state.status_msg.contains("discarded"),
            "status updated even for nonexistent"
        );
        assert_eq!(task.units(), 0);
    }
}
