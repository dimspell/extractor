use iced::widget::{container, text};
use iced::{Element, Fill};

use crate::editors::save_file_viewer::state::SaveFileViewerState;
use crate::message::Message;
use gui_widgets::components::paragraph_cache::ParagraphCache;
use gui_widgets::{RowFlags, TableColumn, TableWidget};

/// Events section: virtualized table of event script records.
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    let n = state.events_filtered_indices.len();
    if n == 0 {
        return container(text("No events"))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into();
    }

    let columns = vec![
        TableColumn {
            width_px: 40.0,
            label: "ID".into(),
            sort: None,
            has_filter: false,
        },
        TableColumn {
            width_px: 100.0,
            label: "State".into(),
            sort: None,
            has_filter: false,
        },
        TableColumn {
            width_px: 400.0,
            label: "Script".into(),
            sort: None,
            has_filter: false,
        },
    ];

    TableWidget::new(
        &state.events_display_cache,
        &state.events_filtered_indices,
        columns,
        0.0, // no separate id col
        |_| RowFlags::default(),
        22.0,
        ParagraphCache::default(),
    )
    .into()
}
