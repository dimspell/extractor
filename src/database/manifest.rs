//! Single source of truth for the SQLite schema.
//!
//! Everything about the shape of the database lives in this one list:
//!
//! * [`super::create_all`] executes every `create_sql` in declaration order.
//! * [`super::drop_all`] executes `DROP TABLE` in *reverse* declaration order,
//!   so the set of dropped tables is always exactly the set of created tables.
//!   A table can no longer be created but never dropped (the old
//!   `party_pgps` drift) because there is no separate hand-written list.
//! * [`super::insert_sql`] returns the canonical INSERT statement for a table,
//!   so parsers no longer embed `include_str!("../queries/insert_*.sql")`
//!   paths of their own.
//!
//! To add a table, add one `table!(...)` entry below. To remove one, delete the
//! entry. Nothing else needs to be kept in sync.

/// One table in the schema.
pub(crate) struct TableSpec {
    /// SQL table name. Also the key used by [`super::insert_sql`].
    pub name: &'static str,
    /// `CREATE TABLE IF NOT EXISTS ...` statement.
    pub create_sql: &'static str,
    /// Canonical `INSERT OR REPLACE ...` statement.
    pub insert_sql: &'static str,
}

macro_rules! table {
    ($name:literal, $create:literal, $insert:literal) => {
        TableSpec {
            name: $name,
            create_sql: include_str!(concat!("../queries/create_table_", $create, ".sql")),
            insert_sql: include_str!(concat!("../queries/insert_", $insert, ".sql")),
        }
    };
}

/// The complete schema, in creation order.
///
/// `create_table_chdata.sql` intentionally creates the `ch_datas` table, hence
/// the mismatch between the first and second column for that entry.
pub(crate) static TABLES: &[TableSpec] = &[
    table!("monster_ref_files", "monster_ref_files", "monster_ref_file"),
    table!("monster_refs", "monster_refs", "monster_ref"),
    table!("messages", "messages", "message"),
    table!("events", "events", "event"),
    table!("extras", "extras", "extra"),
    table!("fog_factors", "fog_factors", "fog_factor"),
    table!("extra_ref_files", "extra_ref_files", "extra_ref_file"),
    table!("extra_refs", "extra_refs", "extra_ref"),
    table!("weapons", "weapons", "weapon"),
    table!("edit_items", "edit_items", "edit_item"),
    table!("event_items", "event_items", "event_item"),
    table!("event_npc_refs", "event_npc_refs", "event_npc_ref"),
    table!("misc_items", "misc_items", "misc_item"),
    table!("heal_items", "heal_items", "heal_item"),
    table!("stores", "stores", "store"),
    table!("store_products", "store_products", "store_product"),
    table!("monsters", "monsters", "monster"),
    table!("maps", "maps", "map"),
    table!("monster_inis", "monster_inis", "monster_ini"),
    table!("npc_inis", "npc_inis", "npc_ini"),
    table!("npc_ref_files", "npc_ref_files", "npc_ref_file"),
    table!("npc_refs", "npc_refs", "npc_ref"),
    table!("wave_inis", "wave_inis", "wave_ini"),
    table!("map_inis", "map_inis", "map_ini"),
    table!("draw_items", "draw_items", "draw_item"),
    table!(
        "dialogue_script_files",
        "dialogue_script_files",
        "dialogue_script_file"
    ),
    table!("dialogue_scripts", "dialogue_scripts", "dialogue_scripts"),
    table!("party_refs", "party_refs", "party_ref"),
    table!("map_tiles", "map_tiles", "map_tile"),
    table!("map_objects", "map_objects", "map_object"),
    table!("map_sprites", "map_sprites", "map_sprite"),
    table!("map_sprite_frames", "map_sprite_frames", "map_sprite_frame"),
    table!("map_metadata", "map_metadata", "map_metadata"),
    table!("map_overlay_modes", "map_overlay_modes", "map_overlay_mode"),
    table!(
        "map_sprite_sequences",
        "map_sprite_sequences",
        "map_sprite_sequence"
    ),
    table!("map_object_refs", "map_object_refs", "map_object_ref"),
    table!("map_bundles", "map_bundles", "map_bundle"),
    table!(
        "map_bundle_records",
        "map_bundle_records",
        "map_bundle_record"
    ),
    table!("map_bundle_items", "map_bundle_items", "map_bundle_item"),
    table!(
        "map_bundle_entries",
        "map_bundle_entries",
        "map_bundle_entry"
    ),
    table!(
        "map_bundle_entry_ids",
        "map_bundle_entry_ids",
        "map_bundle_entry_id"
    ),
    table!("map_bundle_flags", "map_bundle_flags", "map_bundle_flag"),
    table!(
        "dialogue_paragraphs",
        "dialogue_paragraphs",
        "dialogue_paragraphs"
    ),
    table!("party_levels", "party_levels", "party_level"),
    table!("party_inis", "party_inis", "party_ini"),
    table!("magic_spells", "magic_spells", "magic_spell"),
    table!("quests", "quests", "quest"),
    table!("event_scripts", "event_scripts", "event_script"),
    table!("event_variables", "event_variables", "event_variable"),
    table!("event_sprites", "event_sprites", "event_sprite"),
    table!("sprite_files", "sprite_files", "sprite_file"),
    table!("sprite_frames", "sprite_frames", "sprite_frame"),
    table!("sprite_sequences", "sprite_sequences", "sprite_sequence"),
    table!("event_actions", "event_actions", "event_action"),
    table!("ch_datas", "chdata", "chdata"),
    table!("chest_spawns", "chest_spawns", "chest_spawns"),
    table!("monster_spawns", "monster_spawns", "monster_spawns"),
    table!("map_info_multi", "map_info_multi", "map_info_multi"),
    table!("tile_grids", "tile_grids", "tile_grids"),
];
