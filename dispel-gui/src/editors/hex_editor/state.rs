use std::path::{Path, PathBuf};

use super::provider::{BufferProvider, HexProvider};
use super::selection::Selection;

/// Default cell width — 16 bytes per row matches every other hex editor on
/// the planet and keeps the address column the same width across files.
pub const DEFAULT_BYTES_PER_ROW: u8 = 16;

pub struct HexEditorState {
    pub path: PathBuf,
    pub name: String,
    pub provider: BufferProvider,
    pub bytes_per_row: u8,
    pub selection: Selection,
    pub error: Option<String>,
}

impl HexEditorState {
    pub fn load_from_path(path: &Path) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        match std::fs::read(path) {
            Ok(bytes) => Self {
                path: path.to_path_buf(),
                name,
                provider: BufferProvider::from_bytes(bytes),
                bytes_per_row: DEFAULT_BYTES_PER_ROW,
                selection: Selection::default(),
                error: None,
            },
            Err(e) => Self {
                path: path.to_path_buf(),
                name,
                provider: BufferProvider::default(),
                bytes_per_row: DEFAULT_BYTES_PER_ROW,
                selection: Selection::default(),
                error: Some(e.to_string()),
            },
        }
    }

    /// Largest valid byte address, or 0 for an empty file.
    pub fn max_addr(&self) -> u64 {
        self.provider.len().saturating_sub(1)
    }
}
