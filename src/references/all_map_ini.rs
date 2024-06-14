use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader};

use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};

use crate::references::references::parse_null;

#[derive(Debug, Serialize, Deserialize)]
pub struct Map {
    pub id: i32,
    pub map_filename: String,
    pub map_name: String,
    pub pgp_filename: Option<String>,
    pub dlg_filename: Option<String>,
    // light - 0=light, 1=darkness
    pub is_light: bool,
}

pub fn read_all_map_ini(source_path: &Path) -> std::io::Result<Vec<Map>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1250))
            .build(f),
    );

    let mut maps: Vec<Map> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();
                let id: i32 = parts[0].parse::<i32>().unwrap();
                let map_filename = parts[1].to_string();
                let map_name = parts[2].to_string();
                let pgp_filename = parse_null(parts[3]);
                let dlg_filename = parse_null(parts[4]);
                let is_light: bool = parts[5] == "1";

                maps.push(Map {
                    id,
                    map_filename,
                    map_name,
                    pgp_filename,
                    dlg_filename,
                    is_light,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(maps)
}

