use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader};

use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};

use crate::references::references::parse_null;

#[derive(Debug, Serialize, Deserialize)]
pub struct NpcIni {
    pub id: i32,
    pub sprite_filename: Option<String>,
    pub description: String,
}

pub fn read_npc_ini(source_path: &Path) -> std::io::Result<Vec<NpcIni>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(f),
    );
    let mut npc_inis: Vec<NpcIni> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();

                let id = parts[0].parse::<i32>().unwrap();
                let sprite_filename = parse_null(parts[1]);
                let description = parts[2].to_string();

                npc_inis.push(NpcIni {
                    id,
                    sprite_filename,
                    description,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(npc_inis)
}
