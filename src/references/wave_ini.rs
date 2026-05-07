use std::path::Path;

use crate::references::extractor::Extractor;
use dispel_macros::{TextExtractor, TextRecordPatcher};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// Wave.ini - Audio/Sound References
///
/// Maps sound IDs to SNF audio files with playback
/// behavior flags. Used for audio system initialization
/// and sound effect management.
///
/// Reads file: `Wave.ini`
///
/// # ASCII Structure
///
/// ```text
/// +--------------------------------------+
/// | Wave.ini - Audio/Sound References    |
/// +--------------------------------------+
/// | Encoding: EUC-KR                    |
/// | Format: CSV with comments            |
/// | Record Size: Variable (text)        |
/// +--------------------------------------+
/// | ; Comment line                       |
/// | id,snf_filename,unknown_flag           |
/// | 1,music1.snf,loop                     |
/// | 2,effect1.snf,once                    |
/// | ...                                   |
/// +--------------------------------------+
/// ```
///
/// # Field Definitions
///
/// - `id`: Unique sound/audio identifier
/// - `snf_filename`: SNF audio file (or "null")
/// - `unknown_flag`: Playback behavior flag (or "null")
///
/// # Sound Categories
///
/// - `1-50`: Background music tracks
/// - `51-100`: Ambient sounds
/// - `101-200`: Combat sound effects
/// - `201-300`: UI/interface sounds
/// - `301-400`: Character voice effects
/// - `401-500`: Environmental sounds
///
/// # Unknown Flag Values
///
/// - `"loop"`: Looping playback
/// - `"once"`: Play once
/// - `"3d"`: Positional audio
/// - `"stream"`: Stream from disk
/// - `"null"`: Default behavior
///
/// # Special Values
///
/// - `"null"` literal for missing SNF filenames or flags
/// - Lines starting with `;` are comments
/// - CSV format with comma delimiter
///
/// # File Purpose
///
/// Maps sound IDs to SNF audio files with playback
/// behavior flags. Used for audio system initialization
/// and sound effect management.
#[derive(Debug, Clone, Serialize, Deserialize, Default, TextExtractor, TextRecordPatcher)]
#[extractor(encoding = "EUC_KR")]
#[patcher(filename = "Wave.ini")]
pub struct WaveIni {
    /// Sound effect reference identifier.
    #[extractor(field = 0)]
    pub id: i32,
    /// Raw audio filename in .SNF format.
    #[extractor(field = 1, parse_null)]
    pub snf_filename: Option<String>,
    /// Internal unknown string or flag parameter.
    #[extractor(field = 2, parse_null)]
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

    #[test]
    fn serialize_round_trip() {
        let data = b"1,music.snf,loop\r\n2,null,null\r\n";
        let mut c = Cursor::new(data.as_ref());
        let records = WaveIni::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        WaveIni::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = WaveIni::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].id, records2[0].id);
        assert_eq!(records[0].snf_filename, records2[0].snf_filename);
        assert_eq!(records[1].snf_filename, records2[1].snf_filename);
    }
}
