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
            app.global_search.pending_query = input;
            Task::perform(
                async {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                },
                |_| Message::Workspace(WorkspaceMessage::GlobalSearchDebounceTick),
            )
        }
        WorkspaceMessage::GlobalSearchSelect(index) => {
            // In non-legacy mode, use workspace-based navigation
            if let Some(result) = app.global_search.results.get(index) {
                let record_index = result.record_idx;
                match result.catalog_type.as_str() {
                    "Weapon" => app.state.weapon_editor.select(record_index),
                    "HealItem" => app.state.heal_item_editor.select(record_index),
                    "MiscItem" => app.state.misc_item_editor.select(record_index),
                    "EditItem" => app.state.edit_item_editor.select(record_index),
                    "EventItem" => app.state.event_item_editor.select(record_index),
                    "Monster" => app.state.monster_editor.select(record_index),
                    "NpcIni" => app.state.npc_ini_editor.select_npc(record_index),
                    "MagicSpell" => app.state.magic_editor.select(record_index),
                    "Dialog" => {
                        let tab_id = app
                            .state
                            .workspace
                            .active()
                            .map(|t| t.id)
                            .unwrap_or(usize::MAX);
                        if let Some(ed) = app.state.dialog_editors.get_mut(&tab_id) {
                            ed.select_dialog(record_index);
                        }
                    }
                    "DialogueText" => {
                        let tab_id = app
                            .state
                            .workspace
                            .active()
                            .map(|t| t.id)
                            .unwrap_or(usize::MAX);
                        if let Some(ed) = app.state.dialogue_text_editors.get_mut(&tab_id) {
                            ed.select_text(record_index);
                        }
                    }
                    "Store" => app.state.store_editor.select_store(record_index),
                    "PartyRef" => app.state.party_ref_editor.select(record_index),
                    "PartyIni" => app.state.party_ini_editor.select(record_index),
                    "DrawItem" => app.state.draw_item_editor.select(record_index),
                    "Event" => app.state.event_ini_editor.select(record_index),
                    "EventNpcRef" => app.state.event_npc_ref_editor.select(record_index),
                    "Extra" => app.state.extra_ini_editor.select(record_index),
                    "ExtraRef" => {
                        let tab_id = app
                            .state
                            .workspace
                            .active()
                            .map(|t| t.id)
                            .unwrap_or(usize::MAX);
                        if let Some(ed) = app.state.extra_ref_editors.get_mut(&tab_id) {
                            ed.select(record_index);
                        }
                    }
                    "MapIni" => app.state.map_ini_editor.select(record_index),
                    "Message" => app.state.message_scr_editor.select(record_index),
                    "NpcRef" => {
                        let tab_id = app
                            .state
                            .workspace
                            .active()
                            .map(|t| t.id)
                            .unwrap_or(usize::MAX);
                        if let Some(ed) = app.state.npc_ref_editors.get_mut(&tab_id) {
                            ed.select(record_index);
                        }
                    }
                    "PartyLevel" => app.state.party_level_db_editor.select(record_index),
                    "Quest" => app.state.quest_scr_editor.select(record_index),
                    "Wave" => app.state.wave_ini_editor.select(record_index),
                    "ChData" => app.state.chdata_editor.select(record_index),
                    "Map" => app.state.all_map_ini_editor.select(record_index),
                    _ => {}
                }
                app.global_search.is_visible = false;
                app.global_search.query.clear();
                app.state.status_msg = format!("Navigated to {} via search", result.display_text);
            }
            Task::none()
        }
        WorkspaceMessage::GlobalSearchConfirm => {
            if let Some(selected_index) = app.global_search.selected_index.checked_sub(0) {
                if let Some(result) = app.global_search.results.get(selected_index) {
                    {
                        // Workspace-based navigation
                        let record_index = result.record_idx;
                        match result.catalog_type.as_str() {
                            "Weapon" => app.state.weapon_editor.select(record_index),
                            "HealItem" => app.state.heal_item_editor.select(record_index),
                            "MiscItem" => app.state.misc_item_editor.select(record_index),
                            "EditItem" => app.state.edit_item_editor.select(record_index),
                            "EventItem" => app.state.event_item_editor.select(record_index),
                            "Monster" => app.state.monster_editor.select(record_index),
                            "NpcIni" => app.state.npc_ini_editor.select_npc(record_index),
                            "MagicSpell" => app.state.magic_editor.select(record_index),
                            "Dialog" => {
                                let tab_id = app
                                    .state
                                    .workspace
                                    .active()
                                    .map(|t| t.id)
                                    .unwrap_or(usize::MAX);
                                if let Some(ed) = app.state.dialog_editors.get_mut(&tab_id) {
                                    ed.select_dialog(record_index);
                                }
                            }
                            "DialogueText" => {
                                let tab_id = app
                                    .state
                                    .workspace
                                    .active()
                                    .map(|t| t.id)
                                    .unwrap_or(usize::MAX);
                                if let Some(ed) = app.state.dialogue_text_editors.get_mut(&tab_id) {
                                    ed.select_text(record_index);
                                }
                            }
                            "Store" => app.state.store_editor.select_store(record_index),
                            "PartyRef" => app.state.party_ref_editor.select(record_index),
                            "PartyIni" => app.state.party_ini_editor.select(record_index),
                            "DrawItem" => app.state.draw_item_editor.select(record_index),
                            "Event" => app.state.event_ini_editor.select(record_index),
                            "EventNpcRef" => app.state.event_npc_ref_editor.select(record_index),
                            "Extra" => app.state.extra_ini_editor.select(record_index),
                            "ExtraRef" => {
                                let tab_id = app
                                    .state
                                    .workspace
                                    .active()
                                    .map(|t| t.id)
                                    .unwrap_or(usize::MAX);
                                if let Some(ed) = app.state.extra_ref_editors.get_mut(&tab_id) {
                                    ed.select(record_index);
                                }
                            }
                            "MapIni" => app.state.map_ini_editor.select(record_index),
                            "Message" => app.state.message_scr_editor.select(record_index),
                            "NpcRef" => {
                                let tab_id = app
                                    .state
                                    .workspace
                                    .active()
                                    .map(|t| t.id)
                                    .unwrap_or(usize::MAX);
                                if let Some(ed) = app.state.npc_ref_editors.get_mut(&tab_id) {
                                    ed.select(record_index);
                                }
                            }
                            "PartyLevel" => app.state.party_level_db_editor.select(record_index),
                            "Quest" => app.state.quest_scr_editor.select(record_index),
                            "Wave" => app.state.wave_ini_editor.select(record_index),
                            "AllMap" => app.state.all_map_ini_editor.select(record_index),
                            "Chest" => {
                                app.state.chest_editor.selected_idx = Some(record_index);
                                if let Some((_, record)) =
                                    app.state.chest_editor.filtered_chests.get(record_index)
                                {
                                    app.state.chest_editor.edit_name = record.name.clone();
                                    app.state.chest_editor.edit_x = record.x_pos.to_string();
                                    app.state.chest_editor.edit_y = record.y_pos.to_string();
                                    app.state.chest_editor.edit_gold = "0".to_string();
                                    app.state.chest_editor.edit_item_count = "0".to_string();
                                    app.state.chest_editor.edit_item_id = "0".to_string();
                                    app.state.chest_editor.edit_item_type = "0".to_string();
                                    app.state.chest_editor.edit_closed = record.closed.to_string();
                                }
                            }
                            _ => {}
                        }

                        // Open the file in workspace
                        if let Some(path) = &result.source_file {
                            let label = result.display_text.clone();
                            app.state.workspace.open(label, Some(PathBuf::from(path)));
                        }

                        app.global_search.is_visible = false;
                        app.global_search.query.clear();
                        app.state.status_msg =
                            format!("Navigated to {} via search", result.display_text);
                    }
                }
            }
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
        WorkspaceMessage::GlobalSearchDebounceTick => {
            let pending = app.global_search.pending_query.clone();
            if app.global_search.query != pending {
                app.global_search.query = pending;
                app.global_search.search(&app.search_index);
            }
            Task::none()
        }
        // Tools
        WorkspaceMessage::OpenToolTab(editor_type) => {
            use crate::workspace::EditorType;
            let label = match editor_type {
                EditorType::DbViewer => "DB Viewer",
                EditorType::ChestEditor => "Chest Editor",
                _ => "Tool",
            };
            app.state
                .workspace
                .open_tool(label.to_string(), editor_type);
            Task::none()
        }
    }
}
