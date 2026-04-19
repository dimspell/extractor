use crate::db;
use std::collections::HashMap;

pub const PAGE_SIZE: usize = 200;

pub struct DbViewerState {
    pub db_path: String,
    pub tables: Vec<String>,
    pub active_table: Option<String>,
    pub columns: Vec<db::ColumnInfo>,
    pub rows: Vec<Vec<String>>,
    pub total_rows: usize,
    pub page: usize,
    pub search: String,
    pub sort_col: Option<usize>,
    pub sort_dir: db::SortDir,
    pub editing_cell: Option<(usize, usize)>,
    pub edit_buffer: String,
    pub pending_edits: db::PendingEdits,
    pub sql_mode: bool,
    pub sql_query: String,
    pub status_msg: String,
    pub loading_state: crate::loading_state::LoadingState<()>,
}

impl Default for DbViewerState {
    fn default() -> Self {
        Self {
            db_path: String::from("database.sqlite"),
            tables: vec![],
            active_table: None,
            columns: vec![],
            rows: vec![],
            total_rows: 0,
            page: 0,
            search: String::new(),
            sort_col: None,
            sort_dir: db::SortDir::Asc,
            editing_cell: None,
            edit_buffer: String::new(),
            pending_edits: HashMap::new(),
            sql_mode: false,
            sql_query: String::new(),
            status_msg: String::new(),
            loading_state: crate::loading_state::LoadingState::Idle,
        }
    }
}
