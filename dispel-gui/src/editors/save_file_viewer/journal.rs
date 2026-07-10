use iced::widget::{button, container, text, Column};
use iced::{Element, Fill};

use crate::editors::save_file_viewer::state::{JournalSection, SaveFileViewerState};
use crate::editors::save_file_viewer::SaveFileViewerMessage;
use crate::message::Message;
use crate::message::MessageExt;
use gui_widgets::components::paragraph_cache::ParagraphCache;
use gui_widgets::{RowFlags, TableColumn, TableWidget};

/// Journal section: sub-tabs (Main/Side/Trade) + table per section.
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    // Sub-tab bar
    let sections = [
        (JournalSection::Main, "Main"),
        (JournalSection::Side, "Side"),
        (JournalSection::Trade, "Trade"),
    ];

    let mut tab_bar = iced::widget::Row::new().spacing(4).padding(8);
    for (section, label) in &sections {
        let is_active = *section == state.journal_section;
        let mut btn = button(text(*label).size(13));
        if is_active {
            btn = btn.style(iced::widget::button::primary);
        }
        tab_bar = tab_bar.push(
            btn.on_press(Message::save_file_viewer(
                SaveFileViewerMessage::SelectJournalSection(*section),
            ))
            .padding([4, 12]),
        );
    }

    // Table for the active section
    let display_cache = state.journal_display_caches.get(&state.journal_section);
    let filtered_indices = state.journal_filtered_indices.get(&state.journal_section);

    let table: Element<'a, Message> = match (display_cache, filtered_indices) {
        (Some(cache), Some(indices)) if !cache.is_empty() => {
            let columns = vec![
                TableColumn {
                    width_px: 40.0,
                    label: "#".into(),
                    sort: None,
                    has_filter: false,
                },
                TableColumn {
                    width_px: 200.0,
                    label: "Name".into(),
                    sort: None,
                    has_filter: false,
                },
                TableColumn {
                    width_px: 200.0,
                    label: "Flags (hex)".into(),
                    sort: None,
                    has_filter: false,
                },
            ];

            TableWidget::new(
                cache,
                indices,
                columns,
                0.0,
                |_| RowFlags::default(),
                22.0,
                ParagraphCache::default(),
            )
            .into()
        }
        _ => container(text("No entries"))
            .width(Fill)
            .padding(16)
            .into(),
    };

    Column::<Message>::new().push(tab_bar).push(table).into()
}
