use iced::widget::{container, text};
use iced::{Element, Fill};

use crate::editors::save_file_viewer::state::SaveFileViewerState;
use crate::message::Message;
use gui_widgets::{ParagraphCache, RowFlags, TableColumn, TableWidget};

/// Events section: virtualized table of event script records.
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    let sf = match state.save_file.as_ref() {
        Some(sf) => sf,
        None => return container(text("No save file loaded")).into(),
    };

    if sf.events.is_empty() {
        return container(text("No events"))
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into();
    }

    let n = sf.events.len();
    let row_height = 22.0;

    // Build display cache: [row][col] = pre-rendered cell text
    // Columns: #, State, Script Name
    let mut display_cache: Vec<Vec<String>> = Vec::with_capacity(n);
    for (i, ev) in sf.events.iter().enumerate() {
        let state_str = match ev.state {
            0 => "Inactive".into(),
            1 => "Active".into(),
            2 => "Completed".into(),
            s => format!("Unknown({})", s),
        };
        display_cache.push(vec![
            format!("{}", i + 1),
            state_str,
            ev.script_name.clone(),
        ]);
    }

    let filtered_indices: Vec<usize> = (0..n).collect();
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
            width_px: 300.0,
            label: "Script".into(),
            sort: None,
            has_filter: false,
        },
    ];

    let id_col_width = 0.0; // ID is the first real column, no separate id col
    let cache = ParagraphCache::default();

    TableWidget::new(&display_cache, &filtered_indices, columns, id_col_width, |_| RowFlags::default(), row_height, cache)
        .into()
}
