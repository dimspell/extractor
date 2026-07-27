use std::path::PathBuf;

use crate::components::filter::{ColumnFilterAction, GlobalFilterMode};
use crate::editors::save_file_viewer::map_preview::PreviewMessage;
use crate::editors::save_file_viewer::state::{
    InventoryCategory, JournalSection, MapsTableKind, SaveFileSection,
};

/// Messages for the save file viewer.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
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
    /// Toggle between entity table and map preview.
    TogglePreview,
    /// Async map file loaded.
    MapPreviewLoaded(usize, Result<MapPreviewLoaded, String>),
    /// Async tileset decode completed.
    MapPreviewTilesReady(usize, Result<MapPreviewTiles, String>),
    /// Async entity sprite loading completed.
    PreviewSpritesReady(usize, Result<PreviewSpritesLoaded, String>),
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
    InventoryTableSort { cat: InventoryCategory, col: usize },
    /// Begin dragging a column resize handle in an inventory table.
    InventoryTableStartResize { cat: InventoryCategory, col: usize },
    /// Reset a column to its default width (double-click on resize handle).
    InventoryTableResetColumnWidth { cat: InventoryCategory, col: usize },
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
    EventsTableSelect { visible_idx: usize },
    /// Toggle sort by a column in the events table.
    EventsTableSort { col: usize },
    /// Begin dragging a column resize handle in the events table.
    EventsTableStartResize { col: usize },
    /// Reset a column to its default width (double-click on resize handle).
    EventsTableResetColumnWidth { col: usize },
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
    JournalTableSort { section: JournalSection, col: usize },
    /// Begin dragging a column resize handle in a journal table.
    JournalTableStartResize { section: JournalSection, col: usize },
    /// Reset a column to its default width (double-click on resize handle).
    JournalTableResetColumnWidth { section: JournalSection, col: usize },
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
    /// Export a table to CSV (triggers async save dialog).
    ExportCsv(TableKey),
    /// Result of an async CSV export.
    CsvExported(Result<std::path::PathBuf, String>),
    /// Map preview interaction (pan, zoom, layer toggles).
    MapPreview(PreviewMessage),
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

impl From<ColumnFilterAction> for TableFilterAction {
    fn from(a: ColumnFilterAction) -> Self {
        match a {
            ColumnFilterAction::ToggleColumnFilterValue(c, v) => {
                TableFilterAction::ToggleColumnFilterValue(c, v)
            }
            ColumnFilterAction::SelectAllColumnFilter(c) => {
                TableFilterAction::SelectAllColumnFilter(c)
            }
            ColumnFilterAction::ClearAllColumnFilter(c) => {
                TableFilterAction::ClearAllColumnFilter(c)
            }
            ColumnFilterAction::CloseColumnFilterModal => TableFilterAction::CloseColumnFilterModal,
            ColumnFilterAction::ColumnFilterSearch(q) => TableFilterAction::ColumnFilterSearch(q),
            ColumnFilterAction::SetMode(m) => TableFilterAction::SetMode(m),
            ColumnFilterAction::QueryChanged(q) => TableFilterAction::QueryChanged(q),
            ColumnFilterAction::ClearAllFilters => TableFilterAction::ClearAllFilters,
            ColumnFilterAction::NextHighlight => TableFilterAction::NextHighlight,
            ColumnFilterAction::PrevHighlight => TableFilterAction::PrevHighlight,
        }
    }
}

// ── Map preview messages ───────────────────────────────────────────────────

/// Async map preview load completed (after `.map` file parse + tileset decode).
#[derive(Clone)]
pub struct MapPreviewLoaded {
    pub map_data: std::sync::Arc<dispel_core::map::MapData>,
    pub diagonal: i32,
    pub map_stem: String,
}

impl std::fmt::Debug for MapPreviewLoaded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapPreviewLoaded")
            .field("diagonal", &self.diagonal)
            .field("map_stem", &self.map_stem)
            .finish_non_exhaustive()
    }
}

/// Decoded tile pixel data ready from async tileset decode.
#[derive(Clone)]
pub struct MapPreviewTiles {
    pub gtl: std::collections::HashMap<i32, iced::widget::image::Handle>,
    pub btl: std::collections::HashMap<i32, iced::widget::image::Handle>,
    /// Decoded internal sprites (thrones, decor, etc.) from the .map file.
    pub internal_sprites: Vec<crate::components::map_render::InternalSpriteHandle>,
}

impl std::fmt::Debug for MapPreviewTiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapPreviewTiles")
            .field("gtl_len", &self.gtl.len())
            .field("btl_len", &self.btl.len())
            .field("internal_sprite_count", &self.internal_sprites.len())
            .finish()
    }
}

/// Decoded entity sprites ready from async sprite loading.
#[derive(Clone)]
pub struct PreviewSpritesLoaded {
    pub sprites: Vec<Option<crate::components::map_render::EntitySpriteHandle>>,
}

impl std::fmt::Debug for PreviewSpritesLoaded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreviewSpritesLoaded")
            .field("sprite_count", &self.sprites.len())
            .finish()
    }
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
