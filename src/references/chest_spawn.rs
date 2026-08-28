use std::path::Path;

use crate::references::extractor::Extractor;
use dispel_macros::{TextExtractor, TextRecordPatcher};
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

/// `Map/Multi/*.cst` — chest/placeable spawn list (multiplayer).
///
/// Plain-text CSV, EUC-KR encoded. Lines starting with `;` are skipped as
/// comments (the first row is a Korean column header). CRLF line endings.
///
/// Record layout (6 columns):
/// - `serial`: serial number
/// - `x`: tile X coordinate
/// - `y`: tile Y coordinate
/// - `direction`: facing direction
/// - `item_group`: item drop group number
/// - `gold_class`: gold drop class number (0=none, 1=1-20, 2=21-50, ... 6=501-1000)
///
#[derive(Debug, Clone, Default, Serialize, Deserialize, TextExtractor, TextRecordPatcher)]
#[extractor(encoding = "EUC_KR", delimiter = ",", comment_char = ";")]
#[patcher(filename = "*.cst")]
pub struct ChestSpawn {
    /// Serial number.
    #[extractor(field = 0)]
    pub serial: i32,
    /// Tile X coordinate.
    #[extractor(field = 1)]
    pub x: i32,
    /// Tile Y coordinate.
    #[extractor(field = 2)]
    pub y: i32,
    /// Facing direction (0-7 compass orientation, same scale as `NpcLookingDirection`).
    #[extractor(field = 3)]
    pub direction: i32,
    /// Item drop group index.
    #[extractor(field = 4)]
    pub item_group: i32,
    /// Gold class (0=none, 1..6=ascending ranges).
    ///
    /// Verified ranges: 0: none · 1: 1-20 · 2: 21-50 · 3: 51-100 ·
    /// 4: 101-200 · 5: 201-500 · 6: 501-1000.
    #[extractor(field = 5)]
    pub gold_class: i32,
}

pub fn read_chest_spawn(source_path: &Path) -> std::io::Result<Vec<ChestSpawn>> {
    ChestSpawn::read_file(source_path)
}

pub fn save_chest_spawns(conn: &mut Connection, spawns: &[ChestSpawn]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT OR IGNORE INTO chest_spawns (serial, x, y, direction, item_group, gold_class) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;
        for spawn in spawns {
            stmt.execute(params![
                spawn.serial,
                spawn.x,
                spawn.y,
                spawn.direction,
                spawn.item_group,
                spawn.gold_class,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

impl std::fmt::Display for ChestSpawn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ChestSpawn(serial={}, x={}, y={}, dir={}, group={}, gold={})",
            self.serial, self.x, self.y, self.direction, self.item_group, self.gold_class
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_cst_sample() {
        let data = b"; \xb1\xd5\xb1\xc0,\xc2\xc0\xc6\xc5 X ,\xc2\xc0\xc6\xc5 Y,\xb9\xe6\xc7\xc5(\xc1\xf6\xc0\xfc\xc0\xcc 1),\xc0\xc7\xc0\xcd\xb8\xc7 \xb1\xe6\xc7\xc5 \xb9\xf8\xc8\xa3,\xb1\xfb\xc0\xc7 \xba\xd0\xb7\xc7 \xb9\xf8\xc8\xa3\r\n0,211,84,0,1,0\r\n1,207,109,0,1,0\r\n";
        let mut c = Cursor::new(data.as_ref());
        let spawns = ChestSpawn::parse(&mut c, data.len() as u64).unwrap();
        assert!(!spawns.is_empty());
        assert_eq!(spawns[0].serial, 0);
        assert_eq!(spawns[0].x, 211);
        assert_eq!(spawns[0].y, 84);
        assert_eq!(spawns[0].direction, 0);
        assert_eq!(spawns[0].item_group, 1);
        assert_eq!(spawns[0].gold_class, 0);
    }

    #[test]
    fn serialize_round_trip() {
        let data = b"0,211,84,0,1,0\r\n";
        let mut c = Cursor::new(data.as_ref());
        let records = ChestSpawn::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        ChestSpawn::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = ChestSpawn::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].serial, records2[0].serial);
    }
}
