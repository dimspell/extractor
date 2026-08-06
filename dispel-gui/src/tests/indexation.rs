#[cfg(test)]
mod indexation_edge_tests {
    use crate::app::App;
    use crate::message::Message;
    use crate::message::system::SystemMessage;
    use crate::workspace::Workspace;

    #[test]
    fn system_index_complete_does_not_panic() {
        let mut app = App::test_new(Workspace::new());
        let task = app.update(Message::System(SystemMessage::IndexComplete));
        assert_eq!(task.units(), 0);
    }

    #[test]
    fn system_index_save_complete_does_not_panic() {
        let mut app = App::test_new(Workspace::new());
        let task = app.update(Message::System(SystemMessage::IndexSaveComplete));
        assert_eq!(task.units(), 0);
    }

    #[test]
    fn system_cache_indexation_failed_does_not_panic() {
        let mut app = App::test_new(Workspace::new());
        let task = app.update(Message::System(SystemMessage::CacheIndexationFailed));
        assert_eq!(task.units(), 0);
    }
}
