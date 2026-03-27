// use std::io::Write;

use rusqlite::{Connection, Result};

pub fn initialize_database(conn: &Connection) -> Result<()> {
    // Optimization PRAGMAs
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -64000;
         PRAGMA temp_store = MEMORY;
         PRAGMA mmap_size = 30000000000;",
    )?;

    let tables = vec![
        "dialogue_texts",
        "dialogs",
        "draw_items",
        "edit_items",
        "event_items",
        "event_npc_refs",
        "events",
        "extra_refs",
        "extras",
        "heal_items",
        "magic_spells",
        "map_inis",
        "map_metadata",
        "map_objects",
        "map_sprites",
        "map_tiles",
        "maps",
        "messages",
        "misc_items",
        "monster_inis",
        "monster_refs",
        "monsters",
        "npc_inis",
        "npc_refs",
        "party_inis",
        "party_levels",
        "party_pgps",
        "party_refs",
        "quests",
        "store_products",
        "stores",
        "wave_inis",
        "weapons",
    ];

    for table in tables {
        conn.execute(&format!("DROP TABLE IF EXISTS {}", table), [])?;
    }

    conn.execute_batch(include_str!("queries/create_table_npc_refs.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_monster_refs.sql"))?;
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
    conn.execute_batch(include_str!("queries/create_table_events.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_extras.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_monster_inis.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_npc_inis.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_wave_inis.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_inis.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_party_refs.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_draw_items.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_party_pgps.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_dialogs.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_tiles.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_objects.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_sprites.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_map_metadata.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_dialogue_texts.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_party_levels.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_party_inis.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_magic_spells.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_quests.sql"))?;
    conn.execute_batch(include_str!("queries/create_table_messages.sql"))?;

    Ok(())
}
