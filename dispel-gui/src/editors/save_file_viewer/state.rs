use std::collections::HashMap;

use hexedit::HexEditorState;

/// Section tabs displayed in the save file viewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveFileSection {
    Overview,
    Maps,
    Stats,
    Inventory,
    Identity,
    Events,
    Journal,
    Raw,
}

impl SaveFileSection {
    /// Human-readable label for each section tab.
    pub fn label(&self) -> &'static str {
        match self {
            SaveFileSection::Overview => "Overview",
            SaveFileSection::Maps => "Maps",
            SaveFileSection::Stats => "Stats",
            SaveFileSection::Inventory => "Inventory",
            SaveFileSection::Identity => "Identity",
            SaveFileSection::Events => "Events",
            SaveFileSection::Journal => "Journal",
            SaveFileSection::Raw => "Raw",
        }
    }

    /// All sections in display order.
    pub fn all() -> &'static [SaveFileSection] {
        use SaveFileSection::*;
        &[Overview, Maps, Stats, Inventory, Identity, Events, Journal, Raw]
    }
}

/// One embedded hex editor for a raw/unknown block.
pub struct RawHexViewer {
    pub label: &'static str,
    pub state: HexEditorState,
}

/// State for a single save file viewer tab.
pub struct SaveFileViewerState {
    pub save_file: Option<dispel_core::references::save_file::SaveFile>,
    pub raw_hex_viewers: Vec<RawHexViewer>,
    pub active_section: SaveFileSection,
    pub loading: bool,
    pub error: Option<String>,

    // Per-section navigation
    pub selected_map: Option<usize>,
    pub journal_section: JournalSection,
    pub selected_journal_entry: Option<usize>,
    pub inventory_category: Option<InventoryCategory>,

    // Events table display data (built on load, amortized across views)
    pub events_display_cache: Vec<Vec<String>>,
    pub events_filtered_indices: Vec<usize>,

    // Inventory hex viewers (built on load, shown per-selection)
    pub inventory_hex_viewers: HashMap<InventoryCategory, HexEditorState>,
}

impl Default for SaveFileViewerState {
    fn default() -> Self {
        SaveFileViewerState {
            save_file: None,
            raw_hex_viewers: Vec::new(),
            active_section: SaveFileSection::Overview,
            loading: false,
            error: None,
            selected_map: None,
            journal_section: JournalSection::Main,
            selected_journal_entry: None,
            inventory_category: None,
            events_display_cache: Vec::new(),
            events_filtered_indices: Vec::new(),
            inventory_hex_viewers: HashMap::new(),
        }
    }
}

impl InventoryCategory {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            InventoryCategory::Event => "Event Items",
            InventoryCategory::Misc => "Misc Items",
            InventoryCategory::Edit => "Edit Items",
            InventoryCategory::Weapon => "Weapon Items",
            InventoryCategory::Heal => "Heal Items",
        }
    }

    /// Record size in bytes for this category.
    pub fn record_size(&self) -> usize {
        match self {
            InventoryCategory::Event => 244,
            InventoryCategory::Misc => 264,
            InventoryCategory::Edit => 272,
            InventoryCategory::Weapon => 292,
            InventoryCategory::Heal => 256,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalSection {
    Main,
    Side,
    Trade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InventoryCategory {
    Event,
    Misc,
    Edit,
    Weapon,
    Heal,
}
