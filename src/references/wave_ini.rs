use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader, Write};

use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use crate::references::references::{parse_null, Extractor};

#[derive(Debug, Serialize, Deserialize)]
pub struct WaveIni {
    /// Sound effect reference identifier.
    pub id: i32,
    /// Raw audio filename in .SNF format.
    pub snf_filename: Option<String>,
    /// Internal unknown string or flag parameter.
    pub unknown_flag: Option<String>,

}

/// Stores audio references and SNF file mappings.
///
/// Reads file: `Wave.ini`
impl Extractor for WaveIni {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
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
                    let trimmed = line.trim();
                    if trimmed.starts_with(";") || trimmed.is_empty() {
                        continue;
                    }

                    let parts: Vec<&str> = trimmed.split(",").collect();
                    if parts.len() < 3 {
                        continue;
                    }

                    let id = parts[0].trim().parse::<i32>().unwrap();
                    let snf_filename = parse_null(parts[1].trim());
                    let unknown_flag = parse_null(parts[2].trim());

                    waves_inis.push(WaveIni {
                        id,
                        snf_filename,
                        unknown_flag,
                    });
                }
                _ => {}
            }
        }
        Ok(waves_inis)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            let snf = record.snf_filename.as_deref().unwrap_or("null");
            let unk = record.unknown_flag.as_deref().unwrap_or("null");

            let line = format!("{},{},{}\r\n", record.id, snf, unk);
            let (cow, _, _) = EUC_KR.encode(&line);
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_wave_ini(source_path: &Path) -> std::io::Result<Vec<WaveIni>> {
    WaveIni::read_file(source_path)
}