//! Reusable ListView spreadsheet component for all DB/INI/ref file editors.
// Uses SysListView32 in LVS_REPORT mode with column headers, row population,
// cell editing, save, and undo/redo support.

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

/// Custom registered message sent from the edit control to the main window
/// wParam = tab_id, lParam = 1 (apply) or 0 (cancel)
pub static WM_SPREADSHEET_EDIT_COMMAND: std::sync::LazyLock<u32> = std::sync::LazyLock::new(|| {
    unsafe { RegisterWindowMessageW(w!("SpreadsheetEditCommand")) }
});

/// Column definition for a spreadsheet.
#[derive(Clone)]
pub struct ColumnDef {
    pub name: String,
    pub width: i32,
    pub align_right: bool,
    /// If true, the column only accepts integer values.
    pub numeric: bool,
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

    /// Try to parse a string into this CellValue.
    /// If numeric is true, only accept integer values.
    pub fn parse(s: &str, numeric: bool) -> Result<Self, String> {
        if numeric {
            s.trim().parse::<i64>()
                .map(CellValue::Integer)
                .map_err(|_| format!("'{}' is not a valid integer", s))
        } else {
            Ok(CellValue::String(s.to_string()))
        }
    }
}

/// A single row of data.
pub type Row = Vec<CellValue>;

/// Undo/redo entry.
#[derive(Clone)]
struct EditEntry {
    row: usize,
    col: usize,
    old_value: CellValue,
    new_value: CellValue,
}

/// The reusable spreadsheet control.
pub struct Spreadsheet {
    pub hwnd: HWND,
    /// The main application window (for edit control parenting and messaging).
    pub hwnd_main: HWND,
    pub columns: Vec<ColumnDef>,
    pub rows: Vec<Row>,
    pub file_path: Option<std::path::PathBuf>,
    pub undo_stack: Vec<EditEntry>,
    pub redo_stack: Vec<EditEntry>,
    pub tab_id: usize,
    /// In-place edit control (created on double-click).
    pub edit_hwnd: Option<HWND>,
    /// Row currently being edited.
    pub edit_row: usize,
    /// Column currently being edited.
    pub edit_col: usize,
}

impl Spreadsheet {
    pub fn new(parent: HWND, hwnd_main: HWND, columns: Vec<ColumnDef>, tab_id: usize) -> Result<Self> {
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

            if hwnd.0.is_null() {
                return Err(Error::from(HRESULT(0x80004005))); // E_FAIL
            }

            // Enable extended styles for full row select and grid lines
            let mut ex_style = ListView_GetExtendedListViewStyle(hwnd);
            ex_style |= LVS_EX_FULLROWSELECT | LVS_EX_GRIDLINES | LVS_EX_DOUBLEBUFFER
                | LVS_EX_HEADERDRAGDROP;
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
                hwnd_main,
                columns,
                rows: Vec::new(),
                file_path: None,
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
                tab_id,
                edit_hwnd: None,
                edit_row: 0,
                edit_col: 0,
            })
        }
    }

    /// Load rows into the spreadsheet.
    pub fn load_rows(&mut self, rows: Vec<Row>) {
        self.cancel_edit();
        self.populate(rows);
    }

    /// Clear all rows and columns.
    pub fn clear(&mut self) {
        self.cancel_edit();
        unsafe {
            ListView_DeleteAllItems(self.hwnd);
            while ListView_DeleteColumn(self.hwnd, 0).as_bool() {}
        }
        self.columns.clear();
        self.rows.clear();
    }

    /// Populate rows from data.
    pub fn populate(&mut self, rows: Vec<Row>) {
        self.rows = rows;
        unsafe {
            ListView_DeleteAllItems(self.hwnd);
            for (row_idx, row) in self.rows.iter().enumerate() {
                let text_first = wide_from_str(&row.get(0).map(|c| c.as_string()).unwrap_or_default()).unwrap();
                let lvi = LVITEMW {
                    mask: LVIF_TEXT | LVIF_PARAM,
                    iItem: row_idx as i32,
                    iSubItem: 0,
                    pszText: PCWSTR(text_first.as_ptr()),
                    lParam: row_idx as LPARAM,
                    ..Default::default()
                };
                let new_item = ListView_InsertItem(self.hwnd, &lvi);

                for (col_idx, cell) in row.iter().enumerate().skip(1) {
                    let text = wide_from_str(&cell.as_string()).unwrap();
                    let mut sub_item = LVITEMW {
                        mask: LVIF_TEXT,
                        iItem: new_item,
                        iSubItem: col_idx as i32,
                        pszText: PCWSTR(text.as_ptr()),
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
            let text = wide_from_str(&self.rows[row][col].as_string()).unwrap();
            let mut sub_item = LVITEMW {
                mask: LVIF_TEXT,
                iItem: row as i32,
                iSubItem: col as i32,
                pszText: PCWSTR(text.as_ptr()),
                ..Default::default()
            };
            ListView_SetItem(self.hwnd, &mut sub_item);
        }

        Ok(())
    }

    /// Get a reference to the rows for save operations.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// Undo the last edit.
    pub fn undo(&mut self) -> bool {
        self.cancel_edit();
        if let Some(entry) = self.undo_stack.pop() {
            self.rows[entry.row][entry.col] = entry.old_value.clone();
            self.redo_stack.push(entry);
            self.refresh_row(entry.row);
            true
        } else {
            false
        }
    }

    /// Redo the last undone edit.
    pub fn redo(&mut self) -> bool {
        self.cancel_edit();
        if let Some(entry) = self.redo_stack.pop() {
            self.rows[entry.row][entry.col] = entry.new_value.clone();
            self.undo_stack.push(entry);
            self.refresh_row(entry.row);
            true
        } else {
            false
        }
    }

    /// Check if undo is available.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    fn refresh_row(&self, row: usize) {
        unsafe {
            for col_idx in 0..self.columns.len() {
                let text = wide_from_str(&self.rows[row][col_idx].as_string()).unwrap();
                let mut sub_item = LVITEMW {
                    mask: LVIF_TEXT,
                    iItem: row as i32,
                    iSubItem: col_idx as i32,
                    pszText: PCWSTR(text.as_ptr()),
                    ..Default::default()
                };
                ListView_SetItem(self.hwnd, &mut sub_item);
            }
        }
    }

    // ── In-place cell editing ─────────────────────────────────────────

    /// Begin editing a cell: create an EDIT control overlay.
    pub fn begin_edit(&mut self, row: usize, col: usize) {
        self.cancel_edit();

        if row >= self.rows.len() || col >= self.columns.len() {
            return;
        }

        self.edit_row = row;
        self.edit_col = col;

        unsafe {
            // Get the cell rectangle relative to the ListView
            let mut rect = RECT::default();
            rect.top = col as i32; // LVM_GETSUBITEMRECT: top = subitem index
            rect.left = LVIR_BOUNDS.0;

            let lparam = LPARAM(&mut rect as *mut RECT as isize);
            let sent = SendMessageW(self.hwnd, LVM_GETSUBITEMRECT, WPARAM(row as usize), lparam);
            if sent.0 == 0 {
                return;
            }

            // Convert rect from ListView client coords to main window client coords
            MapWindowPoints(self.hwnd, self.hwnd_main, &mut rect, 2);

            // Create the EDIT control as a child of the main window
            let h = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                w!("EDIT"),
                None,
                WS_CHILD | WS_VISIBLE | ES_LEFT | ES_AUTOHSCROLL | ES_WANTRETURN,
                rect.left, rect.top,
                (rect.right - rect.left).max(30),
                (rect.bottom - rect.top).max(18),
                self.hwnd_main,
                None,
                GetModuleHandleW(None).unwrap(),
                None,
            );

            if h.0.is_null() {
                return;
            }

            // Subclass the edit control
            let old_proc = SetWindowLongPtrW(h, GWLP_WNDPROC, Self::edit_subclass_proc as isize);
            // Store the original wndproc in GWLP_USERDATA
            SetWindowLongPtrW(h, GWLP_USERDATA, old_proc);

            // Set the edit text to the current cell value
            let cell_text = self.rows[row][col].as_string();
            let wide_text = wide_from_str(&cell_text).unwrap_or_default();
            SetWindowTextW(h, PCWSTR(wide_text.as_ptr()));

            SetFocus(h);
            // Select all text
            SendMessageW(h, EM_SETSEL, WPARAM(0), LPARAM(-1));

            self.edit_hwnd = Some(h);
        }
    }

    /// Apply the in-progress edit (validate and set cell).
    pub fn apply_edit(&mut self) -> bool {
        let edit_hwnd = match self.edit_hwnd {
            Some(hwnd) => hwnd,
            None => return false,
        };

        let row = self.edit_row;
        let col = self.edit_col;

        // Read the edit text
        let text = unsafe {
            let len = GetWindowTextLengthW(edit_hwnd) as usize;
            if len == 0 {
                String::new()
            } else {
                let mut buf = vec![0u16; len + 1];
                GetWindowTextW(edit_hwnd, &mut buf);
                wide_to_string(&buf)
            }
        };

        // Validate based on column type
        let is_numeric = self.columns.get(col).map(|c| c.numeric).unwrap_or(false);
        match CellValue::parse(&text, is_numeric) {
            Ok(value) => {
                self.destroy_edit();
                self.set_cell(row, col, value).is_ok()
            }
            Err(err_msg) => {
                self.destroy_edit();
                unsafe {
                    let wide_msg = wide_from_str(&format!("Validation error:\n{}", err_msg)).unwrap_or_default();
                    MessageBoxW(
                        self.hwnd_main,
                        PCWSTR(wide_msg.as_ptr()),
                        w!("Invalid Input"),
                        MB_OK | MB_ICONWARNING,
                    );
                }
                false
            }
        }
    }

    /// Cancel the in-progress edit without applying.
    pub fn cancel_edit(&mut self) {
        self.destroy_edit();
    }

    fn destroy_edit(&mut self) {
        if let Some(hwnd) = self.edit_hwnd.take() {
            unsafe {
                DestroyWindow(hwnd);
            }
        }
    }

    /// Handle double-click for cell editing.
    /// Returns true if an edit was started.
    pub fn on_double_click(&mut self, row: usize, col: usize) -> bool {
        if row < self.rows.len() && col < self.columns.len() {
            self.begin_edit(row, col);
            true
        } else {
            false
        }
    }

    /// Subclass window proc for the EDIT control.
    /// Enter → post apply message, Escape → post cancel message.
    unsafe extern "system" fn edit_subclass_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        // Get the original window proc stored in GWLP_USERDATA
        let original_proc = GetWindowLongPtrW(hwnd, GWLP_USERDATA);

        match msg {
            WM_CHAR => {
                let ch = wparam.0 as u32;
                if ch == VK_RETURN as u32 || ch == VK_ESCAPE as u32 {
                    // Post a registered custom message to the parent (main window)
                    // wParam = 1 for apply (Enter), 0 for cancel (Escape)
                    let parent = GetParent(hwnd);
                    let cmd_msg = *WM_SPREADSHEET_EDIT_COMMAND;
                    if cmd_msg != 0 {
                        let apply = if ch == VK_RETURN as u32 { WPARAM(1) } else { WPARAM(0) };
                        PostMessageW(parent, cmd_msg, apply, LPARAM(hwnd.0 as isize));
                    }
                    return LRESULT(0);
                }
            }
            WM_NCDESTROY => {
                // Forward to original and return
                if original_proc != 0 {
                    let proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT =
                        std::mem::transmute(original_proc);
                    return proc(hwnd, msg, wparam, lparam);
                }
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            }
            _ => {}
        }

        if original_proc != 0 {
            let proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT =
                std::mem::transmute(original_proc);
            proc(hwnd, msg, wparam, lparam)
        } else {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}

fn wide_from_str(s: &str) -> Result<Vec<u16>> {
    use std::os::windows::ffi::OsStrExt;
    let os_str = std::ffi::OsStr::new(s);
    let mut v: Vec<u16> = os_str.encode_wide().collect();
    v.push(0);
    Ok(v)
}

fn wide_to_string(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}

// LVM_GETSUBITEMRECT requires LVIR_BOUNDS constant
const LVIR_BOUNDS: WPARAM = WPARAM(0);
// LVM_GETSUBITEMRECT = LVM_FIRST + 56
const LVM_GETSUBITEMRECT: u32 = 0x1000 + 56;
// EM_SETSEL
const EM_SETSEL: u32 = 0x00B1;
