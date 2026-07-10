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

/// Cached display rows for one map's entity tables.
/// `maps_display_caches[i]` corresponds to `save_file.maps[i]` (positional index).
pub struct MapsDisplayCaches {
    pub monsters: Vec<Vec<String>>,
    pub monsters_indices: Vec<usize>,
    pub npcs: Vec<Vec<String>>,
    pub npcs_indices: Vec<usize>,
    pub extra_objects: Vec<Vec<String>>,
    pub extra_objects_indices: Vec<usize>,
    pub draw_items_weapon: Vec<Vec<String>>,
    pub draw_items_weapon_indices: Vec<usize>,
    pub draw_items_heal: Vec<Vec<String>>,
    pub draw_items_heal_indices: Vec<usize>,
    pub draw_items_edit: Vec<Vec<String>>,
    pub draw_items_edit_indices: Vec<usize>,
    pub draw_items_misc: Vec<Vec<String>>,
    pub draw_items_misc_indices: Vec<usize>,
    pub draw_items_event: Vec<Vec<String>>,
    pub draw_items_event_indices: Vec<usize>,
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

    // Journal display caches (built on load, indexed by JournalSection)
    pub journal_display_caches: std::collections::HashMap<JournalSection, Vec<Vec<String>>>,
    pub journal_filtered_indices: std::collections::HashMap<JournalSection, Vec<usize>>,

    // Inventory display caches (built on load, rendered as TableWidget per category)
    pub inventory_display_caches: HashMap<InventoryCategory, Vec<Vec<String>>>,

    // Inventory filtered indices (always `(0..n).collect()` — no filtering yet)
    pub inventory_filtered_indices: HashMap<InventoryCategory, Vec<usize>>,

    // Maps display caches (built on load, one per map at positional index)
    pub maps_display_caches: Vec<MapsDisplayCaches>,
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
            journal_display_caches: HashMap::new(),
            journal_filtered_indices: HashMap::new(),
            inventory_display_caches: HashMap::new(),
            inventory_filtered_indices: HashMap::new(),
            maps_display_caches: Vec::new(),
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
