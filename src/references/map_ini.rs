use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader};

use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use crate::references::references::parse_null;

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
    pub cd_music_track_number: i32,        // CD music track number
}

pub fn read_map_ini(source_path: &Path) -> std::io::Result<Vec<MapIni>> {
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
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();
                let id: i32 = parts[0].parse::<i32>().unwrap();
                let event_id_on_camera_move = parts[1].parse::<i32>().unwrap();
                let start_pos_x = parts[2].parse::<i32>().unwrap();
                let start_pos_y = parts[3].parse::<i32>().unwrap();
                let map_id = parts[4].parse::<i32>().unwrap();
                let monsters_filename = parse_null(parts[5]);
                let npc_filename = parse_null(parts[6]);
                let extra_filename = parse_null(parts[7]);
                let cd_music_track_number = parts[8].parse::<i32>().unwrap();

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
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(map_inis)
}
