//! Save file viewer for .sav files.

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use std::path::Path;

/// Save file viewer.
pub struct SaveViewer {
    pub hwnd: HWND,
    pub file_path: Option<PathBuf>,
    pub save_data: Vec<u8>,
}

impl SaveViewer {
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
                save_data: Vec::new(),
            })
        }
    }

    /// Load a save file.
    pub fn load(&mut self, path: &Path) -> Result<()> {
        self.save_data = std::fs::read(path)?;
        self.file_path = Some(path.to_path_buf());

        // TODO: Parse save file structure and display fields
        // Save files have a specific binary format with monster data,
        // inventory, events, etc.

        Ok(())
    }

    /// Refresh the view.
    pub fn refresh(&self) {
        // TODO: Re-parse and display save data
    }
}
