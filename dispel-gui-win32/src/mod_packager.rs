//! Mod packager UI for creating and packaging game mods.

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use std::path::Path;

/// Mod packager state.
pub struct ModPackager {
    pub hwnd: HWND,
    pub mod_name: String,
    pub mod_version: String,
    pub mod_description: String,
    pub source_path: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub files: Vec<ModFileEntry>,
}

/// A single file entry in a mod package.
pub struct ModFileEntry {
    pub source_path: PathBuf,
    pub relative_path: String,
    pub included: bool,
}

impl ModPackager {
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
                mod_name: String::new(),
                mod_version: String::from("1.0.0"),
                mod_description: String::new(),
                source_path: None,
                output_path: None,
                files: Vec::new(),
            })
        }
    }

    /// Set the source directory for the mod.
    pub fn set_source(&mut self, path: &Path) {
        self.source_path = Some(path.to_path_buf());
        // TODO: Scan directory for game files to include
    }

    /// Set the output path for the mod package.
    pub fn set_output(&mut self, path: &Path) {
        self.output_path = Some(path.to_path_buf());
    }

    /// Build the mod package.
    pub fn build(&self) -> Result<()> {
        // TODO: Create zip archive with selected files and manifest
        Ok(())
    }
}
