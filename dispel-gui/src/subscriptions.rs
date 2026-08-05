use crate::components::tab_bar::TabBarMessage;
use crate::dispatch_table::spreadsheet_nav_msg;
use crate::message::Message;
use crate::message::MessageExt;
use crate::message::SystemMessage;
use crate::message::WorkspaceMessage;
use crate::workspace::EditorType;
use iced::Subscription;

pub fn subscription(app: &crate::app::App) -> Subscription<Message> {
    use iced::keyboard::{self, key::Named, Key};
    use iced::window;

    let close = window::close_requests().map(|_| Message::System(SystemMessage::CloseRequested));

    let keyboard_sub = keyboard::listen().filter_map(|event| {
        if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
            if modifiers.control() || modifiers.command() {
                if let Key::Character(c) = key.as_ref() {
                    let ch = c.chars().next()?;
                    if modifiers.shift() {
                        return match ch {
                            'x' => Some(Message::Workspace(WorkspaceMessage::ReopenActiveTabAsHex)),
                            'p' => Some(Message::Workspace(WorkspaceMessage::ToggleCommandPalette)),
                            _ => None,
                        };
                    }
                    return match ch {
                        'z' => Some(Message::System(SystemMessage::Undo)),
                        'y' => Some(Message::System(SystemMessage::Redo)),
                        's' => Some(Message::System(SystemMessage::Save)),
                        'h' => Some(Message::Workspace(WorkspaceMessage::ToggleHistoryPanel)),
                        'p' => Some(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch)),
                        'w' => Some(Message::tab_bar(TabBarMessage::CloseActiveTab)),
                        _ => None,
                    };
                }
            }
            if let Key::Named(named) = key.as_ref() {
                match named {
                    Named::Escape => {
                        Some(Message::Workspace(WorkspaceMessage::CommandPaletteClose))
                    }
                    Named::Enter => {
                        Some(Message::Workspace(WorkspaceMessage::CommandPaletteConfirm))
                    }
                    Named::ArrowUp => {
                        Some(Message::Workspace(WorkspaceMessage::CommandPaletteArrowUp))
                    }
                    Named::ArrowDown => Some(Message::Workspace(
                        WorkspaceMessage::CommandPaletteArrowDown,
                    )),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    });

    // Global search keyboard handling (only when active)
    let global_search_keyboard_sub = keyboard::listen().filter_map(move |event| {
        if let keyboard::Event::KeyPressed { key, .. } = event {
            if let Key::Named(named) = key.as_ref() {
                match named {
                    Named::Escape => Some(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch)),
                    Named::Enter => Some(Message::Workspace(WorkspaceMessage::GlobalSearchConfirm)),
                    Named::ArrowUp => {
                        Some(Message::Workspace(WorkspaceMessage::GlobalSearchArrowUp))
                    }
                    Named::ArrowDown => {
                        Some(Message::Workspace(WorkspaceMessage::GlobalSearchArrowDown))
                    }
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    });

    // Only include global search keyboard handling when it's active
    let mut subscriptions: Vec<Subscription<Message>> = vec![close, keyboard_sub];
    if app.global_search.is_visible {
        subscriptions.push(global_search_keyboard_sub);
    }

    // Drive animation playback when any sprite viewer is playing.
    if app
        .state
        .editors
        .sprite_viewers
        .values()
        .any(|v| v.is_playing)
    {
        use crate::editors::sprite_editor::SpriteViewerMessage;
        let anim = iced::time::every(std::time::Duration::from_millis(16))
            .map(|_| Message::sprite_viewer(SpriteViewerMessage::Tick));
        subscriptions.push(anim);
    }

    // Poll for SNF playback completion so the Play/Pause button stays in sync.
    if app
        .state
        .editors
        .snf_editors
        .values()
        .any(|e| e.playback.is_some())
    {
        use crate::editors::snf_editor::SnfEditorMessage;
        let snf_tick = iced::time::every(std::time::Duration::from_millis(250))
            .map(|_| Message::snf_editor(SnfEditorMessage::Tick));
        subscriptions.push(snf_tick);
    }

    // Poll for event script indexing progress.
    if matches!(
        app.state.editors.event_scr_editor.index_state,
        crate::editors::event_scr::FunctionIndexState::Indexing { .. }
    ) {
        let index_tick = iced::time::every(std::time::Duration::from_millis(100)).map(|_| {
            Message::event_scr(crate::editors::event_scr::EventScrEditorMessage::IndexTick)
        });
        subscriptions.push(index_tick);
    }

    // Spreadsheet row navigation (Arrow / Home / End keys).
    let active_et = app.state.workspace.active().map(|t| t.editor_type);
    let palette_open = app.command_palette.is_some();
    let search_open = app.global_search.is_visible;

    if !palette_open && !search_open {
        if let Some(et) = active_et {
            use crate::view::editor::SpreadsheetMessage as SM;
            // Probe whether this editor type has a spreadsheet.
            if spreadsheet_nav_msg(et, SM::NavigateUp).is_some() {
                let ss_sub = keyboard::listen().with(et).filter_map(|(et, event)| {
                    if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                        if modifiers.control() || modifiers.command() || modifiers.shift() {
                            return None;
                        }
                        if let Key::Named(named) = key.as_ref() {
                            use crate::view::editor::SpreadsheetMessage as SM;
                            let sm = match named {
                                Named::ArrowUp => SM::NavigateUp,
                                Named::ArrowDown => SM::NavigateDown,
                                Named::Home => SM::NavigateTop,
                                Named::End => SM::NavigateBottom,
                                Named::Escape => SM::CancelEdit,
                                _ => return None,
                            };
                            return spreadsheet_nav_msg(et, sm);
                        }
                    }
                    None
                });
                subscriptions.push(ss_sub);
            }
        }
    }

    // Event Script Editor keyboard shortcuts.
    if !palette_open && !search_open && active_et == Some(EditorType::EventScrEditor) {
        use crate::editors::event_scr::{EventScrEditorMessage, KeyboardShortcut};
        let esc_sub = keyboard::listen().filter_map(|event| {
            if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                if modifiers.control() || modifiers.command() {
                    if let Key::Named(Named::Enter) = key.as_ref() {
                        return Some(Message::event_scr(EventScrEditorMessage::KeyboardShortcut(
                            KeyboardShortcut::InsertActionBelow,
                        )));
                    }
                    if let Key::Named(Named::Space) = key.as_ref() {
                        return Some(Message::event_scr(EventScrEditorMessage::KeyboardShortcut(
                            KeyboardShortcut::TogglePicker,
                        )));
                    }
                    return None;
                }
                if let Key::Named(named) = key.as_ref() {
                    match named {
                        Named::ArrowUp => {
                            return Some(Message::event_scr(
                                EventScrEditorMessage::KeyboardShortcut(
                                    KeyboardShortcut::MoveActionUp,
                                ),
                            ))
                        }
                        Named::ArrowDown => {
                            return Some(Message::event_scr(
                                EventScrEditorMessage::KeyboardShortcut(
                                    KeyboardShortcut::MoveActionDown,
                                ),
                            ))
                        }
                        _ => {}
                    }
                }
            }
            None
        });
        subscriptions.push(esc_sub);
    }

    Subscription::batch(subscriptions)
}
