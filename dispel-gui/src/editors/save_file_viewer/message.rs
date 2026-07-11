use std::path::PathBuf;

use crate::editors::save_file_viewer::state::{
    GlobalFilterMode, InventoryCategory, JournalSection, MapsTableKind, SaveFileSection,
};

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
    /// Select which entity sub-table (monsters, NPCs, ground items, etc.) to view.
    SelectEntityKind(MapsTableKind),
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
    /// Select a row in the inventory table for a category.
    InventoryTableSelect {
        cat: InventoryCategory,
        visible_idx: usize,
    },
    /// Toggle sort by a column in the inventory table for a category.
    InventoryTableSort {
        cat: InventoryCategory,
        col: usize,
    },
    /// Begin dragging a column resize handle in an inventory table.
    InventoryTableStartResize {
        cat: InventoryCategory,
        col: usize,
    },
    /// Reset a column to its default width (double-click on resize handle).
    InventoryTableResetColumnWidth {
        cat: InventoryCategory,
        col: usize,
    },
    /// Cursor moved while a column resize drag is active.
    InventoryTableResizeCursor(f32),
    /// Column resize drag finished.
    InventoryTableEndResize,
    /// Inventory table scrolled; persist the offset for stable re-renders.
    InventoryTableScroll {
        cat: InventoryCategory,
        x: f32,
        y: f32,
        viewport_height: f32,
    },
    /// Select a row in the events table.
    EventsTableSelect {
        visible_idx: usize,
    },
    /// Toggle sort by a column in the events table.
    EventsTableSort {
        col: usize,
    },
    /// Begin dragging a column resize handle in the events table.
    EventsTableStartResize {
        col: usize,
    },
    /// Reset a column to its default width (double-click on resize handle).
    EventsTableResetColumnWidth {
        col: usize,
    },
    /// Cursor moved while a column resize drag is active.
    EventsTableResizeCursor(f32),
    /// Column resize drag finished.
    EventsTableEndResize,
    /// Events table scrolled; persist the offset for stable re-renders.
    EventsTableScroll {
        x: f32,
        y: f32,
        viewport_height: f32,
    },
    /// Select a row in a journal table.
    JournalTableSelect {
        section: JournalSection,
        visible_idx: usize,
    },
    /// Toggle sort by a column in a journal table.
    JournalTableSort {
        section: JournalSection,
        col: usize,
    },
    /// Begin dragging a column resize handle in a journal table.
    JournalTableStartResize {
        section: JournalSection,
        col: usize,
    },
    /// Reset a column to its default width (double-click on resize handle).
    JournalTableResetColumnWidth {
        section: JournalSection,
        col: usize,
    },
    /// Cursor moved while a column resize drag is active.
    JournalTableResizeCursor(f32),
    /// Column resize drag finished.
    JournalTableEndResize,
    /// Journal table scrolled; persist the offset for stable re-renders.
    JournalTableScroll {
        section: JournalSection,
        x: f32,
        y: f32,
        viewport_height: f32,
    },
    /// A unified column-filtering action routed to one of the viewer tables.
    TableFilter {
        /// Which table (keyed identically to the per-table select/sort messages).
        key: TableKey,
        /// What to do with that table's filter state.
        action: TableFilterAction,
    },
}

/// Identifies a single save-file-viewer table for filter routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableKey {
    /// One of a map's entity tables (map position + table kind).
    Map(usize, MapsTableKind),
    /// An inventory table for a single category.
    Inventory(InventoryCategory),
    /// The single events table.
    Events,
    /// A journal table for one sub-section.
    Journal(JournalSection),
}

/// Filtering actions shared by every viewer table (mirrors the spreadsheet editor).
#[derive(Debug, Clone)]
pub enum TableFilterAction {
    /// Open the multi-select filter modal for a column.
    OpenColumnFilter(usize),
    /// Toggle whether a single value is selected in the active column filter.
    ToggleColumnFilterValue(usize, String),
    /// Select every (search-filtered) value in the active column filter.
    SelectAllColumnFilter(usize),
    /// Clear every (search-filtered) value in the active column filter.
    ClearAllColumnFilter(usize),
    /// Update the search box text inside the column filter modal.
    ColumnFilterSearch(String),
    /// Close the column filter modal without applying (state is kept).
    CloseColumnFilterModal,
    /// Clear the hard filter on a specific column.
    ClearColumnFilter(usize),
    /// Right-click quick-filter: set this column's filter to exactly this value.
    QuickFilter(usize, String),
    /// Update the free-text global query box.
    QueryChanged(String),
    /// Switch between FilterOut and Highlight global query behaviour.
    SetMode(GlobalFilterMode),
    /// Clear every column filter and the global query.
    ClearAllFilters,
    /// Jump to the next highlighted row (Highlight mode).
    NextHighlight,
    /// Jump to the previous highlighted row (Highlight mode).
    PrevHighlight,
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
