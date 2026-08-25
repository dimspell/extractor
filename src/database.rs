// use std::io::Write;

use rusqlite::{Connection, Result};

pub fn initialize_database(conn: &Connection) -> Result<()> {
    // Optimization and safety PRAGMAs
    conn.execute_batch(
        "PRAGMA foreign_keys = OFF;
         PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -64000;
         PRAGMA temp_store = MEMORY;
         PRAGMA mmap_size = 30000000000;",
    )?;

    let tables = vec![
        "ch_datas",
        "dialogue_paragraphs",
        "dialogue_script_files",
        "dialogue_scripts",
        "draw_items",
        "edit_items",
        "event_actions",
        "event_items",
        "event_npc_refs",
        "event_scripts",
        "event_sprites",
        "event_variables",
        "events",
        "extra_refs",
        "extra_ref_files",
        "extras",
        "fog_factors",
        "heal_items",
        "magic_spells",
        "map_inis",
        "map_metadata",
        "map_object_metadata",
        "map_object_refs",
        "map_objects",
        "map_overlay_modes",
        "map_sprite_frames",
        "map_sprite_sequences",
        "map_sprites",
        "map_tiles",
        "maps",
        "messages",
        "misc_items",
        "monster_inis",
        "monster_ref_files",
        "monster_refs",
        "monsters",
        "npc_inis",
        "npc_ref_files",
        "npc_refs",
        "party_inis",
        "party_levels",
        "party_pgps",
        "party_refs",
        "quests",
        "sprite_frames",
        "sprite_sequences",
        "sprite_files",
        "store_products",
        "stores",
        "wave_inis",
        "weapons",
    ];

    for table in tables {
        conn.execute(&format!("DROP TABLE IF EXISTS {}", table), [])?;
    }

    conn.execute_batch(include_str!("queries/create_table_monster_ref_files.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_monster_refs.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_messages.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_events.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_extras.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_fog_factors.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_extra_ref_files.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_extra_refs.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_weapons.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_edit_items.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_event_items.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_event_npc_refs.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_misc_items.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_heal_items.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_stores.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_store_products.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_monsters.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_maps.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_monster_inis.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_npc_inis.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_npc_ref_files.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_npc_refs.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_wave_inis.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_inis.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_draw_items.sql"))?;
    conn.execute_batch(include_str!(
        "queries/create_table_dialogue_script_files.sql"
    ))?;
    conn.execute_batch(include_str!("queries/create_table_dialogue_scripts.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_party_refs.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_tiles.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_objects.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_sprites.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_sprite_frames.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_metadata.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_overlay_modes.sql"))?;
    conn.execute_batch(include_str!(
        "queries/create_table_map_sprite_sequences.sql"
    ))?;
    conn.execute_batch(include_str!("queries/create_table_map_object_refs.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_object_metadata.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_dialogue_paragraphs.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_party_levels.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_party_inis.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_magic_spells.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_quests.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_event_scripts.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_event_variables.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_event_sprites.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_sprite_files.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_sprite_frames.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_sprite_sequences.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_event_actions.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_chdata.sql"))?;

    // Re-enable foreign key enforcement now that the schema is rebuilt.
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    Ok(())
}
