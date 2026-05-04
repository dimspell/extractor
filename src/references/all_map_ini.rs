use std::path::Path;

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use crate::references::enums::MapLighting;
use crate::references::extractor::Extractor;
use dispel_macros::TextExtractor;

/// Stores the general list of all maps in the game.
///
/// Reads file: `AllMap.ini`
///
/// # Contents
///
/// Each line follows the CSV format:
/// ```text
/// id,map_filename,map_name,pgp_filename,dlg_filename,lighting
/// ```
///
/// # Field Definitions
///
/// - `id`: Unique map identifier (1, 2, 3, ...)
/// - `map_filename`: .map filename (e.g., "cat1")
/// - `map_name`: Display name shown in game
/// - `pgp_filename`: Conversation script filename or "null"
/// - `dlg_filename`: Dialog text filename or "null"
/// - `lighting`: 0 = Light, 1 = Dark
///
/// # Special Values
///
/// - `"null"` literal for missing PGP/DLG files
/// - Lines starting with `;` are comments
/// - CSV format with comma delimiter
///
/// # File Purpose
///
/// Master index of all game maps, linking map IDs to filenames and metadata.
/// Used by the game engine to load the correct map files and associated assets.
#[derive(Debug, Clone, Serialize, Deserialize, Default, TextExtractor)]
#[extractor(encoding = "WINDOWS_1250")]
pub struct Map {
    /// Map identifier.
    #[extractor(field = 0)]
    pub id: i32,
    /// Filename of the .map file.
    #[extractor(field = 1)]
    pub map_filename: String,
    /// Display name of the map.
    #[extractor(field = 2)]
    pub map_name: String,
    /// Filename of the associated converstation script file.
    #[extractor(field = 3, parse_null)]
    pub pgp_filename: Option<String>,
    /// Filename of the associated dialog file.
    #[extractor(field = 4, parse_null)]
    pub dlg_filename: Option<String>,
    /// Light indicator (0 = Light, 1 = Dark).
    #[extractor(field = 5, enum_from_i32(type = "MapLighting"))]
    pub lighting: MapLighting,
}

pub fn read_all_map_ini(source_path: &Path) -> std::io::Result<Vec<Map>> {
    Map::read_file(source_path)
}

pub fn save_maps(conn: &mut Connection, maps: &[Map]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_map.sql"))?;
        for map in maps {
            stmt.execute(params![
                map.id,
                map.map_filename,
                map.map_name,
                map.pgp_filename,
                map.dlg_filename,
                i32::from(map.lighting),
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::references::enums::MapLighting;
    use std::io::Cursor;

    #[test]
    fn parse_two_entries() {
        let data = b"; comment\n1,cat1,Forest,null,null,0\r\n2,cat2,Dungeon,pgp2,dlg2,1\r\n";
        let mut cursor = Cursor::new(data.as_ref());
        let maps = Map::parse(&mut cursor, data.len() as u64).unwrap();

        assert_eq!(maps.len(), 2);

        assert_eq!(maps[0].id, 1);
        assert_eq!(maps[0].map_filename, "cat1");
        assert_eq!(maps[0].map_name, "Forest");
        assert!(maps[0].pgp_filename.is_none());
        assert!(maps[0].dlg_filename.is_none());
        assert_eq!(maps[0].lighting, MapLighting::Light);

        assert_eq!(maps[1].id, 2);
        assert_eq!(maps[1].pgp_filename.as_deref(), Some("pgp2"));
        assert_eq!(maps[1].dlg_filename.as_deref(), Some("dlg2"));
        assert_eq!(maps[1].lighting, MapLighting::Dark);
    }

    #[test]
    fn parse_skips_comments_and_empty_lines() {
        let data = b"; header\n; another comment\n\n1,m1,Map One,null,null,0\n";
        let mut cursor = Cursor::new(data.as_ref());
        let maps = Map::parse(&mut cursor, data.len() as u64).unwrap();
        assert_eq!(maps.len(), 1);
        assert_eq!(maps[0].map_name, "Map One");
    }

    #[test]
    fn parse_empty() {
        let mut cursor = Cursor::new(b"" as &[u8]);
        let maps = Map::parse(&mut cursor, 0).unwrap();
        assert!(maps.is_empty());
    }

    #[test]
    fn serialize_round_trip() {
        let data = b"1,cat1,Forest,null,null,0\r\n2,cat2,Dungeon,pgp2,dlg2,1\r\n";
        let mut c = Cursor::new(data.as_ref());
        let records = Map::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        Map::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = Map::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].id, records2[0].id);
        assert_eq!(records[0].map_name, records2[0].map_name);
        assert_eq!(records[1].pgp_filename, records2[1].pgp_filename);
    }
}
