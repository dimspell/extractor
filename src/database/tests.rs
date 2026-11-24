//! Boundary tests for the SQLite layer, run against an in-memory database.
//!
//! These do not depend on any game fixtures: they assert the contract of the
//! schema manifest (create/drop symmetry, no stale tables) and one real
//! parse → write → read round trip through the registered INSERT statement.

use super::{
    bootstrap, create_all, drop_all, import_sprite_files, insert_sql, table_names, try_insert_sql,
};
use crate::references::extractor::Extractor;
use crate::references::weapons_db::{WeaponItem, save_weapons};
use rusqlite::Connection;
use std::io::Cursor;

/// Builds one 284-byte `WeaponItem` record preceded by the file's count word.
fn weapon_bytes(name: &str, base_price: i32, weapon_item_id: i32, attack: i16) -> Vec<u8> {
    let mut rec = Vec::with_capacity(284);
    let mut name_buf = [0u8; 30];
    name_buf[..name.len().min(29)].copy_from_slice(&name.as_bytes()[..name.len().min(29)]);
    rec.extend_from_slice(&name_buf);
    rec.extend(vec![0u8; 202]); // description
    rec.extend_from_slice(&base_price.to_le_bytes());
    rec.extend_from_slice(&weapon_item_id.to_le_bytes());
    rec.extend(vec![0u8; 8]); // hp, mp, str, agi
    rec.extend(vec![0u8; 8]); // wis, con, dodge, to_hit
    rec.extend_from_slice(&attack.to_le_bytes());
    rec.extend(vec![0u8; 26]); // defense, magical_str, durability, pads, reqs
    assert_eq!(rec.len(), 284);

    let mut data = 1i32.to_le_bytes().to_vec();
    data.extend(rec);
    data
}

fn user_tables(conn: &Connection) -> Vec<String> {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'")
        .expect("prepare sqlite_master query");
    let mut names: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .expect("query sqlite_master")
        .map(|row| row.expect("read table name"))
        .collect();
    names.sort();
    names
}

#[test]
fn test_bootstrap_creates_every_manifest_table() {
    let conn = Connection::open_in_memory().expect("in-memory database");
    bootstrap(&conn).expect("bootstrap");

    let expected: Vec<String> = table_names().map(str::to_owned).collect();
    let mut expected_sorted = expected.clone();
    expected_sorted.sort();
    assert_eq!(
        user_tables(&conn),
        expected_sorted,
        "sqlite_master must contain exactly the manifest tables"
    );

    // Regression guard for the old hand-written drop list: `party_pgps` was
    // dropped but never created by any SQL file.
    assert!(
        !expected.iter().any(|name| name == "party_pgps"),
        "party_pgps must not be part of the schema"
    );
    assert!(
        try_insert_sql("party_pgps").is_none(),
        "no INSERT statement should be registered for a non-existent table"
    );
}

#[test]
fn test_drop_all_removes_every_created_table() {
    let conn = Connection::open_in_memory().expect("in-memory database");
    bootstrap(&conn).expect("bootstrap");
    assert_eq!(user_tables(&conn).len(), table_names().count());

    // Seed a sentinel row; a subsequent bootstrap must clear it, proving that
    // the derived DROP list covers every created table.
    conn.execute(
        insert_sql("messages"),
        rusqlite::params![1i64, "sentinel", "x", "y"],
    )
    .expect("seed sentinel row");
    let before: i64 = conn
        .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
        .expect("count messages");
    assert_eq!(before, 1);

    drop_all(&conn).expect("drop_all");
    assert!(
        user_tables(&conn).is_empty(),
        "no user tables should remain"
    );

    bootstrap(&conn).expect("re-bootstrap");
    let after: i64 = conn
        .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
        .expect("count messages after re-bootstrap");
    assert_eq!(after, 0, "re-bootstrap must discard all previous rows");

    // `create_all` alone is idempotent and never drops data.
    conn.execute(
        insert_sql("messages"),
        rusqlite::params![2i64, "kept", "x", "y"],
    )
    .expect("seed second row");
    create_all(&conn).expect("create_all");
    let still_there: i64 = conn
        .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
        .expect("count messages after create_all");
    assert_eq!(still_there, 1, "create_all must preserve existing rows");
}

#[test]
fn test_import_round_trip_weapon_into_memory() {
    let mut conn = Connection::open_in_memory().expect("in-memory database");
    bootstrap(&conn).expect("bootstrap");

    let data = weapon_bytes("Sword", 300, 17, 25);
    let weapons = WeaponItem::parse(&mut Cursor::new(&data[..]), data.len() as u64)
        .expect("parse weapon bytes");
    assert_eq!(weapons.len(), 1);
    assert_eq!(weapons[0].name, "Sword");

    save_weapons(&mut conn, &weapons).expect("save weapons");

    let (name, base_price, attack): (String, i32, i16) = conn
        .query_row(
            "SELECT name, base_price, attack FROM weapons WHERE weapon_item_id = ?1",
            [17i32],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("read saved weapon");
    assert_eq!(name, "Sword");
    assert_eq!(base_price, 300);
    assert_eq!(attack, 25);

    // `weapons` declares no primary key, so INSERT OR REPLACE has no conflict
    // target and a re-import appends rather than replaces. Idempotency is
    // asserted separately on a table that does have a key.
}

#[test]
fn test_insert_sql_registry_covers_every_table() {
    for table in table_names() {
        let sql = insert_sql(table);
        let expected = if table == "sprite_files" {
            "INSERT OR IGNORE INTO"
        } else {
            "INSERT OR REPLACE INTO"
        };
        assert!(
            sql.to_ascii_uppercase().starts_with(expected),
            "INSERT for `{table}` must use `{expected}` (the documented conflict policy)"
        );
    }
}

#[test]
fn test_insert_replace_is_idempotent_for_keyed_table() {
    let conn = Connection::open_in_memory().expect("in-memory database");
    bootstrap(&conn).expect("bootstrap");

    for line in ["first", "second"] {
        conn.execute(
            insert_sql("messages"),
            rusqlite::params![1i64, line, "x", "y"],
        )
        .expect("upsert message");
    }
    let (line, count): (String, i64) = conn
        .query_row(
            "SELECT line1, (SELECT COUNT(*) FROM messages) FROM messages WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("read message");
    assert_eq!(line, "second", "last write must win");
    assert_eq!(count, 1, "upsert must not duplicate keyed rows");
}

/// A temp game directory holding one valid archive and one whose byte stream
/// ends before its sequence header. The corrupt file exercises parse failure,
/// which must not leave a `sprite_files` row behind.
#[test]
fn test_import_sprite_files_skips_corrupt_file_and_imports_valid_one() {
    use crate::sprite::{
        SpriteFile, SpriteFrameData, SpritePixelFormat, SpriteSequence, write_sprite_to_path,
    };

    let dir = std::env::temp_dir().join(format!(
        "dispel-sprite-import-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("create temp game dir");

    // Two RGB565 pixels per frame so a clean import yields exactly two rows.
    let frame = SpriteFrameData {
        unknown: [0u8; 24],
        origin_x: 0,
        origin_y: 0,
        width: 2,
        height: 2,
        pixel_format: SpritePixelFormat::Rgb565,
        raw_pixels: vec![0x00, 0xF8, 0x00, 0xF8, 0x00, 0xF8, 0x00, 0xF8],
    };
    let sprite = SpriteFile {
        sequences: vec![SpriteSequence {
            has_stamp: false,
            frames: vec![frame.clone(), frame],
        }],
    };
    write_sprite_to_path(&dir.join("valid.spr"), &sprite).expect("write valid sprite");
    // Sequence count of 1 but no sequence header: parsing must fail.
    std::fs::write(dir.join("corrupt.spr"), [1u8, 0, 0, 0]).expect("write corrupt sprite");

    let mut conn = Connection::open_in_memory().expect("in-memory database");
    bootstrap(&conn).expect("bootstrap");
    import_sprite_files(&dir, &mut conn).expect("import sprite files");

    let paths: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT normalized_path FROM sprite_files")
            .expect("prepare path query");
        stmt.query_map([], |row| row.get(0))
            .expect("query sprite_files")
            .map(|row| row.expect("read path"))
            .collect()
    };
    let frame_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM sprite_frames", [], |row| row.get(0))
        .expect("count frames");
    let seq_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM sprite_sequences", [], |row| {
            row.get(0)
        })
        .expect("count sequences");
    let valid_frames: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sprite_frames WHERE sprite_file_id = \
             (SELECT id FROM sprite_files WHERE normalized_path = 'valid.spr')",
            [],
            |row| row.get(0),
        )
        .expect("count frames of valid sprite");

    std::fs::remove_dir_all(&dir).expect("remove temp game dir");

    assert_eq!(
        paths,
        vec!["valid.spr".to_owned()],
        "corrupt file must have no row"
    );
    assert_eq!(frame_count, 2, "valid sprite must import both frames");
    assert_eq!(seq_count, 1);
    assert_eq!(valid_frames, 2);
}

/// A frame insert failure must roll back the file row written earlier in the
/// same per-file transaction, leaving all three sprite tables empty.
#[test]
fn test_import_sprite_files_rolls_back_on_insert_failure() {
    use crate::sprite::{
        SpriteFile, SpriteFrameData, SpritePixelFormat, SpriteSequence, write_sprite_to_path,
    };

    let dir = std::env::temp_dir().join(format!(
        "dispel-sprite-rollback-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("create temp game dir");

    let frame = SpriteFrameData {
        unknown: [0u8; 24],
        origin_x: 0,
        origin_y: 0,
        width: 1,
        height: 1,
        pixel_format: SpritePixelFormat::Rgb565,
        raw_pixels: vec![0x00, 0xF8],
    };
    let sprite = SpriteFile {
        sequences: vec![SpriteSequence {
            has_stamp: false,
            frames: vec![frame],
        }],
    };
    write_sprite_to_path(&dir.join("sprite.spr"), &sprite).expect("write sprite");

    let mut conn = Connection::open_in_memory().expect("in-memory database");
    bootstrap(&conn).expect("bootstrap");
    conn.execute_batch(
        "CREATE TRIGGER fail_frame BEFORE INSERT ON sprite_frames
         BEGIN SELECT RAISE(ABORT, 'forced frame failure'); END;",
    )
    .expect("create failing trigger");

    let result = import_sprite_files(&dir, &mut conn);

    std::fs::remove_dir_all(&dir).expect("remove temp game dir");

    assert!(result.is_err(), "frame insert failure must propagate");
    for (table, label) in [
        ("sprite_files", "file"),
        ("sprite_frames", "frame"),
        ("sprite_sequences", "sequence"),
    ] {
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .expect("count rows");
        assert_eq!(count, 0, "{label} rows must roll back on insert failure");
    }
}
