//! Application state for the Win32 native GUI.
// This struct holds all persistent UI state for the application.

use windows::core::HWND;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::file_tree::FileTree;
use crate::editors::EditorTypeId;
use crate::spreadsheet::Spreadsheet;

/// The main application state.
pub struct App {
    /// The main window handle.
    pub hwnd: HWND,
    /// The currently selected game path.
    pub game_path: Option<PathBuf>,
    /// Currently open file paths (tab_id -> path).
    pub open_files: HashMap<usize, PathBuf>,
    /// Open spreadsheets per tab (tab_id -> Spreadsheet).
    pub spreadsheets: HashMap<usize, Spreadsheet>,
    /// Editor type per tab (tab_id -> EditorTypeId).
    pub editor_types: HashMap<usize, EditorTypeId>,
    /// Raw file bytes per tab, for save round-tripping.
    pub original_file_data: HashMap<usize, Vec<u8>>,
    /// The next tab ID to assign.
    pub next_tab_id: usize,
    /// The currently active tab.
    pub active_tab: Option<usize>,
    /// Status bar text.
    pub status_text: String,
    /// File tree filter (None = show all).
    pub file_tree_filter: Option<String>,
    /// The file tree control in the sidebar.
    pub file_tree: Option<FileTree>,
}

impl App {
    pub fn new(hwnd: HWND) -> Self {
        Self {
            hwnd,
            game_path: None,
            open_files: HashMap::new(),
            spreadsheets: HashMap::new(),
            editor_types: HashMap::new(),
            original_file_data: HashMap::new(),
            next_tab_id: 1,
            active_tab: None,
            status_text: String::new(),
            file_tree_filter: None,
            file_tree: None,
        }
    }

    pub fn set_status(&mut self, text: &str) {
        self.status_text = text.to_string();
    }

    /// Get the spreadsheet for the active tab, if any.
    pub fn active_spreadsheet(&self) -> Option<&Spreadsheet> {
        let tab_id = self.active_tab.as_ref()?;
        self.spreadsheets.get(tab_id)
    }

    /// Get the mutable spreadsheet for the active tab, if any.
    pub fn active_spreadsheet_mut(&mut self) -> Option<&mut Spreadsheet> {
        let tab_id = self.active_tab?;
        self.spreadsheets.get_mut(&tab_id)
    }

    /// Open a new tab with a spreadsheet editor.
    pub fn open_spreadsheet_tab(
        &mut self,
        path: PathBuf,
        editor_type: EditorTypeId,
        spreadsheet: Spreadsheet,
        raw_data: Vec<u8>,
    ) -> usize {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;
        self.open_files.insert(tab_id, path);
        self.editor_types.insert(tab_id, editor_type);
        self.spreadsheets.insert(tab_id, spreadsheet);
        self.original_file_data.insert(tab_id, raw_data);
        self.active_tab = Some(tab_id);
        tab_id
    }

    /// Close a tab and clean up its data.
    pub fn close_tab(&mut self, tab_id: usize) {
        self.open_files.remove(&tab_id);
        self.editor_types.remove(&tab_id);
        self.spreadsheets.remove(&tab_id);
        self.original_file_data.remove(&tab_id);
        if self.active_tab == Some(tab_id) {
            self.active_tab = None;
        }
    }
}
