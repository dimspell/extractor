use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader, Write};

use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};

use crate::references::references::{parse_null, Extractor};

#[derive(Debug, Serialize, Deserialize)]
pub struct NpcIni {
    /// NPC visual type identifier.
    pub id: i32,
    /// Sprite filename representing the NPC.
    pub sprite_filename: Option<String>,
    /// Internal description or role of the NPC.
    pub description: String,

}

/// Stores visual references and configuration for NPCs.
///
/// Reads file: `Npc.ini`
/// # File Format: `Npc.ini`
///
/// Text file, WINDOWS-1250 encoded. One record per line, CSV format:
/// ```text
/// id,sprite_filename,description
/// ```
/// - `sprite_filename` uses literal `null` when absent.
impl Extractor for NpcIni {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
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
                    let trimmed = line.trim();
                    if trimmed.starts_with(";") || trimmed.is_empty() {
                        continue;
                    }

                    let parts: Vec<&str> = trimmed.split(",").collect();
                    if parts.len() < 3 {
                        continue;
                    }

                    let id = parts[0].trim().parse::<i32>().unwrap();
                    let sprite_filename = parse_null(parts[1].trim());
                    let description = parts[2].trim().to_string();

                    npc_inis.push(NpcIni {
                        id,
                        sprite_filename,
                        description,
                    });
                }
                _ => {}
            }
        }
        Ok(npc_inis)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            let sprite = record.sprite_filename.as_deref().unwrap_or("null");
            let line = format!("{},{},{}\r\n", record.id, sprite, record.description);
            let (cow, _, _) = EUC_KR.encode(&line);
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_npc_ini(source_path: &Path) -> std::io::Result<Vec<NpcIni>> {
    NpcIni::read_file(source_path)
}
