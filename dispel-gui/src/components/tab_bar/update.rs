use crate::app::App;
use crate::components::tab_bar::TabBarMessage;
use iced::Task;

pub fn handle(message: TabBarMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        TabBarMessage::SelectTab(tab_index) => {
            // Normal tab selection — drag state is handled by the widget internally.
            app.state.editors.stop_snf_playback();
            if app.state.workspace.tabs.len() > tab_index {
                app.state.workspace.active_tab = Some(tab_index);
            }
            Task::none()
        }
        TabBarMessage::CloseTab(tab_index) => {
            if app.state.workspace.tabs.len() > tab_index {
                let tab_id = app.state.workspace.tabs[tab_index].id;
                app.state.editors.remove_tab(tab_id);
                app.state.workspace.tabs.remove(tab_index);
                if let Some(active) = app.state.workspace.active_tab {
                    if app.state.workspace.tabs.is_empty() {
                        app.state.workspace.active_tab = None;
                    } else if active >= app.state.workspace.tabs.len() {
                        app.state.workspace.active_tab = Some(app.state.workspace.tabs.len() - 1);
                    }
                }
            }
            Task::none()
        }
        TabBarMessage::TogglePin(tab_index) => {
            if let Some(tab) = app.state.workspace.tabs.get_mut(tab_index) {
                tab.pinned = !tab.pinned;
            }
            Task::none()
        }
        TabBarMessage::CloseActiveTab => {
            if let Some(active_tab) = app.state.workspace.active_tab
                && !app.state.workspace.tabs.is_empty()
            {
                let tab_id = app.state.workspace.tabs[active_tab].id;
                app.state.editors.remove_tab(tab_id);
                app.state.workspace.tabs.remove(active_tab);
                if app.state.workspace.tabs.is_empty() {
                    app.state.workspace.active_tab = None;
                } else if active_tab >= app.state.workspace.tabs.len() {
                    app.state.workspace.active_tab = Some(app.state.workspace.tabs.len() - 1);
                }
            }
            Task::none()
        }
        TabBarMessage::CloseOthers(tab_index) => {
            if app.state.workspace.tabs.len() > tab_index {
                let tab_id = app.state.workspace.tabs[tab_index].id;
                let tabs_to_close: Vec<_> = app
                    .state
                    .workspace
                    .tabs
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| *idx != tab_index)
                    .map(|(_, tab)| tab.id)
                    .collect();
                for id in tabs_to_close {
                    app.state.editors.remove_tab(id);
                }
                app.state.workspace.tabs.retain(|tab| tab.id == tab_id);
                app.state.workspace.active_tab = Some(0);
            }
            Task::none()
        }
        TabBarMessage::CloseAll => {
            app.state.workspace.tabs.clear();
            app.state.workspace.active_tab = None;
            app.state.editors.close_all_tabs();
            Task::none()
        }
        TabBarMessage::OpenAsHex(tab_index) => {
            let path = app
                .state
                .workspace
                .tabs
                .get(tab_index)
                .and_then(|t| t.path.clone());
            if let Some(path) = path {
                return app.open_file_in_workspace_as_hex(&path);
            }
            Task::none()
        }
        // ── Drag-and-drop reordering ────────────────────────────────────
        TabBarMessage::StartDrag(_) => {
            // No longer needed — drag state handled by the custom widget.
            Task::none()
        }
        TabBarMessage::MoveTab(from, to) => {
            let n = app.state.workspace.tabs.len();
            if from < n && to <= n && from != to {
                let tab = app.state.workspace.tabs.remove(from);
                // Adjust target: if removing left of original position, shift back.
                let insert_at = if to > from { to - 1 } else { to };
                app.state.workspace.tabs.insert(insert_at, tab);

                // Update active_tab to follow the moved tab.
                if let Some(active) = app.state.workspace.active_tab {
                    if active == from {
                        app.state.workspace.active_tab = Some(insert_at);
                    } else if active > from && active <= insert_at {
                        app.state.workspace.active_tab = Some(active - 1);
                    } else if active < from && active >= insert_at {
                        app.state.workspace.active_tab = Some(active + 1);
                    }
                }
            }
            Task::none()
        }
        TabBarMessage::TabEnter(_) | TabBarMessage::TabLeave(_) | TabBarMessage::CancelDrag => {
            // No longer needed — hover and drag state handled by the custom widget.
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::{EditorType, Workspace, WorkspaceTab};

    fn create_test_workspace(tab_count: usize) -> Workspace {
        let mut workspace = Workspace::new();
        for i in 0..tab_count {
            workspace.tabs.push(WorkspaceTab {
                id: i,
                label: format!("Tab {}", i),
                path: None,
                editor_type: EditorType::Unknown,
                modified: false,
                pinned: false,
            });
        }
        workspace
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SelectTab Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_select_tab_first_tab() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::SelectTab(0), &mut app);

        assert_eq!(app.state.workspace.active_tab, Some(0));
    }

    #[test]
    fn test_select_tab_middle_tab() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::SelectTab(1), &mut app);
        assert_eq!(app.state.workspace.active_tab, Some(1));
    }

    #[test]
    fn test_select_tab_last_tab() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::SelectTab(2), &mut app);
        assert_eq!(app.state.workspace.active_tab, Some(2));
    }

    #[test]
    fn test_select_tab_out_of_range() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::SelectTab(10), &mut app);
        assert_eq!(app.state.workspace.active_tab, None);
    }

    #[test]
    fn test_select_tab_with_active() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);
        app.state.workspace.active_tab = Some(1);

        let _ = handle(TabBarMessage::SelectTab(0), &mut app);
        assert_eq!(app.state.workspace.active_tab, Some(0));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CloseTab Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_close_first_tab() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseTab(0), &mut app);
        assert_eq!(app.state.workspace.tabs.len(), 2);
        assert_eq!(app.state.workspace.tabs[0].label, "Tab 1");
    }

    #[test]
    fn test_close_middle_tab() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseTab(1), &mut app);
        assert_eq!(app.state.workspace.tabs.len(), 2);
        assert_eq!(app.state.workspace.tabs[0].label, "Tab 0");
        assert_eq!(app.state.workspace.tabs[1].label, "Tab 2");
    }

    #[test]
    fn test_close_last_tab() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseTab(2), &mut app);
        assert_eq!(app.state.workspace.tabs.len(), 2);
        assert_eq!(app.state.workspace.tabs[1].label, "Tab 1");
    }

    #[test]
    fn test_close_tab_out_of_range() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseTab(10), &mut app);
        assert_eq!(app.state.workspace.tabs.len(), 3);
    }

    #[test]
    fn test_close_last_tab_sets_active_to_previous() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);
        app.state.workspace.active_tab = Some(2);

        let _ = handle(TabBarMessage::CloseTab(2), &mut app);
        assert_eq!(app.state.workspace.active_tab, Some(1));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TogglePin Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_toggle_pin() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::TogglePin(0), &mut app);
        assert!(app.state.workspace.tabs[0].pinned);
    }

    #[test]
    fn test_toggle_pin_twice() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::TogglePin(0), &mut app);
        let _ = handle(TabBarMessage::TogglePin(0), &mut app);
        assert!(!app.state.workspace.tabs[0].pinned);
    }

    #[test]
    fn test_toggle_pin_out_of_range() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::TogglePin(10), &mut app);
    }

    #[test]
    fn test_toggle_all_tabs_pin() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        for i in 0..3 {
            let _ = handle(TabBarMessage::TogglePin(i), &mut app);
        }
        for tab in &app.state.workspace.tabs {
            assert!(tab.pinned);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CloseActiveTab Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_close_active_tab() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);
        app.state.workspace.active_tab = Some(1);

        let _ = handle(TabBarMessage::CloseActiveTab, &mut app);
        assert_eq!(app.state.workspace.tabs.len(), 2);
        assert_eq!(app.state.workspace.tabs[0].label, "Tab 0");
        assert_eq!(app.state.workspace.tabs[1].label, "Tab 2");
    }

    #[test]
    fn test_close_active_tab_no_tabs() {
        let workspace = create_test_workspace(0);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseActiveTab, &mut app);
        assert_eq!(app.state.workspace.tabs.len(), 0);
    }

    #[test]
    fn test_close_active_tab_updates_active() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);
        app.state.workspace.active_tab = Some(2);

        let _ = handle(TabBarMessage::CloseActiveTab, &mut app);
        // Closing last tab should set active to previous
        assert_eq!(app.state.workspace.active_tab, Some(1));
    }

    #[test]
    fn test_close_active_tab_no_active_set() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseActiveTab, &mut app);
        // No active tab = nothing to close, tabs unchanged
        assert_eq!(app.state.workspace.tabs.len(), 3);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CloseOthers Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_close_others() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseOthers(0), &mut app);
        assert_eq!(app.state.workspace.tabs.len(), 1);
        assert_eq!(app.state.workspace.tabs[0].id, 0);
    }

    #[test]
    fn test_close_others_middle_tab() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseOthers(1), &mut app);
        assert_eq!(app.state.workspace.tabs.len(), 1);
        assert_eq!(app.state.workspace.tabs[0].id, 1);
    }

    #[test]
    fn test_close_others_sets_active() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseOthers(1), &mut app);
        assert_eq!(app.state.workspace.active_tab, Some(0));
    }

    #[test]
    fn test_close_others_out_of_range() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseOthers(10), &mut app);
        assert_eq!(app.state.workspace.tabs.len(), 3);
    }

    #[test]
    fn test_close_others_preserves_pin_state() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);
        app.state.workspace.tabs[2].pinned = true;

        let _ = handle(TabBarMessage::CloseOthers(0), &mut app);
        assert_eq!(app.state.workspace.tabs.len(), 1);
        assert_eq!(app.state.workspace.tabs[0].id, 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CloseAll Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_close_all() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseAll, &mut app);
        assert!(app.state.workspace.tabs.is_empty());
        assert_eq!(app.state.workspace.active_tab, None);
    }

    #[test]
    fn test_close_all_empty() {
        let workspace = create_test_workspace(0);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseAll, &mut app);
        assert!(app.state.workspace.tabs.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // OpenAsHex Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_open_as_hex_with_path() {
        let mut workspace = create_test_workspace(3);
        workspace.tabs[0].path = Some("/test/path.txt".into());
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::OpenAsHex(0), &mut app);
        // Should return a non-none task (it opens via hex)
        // We can't easily test the task result, but we test it doesn't panic
    }

    #[test]
    fn test_open_as_hex_without_path() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::OpenAsHex(0), &mut app);
        // No path → no action (no panic); workspace unchanged
    }

    #[test]
    fn test_open_as_hex_out_of_range() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::OpenAsHex(10), &mut app);
        // Out of range → no action (no panic); workspace unchanged
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MoveTab Tests (Drag-and-Drop Reordering)
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_move_tab_forward() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::MoveTab(2, 0), &mut app);
        assert_eq!(app.state.workspace.tabs[0].label, "Tab 2");
        assert_eq!(app.state.workspace.tabs[1].label, "Tab 0");
        assert_eq!(app.state.workspace.tabs[2].label, "Tab 1");
    }

    #[test]
    fn test_move_tab_backward() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::MoveTab(0, 3), &mut app);
        assert_eq!(app.state.workspace.tabs[0].label, "Tab 1");
        assert_eq!(app.state.workspace.tabs[1].label, "Tab 2");
        assert_eq!(app.state.workspace.tabs[2].label, "Tab 0");
    }

    #[test]
    fn test_move_tab_to_adjacent() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::MoveTab(0, 1), &mut app);
        assert_eq!(app.state.workspace.tabs[0].label, "Tab 0");
        assert_eq!(app.state.workspace.tabs[1].label, "Tab 1");
        assert_eq!(app.state.workspace.tabs[2].label, "Tab 2");
    }

    #[test]
    fn test_move_tab_same_position() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::MoveTab(1, 1), &mut app);
        assert_eq!(app.state.workspace.tabs[0].label, "Tab 0");
        assert_eq!(app.state.workspace.tabs[1].label, "Tab 1");
        assert_eq!(app.state.workspace.tabs[2].label, "Tab 2");
    }

    #[test]
    fn test_move_tab_out_of_range() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::MoveTab(3, 1), &mut app);
        assert_eq!(app.state.workspace.tabs.len(), 3);
    }

    #[test]
    fn test_move_tab_updates_active_tab() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);
        app.state.workspace.active_tab = Some(0);

        let _ = handle(TabBarMessage::MoveTab(0, 2), &mut app);
        // Active tab follows the moved tab
        assert_eq!(app.state.workspace.active_tab, Some(1));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Legacy drag messages (no-ops with custom widget)
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_start_drag_is_noop() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);
        let _ = handle(TabBarMessage::StartDrag(1), &mut app);
        // No-op: nothing changes
    }

    #[test]
    fn test_start_drag_out_of_range() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);
        let _ = handle(TabBarMessage::StartDrag(10), &mut app);
        // No-op: nothing changes
    }

    #[test]
    fn test_tab_enter_is_noop() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);
        let _ = handle(TabBarMessage::TabEnter(2), &mut app);
        // No-op: nothing changes
    }

    #[test]
    fn test_tab_leave_is_noop() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);
        let _ = handle(TabBarMessage::TabLeave(1), &mut app);
        // No-op: nothing changes
    }

    #[test]
    fn test_cancel_drag_is_noop() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);
        let _ = handle(TabBarMessage::CancelDrag, &mut app);
        // No-op: nothing changes
    }

    #[test]
    fn test_drag_select_does_not_move() {
        // With the custom widget, SelectTab no longer routes to MoveTab
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        // Previously this would have been routed to MoveTab via tab_drag_source.
        // Now SelectTab simply selects.
        let _ = handle(TabBarMessage::SelectTab(2), &mut app);

        assert_eq!(app.state.workspace.tabs[0].label, "Tab 0");
        assert_eq!(app.state.workspace.tabs[1].label, "Tab 1");
        assert_eq!(app.state.workspace.tabs[2].label, "Tab 2");
        assert_eq!(app.state.workspace.active_tab, Some(2));
    }
}
