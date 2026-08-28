use std::path::Path;

use crate::references::enums::BooleanFlag;
use crate::references::extractor::Extractor;
use dispel_macros::{TextExtractor, TextRecordPatcher};
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

/// `Map/Multi/*.mon` — monster spawn list (multiplayer).
///
/// Plain-text CSV, EUC-KR encoded. Lines starting with `;` are skipped.
/// LF-only row endings on disk (the text extractor handles both LF and CRLF).
///
/// Record layout (8 columns):
/// - `serial`: serial number
/// - `monster_no`: species number
/// - `x`: tile X coordinate
/// - `y`: tile Y coordinate
/// - `persistent`: persistent flag (`1` = persistent)
/// - `burrow`: burrow flag (`1` = burrow)
/// - `item_group`: drop group index (same cascade as `.cst` drop mechanics)
/// - `drop_attribute`: drop attribute feeding the cascade count
///
#[derive(Debug, Clone, Default, Serialize, Deserialize, TextExtractor, TextRecordPatcher)]
#[extractor(encoding = "EUC_KR", delimiter = ",", comment_char = ";")]
#[patcher(filename = "*.mon")]
pub struct MonsterSpawn {
    /// Serial number.
    #[extractor(field = 0)]
    pub serial: i32,
    /// Monster species number (1-based index into `.spr` name table after `v - 1`).
    #[extractor(field = 1)]
    pub monster_no: i32,
    /// Tile X coordinate.
    #[extractor(field = 2)]
    pub x: i32,
    /// Tile Y coordinate.
    #[extractor(field = 3)]
    pub y: i32,
    /// Persistent spawn (`1` = persistent).
    #[extractor(field = 4, enum_from_i32(type = "BooleanFlag"))]
    pub persistent: BooleanFlag,
    /// Burrow behavior (`1` = burrow).
    #[extractor(field = 5, enum_from_i32(type = "BooleanFlag"))]
    pub burrow: BooleanFlag,
    /// Drop group index (same tiered cascade rules as `.cst` item groups).
    #[extractor(field = 6)]
    pub item_group: i32,
    /// Drop attribute feeding the cascade count (`N` = number of guaranteed rolls).
    #[extractor(field = 7)]
    pub drop_attribute: i32,
}

pub fn read_monster_spawn(source_path: &Path) -> std::io::Result<Vec<MonsterSpawn>> {
    MonsterSpawn::read_file(source_path)
}

pub fn save_monster_spawns(conn: &mut Connection, spawns: &[MonsterSpawn]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT OR IGNORE INTO monster_spawns (serial, monster_no, x, y, persistent, burrow, item_group, drop_attribute) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        for spawn in spawns {
            stmt.execute(params![
                spawn.serial,
                spawn.monster_no,
                spawn.x,
                spawn.y,
                i32::from(spawn.persistent),
                i32::from(spawn.burrow),
                spawn.item_group,
                spawn.drop_attribute,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

impl std::fmt::Display for MonsterSpawn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MonsterSpawn(serial={}, species={}, x={}, y={}, persistent={}, burrow={}, group={}, attr={})",
            self.serial,
            self.monster_no,
            self.x,
            self.y,
            i32::from(self.persistent),
            i32::from(self.burrow),
            self.item_group,
            self.drop_attribute
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_mon_sample() {
        let data = b"; \xb1\xd5\xb1\xc0,\xb8\xe9\xbd\xba\xc5\xf6 \xb9\xf8\xc8\xa3,\xc2\xc0\xc6\xc5 X,\xc2\xc0\xc6\xc5 Y,\xc5\xb8\xc0\xd4(\xc1\xf6\xc0\xfc\xc0\xcc 1),\xba\xb4\xc0\xce\xc2\xf0(1\xc0\xcc \xba\xb4\xc0\xce\xc2\xf2),\xc0\xc7\xc0\xdd \xb1\xe6\xc7\xc5,\xb6\xb6\xbe\xc5\xc2\xb2 \xbc\xd3\xbc\xba\xc0\xc7 \xc0\xc7\xc6\xa8\xc1\xc0 \xc0\xcf\xb9\xc7 \xc5\xcb\xc0\xd4\xc3\xc7 \xc0\xcf\xc7\xc1 \xc5\xcd\xc0\xcc\xc5\xd7\xc0\xdf\xc0\xd4\xc0\xd4\xc0\xdf\xb1\xdb\xc0\xce \xb1\xc2 \xc7\xc3\xc0\xcc\xeb\xbc\xc5\xc0\xda\xc1\xa1\xc1\xc0\xc6\xa8\xb8\xef\xcc\xc0\xcc\xc0\xce \xc5\xa1\xbc\xbc\xbc\xba\xb8\xef\xc4\xd8.\r\n0,5,209,93,1,0,0,0\r\n1,5,209,93,1,1,0,0\r\n";
        let mut c = Cursor::new(data.as_ref());
        let spawns = MonsterSpawn::parse(&mut c, data.len() as u64).unwrap();
        assert!(!spawns.is_empty());
        assert_eq!(spawns[0].serial, 0);
        assert_eq!(spawns[0].monster_no, 5);
        assert_eq!(spawns[0].x, 209);
        assert_eq!(spawns[0].y, 93);
        assert_eq!(spawns[0].persistent, BooleanFlag::True);
        assert_eq!(spawns[0].burrow, BooleanFlag::False);
    }
}
