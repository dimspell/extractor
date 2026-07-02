//! Viewer data-fetching functions extracted from `app.rs`.

use crate::editors::db_viewer::db;
use crate::editors::db_viewer::DbViewerState;
use crate::editors::db_viewer::PAGE_SIZE;
use crate::message::{Message, ViewerMessage};
use iced::Task;

/// Fetch data using the built table query (filters + sorting).
pub fn fetch_table_data(state: &mut DbViewerState) -> Task<Message> {
    let table = match &state.active_table {
        Some(t) => t.clone(),
        None => return Task::none(),
    };
    state.loading_state = crate::components::loading_state::LoadingState::Loading;

    let path = state.db_path.clone();
    let search = state.search.clone();
    let sort_col = state.sort_col;
    let sort_dir = state.sort_dir;
    let page = state.page;

    Task::perform(
        async move {
            let cols = db::table_columns(&path, &table)?;
            let sql = db::build_table_query(&table, &cols, &search, sort_col, sort_dir);
            let mut result = db::execute_query(&path, &sql, PAGE_SIZE, page * PAGE_SIZE)?;
            result.columns = cols;
            Ok(result)
        },
        |result| Message::Viewer(ViewerMessage::DataLoaded(result)),
    )
}

/// Fetch data using the custom SQL query.
pub fn fetch_sql_data(state: &mut DbViewerState) -> Task<Message> {
    state.loading_state = crate::components::loading_state::LoadingState::Loading;
    let path = state.db_path.clone();
    let sql = state.sql_query.clone();
    let page = state.page;

    Task::perform(
        async move { db::execute_query(&path, &sql, PAGE_SIZE, page * PAGE_SIZE) },
        |result| Message::Viewer(ViewerMessage::DataLoaded(result)),
    )
}
