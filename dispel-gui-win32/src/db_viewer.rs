//! SQLite database viewer using rusqlite.
// Provides a table-based view of SQLite databases.

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use std::path::Path;

/// SQLite database viewer.
pub struct DbViewer {
    pub hwnd: HWND,
    pub file_path: Option<PathBuf>,
    pub table_name: Option<String>,
}

impl DbViewer {
    pub fn new(parent: HWND) -> Result<Self> {
        unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("SysListView32"),
                None,
                WS_CHILD | WS_VISIBLE | LVS_REPORT | LVS_SINGLESEL | LVS_SHOWSELALWAYS,
                0, 0, 0, 0,
                parent,
                None,
                GetModuleHandleW(None)?,
                None,
            );

            // Enable extended styles
            let mut ex_style = ListView_GetExtendedListViewStyle(hwnd);
            ex_style |= LVS_EX_FULLROWSELECT | LVS_EX_GRIDLINES | LVS_EX_DOUBLEBUFFER;
            ListView_SetExtendedListViewStyle(hwnd, ex_style);

            Ok(Self {
                hwnd,
                file_path: None,
                table_name: None,
            })
        }
    }

    /// Open a SQLite database file.
    pub fn open(&mut self, path: &Path) -> Result<()> {
        self.file_path = Some(path.to_path_buf());

        // TODO: Use rusqlite to open the database and list tables
        // For now, just set the file path

        Ok(())
    }

    /// Load a table's contents into the viewer.
    pub fn load_table(&mut self, table_name: &str) -> Result<()> {
        self.table_name = Some(table_name.to_string());

        // TODO: Query the table and populate the ListView
        // This would use rusqlite to execute SELECT * FROM table_name

        Ok(())
    }

    /// Refresh the current view.
    pub fn refresh(&self) {
        // TODO: Re-query the current table
    }
}
