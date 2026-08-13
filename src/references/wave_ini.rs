use std::path::Path;

use crate::references::extractor::Extractor;
use dispel_macros::{TextExtractor, TextRecordPatcher};
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

/// Wave.ini - Audio/Sound References
///
/// Maps sound IDs to SNF audio files and their simultaneous-playback limits.
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
/// | id,snf_filename,max_simultaneous_plays |
/// | 1,music1.snf,5                        |
/// | 2,effect1.snf,1                       |
/// | ...                                   |
/// +--------------------------------------+
/// ```
///
/// # Field Definitions
///
/// - `id`: Unique sound/audio identifier
/// - `snf_filename`: SNF audio file (or "null")
/// - `max_simultaneous_plays`: Number of copies allocated for concurrent
///   playback of this sound
///
///
/// # Special Values
///
/// - `"null"` literal for missing SNF filenames
/// - `max_simultaneous_plays`: `5` is the usual limit; `1` prevents overlap
/// - Lines starting with `;` are comments
/// - CSV format with comma delimiter
///
/// # File Purpose
///
/// Maps sound IDs to SNF audio files with playback
/// simultaneous-playback limits. Used for audio system initialization and
/// sound effect management.
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
    /// Maximum number of simultaneous instances of this sound.
    ///
    /// The loader stores this value as the number of DirectSound buffer copies
    /// for the entry. Playback uses the first free copy, so a value of `1`
    /// prevents the sound from overlapping with itself.
    #[extractor(field = 2)]
    pub max_simultaneous_plays: i32,
}

/// Stores audio references and SNF file mappings.
///
/// Reads file: `Wave.ini`
/// # File Format: `Wave.ini`
///
/// Text file, EUC-KR encoded. One record per line, CSV format:
/// ```text
/// id,snf_filename,max_simultaneous_plays
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
                wave_ini.max_simultaneous_plays,
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
        let data = b"1,music.snf,5\n2,null,1\n";
        let mut c = Cursor::new(data.as_ref());
        let waves = WaveIni::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(waves.len(), 2);
        assert_eq!(waves[0].id, 1);
        assert_eq!(waves[0].snf_filename.as_deref(), Some("music.snf"));
        assert_eq!(waves[0].max_simultaneous_plays, 5);
        assert_eq!(waves[1].snf_filename, None);
        assert_eq!(waves[1].max_simultaneous_plays, 1);
    }

    #[test]
    fn serialize_round_trip() {
        let data = b"1,music.snf,5\r\n2,null,1\r\n";
        let mut c = Cursor::new(data.as_ref());
        let records = WaveIni::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        WaveIni::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = WaveIni::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].id, records2[0].id);
        assert_eq!(records[0].snf_filename, records2[0].snf_filename);
        assert_eq!(
            records[0].max_simultaneous_plays,
            records2[0].max_simultaneous_plays
        );
        assert_eq!(records[1].snf_filename, records2[1].snf_filename);
    }
}
