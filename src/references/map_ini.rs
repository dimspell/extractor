use std::io::{BufRead, BufReader, Write};
use std::{fs::File, path::Path};

use crate::references::references::{parse_null, Extractor};
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

#[derive(Debug, Serialize, Deserialize)]
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
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let f = File::open(source_path)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(EUC_KR))
                .build(f),
        );
        let mut map_inis: Vec<MapIni> = Vec::new();
        for line in reader.lines() {
            match line {
                Ok(line) => {
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
                _ => {}
            }
        }
        Ok(map_inis)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
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
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_map_ini(source_path: &Path) -> std::io::Result<Vec<MapIni>> {
    MapIni::read_file(source_path)
}

pub fn save_map_inis(conn: &mut Connection, map_inis: &Vec<MapIni>) -> Result<()> {
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
