#[cfg(test)]
mod command_palette_tests {
    use crate::app::App;
    use crate::message::Message;
    use crate::message::workspace::WorkspaceMessage;
    use crate::workspace::Workspace;

    #[test]
    fn toggle_opens_and_closes() {
        let mut app = App::test_new(Workspace::new());
        assert!(app.command_palette.is_none(), "starts closed");

        let task = app.update(Message::Workspace(WorkspaceMessage::ToggleCommandPalette));
        assert!(app.command_palette.is_some(), "opened after toggle");
        assert!(task.units() > 0, "returns focus task when opening");

        let task = app.update(Message::Workspace(WorkspaceMessage::ToggleCommandPalette));
        assert!(app.command_palette.is_none(), "closed after second toggle");
        assert_eq!(task.units(), 0, "no task when closing");
    }

    #[test]
    fn close_clears_state() {
        let mut app = App::test_new(Workspace::new());
        app.update(Message::Workspace(WorkspaceMessage::ToggleCommandPalette));
        assert!(app.command_palette.is_some());

        let task = app.update(Message::Workspace(WorkspaceMessage::CommandPaletteClose));
        assert!(app.command_palette.is_none(), "closed via CommandPaletteClose");
        assert_eq!(task.units(), 0);
    }

    #[test]
    fn input_updates_query_and_filters() {
        let mut app = App::test_new(Workspace::new());
        app.update(Message::Workspace(WorkspaceMessage::ToggleCommandPalette));

        let task = app.update(Message::Workspace(WorkspaceMessage::CommandPaletteInput(
            "undo".into(),
        )));
        let palette = app.command_palette.as_ref().unwrap();
        assert_eq!(palette.input_value, "undo", "input value updated");
        assert!(
            palette.filtered_commands.len() < palette.all_commands.len(),
            "filtered reduced from all"
        );
        assert!(
            palette.filtered_commands.iter().any(|c| c.id == "undo"),
            "undo command in filtered results"
        );
        assert_eq!(task.units(), 0, "input returns no task");
    }

    #[test]
    fn confirm_fires_action_and_re_dispatches() {
        // The confirm handler calls app.update(action_msg) for the selected
        // command, re-dispatching it through the update loop.
        let mut app = App::test_new(Workspace::new());
        app.update(Message::Workspace(WorkspaceMessage::ToggleCommandPalette));

        // Filter to "sidebar" (matches against label "Toggle Sidebar")
        app.update(Message::Workspace(WorkspaceMessage::CommandPaletteInput(
            "sidebar".into(),
        )));
        assert!(app.sidebar_visible, "sidebar starts visible");
        assert!(!app.command_palette.as_ref().unwrap().filtered_commands.is_empty(),
            "at least one command matches 'sidebar'");

        // Confirm fires ToggleSidebar (via selected_command + re-dispatch)
        let _ = app.update(Message::Workspace(WorkspaceMessage::CommandPaletteConfirm));
        assert!(!app.sidebar_visible, "sidebar toggled by confirmed command");
    }

    #[test]
    fn confirm_on_empty_palette_does_not_panic() {
        let mut app = App::test_new(Workspace::new());
        let task = app.update(Message::Workspace(WorkspaceMessage::CommandPaletteConfirm));
        assert_eq!(task.units(), 0);
    }

    #[test]
    fn select_fires_action_and_re_dispatches() {
        let mut app = App::test_new(Workspace::new());
        app.update(Message::Workspace(WorkspaceMessage::ToggleCommandPalette));

        // Filter to "history" (matches label "Toggle Edit History")
        app.update(Message::Workspace(WorkspaceMessage::CommandPaletteInput(
            "history".into(),
        )));

        let idx = app
            .command_palette
            .as_ref()
            .unwrap()
            .filtered_commands
            .iter()
            .position(|c| c.id == "toggle-history")
            .expect("toggle-history command found");
        assert!(!app.history_panel_visible, "history panel starts hidden");

        // Select fires ToggleHistoryPanel (via action + re-dispatch)
        let _ = app.update(Message::Workspace(WorkspaceMessage::CommandPaletteSelect(idx)));
        assert!(app.history_panel_visible, "history panel toggled by selected command");
    }

    #[test]
    fn select_out_of_bounds_does_not_close_palette() {
        let mut app = App::test_new(Workspace::new());
        app.update(Message::Workspace(WorkspaceMessage::ToggleCommandPalette));

        let task = app.update(Message::Workspace(WorkspaceMessage::CommandPaletteSelect(999)));
        // Handler only sets palette to None when it finds a valid command
        assert!(app.command_palette.is_some(), "palette still open on bad index");
        assert_eq!(task.units(), 0);
    }

    #[test]
    fn arrow_down_selects_next() {
        let mut app = App::test_new(Workspace::new());
        app.update(Message::Workspace(WorkspaceMessage::ToggleCommandPalette));

        let initial = app.command_palette.as_ref().unwrap().selected_index;
        let task = app.update(Message::Workspace(WorkspaceMessage::CommandPaletteArrowDown));
        let new_idx = app.command_palette.as_ref().unwrap().selected_index;
        let count = app.command_palette.as_ref().unwrap().filtered_commands.len();
        assert_eq!(
            new_idx, (initial + 1) % count,
            "selection wraps forward"
        );
        assert!(task.units() > 0, "returns scroll task");
    }

    #[test]
    fn arrow_up_selects_previous() {
        let mut app = App::test_new(Workspace::new());
        app.update(Message::Workspace(WorkspaceMessage::ToggleCommandPalette));

        // Move down once so we're not at 0
        app.update(Message::Workspace(WorkspaceMessage::CommandPaletteArrowDown));
        let idx_after_down = app.command_palette.as_ref().unwrap().selected_index;

        let task = app.update(Message::Workspace(WorkspaceMessage::CommandPaletteArrowUp));
        let idx_after_up = app.command_palette.as_ref().unwrap().selected_index;
        let count = app.command_palette.as_ref().unwrap().filtered_commands.len();
        assert_eq!(
            idx_after_up,
            (idx_after_down + count - 1) % count,
            "selection wraps backward"
        );
        assert!(task.units() > 0, "returns scroll task");
    }

    #[test]
    fn arrow_on_closed_palette_does_not_panic() {
        let mut app = App::test_new(Workspace::new());
        let task = app.update(Message::Workspace(WorkspaceMessage::CommandPaletteArrowDown));
        assert_eq!(task.units(), 0);
        let task = app.update(Message::Workspace(WorkspaceMessage::CommandPaletteArrowUp));
        assert_eq!(task.units(), 0);
    }
}
