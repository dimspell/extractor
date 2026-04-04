use iced::widget::{button, container, row, scrollable, text};
use iced::{Element, Length};

use crate::style;
use crate::workspace::Workspace;

/// Messages from the tab bar.
#[derive(Debug, Clone)]
pub enum TabBarMessage {
    SelectTab(usize),
    CloseTab(usize),
    TogglePin(usize),
}

/// Render the workspace tab bar.
pub fn view_tab_bar(workspace: &Workspace) -> Element<'_, TabBarMessage> {
    if workspace.tabs.is_empty() {
        return container(text("No tabs open").size(12).style(style::subtle_text))
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
                if tab.pinned {
                    text("📌").size(10)
                } else {
                    text("").size(10)
                },
                text(label).size(11),
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

            let btn = button(tab_row)
                .on_press(TabBarMessage::SelectTab(idx))
                .style(if is_active {
                    style::active_chip
                } else {
                    style::chip
                })
                .padding([4, 8]);

            btn.into()
        })
        .collect();

    scrollable(row(tabs).spacing(2))
        .height(32)
        .width(Length::Fill)
        .into()
}
