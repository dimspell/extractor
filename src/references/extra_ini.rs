use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader};

use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use crate::references::references::parse_null;

#[derive(Debug, Serialize, Deserialize)]
pub struct Extra {
    pub id: i32,
    pub sprite_filename: Option<String>,
    pub unknown: i32,
    pub description: Option<String>,
}

pub fn read_extra_ini(source_path: &Path) -> std::io::Result<Vec<Extra>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(f),
    );
    let mut extras: Vec<Extra> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();
                let id: i32 = parts[0].parse::<i32>().unwrap();
                let sprite_filename = parse_null(parts[1]);
                let unknown = parts[2].parse::<i32>().unwrap();
                let description = parse_null(parts[3]);

                extras.push(Extra {
                    id,
                    sprite_filename,
                    unknown,
                    description,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(extras)
}

