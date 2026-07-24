//! Application state for the Win32 native GUI.
// This struct holds all persistent UI state for the application.

use windows::core::HWND;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::file_tree::FileTree;

/// The main application state.
pub struct App {
    /// The main window handle.
    pub hwnd: HWND,
    /// The currently selected game path.
    pub game_path: Option<PathBuf>,
    /// Currently open file paths (tab_id -> path).
    pub open_files: HashMap<usize, PathBuf>,
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
}
