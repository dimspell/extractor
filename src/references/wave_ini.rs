use std::io::{BufRead, BufReader, Read, Seek, Write};
use std::{fs::File, path::Path};

use crate::references::extractor::{parse_null, Extractor};
use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

// ===========================================================================
// WAVE.INI FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | Wave.ini - Audio/Sound References    |
// +--------------------------------------+
// | Encoding: EUC-KR                    |
// | Format: CSV with comments            |
// | Record Size: Variable (text)        |
// +--------------------------------------+
// | ; Comment line                       |
// | id,snf_filename,unknown_flag           |
// | 1,music1.snf,loop                     |
// | 2,effect1.snf,once                    |
// | ...                                   |
// +--------------------------------------+
//
// FIELD DEFINITIONS:
// - id: Unique sound/audio identifier
// - snf_filename: SNF audio file or "null"
// - unknown_flag: Playback behavior flag
//
// SOUND CATEGORIES:
// - 1-50: Background music tracks
// - 51-100: Ambient sounds
// - 101-200: Combat sound effects
// - 201-300: UI/interface sounds
// - 301-400: Character voice effects
// - 401-500: Environmental sounds
//
// UNKNOWN_FLAG VALUES:
// - "loop": Looping playback
// - "once": Play once
// - "3d": Positional audio
// - "stream": Stream from disk
// - "null": Default behavior
//
// SPECIAL VALUES:
// - "null" literal for missing SNF filenames
// - Lines starting with ";" are comments
// - CSV format with comma delimiter
//
// FILE PURPOSE:
// Maps sound IDs to SNF audio files with playback
// behavior flags. Used for audio system initialization
// and sound effect management.
//
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
/// # File Format: `Wave.ini`
///
/// Text file, EUC-KR encoded. One record per line, CSV format:
/// ```text
/// id,snf_filename,unknown_flag
/// ```
/// - `snf_filename` use literal `null` when absent.
impl Extractor for WaveIni {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        let decoded = DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(reader.by_ref());
        let buf_reader = BufReader::new(decoded);
        let mut waves_inis: Vec<WaveIni> = Vec::new();
        for line in buf_reader.lines().map_while(std::io::Result::ok) {
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

pub fn save_wave_inis(conn: &mut Connection, wave_inis: &[WaveIni]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_wave_ini.sql"))?;
        for wave_ini in wave_inis {
            stmt.execute(params![
                wave_ini.id,
                wave_ini.snf_filename,
                wave_ini.unknown_flag,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_entries() {
        let data = b"1,music.snf,loop\n2,null,null\n";
        let mut c = Cursor::new(data.as_ref());
        let waves = WaveIni::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(waves.len(), 2);
        assert_eq!(waves[0].id, 1);
        assert_eq!(waves[0].snf_filename.as_deref(), Some("music.snf"));
        assert_eq!(waves[0].unknown_flag.as_deref(), Some("loop"));
        assert_eq!(waves[1].snf_filename, None);
        assert_eq!(waves[1].unknown_flag, None);
    }
}
