use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader, Write};

use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use crate::references::references::{parse_null, Extractor};

#[derive(Debug, Serialize, Deserialize)]
pub struct PartyRef {
    pub id: i32,
    pub full_name: Option<String>,
    pub job_name: Option<String>,
    pub root_map_id: i32,
    pub npc_id: i32,
    pub dlg_when_not_in_party: i32,
    pub dlg_when_in_party: i32,
    pub ghost_face_id: i32,
}

impl Extractor for PartyRef {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let f = File::open(source_path)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1250))
                .build(f),
        );
        let mut party_refs: Vec<PartyRef> = Vec::new();
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.starts_with(";") || trimmed.is_empty() {
                        continue;
                    }

                    let parts: Vec<&str> = trimmed.split(",").collect();
                    if parts.len() < 8 {
                        continue;
                    }

                    let id = parts[0].trim().parse::<i32>().unwrap();
                    let full_name = parse_null(parts[1].trim());
                    let job_name = parse_null(parts[2].trim());
                    let root_map_id = parts[3].trim().parse::<i32>().unwrap();
                    let npc_id = parts[4].trim().parse::<i32>().unwrap();
                    let dlg_when_not_in_party = parts[5].trim().parse::<i32>().unwrap();
                    let dlg_when_in_party = parts[6].trim().parse::<i32>().unwrap();
                    let ghost_face_id = parts[7].trim().parse::<i32>().unwrap();

                    party_refs.push(PartyRef {
                        id,
                        full_name,
                        job_name,
                        root_map_id,
                        npc_id,
                        dlg_when_not_in_party,
                        dlg_when_in_party,
                        ghost_face_id,
                    });
                }
                _ => {}
            }
        }
        Ok(party_refs)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            let full_name = record.full_name.as_deref().unwrap_or("null");
            let job_name = record.job_name.as_deref().unwrap_or("null");

            let line = format!(
                "{},{},{},{},{},{},{},{}\r\n",
                record.id,
                full_name,
                job_name,
                record.root_map_id,
                record.npc_id,
                record.dlg_when_not_in_party,
                record.dlg_when_in_party,
                record.ghost_face_id
            );
            let (cow, _, _) = WINDOWS_1250.encode(&line);
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_part_refs(source_path: &Path) -> std::io::Result<Vec<PartyRef>> {
    PartyRef::read_file(source_path)
}
