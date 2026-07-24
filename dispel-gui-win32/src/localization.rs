//! Localization manager UI for translation management.

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use std::path::Path;

/// Localization manager state.
pub struct LocalizationManager {
    pub hwnd: HWND,
    pub language: String,
    pub source_path: Option<PathBuf>,
    pub translations: Vec<TranslationEntry>,
}

/// A single translation entry.
pub struct TranslationEntry {
    pub key: String,
    pub source_text: String,
    pub translated_text: String,
}

impl LocalizationManager {
    pub fn new(parent: HWND) -> Result<Self> {
        unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                None,
                WS_CHILD | WS_VISIBLE | SS_OWNERDRAW,
                0, 0, 0, 0,
                parent,
                None,
                GetModuleHandleW(None)?,
                None,
            );

            Ok(Self {
                hwnd,
                language: String::from("en"),
                source_path: None,
                translations: Vec::new(),
            })
        }
    }

    /// Load translation source file.
    pub fn load_source(&mut self, path: &Path) -> Result<()> {
        self.source_path = Some(path.to_path_buf());
        // TODO: Parse translation file and populate entries
        Ok(())
    }

    /// Set the target language.
    pub fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
    }

    /// Export translations to a file.
    pub fn export_translations(&self, path: &Path) -> Result<()> {
        // TODO: Write translations to file
        Ok(())
    }
}
