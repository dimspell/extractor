use std::path::PathBuf;

use crate::editors::save_file_viewer::state::{InventoryCategory, SaveFileSection};

/// Messages for the save file viewer.
#[derive(Debug, Clone)]
pub enum SaveFileViewerMessage {
    /// Load a save file from disk.
    Load(PathBuf),
    /// Result of loading a save file.
    Loaded(Result<SaveFileLoaded, String>),
    /// Switch to a different section.
    SelectSection(SaveFileSection),
    /// Select an inventory category to view.
    SelectCategory(InventoryCategory),
    /// Route a hex editor message to an embedded raw-section viewer.
    HexViewer(usize, hexedit::HexEditorMessage),
}

/// Data returned after a successful save file load.
#[derive(Debug, Clone)]
pub struct SaveFileLoaded {
    pub save_file: dispel_core::references::save_file::SaveFile,
    pub hex_editors: Vec<RawHexEditorData>,
}

/// Data to initialize one embedded hex editor for a raw section.
#[derive(Debug, Clone)]
pub struct RawHexEditorData {
    pub label: &'static str,
    pub data: Vec<u8>,
}
