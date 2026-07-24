//! Reusable ListView spreadsheet component for all DB/INI/ref file editors.
// Uses SysListView32 in LVS_REPORT mode with column headers, row population,
// cell editing, save, and undo/redo support.

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Storage::FileSystem::*;
use std::path::Path;

const ID_SPREADSHEET_UNDO: u16 = 4001;
const ID_SPREADSHEET_REDO: u16 = 4002;
const ID_SPREADSHEET_SAVE: u16 = 4003;

/// Column definition for a spreadsheet.
pub struct ColumnDef {
    pub name: String,
    pub width: i32,
    pub align_right: bool,
}

/// A single cell value in the spreadsheet.
#[derive(Clone, Debug)]
pub enum CellValue {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
}

impl CellValue {
    pub fn as_string(&self) -> String {
        match self {
            CellValue::String(s) => s.clone(),
            CellValue::Integer(i) => i.to_string(),
            CellValue::Float(f) => f.to_string(),
            CellValue::Bool(b) => b.to_string(),
        }
    }
}

/// A single row of data.
pub type Row = Vec<CellValue>;

/// Undo/redo entry.
struct EditEntry {
    row: usize,
    col: usize,
    old_value: CellValue,
    new_value: CellValue,
}

/// The reusable spreadsheet control.
pub struct Spreadsheet {
    pub hwnd: HWND,
    pub columns: Vec<ColumnDef>,
    pub rows: Vec<Row>,
    pub file_path: Option<std::path::PathBuf>,
    pub undo_stack: Vec<EditEntry>,
    pub redo_stack: Vec<EditEntry>,
}

impl Spreadsheet {
    pub fn new(parent: HWND, columns: Vec<ColumnDef>) -> Result<Self> {
        unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("SysListView32"),
                None,
                WS_CHILD | WS_VISIBLE | LVS_REPORT | LVS_SINGLESEL | LVS_SHOWSELALWAYS
                    | WS_BORDER | WS_HSCROLL | WS_VSCROLL,
                0, 0, 0, 0,
                parent,
                None,
                GetModuleHandleW(None)?,
                None,
            );

            // Enable extended styles for full row select and grid lines
            let mut ex_style = ListView_GetExtendedListViewStyle(hwnd);
            ex_style |= LVS_EX_FULLROWSELECT | LVS_EX_GRIDLINES | LVS_EX_DOUBLEBUFFER;
            ListView_SetExtendedListViewStyle(hwnd, ex_style);

            // Create columns
            let lvc = LVCOLUMNW {
                mask: LVCF_TEXT | LVCF_WIDTH | LVCF_FMT,
                fmt: LVCFMT_LEFT,
                cx: 100,
                pszText: PCWSTR(w!("").as_ptr()),
                ..Default::default()
            };

            for (i, col) in columns.iter().enumerate() {
                let mut col_def = lvc.clone();
                col_def.iSubItem = i as i32;
                col_def.cx = col.width;
                col_def.pszText = PCWSTR(wide_from_str(&col.name)?.as_ptr());
                col_def.fmt = if col.align_right {
                    LVCFMT_RIGHT
                } else {
                    LVCFMT_LEFT
                };
                ListView_InsertColumn(hwnd, i as i32, &col_def);
            }

            Ok(Self {
                hwnd,
                columns,
                rows: Vec::new(),
                file_path: None,
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
            })
        }
    }

    /// Clear all rows and columns.
    pub fn clear(&mut self) {
        unsafe {
            ListView_DeleteAllItems(self.hwnd);
            while ListView_DeleteColumn(self.hwnd, 0).as_bool() {}
        }
        self.columns.clear();
        self.rows.clear();
    }

    /// Set column headers from definitions.
    pub fn set_columns(&mut self, columns: Vec<ColumnDef>) {
        self.clear();
        self.columns = columns;
        unsafe {
            let lvc = LVCOLUMNW {
                mask: LVCF_TEXT | LVCF_WIDTH | LVCF_FMT,
                fmt: LVCFMT_LEFT,
                cx: 100,
                pszText: PCWSTR(w!("").as_ptr()),
                ..Default::default()
            };
            for (i, col) in self.columns.iter().enumerate() {
                let mut col_def = lvc.clone();
                col_def.iSubItem = i as i32;
                col_def.cx = col.width;
                col_def.pszText = PCWSTR(wide_from_str(&col.name).unwrap().as_ptr());
                col_def.fmt = if col.align_right {
                    LVCFMT_RIGHT
                } else {
                    LVCFMT_LEFT
                };
                ListView_InsertColumn(self.hwnd, i as i32, &col_def);
            }
        }
    }

    /// Populate rows from data.
    pub fn populate(&mut self, rows: Vec<Row>) {
        self.rows = rows;
        unsafe {
            ListView_DeleteAllItems(self.hwnd);
            for (row_idx, row) in self.rows.iter().enumerate() {
                let lvi = LVITEMW {
                    mask: LVIF_TEXT | LVIF_PARAM,
                    iItem: row_idx as i32,
                    iSubItem: 0,
                    pszText: PCWSTR(wide_from_str(&row.get(0).map(|c| c.as_string()).unwrap_or_default()).unwrap().as_ptr()),
                    lParam: row_idx as LPARAM,
                    ..Default::default()
                };
                let new_item = ListView_InsertItem(self.hwnd, &lvi);

                for (col_idx, cell) in row.iter().enumerate().skip(1) {
                    let mut sub_item = LVITEMW {
                        mask: LVIF_TEXT,
                        iItem: new_item,
                        iSubItem: col_idx as i32,
                        pszText: PCWSTR(wide_from_str(&cell.as_string()).unwrap().as_ptr()),
                        ..Default::default()
                    };
                    ListView_SetItem(self.hwnd, &mut sub_item);
                }
            }
        }
    }

    /// Get the currently selected row index.
    pub fn selected_row(&self) -> Option<usize> {
        unsafe {
            let idx = ListView_GetNextItem(self.hwnd, -1, LVNI_SELECTED);
            if idx >= 0 {
                Some(idx as usize)
            } else {
                None
            }
        }
    }

    /// Get the value of a cell.
    pub fn get_cell(&self, row: usize, col: usize) -> Option<String> {
        self.rows.get(row).and_then(|r| r.get(col)).map(|c| c.as_string())
    }

    /// Set a cell value and record undo.
    pub fn set_cell(&mut self, row: usize, col: usize, value: CellValue) -> Result<()> {
        if row >= self.rows.len() || col >= self.columns.len() {
            return Err(Error::from(HRESULT(0x80070057))); // E_INVALIDARG
        }

        let old_value = self.rows[row][col].clone();
        self.undo_stack.push(EditEntry {
            row,
            col,
            old_value: old_value.clone(),
            new_value: value.clone(),
        });
        self.redo_stack.clear();

        self.rows[row][col] = value;

        // Update the ListView
        unsafe {
            let item_idx = self.selected_row().unwrap_or(row as i32);
            let mut sub_item = LVITEMW {
                mask: LVIF_TEXT,
                iItem: item_idx,
                iSubItem: col as i32,
                pszText: PCWSTR(wide_from_str(&self.rows[row][col].as_string()).unwrap().as_ptr()),
                ..Default::default()
            };
            ListView_SetItem(self.hwnd, &mut sub_item);
        }

        Ok(())
    }

    /// Save the current data back to the file.
    pub fn save(&self) -> Result<()> {
        let path = self.file_path.as_ref().ok_or_else(|| {
            Error::from(HRESULT(0x80070057)) // E_INVALIDARG
        })?;

        // TODO: Implement actual save logic based on file type
        // For .db files, use dispel_core's WeaponItem::save or similar
        // For .ini files, use the text format writer
        // For .ref files, use the binary ref writer

        // Placeholder: just verify the file is writable
        if path.exists() {
            // File exists - we could write back
        }

        Ok(())
    }

    /// Undo the last edit.
    pub fn undo(&mut self) -> bool {
        if let Some(entry) = self.undo_stack.pop() {
            self.rows[entry.row][entry.col] = entry.old_value.clone();
            self.redo_stack.push(entry);

            // Refresh the ListView
            self.refresh_row(entry.row);
            true
        } else {
            false
        }
    }

    /// Redo the last undone edit.
    pub fn redo(&mut self) -> bool {
        if let Some(entry) = self.redo_stack.pop() {
            self.rows[entry.row][entry.col] = entry.new_value.clone();
            self.undo_stack.push(entry);

            // Refresh the ListView
            self.refresh_row(entry.row);
            true
        } else {
            false
        }
    }

    fn refresh_row(&self, row: usize) {
        unsafe {
            let item_idx = row as i32;
            for col_idx in 0..self.columns.len() {
                let mut sub_item = LVITEMW {
                    mask: LVIF_TEXT,
                    iItem: item_idx,
                    iSubItem: col_idx as i32,
                    pszText: PCWSTR(
                        wide_from_str(&self.rows[row][col_idx].as_string()).unwrap().as_ptr(),
                    ),
                    ..Default::default()
                };
                ListView_SetItem(self.hwnd, &mut sub_item);
            }
        }
    }

    /// Handle double-click for cell editing.
    pub fn on_double_click(&mut self, row: usize, col: usize) {
        // TODO: Show an in-place edit control or dialog
        // For now, just log the intent
        let _ = (row, col);
    }
}

fn wide_from_str(s: &str) -> Result<Vec<u16>> {
    use std::os::windows::ffi::OsStrExt;
    let os_str = std::ffi::OsStr::new(s);
    let mut v: Vec<u16> = os_str.encode_wide().collect();
    v.push(0);
    Ok(v)
}
