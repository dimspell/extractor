use std::path::{Path, PathBuf};

use super::provider::BufferProvider;

/// Default cell width — 16 bytes per row matches every other hex editor on
/// the planet and keeps the address column the same width across files.
pub const DEFAULT_BYTES_PER_ROW: u8 = 16;

pub struct HexEditorState {
    pub path: PathBuf,
    pub name: String,
    pub provider: BufferProvider,
    pub bytes_per_row: u8,
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
                error: None,
            },
            Err(e) => Self {
                path: path.to_path_buf(),
                name,
                provider: BufferProvider::default(),
                bytes_per_row: DEFAULT_BYTES_PER_ROW,
                error: Some(e.to_string()),
            },
        }
    }
}
