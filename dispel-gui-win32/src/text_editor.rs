//! Text editor using Scintilla control for script, INI, dialogue, and text files.
// Supports syntax highlighting, undo/redo, find/replace, and file I/O.

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Storage::FileSystem::*;
use std::path::Path;

const ID_TEXT_FIND: u16 = 5001;
const ID_TEXT_REPLACE: u16 = 5002;
const ID_TEXT_GOTO: u16 = 5003;

/// Scintilla lexer constants.
const SCLEX_NULL: i32 = 0;
const SCLEX_CPP: i32 = 3;
const SCLEX_PYTHON: i32 = 2;
const SCLEX_LUA: i32 = 4;

/// Scintilla margin constants.
const SC_MARGIN_NUMBER: i32 = 0;
const SC_MARGIN_SYMBOL: i32 = 1;

/// Scintilla marker constants.
const SC_MARKNUM_FOLDER: i32 = 25;
const SC_MARKNUM_FOLDEROPEN: i32 = 26;
const SC_MARKNUM_SUBFOLDER: i32 = 27;
const SC_MARKNUM_TAIL: i32 = 28;

/// Scintilla search flags.
const SCFIND_FORWARD: u32 = 0;
const SCFIND_BACKWARD: u32 = 1;
const SCFIND_WHOLEWORD: u32 = 2;
const SCFIND_MATCHCASE: u32 = 4;
const SCFIND_REGEXP: u32 = 8;

/// Scintilla code page.
const SC_CP_UTF8: u32 = 65001;

/// Text editor language types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextLanguage {
    Plain,
    Scr,
    Dlg,
    Pgp,
    Ini,
}

/// Scintilla-based text editor.
pub struct TextEditor {
    pub hwnd: HWND,
    pub file_path: Option<PathBuf>,
    pub modified: bool,
    pub language: TextLanguage,
}

impl TextEditor {
    pub fn new(parent: HWND) -> Result<Self> {
        unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("Scintilla"),
                None,
                WS_CHILD | WS_VISIBLE | WS_HSCROLL | WS_VSCROLL | WS_BORDER,
                0, 0, 0, 0,
                parent,
                None,
                GetModuleHandleW(None)?,
                None,
            );

            // Set code page to UTF-8
            SendMessageW(hwnd, SCI_SETCODEPAGE, WPARAM(SC_CP_UTF8), LPARAM(0));

            // Enable margin for line numbers
            SendMessageW(hwnd, SCI_SETMARGINWIDTHN, WPARAM(SC_MARGIN_NUMBER), LPARAM(40));

            // Set default style
            SendMessageW(hwnd, SCI_STYLESETSIZE, WPARAM(0), LPARAM(10));
            let font_name: Vec<u16> = OsStr::new("Consolas")
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            SendMessageW(hwnd, SCI_STYLESETFONT, WPARAM(0), LPARAM(font_name.as_ptr() as isize));

            // Set null lexer initially
            SendMessageW(hwnd, SCI_SETLEXER, WPARAM(SCLEX_NULL), LPARAM(0));

            Ok(Self {
                hwnd,
                file_path: None,
                modified: false,
                language: TextLanguage::Plain,
            })
        }
    }

    /// Load file content into the editor.
    pub fn load_file(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        self.file_path = Some(path.to_path_buf());
        self.modified = false;

        unsafe {
            let wide_content: Vec<u16> = content.encode_utf16().chain(std::iter::once(0)).collect();
            SendMessageW(self.hwnd, SCI_SETTEXT, WPARAM(0), LPARAM(wide_content.as_ptr() as isize));
            SendMessageW(self.hwnd, SCI_SETSAVEPOINT, WPARAM(0), LPARAM(0));
        }

        Ok(())
    }

    /// Save file content to disk.
    pub fn save_file(&self) -> Result<()> {
        let path = self.file_path.as_ref().ok_or_else(|| {
            Error::from(HRESULT(0x80070057))
        })?;

        let content = self.get_text()?;
        std::fs::write(path, content)?;

        unsafe {
            SendMessageW(self.hwnd, SCI_SETSAVEPOINT, WPARAM(0), LPARAM(0));
        }

        Ok(())
    }

    /// Get the current text content.
    pub fn get_text(&self) -> Result<String> {
        unsafe {
            let len = SendMessageW(self.hwnd, SCI_GETTEXTLENGTH, WPARAM(0), LPARAM(0)) as usize;
            let mut buf = vec![0u16; len + 1];
            SendMessageW(
                self.hwnd,
                SCI_GETTEXT,
                WPARAM(len + 1),
                LPARAM(buf.as_mut_ptr() as isize),
            );
            let text = String::from_utf16_lossy(&buf[..len]);
            Ok(text)
        }
    }

    /// Set the text content.
    pub fn set_text(&mut self, text: &str) -> Result<()> {
        unsafe {
            let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            SendMessageW(self.hwnd, SCI_SETTEXT, WPARAM(0), LPARAM(wide.as_ptr() as isize));
            SendMessageW(self.hwnd, SCI_SETSAVEPOINT, WPARAM(0), LPARAM(0));
        }
        self.modified = false;
        Ok(())
    }

    /// Undo the last edit.
    pub fn undo(&self) -> bool {
        unsafe {
            SendMessageW(self.hwnd, SCI_UNDO, WPARAM(0), LPARAM(0)).as_bool()
        }
    }

    /// Redo the last undone edit.
    pub fn redo(&self) -> bool {
        unsafe {
            SendMessageW(self.hwnd, SCI_REDO, WPARAM(0), LPARAM(0)).as_bool()
        }
    }

    /// Check if undo is available.
    pub fn can_undo(&self) -> bool {
        unsafe {
            SendMessageW(self.hwnd, SCI_CANUNDO, WPARAM(0), LPARAM(0)).as_bool()
        }
    }

    /// Check if redo is available.
    pub fn can_redo(&self) -> bool {
        unsafe {
            SendMessageW(self.hwnd, SCI_CANREDO, WPARAM(0), LPARAM(0)).as_bool()
        }
    }

    /// Find text in the editor.
    pub fn find(&self, text: &str) -> bool {
        unsafe {
            let search_text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let lfr = SCFINDREGEX {
                chrg: CHARRANGE { cpMin: 0, cpMax: -1 },
                lpstrText: search_text.as_ptr(),
                iMessage: SCI_SEARCHINTARGET,
            };
            let result = SendMessageW(
                self.hwnd,
                SCI_SEARCHINTARGET,
                WPARAM(search_text.len()),
                LPARAM(search_text.as_ptr() as isize),
            );
            result >= 0
        }
    }

    /// Replace text in the editor.
    pub fn replace(&self, find_text: &str, replace_text: &str) -> bool {
        unsafe {
            if !self.find(find_text) {
                return false;
            }

            let replace_wide: Vec<u16> = replace_text.encode_utf16().chain(std::iter::once(0)).collect();
            SendMessageW(
                self.hwnd,
                SCI_REPLACETARGET,
                WPARAM(replace_wide.len()),
                LPARAM(replace_wide.as_ptr() as isize),
            );
            true
        }
    }

    /// Apply syntax highlighting for the current language.
    pub fn apply_syntax_highlighting(&self) {
        let lexer = match self.language {
            TextLanguage::Plain => SCLEX_NULL,
            TextLanguage::Scr => SCLEX_CPP,
            TextLanguage::Dlg => SCLEX_NULL,
            TextLanguage::Pgp => SCLEX_NULL,
            TextLanguage::Ini => SCLEX_NULL,
        };
        unsafe {
            SendMessageW(self.hwnd, SCI_SETLEXER, WPARAM(lexer), LPARAM(0));
        }
    }

    /// Set the language for syntax highlighting.
    pub fn set_language(&mut self, language: TextLanguage) {
        self.language = language;
        self.apply_syntax_highlighting();
    }

    /// Check if the document has been modified.
    pub fn is_modified(&self) -> bool {
        unsafe {
            SendMessageW(self.hwnd, SCI_GETMODIFY, WPARAM(0), LPARAM(0)).as_bool()
        }
    }
}
