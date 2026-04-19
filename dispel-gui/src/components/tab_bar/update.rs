use crate::app::App;
use crate::components::tab_bar::TabBarMessage;
use iced::Task;

pub fn handle(message: TabBarMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        TabBarMessage::SelectTab(tab_index) => {
            // Stop SNF playback when switching away from a tab.
            for editor in app.state.snf_editors.values_mut() {
                editor.playback = None;
            }
            if app.state.workspace.tabs.len() > tab_index {
                app.state.workspace.active_tab = Some(tab_index);
            }
            Task::none()
        }
        TabBarMessage::CloseTab(tab_index) => {
            if app.state.workspace.tabs.len() > tab_index {
                let tab_id = app.state.workspace.tabs[tab_index].id;
                app.state.sprite_viewers.remove(&tab_id);
                app.state.extra_ref_editors.remove(&tab_id);
                app.state.npc_ref_editors.remove(&tab_id);
                app.state.monster_ref_editors.remove(&tab_id);
                app.state.dialog_editors.remove(&tab_id);
                app.state.dialogue_text_editors.remove(&tab_id);
                app.state.snf_editors.remove(&tab_id);
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
            if let Some(active_tab) = app.state.workspace.active_tab {
                if !app.state.workspace.tabs.is_empty() {
                    let tab_id = app.state.workspace.tabs[active_tab].id;
                    app.state.sprite_viewers.remove(&tab_id);
                    app.state.extra_ref_editors.remove(&tab_id);
                    app.state.npc_ref_editors.remove(&tab_id);
                    app.state.monster_ref_editors.remove(&tab_id);
                    app.state.dialog_editors.remove(&tab_id);
                    app.state.dialogue_text_editors.remove(&tab_id);
                    app.state.snf_editors.remove(&tab_id);
                    app.state.workspace.tabs.remove(active_tab);
                    if app.state.workspace.tabs.is_empty() {
                        app.state.workspace.active_tab = None;
                    } else if active_tab >= app.state.workspace.tabs.len() {
                        app.state.workspace.active_tab = Some(app.state.workspace.tabs.len() - 1);
                    }
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
                    app.state.sprite_viewers.remove(&id);
                    app.state.extra_ref_editors.remove(&id);
                    app.state.npc_ref_editors.remove(&id);
                    app.state.monster_ref_editors.remove(&id);
                    app.state.dialog_editors.remove(&id);
                    app.state.dialogue_text_editors.remove(&id);
                    app.state.snf_editors.remove(&id);
                }
                app.state.workspace.tabs.retain(|tab| tab.id == tab_id);
                app.state.workspace.active_tab = Some(0);
            }
            Task::none()
        }
        TabBarMessage::CloseAll => {
            app.state.workspace.tabs.clear();
            app.state.workspace.active_tab = None;
            app.state.sprite_viewers.clear();
            app.state.extra_ref_editors.clear();
            app.state.npc_ref_editors.clear();
            app.state.monster_ref_editors.clear();
            app.state.dialog_editors.clear();
            app.state.dialogue_text_editors.clear();
            app.state.snf_editors.clear();
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
    fn test_select_tab_out_of_bounds() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::SelectTab(10), &mut app);

        assert_eq!(app.state.workspace.active_tab, None);
    }

    #[test]
    fn test_select_tab_empty_workspace() {
        let workspace = Workspace::new();
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::SelectTab(0), &mut app);

        assert_eq!(app.state.workspace.active_tab, None);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CloseTab Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_close_tab_first_tab() {
        let mut workspace = create_test_workspace(3);
        workspace.active_tab = Some(0);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseTab(0), &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 2);
        assert_eq!(app.state.workspace.tabs[0].label, "Tab 1");
    }

    #[test]
    fn test_close_tab_middle_tab() {
        let mut workspace = create_test_workspace(3);
        workspace.active_tab = Some(1);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseTab(1), &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 2);
    }

    #[test]
    fn test_close_tab_last_tab() {
        let mut workspace = create_test_workspace(3);
        workspace.active_tab = Some(2);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseTab(2), &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 2);
    }

    #[test]
    fn test_close_tab_out_of_bounds() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseTab(10), &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 3);
    }

    #[test]
    fn test_close_active_tab_adjusts_index() {
        let mut workspace = create_test_workspace(3);
        workspace.active_tab = Some(2);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseTab(2), &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 2);
        assert_eq!(app.state.workspace.active_tab, Some(1));
    }

    #[test]
    fn test_close_tab_clears_active_when_last() {
        let mut workspace = create_test_workspace(1);
        workspace.active_tab = Some(0);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseTab(0), &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 0);
        assert_eq!(app.state.workspace.active_tab, None);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TogglePin Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_toggle_pin_unpinned_to_pinned() {
        let workspace = create_test_workspace(1);
        let mut app = crate::app::App::test_new(workspace);

        assert!(!app.state.workspace.tabs[0].pinned);

        let _ = handle(TabBarMessage::TogglePin(0), &mut app);

        assert!(app.state.workspace.tabs[0].pinned);
    }

    #[test]
    fn test_toggle_pin_pinned_to_unpinned() {
        let mut workspace = create_test_workspace(1);
        workspace.tabs[0].pinned = true;
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::TogglePin(0), &mut app);

        assert!(!app.state.workspace.tabs[0].pinned);
    }

    #[test]
    fn test_toggle_pin_out_of_bounds_no_panic() {
        let workspace = create_test_workspace(1);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::TogglePin(10), &mut app);

        assert!(!app.state.workspace.tabs[0].pinned);
    }

    #[test]
    fn test_toggle_pin_preserves_other_tabs() {
        let mut workspace = create_test_workspace(3);
        workspace.tabs[0].pinned = true;
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::TogglePin(1), &mut app);

        assert!(app.state.workspace.tabs[0].pinned);
        assert!(app.state.workspace.tabs[1].pinned);
        assert!(!app.state.workspace.tabs[2].pinned);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CloseActiveTab Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_close_active_tab_first_tab() {
        let mut workspace = create_test_workspace(3);
        workspace.active_tab = Some(0);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseActiveTab, &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 2);
        assert_eq!(app.state.workspace.active_tab, Some(0));
    }

    #[test]
    fn test_close_active_tab_last_tab() {
        let mut workspace = create_test_workspace(3);
        workspace.active_tab = Some(2);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseActiveTab, &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 2);
        assert_eq!(app.state.workspace.active_tab, Some(1));
    }

    #[test]
    fn test_close_active_tab_no_active() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);
        app.state.workspace.active_tab = None;

        let _ = handle(TabBarMessage::CloseActiveTab, &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 3);
    }

    #[test]
    fn test_close_active_tab_empty_workspace() {
        let workspace = Workspace::new();
        let mut app = crate::app::App::test_new(workspace);
        app.state.workspace.active_tab = Some(0);

        let _ = handle(TabBarMessage::CloseActiveTab, &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CloseOthers Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_close_others_keeps_first() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseOthers(0), &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 1);
        assert_eq!(app.state.workspace.tabs[0].label, "Tab 0");
        assert_eq!(app.state.workspace.active_tab, Some(0));
    }

    #[test]
    fn test_close_others_keeps_middle() {
        let workspace = create_test_workspace(5);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseOthers(2), &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 1);
        assert_eq!(app.state.workspace.tabs[0].label, "Tab 2");
    }

    #[test]
    fn test_close_others_keeps_last() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseOthers(2), &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 1);
        assert_eq!(app.state.workspace.tabs[0].label, "Tab 2");
    }

    #[test]
    fn test_close_others_out_of_bounds() {
        let workspace = create_test_workspace(3);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseOthers(10), &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 3);
    }

    #[test]
    fn test_close_others_with_single_tab() {
        let workspace = create_test_workspace(1);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseOthers(0), &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 1);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CloseAll Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_close_all_clears_tabs() {
        let workspace = create_test_workspace(5);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseAll, &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 0);
        assert_eq!(app.state.workspace.active_tab, None);
    }

    #[test]
    fn test_close_all_empty_workspace() {
        let workspace = Workspace::new();
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::CloseAll, &mut app);

        assert_eq!(app.state.workspace.tabs.len(), 0);
        assert_eq!(app.state.workspace.active_tab, None);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Edge Cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_sequential_tab_operations() {
        let workspace = create_test_workspace(5);
        let mut app = crate::app::App::test_new(workspace);

        let _ = handle(TabBarMessage::SelectTab(2), &mut app);
        assert_eq!(app.state.workspace.active_tab, Some(2));

        let _ = handle(TabBarMessage::TogglePin(2), &mut app);
        assert!(app.state.workspace.tabs[2].pinned);

        let _ = handle(TabBarMessage::SelectTab(0), &mut app);
        assert_eq!(app.state.workspace.active_tab, Some(0));

        let _ = handle(TabBarMessage::CloseTab(0), &mut app);
        assert_eq!(app.state.workspace.tabs.len(), 4);
        assert!(app.state.workspace.tabs[1].pinned);
    }
}
