use std::io::{BufRead, BufReader, Write};
use std::{fs::File, path::Path};

use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use crate::references::references::{parse_null, Extractor};

// ===========================================================================
// NPC.INI FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | Npc.ini - NPC Visual References      |
// +--------------------------------------+
// | Encoding: EUC-KR                    |
// | Format: CSV with comments            |
// | Record Size: Variable (text)        |
// +--------------------------------------+
// | ; Comment line                       |
// | id,sprite_filename,description       |
// | 1,guard.spr,City Guard              |
// | 2,merchant.spr,Shopkeeper           |
// | ...                                  |
// +--------------------------------------+
//
// FIELD DEFINITIONS:
// - id: Unique NPC visual type identifier
// - sprite_filename: SPR filename or "null"
// - description: NPC role/appearance description
//
// SPECIAL VALUES:
// - "null" literal for missing sprite filenames
// - Lines starting with ";" are comments
// - CSV format with comma delimiter
//
// FILE PURPOSE:
// Defines visual appearances for NPC characters with sprite
// filenames and descriptions. Used for rendering NPCs in
// the game world and linking to NPC behavior scripts.
//
// ===========================================================================

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
            if let Ok(line) = line {
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

pub fn save_npc_inis(conn: &mut Connection, npc_inis: &Vec<NpcIni>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_npc_ini.sql"))?;
        for npc_ini in npc_inis {
            stmt.execute(params![
                npc_ini.id,
                npc_ini.sprite_filename,
                npc_ini.description,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
