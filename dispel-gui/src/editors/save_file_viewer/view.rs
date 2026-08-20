use iced::widget::{Column, button, container, row, text};
use iced::{Element, Fill};

use crate::app::App;
use crate::editors::save_file_viewer::state::SaveFileSection;
use crate::message::Message;
use crate::message::MessageExt;
use crate::style;

pub fn view<'a>(app: &'a App) -> Element<'a, Message> {
    let tab_id = match app.state.workspace.active() {
        Some(t) => t.id,
        None => return placeholder("No active tab"),
    };

    let state = match app.state.editors.save_file_viewers.get(&tab_id) {
        Some(s) => s,
        None => return placeholder("Save file not loaded"),
    };

    if state.loading {
        return placeholder("Loading save file...");
    }

    if let Some(ref err) = state.error {
        return placeholder(&format!("Error: {}", err));
    }

    if state.save_file.is_none() {
        return placeholder("No save file loaded");
    }

    // Section tab bar
    let buttons: Vec<Element<'a, Message>> = SaveFileSection::all()
        .iter()
        .map(|section| {
            let is_active = *section == state.active_section;
            let mut btn = button(text(section.label().to_string()).size(13));
            if is_active {
                btn = btn.style(style::active_tab_button);
            } else {
                btn = btn.style(style::tab_button);
            }
            btn.on_press(Message::save_file_viewer(
                crate::editors::save_file_viewer::SaveFileViewerMessage::SelectSection(*section),
            ))
            .padding([4, 12])
            .into()
        })
        .collect();

    let section_tabs = row(buttons).spacing(4).padding(8);

    // Section content
    let content: Element<'a, Message> = match state.active_section {
        SaveFileSection::Overview => crate::editors::save_file_viewer::overview::view(state),
        SaveFileSection::Stats => crate::editors::save_file_viewer::stats::view(state),
        SaveFileSection::PartyMembers => {
            crate::editors::save_file_viewer::party_members::view(state)
        }
        SaveFileSection::Inventory => crate::editors::save_file_viewer::inventory::view(state),
        SaveFileSection::Character => crate::editors::save_file_viewer::character::view(state),
        SaveFileSection::Raw => crate::editors::save_file_viewer::raw::view(state),
        SaveFileSection::Events => crate::editors::save_file_viewer::events::view(state),
        SaveFileSection::Journal => crate::editors::save_file_viewer::journal::view(state),
        SaveFileSection::Maps => crate::editors::save_file_viewer::maps::view(state),
        SaveFileSection::SavedViewport => {
            crate::editors::save_file_viewer::saved_viewport::view(state)
        }
    };

    container(Column::new().push(section_tabs).push(content))
        .width(Fill)
        .height(Fill)
        .into()
}

fn placeholder(text_str: &str) -> Element<'static, Message> {
    container(text(text_str.to_string()))
        .width(Fill)
        .height(Fill)
        .padding(16)
        .into()
}
