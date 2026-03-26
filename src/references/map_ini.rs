use std::io::{BufRead, BufReader, Write};
use std::{fs::File, path::Path};

use crate::references::references::{parse_null, Extractor};
use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MapIni {
    pub id: i32,
    // id
    pub event_id_on_camera_move: i32,
    // event that occurs when camera moves
    pub start_pos_x: i32,
    // start position X
    pub start_pos_y: i32,
    // start position Y
    pub map_id: i32,
    // map id
    pub monsters_filename: Option<String>,
    // monsters filename
    pub npc_filename: Option<String>,
    // NPC filename
    pub extra_filename: Option<String>,
    // extra filename
    pub cd_music_track_number: i32, // CD music track number
}

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
