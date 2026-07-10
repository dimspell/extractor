use std::collections::HashMap;

use gui_widgets::TableColumn;
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

/// Identifies one of the entity tables rendered for a map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapsTableKind {
    Monsters,
    Npcs,
    ExtraObjects,
    Weapon,
    Heal,
    Edit,
    Misc,
    Event,
}

impl MapsTableKind {
    /// All table kinds in the order they are rendered for a map.
    pub fn all() -> &'static [MapsTableKind] {
        use MapsTableKind::*;
        &[Monsters, Npcs, ExtraObjects, Weapon, Heal, Edit, Misc, Event]
    }

    /// Default column layout (widths + labels) for this table kind.
    /// `sort`/`has_filter` are left at their defaults; the view overrides
    /// `width_px` from the per-table state and `sort` from the active sort.
    pub fn default_columns(&self) -> Vec<TableColumn> {
        use MapsTableKind::*;
        let defs: &[(&str, f32)] = match self {
            Monsters => &[
                ("Name", 130.0), ("HP", 80.0), ("MP", 80.0), ("Atk", 42.0),
                ("Def", 42.0), ("Dodge", 48.0), ("Hit", 42.0), ("XP", 42.0),
                ("Gold", 42.0), ("Sight", 42.0), ("Range", 42.0), ("AI", 65.0),
                ("X", 42.0), ("Y", 42.0),
            ],
            Npcs => &[
                ("Name", 130.0), ("Role", 130.0), ("DialogID", 55.0),
                ("PartyScript", 65.0), ("ShowOnEvent", 70.0), ("LookDir", 55.0),
                ("Waypoints", 200.0),
            ],
            ExtraObjects => &[
                ("Name", 130.0), ("X", 50.0), ("Y", 50.0), ("Unk6", 60.0),
                ("Unk11", 60.0), ("Unk32", 60.0),
            ],
            Weapon => &[
                ("Name", 130.0), ("Price", 50.0), ("Atk", 38.0), ("Def", 38.0),
                ("MagStr", 50.0), ("Coords", 90.0),
            ],
            Heal => &[
                ("Name", 130.0), ("Price", 50.0), ("HP", 38.0), ("MP", 38.0),
                ("Coords", 90.0),
            ],
            Edit => &[
                ("Name", 130.0), ("Price", 50.0), ("HP", 38.0), ("MP", 38.0),
                ("Str", 38.0), ("Agi", 38.0), ("Coords", 90.0),
            ],
            Misc => &[("Name", 130.0), ("Price", 50.0), ("Coords", 90.0)],
            Event => &[
                ("Name", 130.0), ("Price", 50.0), ("EventID", 60.0), ("Coords", 90.0),
            ],
        };
        defs.iter()
            .map(|(label, width_px)| TableColumn {
                width_px: *width_px,
                label: (*label).to_string(),
                sort: None,
                has_filter: false,
            })
            .collect()
    }
}

/// Per-table interaction state for one map's entity tables.
#[derive(Debug, Clone, Default)]
pub struct MapTableState {
    /// Currently selected original row index (highlighted).
    pub selected_orig: Option<usize>,
    /// Active sort column, if any.
    pub sort_column: Option<usize>,
    /// Sort direction for `sort_column`.
    pub sort_ascending: bool,
    /// Per-column widths (px), parallel to `default_columns()`.
    pub column_widths: Vec<f32>,
    /// Last reported scroll offset (x, y) for stable scroll across re-renders.
    pub scroll_offset: (f32, f32),
}

/// Active column-resize drag for a maps table.
#[derive(Debug, Clone)]
pub struct MapsTableResizeDrag {
    pub map: usize,
    pub kind: MapsTableKind,
    pub col: usize,
    pub anchor_width: f32,
    pub anchor_cursor_x: Option<f32>,
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

    // Maps table interaction state, indexed by map position then table kind.
    pub maps_table_states: Vec<HashMap<MapsTableKind, MapTableState>>,
    // Active column-resize drag for a maps table, if any.
    pub maps_resizing: Option<MapsTableResizeDrag>,
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
            maps_table_states: Vec::new(),
            maps_resizing: None,
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
