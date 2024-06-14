use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader};

use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use crate::references::references::parse_null;

#[derive(Debug, Serialize, Deserialize)]
pub struct WaveIni {
    pub id: i32,
    pub snf_filename: Option<String>,
    pub unknown_flag: Option<String>,
}

pub fn read_wave_ini(source_path: &Path) -> std::io::Result<Vec<WaveIni>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(f),
    );
    let mut waves_inis: Vec<WaveIni> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();

                let id = parts[0].parse::<i32>().unwrap();
                let snf_filename = parse_null(parts[1]);
                let unknown_flag = parse_null(parts[2]);

                waves_inis.push(WaveIni {
                    id,
                    snf_filename,
                    unknown_flag,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(waves_inis)
}