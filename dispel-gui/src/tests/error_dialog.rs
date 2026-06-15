#[cfg(test)]
mod error_dialog_tests {
    use crate::app::App;
    use crate::message::system::SystemMessage;
    use crate::message::Message;
    use crate::workspace::Workspace;

    #[test]
    fn show_error_sets_dialog() {
        let mut app = App::test_new(Workspace::new());
        assert!(app.error_dialog.is_none());

        let task = app.update(Message::System(SystemMessage::ShowError(
            "Something went wrong".into(),
        )));
        assert_eq!(app.error_dialog.as_deref(), Some("Something went wrong"));
        assert_eq!(task.units(), 0);
    }

    #[test]
    fn dismiss_error_clears_dialog() {
        let mut app = App::test_new(Workspace::new());
        let _ = app.update(Message::System(SystemMessage::ShowError("error".into())));
        assert!(app.error_dialog.is_some());

        let task = app.update(Message::System(SystemMessage::DismissError));
        assert!(app.error_dialog.is_none());
        assert_eq!(task.units(), 0);
    }

    #[test]
    fn dismiss_when_no_error_does_not_panic() {
        let mut app = App::test_new(Workspace::new());
        let task = app.update(Message::System(SystemMessage::DismissError));
        assert!(app.error_dialog.is_none());
        assert_eq!(task.units(), 0);
    }

    #[test]
    fn show_error_overwrites_previous() {
        let mut app = App::test_new(Workspace::new());
        let _ = app.update(Message::System(SystemMessage::ShowError("first".into())));
        let _ = app.update(Message::System(SystemMessage::ShowError("second".into())));
        assert_eq!(app.error_dialog.as_deref(), Some("second"));
    }
}
