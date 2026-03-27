use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader, Write};

use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use crate::references::references::{parse_int, parse_null, Extractor};

#[derive(Debug, Serialize, Deserialize)]
pub struct PartyPgp {
    /// Logic block identifier.
    pub id: i32,
    /// Literal string or branch condition for dialogue.
    pub dialog_text: Option<String>,
    /// Internal script parameter 1.
    pub unknown_id1: Option<i32>,
    /// Internal script parameter 2.
    pub unknown_id2: Option<i32>,

}

/// Stores party dialogue logic and ID references.
///
/// Reads file: `NpcInGame/PartyPgp.pgp`
/// # File Format: `NpcInGame/PartyPgp.pgp`
///
/// Text file, WINDOWS-1250 encoded. One record per line, pipe-delimited:
/// ```text
/// id|dialog_text|unknown_id1|unknown_id2
/// ```
/// - `dialog_text`, `unknown_id1`, `unknown_id2` use literal `null` when absent.
impl Extractor for PartyPgp {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let f = File::open(source_path)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1250))
                .build(f),
        );
        let mut pgps: Vec<PartyPgp> = Vec::new();
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.starts_with(";") || trimmed.is_empty() {
                        continue;
                    }
                    let parts: Vec<&str> = trimmed.split("|").collect();
                    if parts.len() < 4 {
                        continue;
                    }

                    let id: i32 = parts[0].trim().parse::<i32>().unwrap();
                    let dialog_text = parse_null(parts[1].trim());
                    let unknown_id1 = parse_int(parts[2].trim());
                    let unknown_id2 = parse_int(parts[3].trim());
                    pgps.push(PartyPgp {
                        id,
                        dialog_text,
                        unknown_id1,
                        unknown_id2,
                    });
                }
                _ => {}
            }
        }
        Ok(pgps)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            let dlg = record.dialog_text.as_deref().unwrap_or("null");
            let unk1 = record.unknown_id1.map_or("null".to_string(), |v| v.to_string());
            let unk2 = record.unknown_id2.map_or("null".to_string(), |v| v.to_string());

            let line = format!("{}|{}|{}|{}\r\n", record.id, dlg, unk1, unk2);
            let (cow, _, _) = WINDOWS_1250.encode(&line);
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_party_pgps(source_path: &Path) -> std::io::Result<Vec<PartyPgp>> {
    PartyPgp::read_file(source_path)
}