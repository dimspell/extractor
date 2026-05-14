use crate::components::context_menu::{ContextMenu, Entry};
use iced::widget::{button, container, row, scrollable, text};
use iced::{Element, Length};

use super::message::TabBarMessage;
use crate::style;
use crate::workspace::Workspace;

/// Render the workspace tab bar.
pub fn view_tab_bar(workspace: &Workspace) -> Element<'_, TabBarMessage> {
    if workspace.tabs.is_empty() {
        return container(text("No file opened").size(12).style(style::subtle_text))
            .padding(8)
            .into();
    }

    let tabs: Vec<Element<'_, TabBarMessage>> = workspace
        .tabs
        .iter()
        .enumerate()
        .map(|(idx, tab)| {
            let is_active = workspace.active_tab == Some(idx);
            let label = if tab.modified {
                format!("{} ●", tab.label)
            } else {
                tab.label.clone()
            };

            let mut tab_row = row![
                // Pin indicator
                if tab.pinned {
                    text("📌 ").size(10)
                } else {
                    text("  ").size(10) // Maintain spacing
                },
                // Tab label
                text(label.clone()).size(11),
            ]
            .spacing(4);

            if !tab.pinned {
                tab_row = tab_row.push(
                    button(text("✕").size(10))
                        .on_press(TabBarMessage::CloseTab(idx))
                        .style(style::chip)
                        .padding([2, 4]),
                );
            }

            // Create the tab button
            let btn = button(tab_row)
                .on_press(TabBarMessage::SelectTab(idx))
                .style(if is_active {
                    style::active_chip
                } else {
                    style::chip
                })
                .padding([4, 8]);

            // Add context menu for right-click actions
            let mut context_entries = vec![
                Entry::item("Close", TabBarMessage::CloseTab(idx)),
                Entry::item("Close Others", TabBarMessage::CloseOthers(idx)),
                Entry::item("Close All", TabBarMessage::CloseAll),
                Entry::item("Pin/Unpin", TabBarMessage::TogglePin(idx)),
            ];
            // Only show "Open as Hex" for file-backed tabs (not tool tabs)
            if tab.path.is_some() {
                context_entries.push(Entry::separator());
                context_entries.push(Entry::item(
                    "Open as Hex",
                    TabBarMessage::OpenAsHex(idx),
                ));
            }

            ContextMenu::new(btn, context_entries).into()
        })
        .collect();

    // Create scrollable tab bar with better spacing and overflow handling
    container(
        scrollable(
            row(tabs)
                .spacing(4) // Reduced spacing for more tabs
                .padding(8) // Vertical: 8, Horizontal: 4
                .wrap(),
        )
        .direction(iced::widget::scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new(),
        )),
    )
    .width(Length::Fill)
    .into()
}
