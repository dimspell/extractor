use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader, Write};

use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use crate::references::references::{parse_null, Extractor};

#[derive(Debug, Serialize, Deserialize)]
pub struct Extra {
    /// Tool or object identifier.
    pub id: i32,
    /// Base SPR/SPX filename for the object.
    pub sprite_filename: Option<String>,
    /// Internal unknown flag.
    pub unknown: i32,
    /// Optional description for the interactive object.
    pub description: Option<String>,

}

/// Stores definitions and types for interactive objects (extras).
///
/// Reads file: `Extra.ini`
impl Extractor for Extra {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
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
                    if parts.len() < 4 {
                        continue;
                    }
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
                _ => {}
            }
        }
        Ok(extras)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            let sprite = record.sprite_filename.as_deref().unwrap_or("null");
            let desc = record.description.as_deref().unwrap_or("null");
            let line = format!("{},{},{},{}\r\n", record.id, sprite, record.unknown, desc);
            let (cow, _, _) = EUC_KR.encode(&line);
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_extra_ini(source_path: &Path) -> std::io::Result<Vec<Extra>> {
    Extra::read_file(source_path)
}

