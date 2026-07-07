use iced::widget::{button, container, scrollable, text, Column, Row};
use iced::{Element, Fill};

use crate::editors::save_file_viewer::state::{JournalSection, SaveFileViewerState};
use crate::message::Message;
use crate::message::MessageExt;

/// Journal section: sub-tabs (Main/Side/Trade) + scrollable entry list.
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    let sf = match state.save_file.as_ref() {
        Some(sf) => sf,
        None => return container(text("No save file loaded")).into(),
    };

    // Sub-tab bar
    let sections = [
        (JournalSection::Main, "Main"),
        (JournalSection::Side, "Side"),
        (JournalSection::Trade, "Trade"),
    ];

    let tab_buttons: Vec<Element<'a, Message>> = sections
        .iter()
        .map(|(section, label)| {
            let is_active = *section == state.journal_section;
            let mut btn = button(text(*label).size(13));
            if is_active {
                btn = btn.style(iced::widget::button::primary);
            }
            btn.on_press(Message::save_file_viewer(
                crate::editors::save_file_viewer::SaveFileViewerMessage::SelectJournalSection(
                    *section,
                ),
            ))
            .padding([4, 12])
            .into()
        })
        .collect();

    let tab_bar = Row::new().spacing(4).padding(8).extend(tab_buttons);

    // Get entries for this section
    let entries: Vec<&dispel_core::references::save_file::JournalEntry> = match state.journal_section
    {
        JournalSection::Main => sf.journal.main.iter().collect(),
        JournalSection::Side => sf.journal.side.iter().collect(),
        JournalSection::Trade => sf.journal.trade.iter().collect(),
    };

    // Entry list
    let list = if entries.is_empty() {
        container(text("No entries"))
            .width(Fill)
            .padding(16)
            .into()
    } else {
        let mut col = Column::new().spacing(2).padding(8);
        for (i, entry) in entries.iter().enumerate() {
            let is_selected = state.selected_journal_entry == Some(i);
            let flags = format_flags(entry.flags);

            let row = Row::new()
                .spacing(8)
                .push(text(format!("{}", i + 1)).size(12).width(30))
                .push(text(&entry.name).size(12).width(200))
                .push(text(&flags).size(11).color(iced::Color::from_rgb(0.6, 0.6, 0.6)));

            let entry_widget = if is_selected {
                container(row).style(iced::widget::container::bordered)
            } else {
                container(row)
            };

            col = col.push(
                entry_widget
                    .width(Fill)
                    .padding([2, 8])
                    .into(),
            );
        }
        scrollable(col).height(Fill).into()
    };

    Column::new().push(tab_bar).push(list).into()
}

fn format_flags(flags: u32) -> String {
    if flags == 0 {
        "None".into()
    } else {
        let mut parts = Vec::new();
        if flags & 1 != 0 {
            parts.push("READ");
        }
        if flags & 2 != 0 {
            parts.push("NEW");
        }
        if flags & 4 != 0 {
            parts.push("DONE");
        }
        let extra = flags & !7;
        if extra != 0 {
            parts.push(&format!("0x{:X}", extra));
        }
        if parts.is_empty() {
            format!("0x{:X}", flags)
        } else {
            parts.join("|")
        }
    }
}
