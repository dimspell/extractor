use std::io::{BufRead, BufReader, Read, Seek, Write};
use std::path::Path;

use crate::references::extractor::{parse_null, Extractor};
use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

// ===========================================================================
// MAP.INI FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | Map.ini - Map Initialization Data    |
// +--------------------------------------+
// | Encoding: EUC-KR                    |
// | Format: CSV with comments            |
// | Record Size: Variable (text)        |
// +--------------------------------------+
// | ; Comment line                       |
// | id,camera_event,start_x,start_y,map_id,monsters,npcs,extras,cd_track|
// | 1,1001,5,10,1,mon1.ref,npc1.ref,ext1.ref,1|
// | 2,1002,3,8,2,mon2.ref,npc2.ref,ext2.ref,2|
// | ...                                   |
// +--------------------------------------+
//
// FIELD DEFINITIONS:
// - id: Unique map identifier
// - camera_event: Event triggered on camera movement
// - start_x: Initial player X coordinate
// - start_y: Initial player Y coordinate
// - map_id: Target map ID for linking
// - monsters: Monster placement REF file
// - npcs: NPC placement REF file
// - extras: Extra object placement REF file
// - cd_track: Background music track number
//
// SPECIAL VALUES:
// - "null" literal for missing REF files
// - Lines starting with ";" are comments
// - CSV format with comma delimiter
// - Coordinates use isometric tile system
//
// FILE PURPOSE:
// Defines map initialization parameters including
// starting positions, linked files, and music tracks.
// Used for map loading and setup.
//
// ===========================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapIni {
    /// Map ini identifier.
    pub id: i32,
    /// Event ID triggered when camera moves.
    pub event_id_on_camera_move: i32,
    /// Initial spawn X coordinate.
    pub start_pos_x: i32,
    /// Initial spawn Y coordinate.
    pub start_pos_y: i32,
    /// Target map ID to link to.
    pub map_id: i32,
    /// Monster placement .ref filename.
    pub monsters_filename: Option<String>,
    /// NPC placement .ref filename.
    pub npc_filename: Option<String>,
    /// Extra interactive objects .ref filename.
    pub extra_filename: Option<String>,
    /// Audio track index for map background music.
    pub cd_music_track_number: i32, // CD music track number
}

/// Stores specific properties and configuration for a single map.
///
/// Reads file: `Ref/Map.ini`
/// # File Format: `Ref/Map.ini`
///
/// Text file, EUC-KR encoded. One record per line, CSV format:
/// ```text
/// id,event_id_on_camera_move,start_x,start_y,map_id,monsters_file,npc_file,extra_file,cd_track
/// ```
/// - Optional filenames use literal `null`.
/// - `cd_track` is the background music CD track index.
impl Extractor for MapIni {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        let decoded = DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(reader.by_ref());
        let buf_reader = BufReader::new(decoded);
        let mut map_inis: Vec<MapIni> = Vec::new();
        for line in buf_reader.lines().map_while(std::io::Result::ok) {
            let trimmed = line.trim();
            if trimmed.starts_with(";") || trimmed.is_empty() {
                continue;
            }

            let parts: Vec<&str> = trimmed.split(",").collect();
            if parts.len() < 9 {
                continue;
            }

            let id: i32 = parts[0].trim().parse::<i32>().unwrap();
            let event_id_on_camera_move = parts[1].trim().parse::<i32>().unwrap();
            let start_pos_x = parts[2].trim().parse::<i32>().unwrap();
            let start_pos_y = parts[3].trim().parse::<i32>().unwrap();
            let map_id = parts[4].trim().parse::<i32>().unwrap();
            let monsters_filename = parse_null(parts[5].trim());
            let npc_filename = parse_null(parts[6].trim());
            let extra_filename = parse_null(parts[7].trim());
            let cd_music_track_number = parts[8].trim().parse::<i32>().unwrap();

            map_inis.push(MapIni {
                id,
                event_id_on_camera_move,
                start_pos_x,
                start_pos_y,
                map_id,
                monsters_filename,
                npc_filename,
                extra_filename,
                cd_music_track_number,
            });
        }
        Ok(map_inis)
    }

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        for record in records {
            let mon = record.monsters_filename.as_deref().unwrap_or("null");
            let npc = record.npc_filename.as_deref().unwrap_or("null");
            let ext = record.extra_filename.as_deref().unwrap_or("null");

            let line = format!(
                "{},{},{},{},{},{},{},{},{}\r\n",
                record.id,
                record.event_id_on_camera_move,
                record.start_pos_x,
                record.start_pos_y,
                record.map_id,
                mon,
                npc,
                ext,
                record.cd_music_track_number
            );
            let (cow, _, _) = EUC_KR.encode(&line);
            writer.write_all(&cow)?;
        }
        Ok(())
    }
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
