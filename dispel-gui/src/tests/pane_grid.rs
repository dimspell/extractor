#[cfg(test)]
mod pane_grid_tests {
    use crate::app::App;
    use crate::message::Message;
    use crate::message::workspace::WorkspaceMessage;
    use crate::workspace::Workspace;

    #[test]
    fn toggle_sidebar_hides_and_shows() {
        let mut app = App::test_new(Workspace::new());
        assert!(app.sidebar_visible, "sidebar visible by default");

        // Hide sidebar
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleSidebar));
        assert!(!app.sidebar_visible, "sidebar hidden");
        assert_eq!(
            app.state.pane_state.state.len(),
            1,
            "one pane when sidebar hidden"
        );

        // Show sidebar again
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleSidebar));
        assert!(app.sidebar_visible, "sidebar shown again");
        assert_eq!(
            app.state.pane_state.state.len(),
            2,
            "two panes when sidebar visible"
        );
    }

    #[test]
    fn toggle_sidebar_twice_restores_original_state() {
        let mut app = App::test_new(Workspace::new());
        let panes_before = app.state.pane_state.state.len();

        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleSidebar));
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleSidebar));

        assert!(app.sidebar_visible);
        assert_eq!(
            app.state.pane_state.state.len(),
            panes_before,
            "same number of panes after double-toggle"
        );
    }

    #[test]
    fn toggle_history_panel_shows_and_hides() {
        let mut app = App::test_new(Workspace::new());
        assert!(!app.history_panel_visible, "history hidden by default");

        // Show history panel
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleHistoryPanel));
        assert!(app.history_panel_visible);
        assert_eq!(
            app.state.pane_state.state.len(),
            3,
            "three panes with history panel"
        );

        // Hide history panel
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleHistoryPanel));
        assert!(!app.history_panel_visible);
        assert_eq!(
            app.state.pane_state.state.len(),
            2,
            "back to two panes"
        );
    }

    #[test]
    fn toggle_history_panel_with_hidden_sidebar() {
        let mut app = App::test_new(Workspace::new());

        // Hide sidebar first
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleSidebar));
        assert_eq!(app.state.pane_state.state.len(), 1, "only main content");

        // Show history panel (should split the single main pane)
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleHistoryPanel));
        assert!(app.history_panel_visible);
        assert_eq!(
            app.state.pane_state.state.len(),
            2,
            "main + history with no sidebar"
        );

        // Show sidebar again (should rebuild with all three)
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleSidebar));
        assert!(app.sidebar_visible);
        assert_eq!(
            app.state.pane_state.state.len(),
            3,
            "sidebar + main + history"
        );
    }

    #[test]
    fn toggle_maximize_pane_toggles_state() {
        let mut app = App::test_new(Workspace::new());
        assert!(app.state.pane_state.maximized.is_none(), "not maximized");

        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleMaximizePane));
        assert!(
            app.state.pane_state.maximized.is_some(),
            "maximized after toggle"
        );

        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleMaximizePane));
        assert!(
            app.state.pane_state.maximized.is_none(),
            "restored after second toggle"
        );
    }

    #[test]
    fn pane_clicked_changes_focus() {
        let mut app = App::test_new(Workspace::new());
        let initial_focus = app.state.pane_state.focus;

        // Get the other pane
        let other_pane = app
            .state
            .pane_state
            .state
            .iter()
            .find(|(id, _)| **id != initial_focus)
            .map(|(id, _)| *id)
            .expect("at least two panes in default layout");

        let _ = app.update(Message::Workspace(WorkspaceMessage::PaneClicked(other_pane)));
        assert_eq!(
            app.state.pane_state.focus,
            other_pane,
            "focus changed to clicked pane"
        );
    }

    #[test]
    fn pane_resized_does_not_panic() {
        let mut app = App::test_new(Workspace::new());

        // Get the sidebar split from the default layout
        let split = app.state.pane_state.sidebar_split.expect("has sidebar split");

        use iced::widget::pane_grid::ResizeEvent;
        let event = ResizeEvent { split, ratio: 0.3 };
        let _ = app.update(Message::Workspace(WorkspaceMessage::PaneResized(event)));
        // If we get here without panicking, the resize was handled
    }

    #[test]
    fn pane_dragged_does_not_panic() {
        let mut app = App::test_new(Workspace::new());

        // Collect pane IDs
        let panes: Vec<_> = app
            .state
            .pane_state
            .state
            .iter()
            .map(|(id, _)| *id)
            .collect();
        assert!(panes.len() >= 2, "at least two panes");

        use iced::widget::pane_grid::{DragEvent, Target};
        let _ = app.update(Message::Workspace(WorkspaceMessage::PaneDragged(
            DragEvent::Dropped {
                pane: panes[1],
                target: Target::Pane(panes[0], iced::widget::pane_grid::Region::Center),
            },
        )));
        // If we get here without panicking, the drop was handled
    }
}
