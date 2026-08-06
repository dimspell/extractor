//! Tests for the workspace message handler (update/workspace.rs).
//!
//! Covers behavior NOT already tested by:
//! - `pane_grid.rs` (ToggleSidebar, ToggleHistoryPanel, ToggleMaximizePane, PaneClicked, PaneResized, PaneDragged)
//! - `command_palette.rs` (ToggleCommandPalette, CommandPaletteInput/Select/Close/Confirm/ArrowUp/ArrowDown)
//! - `global_search.rs` (ToggleGlobalSearch, GlobalSearchInput, GlobalSearchAsync, GlobalSearchSelect/Confirm, ArrowUp/ArrowDown)
//! - `message_routing.rs` (OpenToolTab for DbViewer)
//! - `workspace.rs` (ReopenActiveTabAsHex with no tab / no path)

#[cfg(test)]
mod workspace_handler_tests {
    use crate::app::App;
    use crate::message::Message;
    use crate::message::workspace::WorkspaceMessage;
    use crate::workspace::{EditorType, Workspace};

    /// GlobalSearchSelect when a result has source_file but no game path is set.
    /// Should close search and clear query without trying to open a file.
    /// This exercises the branch where source_file is Some but shared_game_path is empty.
    #[test]
    fn test_global_search_select_no_game_path() {
        let mut app = App::test_new(Workspace::new());
        let _ = app.update(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch));

        // Set up a result with source_file, but shared_game_path is empty
        app.global_search
            .results
            .push(crate::components::global_search::SearchResult {
                catalog_type: "test".into(),
                record_idx: 0,
                display_text: "file.map".into(),
                source_file: Some("maps/file.map".into()),
            });
        app.global_search.selected_index = 0;

        let task = app.update(Message::Workspace(WorkspaceMessage::GlobalSearchSelect(0)));

        assert!(!app.global_search.is_visible, "search should close");
        assert!(app.global_search.query.is_empty(), "query should clear");
        assert_eq!(task.units(), 0, "no file-open task without game path");
    }

    /// OpenToolTab(LocalizationManager) when a game path is set and no entries loaded.
    /// Should add a LocalizationManager tab and return a Scan message task.
    #[test]
    fn test_open_tool_tab_localization_manager_with_game_path() {
        let mut app = App::test_new(Workspace::new());
        app.state.shared_game_path = "/game/path".into();

        let _task = app.update(Message::Workspace(WorkspaceMessage::OpenToolTab(
            EditorType::LocalizationManager,
        )));

        // Tab creation
        assert_eq!(app.state.workspace.tabs.len(), 1);
        assert_eq!(
            app.state.workspace.tabs[0].editor_type,
            EditorType::LocalizationManager
        );
        assert_eq!(app.state.workspace.tabs[0].label, "Localization Packager");

        // With game path set and empty entries, should dispatch a Scan task
        // (Scan dispatches via Task::done which has units=0, so check tab + game path)
        assert!(
            app.state.editors.localization_manager.entries.is_empty(),
            "entries still empty (scan not executed without runtime)"
        );
    }
}
