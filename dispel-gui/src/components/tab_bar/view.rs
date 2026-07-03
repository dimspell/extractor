use gui_widgets::components::context_menu::{platform, Entry};
use gui_widgets::components::tab_bar::{TabBar, TabBarEvent, TabData};
use iced::Element;

use super::message::TabBarMessage;
use crate::app::App;
use crate::message::ext::MessageExt;
use crate::message::Message;
use crate::workspace::WorkspaceTab;

/// Build the context menu entries for a given tab.
fn context_entries_for_tab(tab_idx: usize, tabs: &[WorkspaceTab]) -> Vec<Entry<TabBarMessage>> {
    let mut entries = vec![
        Entry::item("Close", TabBarMessage::CloseTab(tab_idx)),
        Entry::item("Close Others", TabBarMessage::CloseOthers(tab_idx)),
        Entry::item("Close All", TabBarMessage::CloseAll),
        Entry::item("Pin/Unpin", TabBarMessage::TogglePin(tab_idx)),
        Entry::item(
            "Move Left",
            TabBarMessage::MoveTab(tab_idx, tab_idx.saturating_sub(1)),
        ),
        Entry::item(
            "Move Right",
            TabBarMessage::MoveTab(tab_idx, (tab_idx + 2).min(tabs.len())),
        ),
    ];

    if tabs.get(tab_idx).and_then(|t| t.path.as_ref()).is_some() {
        entries.push(Entry::separator());
        entries.push(Entry::item(
            "Open as Hex",
            TabBarMessage::OpenAsHex(tab_idx),
        ));
    }

    entries
}

/// Show a native context menu for `tab_idx`, returning the resulting message
/// if the user picked an action, or `None` if cancelled / unavailable.
fn try_native_context_menu(
    tab_idx: usize,
    tabs: &[WorkspaceTab],
) -> Option<Message> {
    let entries = context_entries_for_tab(tab_idx, tabs);
    match platform::try_show_native_menu(&entries) {
        Some(platform::NativeResult::Selected(entry_idx)) => {
            entries.get(entry_idx).map(|entry| match entry {
                Entry::Item { action, .. } => Message::tab_bar(action.clone()),
                Entry::Separator | Entry::Disabled { .. } => unreachable!(),
            })
        }
        Some(platform::NativeResult::Cancelled) => None,
        None => None,
    }
}

/// Render the workspace tab bar using the custom [`TabBar`] widget.
pub fn view_tab_bar(app: &App) -> Element<'_, Message> {
    let tabs: Vec<TabData> = app
        .state
        .workspace
        .tabs
        .iter()
        .map(|t| TabData {
            id: t.id,
            label: t.label.clone(),
            modified: t.modified,
            pinned: t.pinned,
        })
        .collect();

    let active_tab = app.state.workspace.active_tab;
    // Snapshot context entries for right-click
    let ws_tabs = &app.state.workspace.tabs;

    TabBar::new(tabs, active_tab).on_event(move |event| match event {
        TabBarEvent::Selected(i) => Message::tab_bar(TabBarMessage::SelectTab(i)),
        TabBarEvent::Closed(i) => Message::tab_bar(TabBarMessage::CloseTab(i)),
        TabBarEvent::Dragged(from, to) => Message::tab_bar(TabBarMessage::MoveTab(from, to)),
        TabBarEvent::DragCanceled(_) => Message::tab_bar(TabBarMessage::CancelDrag),
        TabBarEvent::RightClicked(i) => {
            // Try native menu; fallback to no-op if unavailable.
            try_native_context_menu(i, ws_tabs).unwrap_or_else(|| {
                Message::tab_bar(TabBarMessage::CancelDrag)
            })
        }
    }).into()
}
