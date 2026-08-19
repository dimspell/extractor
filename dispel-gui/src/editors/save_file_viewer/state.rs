use crate::components::filter::{ColumnFilterOption, GlobalFilterMode};
use crate::editors::save_file_viewer::RawHexEditorData;
use crate::editors::save_file_viewer::message::TableKey;
use dispel_core::SaveFile;
use gui_widgets::TableColumn;
use gui_widgets::components::paragraph_cache::ParagraphCache;
use hexedit::HexEditorState;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// Section tabs displayed in the save file viewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveFileSection {
    Overview,
    Maps,
    SavedViewport,
    Stats,
    Inventory,
    PartyMembers,
    Character,
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
            SaveFileSection::SavedViewport => "Saved Viewport",
            SaveFileSection::Stats => "Stats",
            SaveFileSection::Inventory => "Inventory",
            SaveFileSection::PartyMembers => "Party Members",
            SaveFileSection::Character => "Character",
            SaveFileSection::Events => "Events",
            SaveFileSection::Journal => "Journal",
            SaveFileSection::Raw => "Raw",
        }
    }

    /// All sections in display order.
    pub fn all() -> &'static [SaveFileSection] {
        use SaveFileSection::*;
        &[
            Overview,
            Maps,
            SavedViewport,
            Stats,
            Inventory,
            PartyMembers,
            Character,
            Events,
            Journal,
            Raw,
        ]
    }
}

/// One embedded hex editor for a raw/unknown block.
pub struct RawHexViewer {
    pub label: String,
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
        &[
            Monsters,
            Npcs,
            ExtraObjects,
            Weapon,
            Heal,
            Edit,
            Misc,
            Event,
        ]
    }

    /// Default column layout (widths + labels) for this table kind.
    /// `sort`/`has_filter` are left at their defaults; the view overrides
    /// `width_px` from the per-table state and `sort` from the active sort.
    pub fn default_columns(&self) -> Vec<TableColumn> {
        use MapsTableKind::*;
        let defs: &[(&str, f32)] = match self {
            Monsters => &[
                ("monster_state", 60.0),
                ("record_index", 60.0),
                ("sprite_frame_id", 60.0),
                ("name", 160.0),
                ("monster_db_id", 60.0),
                ("hp_current", 60.0),
                ("hp_maximum", 60.0),
                ("mp_current", 60.0),
                ("mp_maximum", 60.0),
                ("walk_speed", 60.0),
                ("hit_rate", 60.0),
                ("dodge_rate", 60.0),
                ("offense_rate", 60.0),
                ("defense_rate", 60.0),
                ("magic_rate", 60.0),
                ("is_undead", 60.0),
                ("has_blood", 60.0),
                ("monster_ai_type", 60.0),
                ("experience_on_kill", 60.0),
                ("gold_drop_on_kill", 60.0),
                ("distance_range_size", 60.0),
                ("detection_sight_size", 60.0),
                ("aggression_flag", 60.0),
                ("spell_slot_1", 60.0),
                ("spell_slot_2", 60.0),
                ("spell_slot_3", 60.0),
                ("oversize", 60.0),
                ("magic_level", 60.0),
                ("patrol_countdown", 60.0),
                ("behavior_flag", 60.0),
                ("ai_state", 60.0),
                ("ai_sub_state", 60.0),
                ("movement_direction", 60.0),
                ("target_position_x", 60.0),
                ("target_position_y", 60.0),
                ("unknown_runtime_1", 60.0),
                ("unknown_runtime_2", 60.0),
                ("awake_flag", 60.0),
                ("unknown_runtime_3", 60.0),
                ("event_id_on_kill", 60.0),
                ("unknown_5", 60.0),
                ("current_position_x", 60.0),
                ("current_position_y", 60.0),
                ("spawn_position_x", 60.0),
                ("spawn_position_y", 60.0),
                ("home_position_x", 60.0),
                ("home_position_y", 60.0),
                ("unknown_patrol_flag", 60.0),
                ("unknown_cleared_on_death_1", 60.0),
                ("unknown_cleared_on_death_2", 60.0),
                ("spawn_group_id", 60.0),
                ("constructor_marker", 60.0),
                ("unknown_cleared_on_death_3", 60.0),
                ("dead_or_removed_flag", 60.0),
                ("unknown_runtime_flag_0", 60.0),
                ("unknown_map_data", 60.0),
                ("unknown_runtime_4", 60.0),
                ("unknown_runtime_5", 60.0),
                ("unknown_runtime_flag_1", 60.0),
                ("unknown_runtime_6", 60.0),
                ("unknown_runtime_flag_2", 60.0),
                ("unknown_runtime_7", 60.0),
                ("constructor_unknown_negative_one", 60.0),
                ("path_buffer_present_flag", 60.0),
                ("unknown_cleared_on_death_4", 60.0),
                ("loot_item1", 60.0),
                ("loot_item2", 60.0),
                ("loot_item3", 60.0),
                ("force_ai_update", 60.0),
                ("drop_all_loot", 60.0),
                ("respawn_timer", 60.0),
                ("unknown_runtime_8", 60.0),
                ("unknown_runtime_9", 60.0),
                ("special_attack", 60.0),
                ("special_attack_chance", 60.0),
                ("special_attack_duration", 60.0),
                ("unknown_runtime_10", 60.0),
                ("unknown_runtime_11", 60.0),
                ("boldness", 60.0),
                ("attack_speed", 60.0),
                ("guard_flag", 60.0),
                ("unknown_runtime_flag_3", 60.0),
                ("unknown_runtime_12", 60.0),
                ("unknown_runtime_flag_4", 60.0),
                ("unknown_runtime_13", 60.0),
                ("unknown_runtime_14", 60.0),
                ("unknown_runtime_flag_5", 60.0),
                ("unknown_runtime_15", 60.0),
                ("ai_tick_counter", 60.0),
                ("sight_backup", 60.0),
                ("patrol_countdown_backup", 60.0),
                ("hidden_or_delisted_flag", 60.0),
                ("path_buffer_position_x", 60.0),
                ("path_buffer_position_y", 60.0),
                ("nested_summon_flag", 60.0),
                ("nested_summon_record", 320.0),
            ],
            Npcs => &[
                ("name", 160.0),
                ("role_description", 160.0),
                ("movement_state", 60.0),
                ("tile_data_entry", 60.0),
                ("path_progress", 60.0),
                ("current_position_x", 60.0),
                ("current_position_y", 60.0),
                ("last_position_x", 60.0),
                ("last_position_y", 60.0),
                ("target_position_x", 60.0),
                ("target_position_y", 60.0),
                ("path_destination_x", 60.0),
                ("path_destination_y", 60.0),
                ("render_direction_flag", 60.0),
                ("cell_offset_x", 60.0),
                ("cell_offset_y", 60.0),
                ("map_npc_index_plus_500", 60.0),
                ("runtime_state_78", 60.0),
                ("runtime_state_79", 60.0),
                ("path_handle", 60.0),
                ("path_step_counter", 60.0),
                ("npc_ini_id", 60.0),
                ("patrol_waypoint_count", 60.0),
                ("current_patrol_waypoint_index", 60.0),
                ("unknown_runtime_7d", 60.0),
                ("unknown_runtime_7e", 60.0),
                ("unknown_runtime_7f", 60.0),
                ("unknown_runtime_80", 60.0),
                ("current_waypoint_index", 60.0),
                ("unknown_runtime_82", 60.0),
                ("wait_tick_counter", 60.0),
                ("unknown_runtime_90", 60.0),
                ("unknown_runtime_94", 60.0),
                ("npc_ref_party_member_slot", 60.0),
                ("npc_ref_show_on_event_id", 60.0),
                ("npc_ref_movement_mode", 60.0),
                ("waypoint1_filled", 60.0),
                ("waypoint1_x", 60.0),
                ("waypoint1_y", 60.0),
                ("waypoint1_wait_time", 60.0),
                ("waypoint1_facing_direction", 60.0),
                ("waypoint1_reserved", 60.0),
                ("waypoint2_filled", 60.0),
                ("waypoint2_x", 60.0),
                ("waypoint2_y", 60.0),
                ("waypoint2_wait_time", 60.0),
                ("waypoint2_facing_direction", 60.0),
                ("waypoint2_reserved", 60.0),
                ("waypoint3_filled", 60.0),
                ("waypoint3_x", 60.0),
                ("waypoint3_y", 60.0),
                ("waypoint3_wait_time", 60.0),
                ("waypoint3_facing_direction", 60.0),
                ("waypoint3_reserved", 60.0),
                ("waypoint4_filled", 60.0),
                ("waypoint4_x", 60.0),
                ("waypoint4_y", 60.0),
                ("waypoint4_wait_time", 60.0),
                ("waypoint4_facing_direction", 60.0),
                ("waypoint4_reserved", 60.0),
                ("activation_rect_x1", 60.0),
                ("activation_rect_y1", 60.0),
                ("activation_rect_x2", 60.0),
                ("activation_rect_y2", 60.0),
                ("npc_ref_interaction_mode", 60.0),
                ("npc_ref_interaction_result", 60.0),
                ("npc_ref_interaction_range", 60.0),
                ("npc_ref_dialog_id", 60.0),
                ("dialogue_face_sprite_id", 60.0),
                ("move_mode", 60.0),
                ("unknown_runtime_1ac", 60.0),
                ("runtime_target_position_x", 60.0),
                ("runtime_target_position_y", 60.0),
                ("unknown_runtime_1b8", 60.0),
                ("freeze_flag", 60.0),
                ("freeze_counter", 60.0),
            ],
            ExtraObjects => &[
                ("render_state_slot", 60.0),
                ("render_variant_index", 60.0),
                ("current_sprite_frame", 60.0),
                ("map_object_id", 60.0),
                ("extra_definition_id", 60.0),
                ("object_name", 160.0),
                ("object_type", 60.0),
                ("map_x", 60.0),
                ("map_y", 60.0),
                ("direction", 60.0),
                ("interaction_state", 60.0),
                ("requires_key", 60.0),
                ("required_item_and_padding", 60.0),
                ("required_item2_and_padding", 60.0),
                ("requirement_range_2_start", 60.0),
                ("requirement_range_2_end", 60.0),
                ("requirement_range_3_start", 60.0),
                ("requirement_range_3_end", 60.0),
                ("gold_amount", 60.0),
                ("loot_item_and_padding", 60.0),
                ("loot_item_count", 60.0),
                ("additional_loot_1", 60.0),
                ("additional_loot_1_count", 60.0),
                ("additional_loot_2", 60.0),
                ("additional_loot_2_count_and_config", 320.0),
                ("interaction_event_id", 60.0),
                ("interaction_message_id", 60.0),
                ("footprint_width", 60.0),
                ("footprint_height", 60.0),
                ("footprint_orientation", 60.0),
                ("interaction_range", 60.0),
                ("interaction_range_padding", 200.0),
                ("is_quest_element", 60.0),
                ("post_activation_tile_flag", 60.0),
                ("post_activation_footprint_mode", 60.0),
                ("preserve_final_sprite_frame", 60.0),
                ("alternate_render_mode", 60.0),
                ("activation_effect_id", 60.0),
                ("unresolved_activation_effect_flag", 60.0),
                ("activation_effect_padding", 60.0),
                ("active_overlay_enabled", 60.0),
                ("map_object_active", 60.0),
                ("interaction_pending", 60.0),
            ],
            Weapon => &[
                ("name", 160.0),
                ("description", 160.0),
                ("base_price", 60.0),
                ("weapon_item_id", 60.0),
                ("health_points", 60.0),
                ("mana_points", 60.0),
                ("strength", 60.0),
                ("agility", 60.0),
                ("wisdom", 60.0),
                ("constitution", 60.0),
                ("to_dodge", 60.0),
                ("to_hit", 60.0),
                ("attack", 60.0),
                ("defense", 60.0),
                ("magical_strength", 60.0),
                ("durability", 60.0),
                ("padding2", 60.0),
                ("padding3", 60.0),
                ("req_strength", 60.0),
                ("padding4", 60.0),
                ("req_agility", 60.0),
                ("padding5", 60.0),
                ("req_wisdom", 60.0),
                ("padding6", 60.0),
                ("padding7", 60.0),
                ("padding8", 60.0),
                ("map_coordinate_x", 60.0),
                ("map_coordinate_y", 60.0),
                ("unknown_1", 60.0),
            ],
            Heal => &[
                ("name", 160.0),
                ("description", 160.0),
                ("base_price", 60.0),
                ("heal_item_id", 60.0),
                ("health_points", 60.0),
                ("mana_points", 60.0),
                ("restore_full_health", 60.0),
                ("restore_full_mana", 60.0),
                ("poison_heal", 60.0),
                ("petrif_heal", 60.0),
                ("polimorph_heal", 60.0),
                ("unknown_1", 60.0),
                ("unknown_2", 60.0),
                ("map_coordinate_x", 60.0),
                ("map_coordinate_y", 60.0),
                ("unknown_3", 60.0),
            ],
            Edit => &[
                ("name", 160.0),
                ("description", 160.0),
                ("base_price", 60.0),
                ("edit_item_id", 60.0),
                ("health_points", 60.0),
                ("mana_points", 60.0),
                ("strength", 60.0),
                ("agility", 60.0),
                ("wisdom", 60.0),
                ("constitution", 60.0),
                ("to_dodge", 60.0),
                ("to_hit", 60.0),
                ("offense", 60.0),
                ("defense", 60.0),
                ("magical_power", 60.0),
                ("item_destroying_power", 60.0),
                ("unknown_3", 60.0),
                ("modifies_item", 60.0),
                ("additional_effect", 60.0),
                ("map_coordinate_x", 60.0),
                ("map_coordinate_y", 60.0),
                ("unknown_4", 60.0),
            ],
            Misc => &[
                ("name", 160.0),
                ("description", 160.0),
                ("base_price", 60.0),
                ("unknown_1", 320.0),
                ("misc_item_id", 60.0),
                ("map_coordinate_x", 60.0),
                ("map_coordinate_y", 60.0),
                ("unknown_7", 60.0),
            ],
            Event => &[
                ("name", 160.0),
                ("description", 160.0),
                ("base_price", 60.0),
                ("event_item_id", 60.0),
                ("map_coordinate_x", 60.0),
                ("map_coordinate_y", 60.0),
                ("unknown_1", 60.0),
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

/// Per-table interaction state (selection / sort / column widths / scroll / filter)
/// shared by all save-file-viewer tables (maps, inventory, events, journal).
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

/// Active column-resize drag for a table, keyed by `TableKey`.

#[derive(Debug, Clone)]
pub struct ResizeDrag {
    /// Which table the resize is happening on.
    pub key: TableKey,
    /// Column index being dragged.
    pub col: usize,
    /// The column width at the start of the drag.
    pub anchor_width: f32,
    /// Cursor x at the start of the drag (None on first cursor event).
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
    /// Map ID → display name lookup from AllMap.ini (empty if unavailable).
    pub map_name_lookup: HashMap<u32, String>,
    pub loading: bool,
    pub error: Option<String>,
    /// Transient status message (CSV export progress/results, etc.).
    pub status_msg: Option<String>,
    /// Shared paragraph cache reused across all tables (avoids per-frame alloc).
    pub paragraph_cache: ParagraphCache,

    // Per-section navigation
    pub selected_map: Option<usize>,
    /// Which entity sub-table is selected in the Maps section.
    pub selected_entity_kind: MapsTableKind,
    pub journal_section: JournalSection,
    pub selected_journal_entry: Option<usize>,
    pub inventory_category: Option<InventoryCategory>,
    /// Which character table (equipment/belt/inventory placement) is selected.
    pub character_kind: Option<CharacterTableKind>,

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
    // Character display caches (built on load, rendered as TableWidget per kind)
    pub character_display_caches: HashMap<CharacterTableKind, Vec<Vec<String>>>,
    // Character filtered indices (always `(0..n).collect()` — no filtering yet)
    pub character_filtered_indices: HashMap<CharacterTableKind, Vec<usize>>,
    // Character table interaction state, keyed by kind.
    pub character_table_states: HashMap<CharacterTableKind, TableInteractionState>,
    // Events table interaction state (single table).
    pub events_table_state: TableInteractionState,
    // Journal table interaction state, keyed by section.
    pub journal_table_states: HashMap<JournalSection, TableInteractionState>,
    // Maps display caches (built on load, one per map at positional index)
    pub maps_display_caches: Vec<MapsDisplayCaches>,
    // Maps table interaction state, indexed by map position then table kind.
    pub maps_table_states: Vec<HashMap<MapsTableKind, TableInteractionState>>,
    // Active column-resize drag, if any (unified across all tables).
    pub resizing: Option<ResizeDrag>,
    /// Tracks the last `*StartResize` press `(key, col, time)` so that a
    /// second press on the same column within 400 ms is recognised as a
    /// double-press and triggers auto-size instead of starting a new drag.
    pub last_resize_press: Option<(TableKey, usize, Instant)>,

    // ── Map preview ────────────────────────────────────────────────────────
    /// Whether the map preview canvas is shown instead of the entity table.
    pub show_preview: bool,
    /// Preview state for the currently selected map (replaced on map switch).
    pub map_preview: Option<crate::editors::save_file_viewer::map_preview::MapPreviewState>,
}

impl Default for SaveFileViewerState {
    fn default() -> Self {
        SaveFileViewerState {
            save_file: None,
            raw_hex_viewers: Vec::new(),
            active_section: SaveFileSection::Overview,
            map_name_lookup: HashMap::new(),
            loading: false,
            error: None,
            status_msg: None,
            paragraph_cache: ParagraphCache::default(),
            selected_map: None,
            selected_entity_kind: MapsTableKind::Monsters,
            journal_section: JournalSection::Main,
            selected_journal_entry: None,
            inventory_category: None,
            character_kind: None,
            events_display_cache: Vec::new(),
            events_filtered_indices: Vec::new(),
            journal_display_caches: HashMap::new(),
            journal_filtered_indices: HashMap::new(),
            inventory_display_caches: HashMap::new(),
            inventory_filtered_indices: HashMap::new(),
            inventory_table_states: HashMap::new(),
            character_display_caches: HashMap::new(),
            character_filtered_indices: HashMap::new(),
            character_table_states: HashMap::new(),
            events_table_state: TableInteractionState::default(),
            journal_table_states: HashMap::new(),
            maps_display_caches: Vec::new(),
            maps_table_states: Vec::new(),
            resizing: None,
            last_resize_press: None,
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
                ("name", 160.0),
                ("description", 160.0),
                ("base_price", 60.0),
                ("weapon_item_id", 60.0),
                ("health_points", 60.0),
                ("mana_points", 60.0),
                ("strength", 60.0),
                ("agility", 60.0),
                ("wisdom", 60.0),
                ("constitution", 60.0),
                ("to_dodge", 60.0),
                ("to_hit", 60.0),
                ("attack", 60.0),
                ("defense", 60.0),
                ("magical_strength", 60.0),
                ("durability", 60.0),
                ("padding2", 60.0),
                ("padding3", 60.0),
                ("req_strength", 60.0),
                ("padding4", 60.0),
                ("req_agility", 60.0),
                ("padding5", 60.0),
                ("req_wisdom", 60.0),
                ("padding6", 60.0),
                ("padding7", 60.0),
                ("padding8", 60.0),
                ("inventory_instance_id", 60.0),
                ("unknown_2", 60.0),
                ("unknown_3", 60.0),
                ("unknown_4", 60.0),
            ],
            InventoryCategory::Heal => &[
                ("name", 160.0),
                ("description", 160.0),
                ("base_price", 60.0),
                ("heal_item_id", 60.0),
                ("health_points", 60.0),
                ("mana_points", 60.0),
                ("restore_full_health", 60.0),
                ("restore_full_mana", 60.0),
                ("poison_heal", 60.0),
                ("petrif_heal", 60.0),
                ("polimorph_heal", 60.0),
                ("unknown_1", 60.0),
                ("item_type_id", 60.0),
                ("position_index", 60.0),
                ("unknown_4", 60.0),
                ("unknown_5", 60.0),
            ],
            InventoryCategory::Edit => &[
                ("name", 160.0),
                ("description", 160.0),
                ("base_price", 60.0),
                ("unknown_1", 60.0),
                ("unknown_2", 60.0),
                ("health_points", 60.0),
                ("mana_points", 60.0),
                ("strength", 60.0),
                ("agility", 60.0),
                ("wisdom", 60.0),
                ("constitution", 60.0),
                ("to_dodge", 60.0),
                ("to_hit", 60.0),
                ("offense", 60.0),
                ("defense", 60.0),
                ("magical_power", 60.0),
                ("item_destroying_power", 60.0),
                ("unknown_3", 60.0),
                ("modifies_item", 60.0),
                ("additional_effect", 60.0),
                ("item_type_id", 60.0),
                ("unknown_5", 60.0),
                ("unknown_6", 60.0),
            ],
            InventoryCategory::Event => &[
                ("name", 160.0),
                ("description", 160.0),
                ("base_price", 60.0),
                ("event_item_id", 60.0),
                ("item_type_id", 60.0),
                ("unknown_3", 60.0),
                ("unknown_4", 60.0),
            ],
            InventoryCategory::Misc => &[
                ("name", 160.0),
                ("description", 160.0),
                ("base_price", 60.0),
                ("unknown_1", 320.0),
                ("misc_item_id", 60.0),
                ("item_type_id", 60.0),
                ("unknown_4", 60.0),
                ("unknown_5", 60.0),
                ("unknown_6", 60.0),
                ("unknown_7", 60.0),
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

/// Identifies one of the character tables (equipment, belt potions,
/// inventory placement) rendered in the Character section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterTableKind {
    Equipment,
    BeltPotions,
    InventoryPlacement,
}

impl CharacterTableKind {
    /// All character tables in the order they are rendered.
    pub fn all() -> &'static [CharacterTableKind] {
        use CharacterTableKind::*;
        &[Equipment, BeltPotions, InventoryPlacement]
    }

    /// Human-readable label for each character table.
    pub fn label(&self) -> &'static str {
        match self {
            CharacterTableKind::Equipment => "Equipped Equipment",
            CharacterTableKind::BeltPotions => "Belt Potions",
            CharacterTableKind::InventoryPlacement => "Inventory Placement",
        }
    }

    /// Default column layout (widths + labels) for this character table.
    /// `sort`/`has_filter` are left at their defaults; the view overrides
    /// `width_px` from the per-table state and `sort` from the active sort.
    pub fn default_columns(&self) -> Vec<TableColumn> {
        let labels: &[&str] = match self {
            CharacterTableKind::Equipment => &[
                "panel_slot_marker",
                "weapon_catalog_index",
                "weapon_inventory_instance_id",
            ],
            CharacterTableKind::BeltPotions => {
                &["item_category", "item_catalog_index", "icon_x", "icon_y"]
            }
            CharacterTableKind::InventoryPlacement => &[
                "item_category",
                "item_catalog_index",
                "icon_x",
                "icon_y",
                "item_instance_index",
            ],
        };
        labels
            .iter()
            .map(|label| TableColumn {
                width_px: 60.0,
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
            TableColumn {
                width_px: 40.0,
                label: "entry_index".into(),
                sort: None,
                has_filter: false,
            },
            TableColumn {
                width_px: 200.0,
                label: "quest_title".into(),
                sort: None,
                has_filter: false,
            },
            TableColumn {
                width_px: 60.0,
                label: "quest_state[0]".into(),
                sort: None,
                has_filter: false,
            },
            TableColumn {
                width_px: 60.0,
                label: "quest_state[1]".into(),
                sort: None,
                has_filter: false,
            },
            TableColumn {
                width_px: 60.0,
                label: "quest_state[2]".into(),
                sort: None,
                has_filter: false,
            },
            TableColumn {
                width_px: 60.0,
                label: "quest_state[3]".into(),
                sort: None,
                has_filter: false,
            },
            TableColumn {
                width_px: 60.0,
                label: "quest_state[4]".into(),
                sort: None,
                has_filter: false,
            },
            TableColumn {
                width_px: 60.0,
                label: "quest_state[5]".into(),
                sort: None,
                has_filter: false,
            },
            TableColumn {
                width_px: 60.0,
                label: "quest_state[6]".into(),
                sort: None,
                has_filter: false,
            },
            TableColumn {
                width_px: 60.0,
                label: "quest_state[7]".into(),
                sort: None,
                has_filter: false,
            },
            TableColumn {
                width_px: 60.0,
                label: "quest_id".into(),
                sort: None,
                has_filter: false,
            },
            TableColumn {
                width_px: 60.0,
                label: "progress_quest_id_1".into(),
                sort: None,
                has_filter: false,
            },
            TableColumn {
                width_px: 60.0,
                label: "progress_quest_id_2".into(),
                sort: None,
                has_filter: false,
            },
            TableColumn {
                width_px: 60.0,
                label: "is_completed".into(),
                sort: None,
                has_filter: false,
            },
        ]
    }
}

/// Default column layout for the events table.
pub fn events_default_columns() -> Vec<TableColumn> {
    vec![
        TableColumn {
            width_px: 60.0,
            label: "event_id".into(),
            sort: None,
            has_filter: false,
        },
        TableColumn {
            width_px: 60.0,
            label: "unknown_1".into(),
            sort: None,
            has_filter: false,
        },
        TableColumn {
            width_px: 60.0,
            label: "unknown_2".into(),
            sort: None,
            has_filter: false,
        },
        TableColumn {
            width_px: 400.0,
            label: "script_name".into(),
            sort: None,
            has_filter: false,
        },
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

pub fn get_hex_editors(save_file: &SaveFile) -> Vec<RawHexEditorData> {
    // Build embedded hex viewers for unknown/raw blocks
    let hex_editors: Vec<RawHexEditorData> = vec![
        // RawHexEditorData {
        //     label: "Equipped Equipment".into(),
        //     data: save_file
        //         .character_identity
        //         .equipped_equipment
        //         .iter()
        //         .flat_map(|s| {
        //             let mut b = Vec::with_capacity(9);
        //             b.push(s.panel_slot_marker);
        //             b.extend_from_slice(&s.weapon_catalog_index.to_le_bytes());
        //             b.extend_from_slice(&s.weapon_inventory_instance_id.to_le_bytes());
        //             b
        //         })
        //         .collect::<Vec<_>>(),
        // },
        // RawHexEditorData {
        //     label: "Belt Potions".into(),
        //     data: save_file
        //         .character_identity
        //         .belt_potions
        //         .iter()
        //         .flat_map(|s| {
        //             let mut b = Vec::with_capacity(16);
        //             b.extend_from_slice(&s.item_category.to_le_bytes());
        //             b.extend_from_slice(&s.item_catalog_index.to_le_bytes());
        //             b.extend_from_slice(&s.icon_x.to_le_bytes());
        //             b.extend_from_slice(&s.icon_y.to_le_bytes());
        //             b
        //         })
        //         .collect(),
        // },
        // RawHexEditorData {
        //     label: "Inventory Placement".into(),
        //     data: save_file
        //         .character_identity
        //         .inventory_placement
        //         .iter()
        //         .flat_map(|e| {
        //             let mut b = Vec::with_capacity(20);
        //             b.extend_from_slice(&e.unknown_a.to_le_bytes());
        //             b.extend_from_slice(&e.unknown_b.to_le_bytes());
        //             b.extend_from_slice(&e.unknown_c.to_le_bytes());
        //             b.extend_from_slice(&e.unknown_d.to_le_bytes());
        //             b.extend_from_slice(&e.unknown_e.to_le_bytes());
        //             b
        //         })
        //         .collect(),
        // },
        // RawHexEditorData {
        //     label: "Character Stats Header".into(),
        //     data: {
        //         let header = &save_file.character;
        //         let mut bytes = Vec::with_capacity(24);
        //         bytes.push(header.unknown_a);
        //         bytes.extend_from_slice(&header.unknown_b.to_le_bytes());
        //         bytes.extend_from_slice(&header.unknown_block);
        //         bytes
        //     },
        // },
        RawHexEditorData {
            label: "character_identity.unknown_00".into(),
            data: save_file.character_identity.unknown_00.clone(),
        },
        RawHexEditorData {
            label: "character_identity.unknown_02".into(),
            data: save_file.character_identity.unknown_02.clone(),
        },
        RawHexEditorData {
            label: "post_events.block_a".into(),
            data: save_file.post_events.block_a.clone(),
        },
        RawHexEditorData {
            label: "post_events.records".into(),
            data: save_file.post_events.records.clone(),
        },
        RawHexEditorData {
            label: "post_events.block_b".into(),
            data: save_file.post_events.block_b.clone(),
        },
    ];

    // for map in &save_file.maps {
    //     let trailer = &map.extra_objects_trailer;
    //     // `tail_size` (4 bytes) plus the seven-byte trailer header; the five
    //     // ground-item sections are exposed separately below.
    //     let mut data = Vec::with_capacity(4 + 7 + trailer.records.len() * 24);
    //     data.extend_from_slice(&trailer.tail_size.to_le_bytes());
    //     data.extend_from_slice(&(trailer.records.len() as u16).to_le_bytes());
    //     for record in &trailer.records {
    //         record
    //             .write(&mut data)
    //             .expect("writing a trailer record to memory cannot fail");
    //     }
    //     data.push(trailer.automatic_placement_active);
    //     data.extend_from_slice(&trailer.automatic_placement_value.to_le_bytes());
    //     data.extend_from_slice(&trailer.automatic_placement_global_item_index.to_le_bytes());
    //     hex_editors.push(RawHexEditorData {
    //         label: format!("Map {} Extra-Object Trailer", map.map_id),
    //         data,
    //     });
    // }

    hex_editors
}
