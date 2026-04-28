use std::path::Path;

use crate::references::extractor::Extractor;
use dispel_macros::TextExtractor;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// Map.ini - Map Initialization Data
///
/// Stores specific properties and configuration for a single map.
///
/// Reads file: `Ref/Map.ini`
///
/// # ASCII Structure
///
/// ```text
/// +--------------------------------------+
/// | Map.ini - Map Initialization Data    |
/// +--------------------------------------+
/// | Encoding: EUC-KR                    |
/// | Format: CSV with comments            |
/// | Record Size: Variable (text)        |
/// +--------------------------------------+
/// | ; Comment line                       |
/// | id,camera_event,start_x,start_y,map_id,monsters,npcs,extras,cd_track|
/// | 1,1001,5,10,1,mon1.ref,npc1.ref,ext1.ref,1|
/// | 2,1002,3,8,2,mon2.ref,npc2.ref,ext2.ref,2|
/// | ...                                   |
/// +--------------------------------------+
/// ```
///
/// # Field Definitions
///
/// - `id`: Unique map identifier
/// - `event_id_on_camera_move`: Event triggered on camera movement
/// - `start_pos_x`: Initial player X coordinate (isometric tile)
/// - `start_pos_y`: Initial player Y coordinate (isometric tile)
/// - `map_id`: Target map ID for linking
/// - `monsters_filename`: Monster placement REF file (or "null")
/// - `npc_filename`: NPC placement REF file (or "null")
/// - `extra_filename`: Extra object placement REF file (or "null")
/// - `cd_music_track_number`: Background music track number
///
/// # Field Categories
///
/// - **Identification**: `id` (map instance ID)
/// - **Initial Position**: `start_pos_x`, `start_pos_y` (isometric tile coordinates)
/// - **Event Link**: `event_id_on_camera_move` (triggered on camera movement)
/// - **Map Linking**: `map_id` (target map for transitions)
/// - **Placements**: `monsters_filename`, `npc_filename`, `extra_filename` (REF files)
/// - **Audio**: `cd_music_track_number` (background music)
///
/// # Special Values
///
/// - `"null"` literal for missing REF files (`monsters_filename`, `npc_filename`, `extra_filename`)
/// - Lines starting with `;` are comments
/// - CSV format with comma delimiter
/// - Coordinates use isometric tile system
///
/// # File Purpose
///
/// Defines map initialization parameters including
/// starting positions, linked files, and music tracks.
/// Used for map loading and setup.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TextExtractor)]
pub struct MapIni {
    /// Map ini identifier.
    #[extractor(field = 0)]
    pub id: i32,
    /// Event ID triggered when camera moves.
    #[extractor(field = 1)]
    pub event_id_on_camera_move: i32,
    /// Initial spawn X coordinate.
    #[extractor(field = 2)]
    pub start_pos_x: i32,
    /// Initial spawn Y coordinate.
    #[extractor(field = 3)]
    pub start_pos_y: i32,
    /// Target map ID to link to.
    #[extractor(field = 4)]
    pub map_id: i32,
    /// Monster placement .ref filename.
    #[extractor(field = 5, parse_null)]
    pub monsters_filename: Option<String>,
    /// NPC placement .ref filename.
    #[extractor(field = 6, parse_null)]
    pub npc_filename: Option<String>,
    /// Extra interactive objects .ref filename.
    #[extractor(field = 7, parse_null)]
    pub extra_filename: Option<String>,
    /// Audio track index for map background music.
    #[extractor(field = 8)]
    pub cd_music_track_number: i32,
}

pub fn read_map_ini(source_path: &Path) -> std::io::Result<Vec<MapIni>> {
    MapIni::read_file(source_path)
}

pub fn save_map_inis(conn: &mut Connection, map_inis: &[MapIni]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_map_ini.sql"))?;
        for map_ini in map_inis {
            stmt.execute(params![
                map_ini.id,
                map_ini.event_id_on_camera_move,
                map_ini.start_pos_x,
                map_ini.start_pos_y,
                map_ini.map_id,
                map_ini.monsters_filename,
                map_ini.npc_filename,
                map_ini.extra_filename,
                map_ini.cd_music_track_number,
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

    #[test]
    fn parse_entry_with_nulls() {
        let data = b"1,0,5,10,2,null,null,null,3\n";
        let mut c = Cursor::new(data.as_ref());
        let maps = MapIni::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(maps.len(), 1);
        assert_eq!(maps[0].id, 1);
        assert_eq!(maps[0].start_pos_x, 5);
        assert_eq!(maps[0].start_pos_y, 10);
        assert_eq!(maps[0].map_id, 2);
        assert_eq!(maps[0].cd_music_track_number, 3);
        assert!(maps[0].monsters_filename.is_none());
        assert!(maps[0].npc_filename.is_none());
    }

    #[test]
    fn parse_entry_with_filenames() {
        let data = b"2,100,3,8,5,mon.ref,npc.ref,ext.ref,7\n";
        let mut c = Cursor::new(data.as_ref());
        let maps = MapIni::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(maps[0].monsters_filename.as_deref(), Some("mon.ref"));
        assert_eq!(maps[0].npc_filename.as_deref(), Some("npc.ref"));
        assert_eq!(maps[0].extra_filename.as_deref(), Some("ext.ref"));
    }

    #[test]
    fn parse_skips_comments_and_empty() {
        let data = b"; header\n\n1,0,0,0,1,null,null,null,1\n";
        let mut c = Cursor::new(data.as_ref());
        let maps = MapIni::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(maps.len(), 1);
    }

    #[test]
    fn serialize_round_trip() {
        let data = b"1,0,5,10,2,null,null,null,3\r\n";
        let mut c = Cursor::new(data.as_ref());
        let records = MapIni::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        MapIni::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = MapIni::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].id, records2[0].id);
        assert_eq!(records[0].map_id, records2[0].map_id);
        assert_eq!(
            records[0].cd_music_track_number,
            records2[0].cd_music_track_number
        );
    }
}
