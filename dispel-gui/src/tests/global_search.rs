#[cfg(test)]
mod global_search_tests {
    use crate::app::App;
    use crate::message::Message;
    use crate::message::workspace::WorkspaceMessage;
    use crate::workspace::Workspace;

    #[test]
    fn toggle_shows_and_hides() {
        let mut app = App::test_new(Workspace::new());
        assert!(!app.global_search.is_visible, "starts hidden");

        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch));
        assert!(app.global_search.is_visible, "shown after toggle");
        assert!(
            app.command_palette.is_none(),
            "command palette closed when search opens"
        );

        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch));
        assert!(!app.global_search.is_visible, "hidden after second toggle");
        assert!(app.global_search.query.is_empty(), "query cleared on hide");
        assert!(app.global_search.results.is_empty(), "results cleared on hide");
    }

    #[test]
    fn input_two_chars_returns_async_task() {
        let mut app = App::test_new(Workspace::new());
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch));

        // 2+ chars triggers async search
        let task = app.update(Message::Workspace(WorkspaceMessage::GlobalSearchInput(
            "te".into(),
        )));
        assert_eq!(app.global_search.query, "te");
        assert!(task.units() > 0, "async search spawned for 2+ chars");
    }

    #[test]
    fn input_empty_returns_async_task() {
        let mut app = App::test_new(Workspace::new());
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch));

        // Empty input also triggers async search (resets results)
        let task = app.update(Message::Workspace(WorkspaceMessage::GlobalSearchInput(
            String::new(),
        )));
        assert!(task.units() > 0, "async search spawned for empty input");
    }

    #[test]
    fn input_one_char_clears_results() {
        let mut app = App::test_new(Workspace::new());
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch));

        // Set some results first
        app.global_search.results.push(
            crate::components::global_search::SearchResult {
                catalog_type: "test".into(),
                record_idx: 0,
                display_text: "foo".into(),
                source_file: None,
            },
        );
        app.global_search.selected_index = 0;

        // Single char clears results
        let task = app.update(Message::Workspace(WorkspaceMessage::GlobalSearchInput(
            "a".into(),
        )));
        assert_eq!(app.global_search.query, "a");
        assert!(app.global_search.results.is_empty(), "results cleared");
        assert_eq!(app.global_search.selected_index, 0, "selection reset");
        assert_eq!(task.units(), 0, "no async task for 1-char input");
    }

    #[test]
    fn select_closes_search() {
        let mut app = App::test_new(Workspace::new());
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch));

        // Set up a result
        app.global_search.results.push(
            crate::components::global_search::SearchResult {
                catalog_type: "test".into(),
                record_idx: 0,
                display_text: "foo".into(),
                source_file: None,
            },
        );

        let task = app.update(Message::Workspace(WorkspaceMessage::GlobalSearchSelect(0)));
        assert!(!app.global_search.is_visible, "search closed after select");
        assert!(app.global_search.query.is_empty(), "query cleared");
        assert_eq!(task.units(), 0, "no task when no game path");
    }

    #[test]
    fn select_out_of_bounds_does_not_panic() {
        let mut app = App::test_new(Workspace::new());
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch));
        let task = app.update(Message::Workspace(WorkspaceMessage::GlobalSearchSelect(999)));
        assert!(!app.global_search.is_visible, "closed on bad index");
        assert!(app.global_search.query.is_empty(), "cleared on bad index");
        assert_eq!(task.units(), 0);
    }

    #[test]
    fn arrow_down_selects_next() {
        let mut app = App::test_new(Workspace::new());
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch));

        // Add some results so navigation works
        for i in 0..3 {
            app.global_search.results.push(
                crate::components::global_search::SearchResult {
                    catalog_type: "test".into(),
                    record_idx: i,
                    display_text: format!("file_{i}"),
                    source_file: None,
                },
            );
        }

        assert_eq!(app.global_search.selected_index, 0);
        let _ = app.update(Message::Workspace(WorkspaceMessage::GlobalSearchArrowDown));
        assert_eq!(app.global_search.selected_index, 1);
        let _ = app.update(Message::Workspace(WorkspaceMessage::GlobalSearchArrowDown));
        assert_eq!(app.global_search.selected_index, 2);
        let _ = app.update(Message::Workspace(WorkspaceMessage::GlobalSearchArrowDown));
        assert_eq!(app.global_search.selected_index, 0, "wraps to start");
    }

    #[test]
    fn arrow_up_selects_previous() {
        let mut app = App::test_new(Workspace::new());
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch));

        for i in 0..3 {
            app.global_search.results.push(
                crate::components::global_search::SearchResult {
                    catalog_type: "test".into(),
                    record_idx: i,
                    display_text: format!("file_{i}"),
                    source_file: None,
                },
            );
        }

        // Start at 0, press up → wraps to last
        let _ = app.update(Message::Workspace(WorkspaceMessage::GlobalSearchArrowUp));
        assert_eq!(app.global_search.selected_index, 2, "wraps to last");
    }

    #[test]
    fn confirm_without_results_does_not_panic() {
        let mut app = App::test_new(Workspace::new());
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch));
        let task = app.update(Message::Workspace(WorkspaceMessage::GlobalSearchConfirm));
        assert!(!app.global_search.is_visible, "closed");
        assert_eq!(task.units(), 0);
    }
}

#[cfg(test)]
mod global_search_confirm_tests {
    use crate::app::App;
    use crate::message::workspace::WorkspaceMessage;
    use crate::message::Message;
    use crate::workspace::Workspace;

    #[test]
    fn confirm_with_selection_index_opens_file_when_game_path_set() {
        let mut app = App::test_new(Workspace::new());
        app.state.shared_game_path = "/game/path".into();
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch));

        // Add a result with a source file
        app.global_search.results.push(
            crate::components::global_search::SearchResult {
                catalog_type: "test".into(),
                record_idx: 0,
                display_text: "file.map".into(),
                source_file: Some("maps/file.map".into()),
            },
        );
        app.global_search.selected_index = 0;

        let task = app.update(Message::Workspace(WorkspaceMessage::GlobalSearchConfirm));
        // With a game path + valid result, it should return an open-file task
        assert!(task.units() > 0, "should open file on confirm");
        assert!(!app.global_search.is_visible, "search closed");
    }

    #[test]
    fn confirm_closes_search_when_no_game_path() {
        let mut app = App::test_new(Workspace::new());
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch));

        app.global_search.results.push(
            crate::components::global_search::SearchResult {
                catalog_type: "test".into(),
                record_idx: 0,
                display_text: "file.map".into(),
                source_file: Some("maps/file.map".into()),
            },
        );

        let task = app.update(Message::Workspace(WorkspaceMessage::GlobalSearchConfirm));
        assert!(!app.global_search.is_visible, "search closed");
        assert!(app.global_search.query.is_empty(), "query cleared");
        assert_eq!(task.units(), 0, "no file task when no game path");
    }
}
