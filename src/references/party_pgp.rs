use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader};

use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use crate::references::references::{parse_int, parse_null};

#[derive(Debug, Serialize, Deserialize)]
pub struct PartyPgp {
    pub id: i32,
    pub dialog_text: Option<String>,
    pub unknown_id1: Option<i32>,
    pub unknown_id2: Option<i32>,
}

pub fn read_party_pgps(source_path: &Path) -> std::io::Result<Vec<PartyPgp>> {
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
                if line.starts_with(";") {
                    continue;
                }
                let parts: Vec<&str> = line.split("|").collect();

                let id: i32 = parts[0].parse::<i32>().unwrap();
                let dialog_text = parse_null(parts[1]);
                let unknown_id1 = parse_int(parts[2]);
                let unknown_id2 = parse_int(parts[3]);
                pgps.push(PartyPgp {
                    id,
                    dialog_text,
                    unknown_id1,
                    unknown_id2,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(pgps)
}