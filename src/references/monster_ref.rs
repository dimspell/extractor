use std::path::Path;

use crate::references::enums::{BooleanFlag, InventoryItem, ItemTypeId, TriStateFlag};
use crate::references::extractor::Extractor;
use dispel_macros::{Extractor, RecordLayout, RecordPatcher};
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

/// MonsterRef.ref - Monster Placements on Maps
///
/// Stores specific placements and configurations for monsters on a given map.
///
/// Reads file: `MonsterInGame/Mondun01.ref` (and other map-specific `.ref` files)
///
/// # Binary Format
///
/// - **Encoding**: Little-endian for all numeric values
/// - **Record Size**: 56 bytes (14 × i32)
/// - **Header**: 4-byte i32 record count, followed by records
///
/// ```text
/// +--------------------------------------+
/// | MonsterRef.ref - Monster Placements  |
/// +--------------------------------------+
/// | Encoding: Binary (Little-Endian)     |
/// | Record Size: 56 bytes               |
/// | Header: 4-byte record count          |
/// +--------------------------------------+
/// | [Header]                             |
/// | - record_count: i32                  |
/// +--------------------------------------+
/// | [Record 1] - 56 bytes               |
/// | - placement_id: i32                  |
/// | - monster_db_id: i32 (-> Monster.db) |
/// | - map_x, map_y: i32                  |
/// | - initial_patrol_countdown: i32      |
/// | - skip_ai_action: i32                 |
/// | - initial_active_flag: i32            |
/// | - ai_type_override: i32               |
/// | - event_id_on_kill: i32               |
/// | - loot_item_1..3: i32                 |
/// | - drop_all_loot: i32                  |
/// | - force_ai_update: i32                |
/// +--------------------------------------+
/// | [Record 2]                           |
/// | ... (same structure) ...             |
/// +--------------------------------------+
/// ```
///
/// # Field Categories
///
/// - **Identification**: `placement_id`, `monster_db_id` (links to `Monster.db`)
/// - **Position**: `map_x`, `map_y` (tile coordinates)
/// - **Event Link**: `event_id_on_kill` (links to `Event.ini`)
/// - **Loot Drops**: 3 packed `InventoryItem` values.
///
/// # Special Values
///
/// - `ai_type_override`: `-1` uses the AI type from `Monster.db`; 0 or 1 overrides it.
/// - `drop_all_loot`: `1` drops every populated loot slot; other observed values select one.
/// - `force_ai_update`: `1` runs the AI update path even when `initial_active_flag` is clear.
///
/// # File Purpose
///
/// Defines monster placements on specific maps with position,
/// event triggers, and loot configurations. Used by game engine
/// for monster spawning and encounter design.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Extractor, RecordPatcher, RecordLayout)]
#[extractor(property_item_size = 56)]
#[patcher(extension = "ref", stem_prefix = "mon")]
pub struct MonsterRef {
    /// Record index relative to the file (0-based).
    #[extractor(index)]
    pub index: i32,
    /// Map-local monster placement identifier, distinct from the record index.
    #[extractor(primitive(type = "i32"))]
    pub placement_id: i32,
    /// One-based ID of the monster type from `Monster.db` and `Monster.ini`.
    #[extractor(primitive(type = "i32"))]
    pub monster_db_id: i32,
    /// Spawn tile X coordinate.
    #[extractor(primitive(type = "i32"))]
    pub map_x: i32,
    /// Spawn tile Y coordinate.
    #[extractor(primitive(type = "i32"))]
    pub map_y: i32,
    /// Initial countdown used by the patrol/scan behavior.
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub initial_patrol_countdown: BooleanFlag,
    /// Skips one branch of the monster AI action logic when set.
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub skip_ai_action: BooleanFlag,
    /// Initial monster active flag. The original maps observed so far use zero.
    #[extractor(primitive(type = "i32"))]
    pub initial_active_flag: i32,
    /// Overrides the AI type from `Monster.db`; `-1` leaves it unchanged.
    #[extractor(enum_from_i32(type = "TriStateFlag"))]
    pub ai_type_override: TriStateFlag,
    /// Event trigger ID run after this monster is killed.
    #[extractor(primitive(type = "i32"))]
    pub event_id_on_kill: i32,
    /// First loot drop (encoded as i32: low 16 bits = item, high 16 bits = padding).
    #[extractor(inventory_item(wire_type = "i32"))]
    pub loot_item_1: InventoryItem,
    /// Second loot drop (encoded as i32: low 16 bits = item, high 16 bits = padding).
    #[extractor(inventory_item(wire_type = "i32"))]
    pub loot_item_2: InventoryItem,
    /// Third loot drop (encoded as i32: low 16 bits = item, high 16 bits = padding).
    #[extractor(inventory_item(wire_type = "i32"))]
    pub loot_item_3: InventoryItem,
    /// When one, drop every populated loot slot rather than selecting one slot.
    #[extractor(enum_from_i32(type = "TriStateFlag"))]
    pub drop_all_loot: TriStateFlag,
    /// Forces the AI update path, including when the normal active flag is clear.
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub force_ai_update: BooleanFlag,
}

pub fn read_monster_ref(source_path: &Path) -> std::io::Result<Vec<MonsterRef>> {
    MonsterRef::read_file(source_path)
}

pub fn save_monster_refs(
    conn: &mut Connection,
    file_id: i32,
    monster_refs: &[MonsterRef],
) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_monster_ref.sql"))?;
        for monster_ref in monster_refs {
            stmt.execute(params![
                file_id,
                monster_ref.index,
                monster_ref.placement_id,
                if monster_ref.monster_db_id == 0 {
                    None
                } else {
                    Some(monster_ref.monster_db_id)
                },
                monster_ref.map_x,
                monster_ref.map_y,
                i32::from(monster_ref.initial_patrol_countdown),
                i32::from(monster_ref.skip_ai_action),
                monster_ref.initial_active_flag,
                i32::from(monster_ref.ai_type_override),
                monster_ref.event_id_on_kill,
                monster_ref.loot_item_1.item_id() as i32,
                u8::from(
                    monster_ref
                        .loot_item_1
                        .item_type()
                        .unwrap_or(ItemTypeId::Other)
                ) as i32,
                monster_ref.loot_item_1.raw(),
                monster_ref.loot_item_2.item_id() as i32,
                u8::from(
                    monster_ref
                        .loot_item_2
                        .item_type()
                        .unwrap_or(ItemTypeId::Other)
                ) as i32,
                monster_ref.loot_item_2.raw(),
                monster_ref.loot_item_3.item_id() as i32,
                u8::from(
                    monster_ref
                        .loot_item_3
                        .item_type()
                        .unwrap_or(ItemTypeId::Other)
                ) as i32,
                monster_ref.loot_item_3.raw(),
                i32::from(monster_ref.drop_all_loot),
                i32::from(monster_ref.force_ai_update),
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn ref_bytes(placement_id: i32, monster_db_id: i32, map_x: i32, map_y: i32) -> Vec<u8> {
        // 14 × i32 = 56 bytes; remaining 10 fields are zero
        let mut buf: Vec<u8> = Vec::with_capacity(56);
        for &v in &[
            placement_id,
            monster_db_id,
            map_x,
            map_y,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ] {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        buf
    }

    #[test]
    fn parse_one_ref() {
        let rec = ref_bytes(1, 5, 10, 20);
        let mut data = 1i32.to_le_bytes().to_vec(); // header
        data.extend(&rec);
        assert_eq!(data.len(), 60);

        let mut c = Cursor::new(&data[..]);
        let refs = MonsterRef::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].placement_id, 1);
        assert_eq!(refs[0].monster_db_id, 5);
        assert_eq!(refs[0].map_x, 10);
        assert_eq!(refs[0].map_y, 20);
    }

    #[test]
    fn parse_two_refs() {
        let mut data = 2i32.to_le_bytes().to_vec();
        data.extend(ref_bytes(1, 2, 3, 4));
        data.extend(ref_bytes(5, 6, 7, 8));

        let mut c = Cursor::new(&data[..]);
        let refs = MonsterRef::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[1].monster_db_id, 6);
    }

    #[test]
    fn serialize_round_trip() {
        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(ref_bytes(1, 2, 3, 4));
        let mut c = Cursor::new(&data[..]);
        let records = MonsterRef::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        MonsterRef::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = MonsterRef::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].placement_id, records2[0].placement_id);
        assert_eq!(records[0].monster_db_id, records2[0].monster_db_id);
        assert_eq!(records[0].map_x, records2[0].map_x);
        assert_eq!(records[0].map_y, records2[0].map_y);
    }
}
