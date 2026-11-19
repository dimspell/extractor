//! SQLite schema bootstrap, INSERT statements, and the full import pipeline.
//!
//! This module is the single owner of the database layer. The schema shape
//! lives in the [`manifest`] table list; `DROP` and `CREATE` order are both
//! derived from it, and every INSERT statement is registered there too. Callers
//! (parsers and the CLI) obtain their statements through [`insert_sql`] instead
//! of embedding `include_str!` paths.
//!
//! # Conflict policy
//!
//! Every registered INSERT uses `INSERT OR REPLACE`. For keyed tables this is
//! an idempotent last-write-wins upsert: re-importing converges on one row per
//! key instead of failing or silently keeping stale values. Tables that
//! declare no primary key / `UNIQUE` constraint (`weapons`, `party_inis`,
//! `party_levels`, `chest_spawns`, `monster_spawns`) have no conflict target,
//! so `REPLACE` behaves exactly like a plain `INSERT` for them.
//!
//! The one deliberate exception is `sprite_files`, which uses `INSERT OR
//! IGNORE`: its `AUTOINCREMENT` id is captured by the caller and referenced by
//! `sprite_frames`, so a repeat import must reuse the existing row rather than
//! delete-and-reinsert it with a new id.
//!
//! The schema currently declares no `FOREIGN KEY` constraints; relationships
//! are enforced by import order in [`import`] and in code. The defensive
//! `PRAGMA foreign_keys = ON` is kept so that adding constraints later does
//! not require touching the bootstrap path.

mod import;
mod manifest;

pub use import::{
    import_all, import_databases, import_dialogues_paragraphs, import_event_scripts, import_maps,
    import_refs, import_rest, import_sprite_files,
};

use manifest::TABLES;
use rusqlite::{Connection, Result};

/// Names of every table in the schema, in creation order.
pub fn table_names() -> impl Iterator<Item = &'static str> {
    TABLES.iter().map(|spec| spec.name)
}

/// Returns the canonical INSERT statement for `table`, or `None` if the table
/// is not part of the manifest.
pub fn try_insert_sql(table: &str) -> Option<&'static str> {
    TABLES
        .iter()
        .find(|spec| spec.name == table)
        .map(|spec| spec.insert_sql)
}

/// Returns the canonical INSERT statement for `table`.
///
/// # Panics
///
/// Panics if `table` is not registered in the schema manifest. This is a
/// programming error (a caller asking for a table that does not exist), not a
/// runtime I/O condition.
pub fn insert_sql(table: &str) -> &'static str {
    try_insert_sql(table)
        .unwrap_or_else(|| panic!("no INSERT statement registered for table `{table}`"))
}

/// Creates every table in the manifest (idempotent; never drops data).
pub fn create_all(conn: &Connection) -> Result<()> {
    for spec in TABLES {
        conn.execute_batch(spec.create_sql)?;
    }
    Ok(())
}

/// Drops every table in the manifest, in reverse creation order.
///
/// The drop set is derived from the same manifest as [`create_all`], so a
/// table can never be created without also being dropped.
pub fn drop_all(conn: &Connection) -> Result<()> {
    for spec in TABLES.iter().rev() {
        conn.execute(&format!("DROP TABLE IF EXISTS \"{}\"", spec.name), [])?;
    }
    Ok(())
}

/// Rebuilds the schema from scratch: applies performance PRAGMAs, drops all
/// known tables and recreates them from the manifest.
pub fn bootstrap(conn: &Connection) -> Result<()> {
    // Optimization and safety PRAGMAs. `foreign_keys` is temporarily disabled
    // so the drop/recreate cycle cannot trip over a future FK constraint.
    conn.execute_batch(
        "PRAGMA foreign_keys = OFF;
         PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -64000;
         PRAGMA temp_store = MEMORY;
         PRAGMA mmap_size = 30000000000;",
    )?;

    drop_all(conn)?;
    create_all(conn)?;

    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(())
}

/// Backwards-compatible alias for [`bootstrap`].
pub fn initialize_database(conn: &Connection) -> Result<()> {
    bootstrap(conn)
}

#[cfg(test)]
mod tests;
