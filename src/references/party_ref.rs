use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader};

use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use crate::references::references::parse_null;

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

pub fn read_part_refs(source_path: &Path) -> std::io::Result<Vec<PartyRef>> {
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
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();

                let id = parts[0].parse::<i32>().unwrap();
                let full_name = parse_null(parts[1]);
                let job_name = parse_null(parts[2]);
                let root_map_id = parts[3].parse::<i32>().unwrap();
                let npc_id = parts[4].parse::<i32>().unwrap();
                let dlg_when_not_in_party = parts[5].parse::<i32>().unwrap();
                let dlg_when_in_party = parts[6].parse::<i32>().unwrap();
                let ghost_face_id = parts[7].parse::<i32>().unwrap();

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
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(party_refs)
}

