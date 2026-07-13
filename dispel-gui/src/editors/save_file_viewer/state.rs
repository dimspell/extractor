use std::collections::{HashMap, HashSet};

use crate::components::filter::{ColumnFilterOption, GlobalFilterMode};
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
                ("signature_a", 60.0), ("record_index", 60.0), ("signature_b", 60.0),
                ("name", 160.0), ("monster_db_id", 60.0), ("hp_current", 60.0),
                ("hp_maximum", 60.0), ("mp_current", 60.0), ("mp_maximum", 60.0),
                ("walk_speed", 60.0), ("hit_rate", 60.0), ("dodge_rate", 60.0),
                ("offense_rate", 60.0), ("defense_rate", 60.0), ("magic_rate", 60.0),
                ("is_undead", 60.0), ("has_blood", 60.0), ("monster_ai_type", 60.0),
                ("experience_on_kill", 60.0), ("gold_drop_on_kill", 60.0),
                ("unknown_1", 60.0), ("sight_range", 60.0), ("attack_range", 60.0),
                ("spell_slot_1", 60.0), ("spell_slot_2", 60.0), ("spell_slot_3", 60.0),
                ("oversize", 60.0), ("magic_level", 60.0), ("unknown_2", 60.0),
                ("unknown_3", 320.0), ("unknown_4", 60.0), ("unknown_5", 60.0),
                ("current_position_x", 60.0), ("current_position_y", 60.0),
                ("spawn_position_x", 60.0), ("spawn_position_y", 60.0),
                ("unknown_10_coordinate", 60.0), ("unknown_11_coordinate", 60.0),
                ("unknown_12", 60.0), ("unknown_13", 60.0), ("unknown_14", 60.0),
                ("unknown_15", 60.0), ("unknown_16", 60.0), ("unknown_17", 60.0),
                ("unknown_18", 60.0), ("unknown_19", 320.0), ("unknown_20", 60.0),
                ("unknown_21", 60.0), ("unknown_22", 60.0), ("loot_item1", 60.0),
                ("loot_item2", 60.0), ("loot_item3", 60.0), ("mon_ref_padding_12", 60.0),
                ("mon_ref_padding_13", 60.0), ("unknown_23", 60.0), ("unknown_24", 60.0),
                ("unknown_25", 60.0), ("unknown_26", 60.0),
                ("special_attack_chance", 60.0), ("special_attack_duration", 60.0),
                ("unknown_27", 200.0), ("boldness", 60.0), ("attack_speed", 60.0),
                ("unknown_28", 200.0), ("unknown_29", 60.0), ("unknown_30", 320.0),
            ],
            Npcs => &[
                ("name", 160.0), ("role_description", 160.0), ("unknown1", 60.0),
                ("unknown2", 60.0), ("unknown3", 60.0), ("unknown4", 60.0),
                ("unknown5", 60.0), ("unknown6", 60.0), ("unknown7", 60.0),
                ("unknown8", 60.0), ("unknown9", 60.0), ("unknown10", 60.0),
                ("unknown11", 60.0), ("unknown12", 200.0), ("npc_ini_id", 60.0),
                ("unknown13", 320.0), ("npc_ref_party_script_id", 60.0),
                ("npc_ref_show_on_event_id", 60.0), ("unknown14", 60.0),
                ("npc_ref_unknown_1", 60.0), ("npc_ref_waypoint1filled", 60.0),
                ("npc_ref_waypoint1x", 60.0), ("npc_ref_waypoint1y", 60.0),
                ("npc_ref_unknown_2", 60.0), ("npc_ref_look_direction", 60.0),
                ("npc_ref_unknown_9", 60.0), ("npc_ref_waypoint2filled", 60.0),
                ("npc_ref_waypoint2x", 60.0), ("npc_ref_waypoint2y", 60.0),
                ("npc_ref_unknown_3", 60.0), ("npc_ref_unknown_6", 60.0),
                ("npc_ref_unknown_10", 60.0), ("npc_ref_waypoint3filled", 60.0),
                ("npc_ref_waypoint3x", 60.0), ("npc_ref_waypoint3y", 60.0),
                ("npc_ref_unknown_4", 60.0), ("npc_ref_unknown_7", 60.0),
                ("npc_ref_unknown_11", 60.0), ("npc_ref_waypoint4filled", 60.0),
                ("npc_ref_waypoint4x", 60.0), ("npc_ref_waypoint4y", 60.0),
                ("npc_ref_unknown_5", 60.0), ("npc_ref_unknown_8", 60.0),
                ("npc_ref_unknown_12", 60.0), ("npc_ref_unknown_13", 60.0),
                ("npc_ref_unknown_14", 60.0), ("npc_ref_unknown_15", 60.0),
                ("npc_ref_unknown_16", 60.0), ("npc_ref_unknown_17", 60.0),
                ("unknown15", 60.0), ("npc_ref_dialog_id", 60.0), ("unknown16", 320.0),
            ],
            ExtraObjects => &[
                ("unknown_1", 60.0), ("unknown_2", 60.0), ("unknown_3", 60.0),
                ("unknown_4", 60.0), ("unknown_5", 60.0), ("name", 160.0),
                ("unknown_6", 60.0), ("unknown_7", 60.0), ("unknown_8", 60.0),
                ("unknown_9", 60.0), ("unknown_10", 200.0), ("unknown_11", 60.0),
                ("unknown_12", 60.0), ("unknown_13", 60.0), ("unknown_14", 60.0),
                ("unknown_15", 60.0), ("unknown_16", 60.0), ("unknown_17", 60.0),
                ("unknown_18", 60.0), ("unknown_19", 60.0), ("unknown_20", 60.0),
                ("unknown_21", 60.0), ("unknown_22", 60.0), ("unknown_23", 320.0),
                ("unknown_24", 60.0), ("unknown_25", 60.0), ("unknown_26", 60.0),
                ("unknown_27", 60.0), ("unknown_28", 60.0), ("unknown_29", 60.0),
                ("unknown_30", 200.0), ("unknown_31", 200.0), ("unknown_32", 60.0),
                ("unknown_33", 60.0), ("unknown_34", 60.0), ("unknown_35", 60.0),
                ("unknown_36", 60.0), ("unknown_37", 60.0), ("unknown_38", 60.0),
            ],
            Weapon => &[
                ("name", 160.0), ("description", 160.0), ("base_price", 60.0),
                ("weapon_item_id", 60.0), ("health_points", 60.0), ("mana_points", 60.0),
                ("strength", 60.0), ("agility", 60.0), ("wisdom", 60.0),
                ("constitution", 60.0), ("to_dodge", 60.0), ("to_hit", 60.0),
                ("attack", 60.0), ("defense", 60.0), ("magical_strength", 60.0),
                ("durability", 60.0), ("padding2", 60.0), ("padding3", 60.0),
                ("req_strength", 60.0), ("padding4", 60.0), ("req_agility", 60.0),
                ("padding5", 60.0), ("req_wisdom", 60.0), ("padding6", 60.0),
                ("padding7", 60.0), ("padding8", 60.0), ("map_coordinate_x", 60.0),
                ("map_coordinate_y", 60.0), ("unknown_1", 60.0),
            ],
            Heal => &[
                ("name", 160.0), ("description", 160.0), ("base_price", 60.0),
                ("heal_item_id", 60.0), ("health_points", 60.0), ("mana_points", 60.0),
                ("restore_full_health", 60.0), ("restore_full_mana", 60.0),
                ("poison_heal", 60.0), ("petrif_heal", 60.0), ("polimorph_heal", 60.0),
                ("unknown_1", 60.0), ("unknown_2", 60.0), ("map_coordinate_x", 60.0),
                ("map_coordinate_y", 60.0), ("unknown_3", 60.0),
            ],
            Edit => &[
                ("name", 160.0), ("description", 160.0), ("base_price", 60.0),
                ("edit_item_id", 60.0), ("health_points", 60.0), ("mana_points", 60.0),
                ("strength", 60.0), ("agility", 60.0), ("wisdom", 60.0),
                ("constitution", 60.0), ("to_dodge", 60.0), ("to_hit", 60.0),
                ("offense", 60.0), ("defense", 60.0), ("magical_power", 60.0),
                ("item_destroying_power", 60.0), ("unknown_3", 60.0),
                ("modifies_item", 60.0), ("additional_effect", 60.0),
                ("map_coordinate_x", 60.0), ("map_coordinate_y", 60.0),
                ("unknown_4", 60.0),
            ],
            Misc => &[
                ("name", 160.0), ("description", 160.0), ("base_price", 60.0),
                ("unknown_1", 320.0), ("unknown_2", 60.0), ("unknown_3", 60.0),
                ("unknown_4", 60.0), ("unknown_5", 60.0), ("unknown_7", 60.0),
            ],
            Event => &[
                ("name", 160.0), ("description", 160.0), ("base_price", 60.0),
                ("event_item_id", 60.0), ("map_coordinate_x", 60.0),
                ("map_coordinate_y", 60.0), ("unknown_1", 60.0),
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

/// Per-table column filtering state, mirroring the spreadsheet editor.
#[derive(Debug, Clone, Default)]
pub struct TableFilterState {
    /// Hard column filters: column index -> set of allowed values.
    /// An empty set (or missing entry) means "no filter" for that column.
    pub column_filters: HashMap<usize, HashSet<String>>,
    /// Free-text global query applied to all columns.
    pub filter_query: String,
    /// How the global query behaves (remove vs highlight).
    pub filter_mode: GlobalFilterMode,
    /// Column whose filter modal is currently open, if any.
    pub active_column_filter: Option<usize>,
    /// Distinct options for the active column filter modal.
    pub column_filter_options: Vec<ColumnFilterOption>,
    /// Search box text inside the column filter modal.
    pub column_filter_search: String,
    /// Original rows highlighted by the global query in Highlight mode,
    /// stored in catalog order so prev/next navigation is stable.
    pub highlighted_indices: Vec<usize>,
    /// Index (into `highlighted_indices`) for next/prev navigation.
    pub current_highlight_pos: Option<usize>,
}

impl TableFilterState {
    /// Whether any filter (column or query) is currently active.
    pub fn is_active(&self) -> bool {
        !self.column_filters.is_empty() || !self.filter_query.is_empty()
    }

    /// Move the highlight cursor to the next match, with wrap-around.
    pub fn navigate_next_highlight(&mut self) {
        if self.highlighted_indices.is_empty() {
            self.current_highlight_pos = None;
            return;
        }
        let len = self.highlighted_indices.len();
        self.current_highlight_pos = Some(match self.current_highlight_pos {
            Some(pos) => (pos + 1) % len,
            None => 0,
        });
    }

    /// Move the highlight cursor to the previous match, with wrap-around.
    pub fn navigate_prev_highlight(&mut self) {
        if self.highlighted_indices.is_empty() {
            self.current_highlight_pos = None;
            return;
        }
        let len = self.highlighted_indices.len();
        self.current_highlight_pos = Some(match self.current_highlight_pos {
            Some(0) | None => len - 1,
            Some(pos) => pos - 1,
        });
    }

    /// `orig_idx` of the row currently focused via highlight navigation.
    pub fn current_highlight_orig_idx(&self) -> Option<usize> {
        self.current_highlight_pos
            .and_then(|p| self.highlighted_indices.get(p).copied())
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
    /// Shared scroll state consumed by the table widget every frame.
    pub table_state: gui_widgets::TableState,
    /// Column filtering state.
    pub filter: TableFilterState,
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

/// Per-table interaction state (selection / sort / column widths / scroll)
/// shared by the inventory, events, and journal tables.
#[derive(Debug, Clone, Default)]
pub struct TableInteractionState {
    /// Currently selected original row index (highlighted).
    pub selected_orig: Option<usize>,
    /// Active sort column, if any.
    pub sort_column: Option<usize>,
    /// Sort direction for `sort_column`.
    pub sort_ascending: bool,
    /// Per-column widths (px), parallel to `default_columns()`.
    pub column_widths: Vec<f32>,
    /// Shared scroll state consumed by the table widget every frame.
    pub table_state: gui_widgets::TableState,
    /// Column filtering state.
    pub filter: TableFilterState,
}

/// Active column-resize drag for an inventory table.
#[derive(Debug, Clone)]
pub struct InventoryResizeDrag {
    pub cat: InventoryCategory,
    pub col: usize,
    pub anchor_width: f32,
    pub anchor_cursor_x: Option<f32>,
}

/// Active column-resize drag for the events table (single table, no key).
#[derive(Debug, Clone)]
pub struct EventsResizeDrag {
    pub col: usize,
    pub anchor_width: f32,
    pub anchor_cursor_x: Option<f32>,
}

/// Active column-resize drag for a journal table (keyed by section).
#[derive(Debug, Clone)]
pub struct JournalResizeDrag {
    pub section: JournalSection,
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
    /// Which entity sub-table is selected in the Maps section.
    pub selected_entity_kind: MapsTableKind,
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

    // Inventory table interaction state, keyed by category.
    pub inventory_table_states: HashMap<InventoryCategory, TableInteractionState>,
    // Active column-resize drag for an inventory table, if any.
    pub inventory_resizing: Option<InventoryResizeDrag>,

    // Events table interaction state (single table).
    pub events_table_state: TableInteractionState,
    // Active column-resize drag for the events table, if any.
    pub events_resizing: Option<EventsResizeDrag>,

    // Journal table interaction state, keyed by section.
    pub journal_table_states: HashMap<JournalSection, TableInteractionState>,
    // Active column-resize drag for a journal table, if any.
    pub journal_resizing: Option<JournalResizeDrag>,

    // Maps display caches (built on load, one per map at positional index)
    pub maps_display_caches: Vec<MapsDisplayCaches>,

    // Maps table interaction state, indexed by map position then table kind.
    pub maps_table_states: Vec<HashMap<MapsTableKind, MapTableState>>,
    // Active column-resize drag for a maps table, if any.
    pub maps_resizing: Option<MapsTableResizeDrag>,

    // ── Map preview ────────────────────────────────────────────────────────
    /// Whether the map preview canvas is shown instead of the entity table.
    pub show_preview: bool,
    /// Preview state for the currently selected map (replaced on map switch).
    pub map_preview: Option<crate::components::map_preview::MapPreviewState>,
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
            selected_entity_kind: MapsTableKind::Monsters,
            journal_section: JournalSection::Main,
            selected_journal_entry: None,
            inventory_category: None,
            events_display_cache: Vec::new(),
            events_filtered_indices: Vec::new(),
            journal_display_caches: HashMap::new(),
            journal_filtered_indices: HashMap::new(),
            inventory_display_caches: HashMap::new(),
            inventory_filtered_indices: HashMap::new(),
            inventory_table_states: HashMap::new(),
            inventory_resizing: None,
            events_table_state: TableInteractionState::default(),
            events_resizing: None,
            journal_table_states: HashMap::new(),
            journal_resizing: None,
            maps_display_caches: Vec::new(),
            maps_table_states: Vec::new(),
            maps_resizing: None,
            show_preview: false,
            map_preview: None,
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

    /// Default column layout (widths + labels) for this inventory category.
    /// `sort`/`has_filter` are left at their defaults; the view overrides
    /// `width_px` from the per-table state and `sort` from the active sort.
    pub fn default_columns(&self) -> Vec<TableColumn> {
        let defs: &[(&str, f32)] = match self {
            InventoryCategory::Weapon => &[
                ("name",160.0),("description",160.0),("base_price",60.0),("weapon_item_id",60.0),("health_points",60.0),("mana_points",60.0),("strength",60.0),("agility",60.0),("wisdom",60.0),("constitution",60.0),("to_dodge",60.0),("to_hit",60.0),("attack",60.0),("defense",60.0),("magical_strength",60.0),("durability",60.0),("padding2",60.0),("padding3",60.0),("req_strength",60.0),("padding4",60.0),("req_agility",60.0),("padding5",60.0),("req_wisdom",60.0),("padding6",60.0),("padding7",60.0),("padding8",60.0),("unknown_1",60.0),("unknown_2",60.0),("unknown_3",60.0),("unknown_4",60.0),
            ],
            InventoryCategory::Heal => &[
                ("name",160.0),("description",160.0),("base_price",60.0),("heal_item_id",60.0),("health_points",60.0),("mana_points",60.0),("restore_full_health",60.0),("restore_full_mana",60.0),("poison_heal",60.0),("petrif_heal",60.0),("polimorph_heal",60.0),("unknown_1",60.0),("unknown_2",60.0),("unknown_3",60.0),("unknown_4",60.0),("unknown_5",60.0),
            ],
            InventoryCategory::Edit => &[
                ("name",160.0),("description",160.0),("base_price",60.0),("unknown_1",60.0),("unknown_2",60.0),("health_points",60.0),("mana_points",60.0),("strength",60.0),("agility",60.0),("wisdom",60.0),("constitution",60.0),("to_dodge",60.0),("to_hit",60.0),("offense",60.0),("defense",60.0),("magical_power",60.0),("item_destroying_power",60.0),("unknown_3",60.0),("modifies_item",60.0),("additional_effect",60.0),("unknown_4",60.0),("unknown_5",60.0),("unknown_6",60.0),
            ],
            InventoryCategory::Event => &[
                ("name",160.0),("description",160.0),("base_price",60.0),("event_item_id",60.0),("unknown_2",60.0),("unknown_3",60.0),("unknown_4",60.0),
            ],
            InventoryCategory::Misc => &[
                ("name",160.0),("description",160.0),("base_price",60.0),("unknown_1",320.0),("unknown_2",60.0),("unknown_3",60.0),("unknown_4",60.0),("unknown_5",60.0),("unknown_6",60.0),("unknown_7",60.0),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JournalSection {
    Main,
    Side,
    Trade,
}

impl JournalSection {
    /// All journal sub-sections in display order.
    pub fn all() -> &'static [JournalSection] {
        use JournalSection::*;
        &[Main, Side, Trade]
    }

    /// Default column layout for a journal table (same across sub-sections).
    pub fn default_columns(&self) -> Vec<TableColumn> {
        vec![
            TableColumn { width_px: 40.0, label: "#".into(), sort: None, has_filter: false },
            TableColumn { width_px: 200.0, label: "Name".into(), sort: None, has_filter: false },
            TableColumn { width_px: 200.0, label: "Flags (hex)".into(), sort: None, has_filter: false },
        ]
    }
}

/// Default column layout for the events table.
pub fn events_default_columns() -> Vec<TableColumn> {
    vec![
        TableColumn { width_px: 60.0, label: "event_id".into(), sort: None, has_filter: false },
        TableColumn { width_px: 60.0, label: "unknown_1".into(), sort: None, has_filter: false },
        TableColumn { width_px: 60.0, label: "unknown_2".into(), sort: None, has_filter: false },
        TableColumn { width_px: 400.0, label: "script_name".into(), sort: None, has_filter: false },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InventoryCategory {
    Event,
    Misc,
    Edit,
    Weapon,
    Heal,
}
