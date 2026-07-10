use std::path::PathBuf;

use crate::editors::save_file_viewer::state::{InventoryCategory, JournalSection, MapsTableKind, SaveFileSection};

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
    /// Select a journal sub-section (Main/Side/Trade).
    SelectJournalSection(JournalSection),
    /// Select a map in the Maps section.
    SelectMap(usize),
    /// Select a row in one of a map's entity tables.
    MapsTableSelect {
        map: usize,
        kind: MapsTableKind,
        visible_idx: usize,
    },
    /// Toggle sort by a column in one of a map's entity tables.
    MapsTableSort {
        map: usize,
        kind: MapsTableKind,
        col: usize,
    },
    /// Begin dragging a column resize handle in one of a map's entity tables.
    MapsTableStartResize {
        map: usize,
        kind: MapsTableKind,
        col: usize,
    },
    /// Reset a column to its default width (double-click on resize handle).
    MapsTableResetColumnWidth {
        map: usize,
        kind: MapsTableKind,
        col: usize,
    },
    /// Cursor moved while a column resize drag is active.
    MapsTableResizeCursor(f32),
    /// Column resize drag finished.
    MapsTableEndResize,
    /// Table scrolled; persist the offset for stable re-renders.
    MapsTableScroll {
        map: usize,
        kind: MapsTableKind,
        x: f32,
        y: f32,
        viewport_height: f32,
    },
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
