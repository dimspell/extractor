//! Hex editor using Scintilla hex mode for byte-level editing.

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use std::path::Path;

/// Hex editor using Scintilla control in hex mode.
pub struct HexEditor {
    pub hwnd: HWND,
    pub file_path: Option<PathBuf>,
    pub data: Vec<u8>,
    pub modified: bool,
}

impl HexEditor {
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
            SendMessageW(hwnd, SCI_SETCODEPAGE, WPARAM(65001), LPARAM(0));

            // Use hex mode lexer
            SendMessageW(hwnd, SCI_SETLEXER, WPARAM(4), LPARAM(0));

            Ok(Self {
                hwnd,
                file_path: None,
                data: Vec::new(),
                modified: false,
            })
        }
    }

    /// Load a binary file into the hex editor.
    pub fn load_file(&mut self, path: &Path) -> Result<()> {
        self.data = std::fs::read(path)?;
        self.file_path = Some(path.to_path_buf());
        self.modified = false;

        unsafe {
            // Display hex data in Scintilla
            let hex_text = self.format_hex_view();
            let wide: Vec<u16> = hex_text.encode_utf16().chain(std::iter::once(0)).collect();
            SendMessageW(self.hwnd, SCI_SETTEXT, WPARAM(0), LPARAM(wide.as_ptr() as isize));
            SendMessageW(self.hwnd, SCI_SETSAVEPOINT, WPARAM(0), LPARAM(0));
        }

        Ok(())
    }

    /// Save the current hex data back to the file.
    pub fn save_file(&self) -> Result<()> {
        let path = self.file_path.as_ref().ok_or_else(|| {
            Error::from(HRESULT(0x80070057))
        })?;

        // TODO: Parse the hex view text back to binary data
        // For now, save the original data
        if !self.data.is_empty() {
            std::fs::write(path, &self.data)?;
        }

        Ok(())
    }

    /// Format binary data as hex view text.
    fn format_hex_view(&self) -> String {
        let mut result = String::new();
        for (i, chunk) in self.data.chunks(16).enumerate() {
            let offset = i * 16;
            result.push_str(&format!("{:08X}  ", offset));

            // Hex bytes
            for (j, byte) in chunk.iter().enumerate() {
                result.push_str(&format!("{:02X} ", byte));
                if j == 7 {
                    result.push(' ');
                }
            }
            // Pad to 16 bytes
            for _ in chunk.len()..16 {
                result.push_str("   ");
            }
            if chunk.len() > 8 {
                result.push(' ');
            }

            // ASCII representation
            result.push(' ');
            for byte in chunk {
                if byte.is_ascii_graphic() || byte == b' ' {
                    result.push(*byte as char);
                } else {
                    result.push('.');
                }
            }

            result.push('\n');
        }
        result
    }

    /// Get the raw binary data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Check if the document has been modified.
    pub fn is_modified(&self) -> bool {
        self.modified
    }
}
