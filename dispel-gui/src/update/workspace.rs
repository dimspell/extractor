// Workspace message handlers

use crate::app::App;
use crate::components::tab_bar;
use crate::message::workspace::WorkspaceMessage;
use crate::message::Message;
use crate::update::file_tree;
use iced::widget::pane_grid;
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: WorkspaceMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        WorkspaceMessage::FileTree(msg) => file_tree::handle(msg, app),
        WorkspaceMessage::TabBar(msg) => {
            // Handle tab bar messages
            tab_bar::handle(msg, app)
        }
        WorkspaceMessage::ToggleSidebar => {
            app.sidebar_visible = !app.sidebar_visible;
            Task::none()
        }
        WorkspaceMessage::ToggleGlobalSearch => {
            app.global_search.toggle();
            if app.global_search.is_visible {
                app.command_palette = None;
                return iced::widget::operation::focus(
                    crate::global_search::GlobalSearch::input_id(),
                );
            }
            Task::none()
        }
        WorkspaceMessage::ToggleHistoryPanel => {
            app.history_panel_visible = !app.history_panel_visible;
            if app.history_panel_visible {
                // Split the focused pane to add a history panel
                if let Some((new_pane, _split)) = app.state.pane_state.state.split(
                    pane_grid::Axis::Vertical,
                    app.state.pane_state.focus,
                    crate::state::state::PaneContent::HistoryPanel,
                ) {
                    app.state.pane_state.focus = new_pane;
                }
            } else {
                // Find and close the history panel pane
                let panes: Vec<_> = app
                    .state
                    .pane_state
                    .state
                    .iter()
                    .filter_map(|(id, content)| {
                        if matches!(content, crate::state::state::PaneContent::HistoryPanel) {
                            Some(*id)
                        } else {
                            None
                        }
                    })
                    .collect();
                for pane_id in panes {
                    if app.state.pane_state.state.len() > 1 {
                        if let Some((_, sibling)) = app.state.pane_state.state.close(pane_id) {
                            app.state.pane_state.focus = sibling;
                        }
                    }
                }
            }
            Task::none()
        }
        WorkspaceMessage::ToggleMaximizePane => {
            if app.state.pane_state.maximized.is_some() {
                app.state.pane_state.state.restore();
                app.state.pane_state.maximized = None;
            } else {
                let focus = app.state.pane_state.focus;
                app.state.pane_state.state.maximize(focus);
                app.state.pane_state.maximized = Some(focus);
            }
            Task::none()
        }
        // Pane messages
        WorkspaceMessage::PaneClicked(pane) => {
            app.state.pane_state.focus = pane;
            Task::none()
        }
        WorkspaceMessage::PaneResized(event) => {
            app.state.pane_state.state.resize(event.split, event.ratio);
            Task::none()
        }
        WorkspaceMessage::PaneDragged(event) => {
            if let iced::widget::pane_grid::DragEvent::Dropped { pane, target } = event {
                app.state.pane_state.state.drop(pane, target);
            }
            Task::none()
        }
        // Command Palette
        WorkspaceMessage::ToggleCommandPalette => {
            if app.command_palette.is_some() {
                app.command_palette = None;
                Task::none()
            } else {
                app.command_palette =
                    Some(crate::components::command_palette::CommandPalette::new());
                iced::widget::operation::focus(
                    crate::components::command_palette::CommandPalette::input_id(),
                )
            }
        }
        WorkspaceMessage::CommandPaletteInput(input) => {
            if let Some(palette) = &mut app.command_palette {
                palette.update_input(input);
            }
            Task::none()
        }
        WorkspaceMessage::CommandPaletteSelect(index) => {
            if let Some(palette) = &app.command_palette {
                if let Some(cmd) = palette.filtered_commands.get(index) {
                    let action_msg = (cmd.action)();
                    app.command_palette = None;
                    return app.update(action_msg);
                }
            }
            Task::none()
        }
        WorkspaceMessage::CommandPaletteClose => {
            app.command_palette = None;
            Task::none()
        }
        WorkspaceMessage::CommandPaletteConfirm => {
            if let Some(palette) = &app.command_palette {
                if let Some(cmd) = palette.selected_command() {
                    let action_msg = (cmd.action)();
                    app.command_palette = None;
                    return app.update(action_msg);
                }
            }
            Task::none()
        }
        WorkspaceMessage::CommandPaletteArrowUp => {
            if let Some(palette) = &mut app.command_palette {
                palette.select_previous();
            }
            Task::none()
        }
        WorkspaceMessage::CommandPaletteArrowDown => {
            if let Some(palette) = &mut app.command_palette {
                palette.select_next();
            }
            Task::none()
        }
        // Global Search
        WorkspaceMessage::GlobalSearchInput(input) => {
            app.global_search.query = input.clone();

            // Optimize: Only search if query has minimum length
            if input.len() >= 2 || input.is_empty() {
                // Use async search to keep UI responsive
                let query = input.clone();
                return Task::perform(
                    async move {
                        // Simulate async work (in real app, this would be actual async I/O)
                        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                        query
                    },
                    |query| Message::Workspace(WorkspaceMessage::GlobalSearchAsync(query)),
                );
            } else if !input.is_empty() {
                // Clear results for very short queries
                app.global_search.results.clear();
                app.global_search.selected_index = 0;
            }
            Task::none()
        }
        WorkspaceMessage::GlobalSearchSelect(index) => {
            if let Some(result) = app.global_search.results.get(index) {
                if let Some(relative_path) = &result.source_file {
                    // Construct full path by combining game path with relative path
                    if !app.state.shared_game_path.is_empty() {
                        let full_path =
                            PathBuf::from(&app.state.shared_game_path).join(relative_path);
                        // Close search dialog and clear query before opening file
                        app.global_search.is_visible = false;
                        app.global_search.query.clear();
                        return app.open_file_in_workspace(&full_path);
                    }
                }
            }
            app.global_search.is_visible = false;
            app.global_search.query.clear();
            Task::none()
        }
        WorkspaceMessage::GlobalSearchAsync(query) => {
            // Process async search query on main thread
            app.global_search.results.clear();
            app.global_search.selected_index = 0;

            if query.is_empty() {
                Task::none()
            } else {
                let file_results = app.search_index.search_files(&query);
                for entry in file_results.iter().take(app.global_search.max_results) {
                    app.global_search
                        .results
                        .push(crate::global_search::SearchResult {
                            catalog_type: entry.editor_type.clone(),
                            record_idx: 0,
                            display_text: entry.file_path.clone(),
                            source_file: Some(entry.file_path.clone()),
                        });
                }
                Task::none()
            }
        }

        WorkspaceMessage::GlobalSearchConfirm => {
            if let Some(selected_index) = app.global_search.selected_index.checked_sub(0) {
                if let Some(result) = app.global_search.results.get(selected_index) {
                    if let Some(relative_path) = &result.source_file {
                        // Construct full path by combining game path with relative path
                        if !app.state.shared_game_path.is_empty() {
                            let full_path =
                                PathBuf::from(&app.state.shared_game_path).join(relative_path);
                            // Close search dialog and clear query before opening file
                            app.global_search.is_visible = false;
                            app.global_search.query.clear();
                            return app.open_file_in_workspace(&full_path);
                        }
                    }
                }
            }
            app.global_search.is_visible = false;
            app.global_search.query.clear();
            Task::none()
        }
        WorkspaceMessage::GlobalSearchArrowUp => {
            app.global_search.select_previous();
            Task::none()
        }
        WorkspaceMessage::GlobalSearchArrowDown => {
            app.global_search.select_next();
            Task::none()
        }

        // Tools
        WorkspaceMessage::OpenToolTab(editor_type) => {
            use crate::message::editor::localization::LocalizationMessage;
            use crate::message::MessageExt;
            use crate::workspace::EditorType;
            let label = match editor_type {
                EditorType::DbViewer => "DB Viewer",
                EditorType::ChestEditor => "Chest Editor",
                EditorType::LocalizationManager => "Localization Packager",
                _ => "Tool",
            };
            app.state
                .workspace
                .open_tool(label.to_string(), editor_type);
            // B3: auto-scan when opening Localization Manager with game path set and no entries loaded
            if editor_type == EditorType::LocalizationManager
                && app.state.localization_manager.entries.is_empty()
                && !app.state.shared_game_path.is_empty()
            {
                return Task::done(crate::message::Message::localization(
                    LocalizationMessage::Scan,
                ));
            }
            Task::none()
        }
    }
}
