//! Parser for `ExtraInGame/fogdata.dat` — observed map-lighting fade tables.
//!
//! The file is a flat table of exactly [`ROWS`] (123) rows × [`ROW_LEN`]
//! (512) bytes = 62,976 bytes total, with no header.
//!
//! Row `L-1` serves light level `L`. Map data carries levels `1..=199`,
//! but the table only covers levels `1..=123`; levels beyond that index
//! past the table (callers must clamp or skip).
//!
//! Each byte is a brightness factor `f` in `0..=[MAX_FACTOR]` (31), i.e. a
//! 5-bit fixed-point value: the effective multiplier on shadowed tile pixels
//! is `f/32`, applied to the **red and green channels only** (blue stays
//! untouched). Rows are animated flicker patterns — values are *not*
//! monotonic in level.
//!
//! A consumer indexes the file as `byte[(level-1)*512 + pair]` where `pair`
//! is the pixel-pair index (`0..512`) within a tile; each factor byte covers
//! two horizontally adjacent pixels.
//!
//! This data feeds the map lighting pass for maps flagged Dark in
//! `AllMap.ini`; see [`super::render::plot_shadows`] and
//! `docs/rendering.md`.

use std::fmt;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Number of fade-table rows (one per light level).
pub const ROWS: usize = 123;
/// Bytes per row (one brightness factor per pixel pair).
pub const ROW_LEN: usize = 512;
/// Total expected file size in bytes.
pub const EXPECTED_LEN: usize = ROWS * ROW_LEN;
/// Maximum valid brightness factor (5-bit fixed point).
pub const MAX_FACTOR: u8 = 31;

/// Errors returned by [`FogData::set_factor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetFactorError {
    /// Factor values above 31 are rejected: out-of-range values wrap when
    /// consumed, and a modding tool refuses them instead.
    ValueOutOfRange(u8),
    /// Light level outside `1..=[ROWS]`.
    LevelOutOfRange(u32),
    /// Pixel-pair index outside `0..[ROW_LEN]`.
    PairOutOfRange(usize),
}

impl fmt::Display for SetFactorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValueOutOfRange(v) => {
                write!(f, "factor {v} exceeds MAX_FACTOR ({MAX_FACTOR})")
            }
            Self::LevelOutOfRange(l) => write!(f, "level {l} out of range 1..={ROWS}"),
            Self::PairOutOfRange(p) => write!(f, "pair {p} out of range 0..{ROW_LEN}"),
        }
    }
}

impl std::error::Error for SetFactorError {}

/// In-memory representation of `ExtraInGame/fogdata.dat`.
///
/// See the [module documentation](self) for the binary layout and semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FogData {
    data: Vec<u8>,
}

impl FogData {
    /// Parses the fade tables from a reader positioned at the start of a
    /// `fogdata.dat` stream.
    ///
    /// Reads exactly [`EXPECTED_LEN`] bytes; short streams are rejected with
    /// `InvalidData`. Extra trailing bytes are tolerated (callers normally
    /// pass exact-length streams).
    ///
    /// # Seek behavior
    /// The reader is rewound to offset 0 before reading, regardless of its
    /// current position.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> std::io::Result<Self> {
        reader.seek(SeekFrom::Start(0))?;
        let mut data = vec![0u8; EXPECTED_LEN];
        reader.read_exact(&mut data).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("fogdata.dat too short: expected {EXPECTED_LEN} bytes ({e})"),
            )
        })?;
        Ok(Self { data })
    }

    /// Loads the fade tables from `<game_path>/ExtraInGame/fogdata.dat`.
    pub fn load(game_path: &Path) -> std::io::Result<Self> {
        let path = game_path.join("ExtraInGame").join("fogdata.dat");
        if path.metadata()?.len() < EXPECTED_LEN as u64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "fogdata.dat too small: expected at least {} bytes",
                    EXPECTED_LEN
                ),
            ));
        }
        let mut file = File::open(&path)?;
        Self::parse(&mut file)
    }

    /// Writes all 62,976 raw bytes to `w`.
    pub fn to_writer<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_all(&self.data)
    }

    /// Saves the fade tables to `path`, creating or truncating the file.
    pub fn save_file(&self, path: &Path) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        self.to_writer(&mut file)
    }

    /// Fade factor for light `level` (1-based) at pixel-pair index `pair`
    /// (0..512).
    ///
    /// # Panics
    /// Panics when `level == 0`, `level > ROWS`, or `pair >= ROW_LEN`.
    /// Hot-path accessor — use [`FogData::get_factor`] for a panic-free
    /// variant.
    pub fn factor(&self, level: u32, pair: usize) -> u8 {
        self.data[((level - 1) as usize) * ROW_LEN + pair]
    }

    /// Panic-free variant of [`FogData::factor`]; returns `None` when
    /// `level` or `pair` is out of range.
    pub fn get_factor(&self, level: u32, pair: usize) -> Option<u8> {
        if level == 0 || level as usize > ROWS || pair >= ROW_LEN {
            return None;
        }
        Some(self.factor(level, pair))
    }

    /// Sets the fade factor for light `level` at pixel-pair index `pair`.
    ///
    /// # Errors
    /// Returns [`SetFactorError::ValueOutOfRange`] when `value > MAX_FACTOR`
    /// (out-of-range values wrap when consumed, so editors must enforce the
    /// range here), [`SetFactorError::LevelOutOfRange`] when
    /// `level == 0 || level > ROWS`, or [`SetFactorError::PairOutOfRange`]
    /// when `pair >= ROW_LEN`.
    pub fn set_factor(&mut self, level: u32, pair: usize, value: u8) -> Result<(), SetFactorError> {
        if value > MAX_FACTOR {
            return Err(SetFactorError::ValueOutOfRange(value));
        }
        if level == 0 || level as usize > ROWS {
            return Err(SetFactorError::LevelOutOfRange(level));
        }
        if pair >= ROW_LEN {
            return Err(SetFactorError::PairOutOfRange(pair));
        }
        self.data[((level - 1) as usize) * ROW_LEN + pair] = value;
        Ok(())
    }

    /// The full 512-byte row serving light `level`.
    ///
    /// # Panics
    /// Panics when `level == 0` or `level > ROWS`.
    pub fn row(&self, level: u32) -> &[u8] {
        let start = ((level - 1) as usize) * ROW_LEN;
        &self.data[start..start + ROW_LEN]
    }

    /// Total byte length (always [`EXPECTED_LEN`]).
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Always `false` — a parsed table always holds [`EXPECTED_LEN`] bytes.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Test-only constructor from raw bytes (sibling-module tests included).
    #[cfg(test)]
    pub(crate) fn from_raw(data: Vec<u8>) -> Self {
        debug_assert_eq!(data.len(), EXPECTED_LEN);
        Self { data }
    }
}

/// Persists the full fade table to the `fog_factors` SQLite table.
///
/// Writes exactly [`ROWS`] × [`ROW_LEN`] = 62,976 rows — one per file byte —
/// so the decoded table is a fully lossless 1:1 representation of
/// `fogdata.dat` (no raw blob is needed; `byte[(level-1)*512 + pair]`
/// round-trips exactly).
pub fn save_to_db(conn: &mut rusqlite::Connection, fog: &FogData) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_fog_factor.sql"))?;
        for level in 1..=ROWS as u32 {
            for pair in 0..ROW_LEN {
                stmt.execute(rusqlite::params![
                    level as i32,
                    pair as i32,
                    fog.factor(level, pair) as i32,
                ])?;
            }
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
    fn test_parse_rejects_short_buffer() {
        let buf = vec![0u8; 100];
        let err = FogData::parse(&mut Cursor::new(buf)).expect_err("short buffer must be rejected");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_parse_and_factor_spot_checks() {
        let mut data = vec![0u8; EXPECTED_LEN];
        for level in 1..=ROWS as u32 {
            for p in 0..ROW_LEN {
                data[((level - 1) as usize) * ROW_LEN + p] = ((level * 7 + p as u32) % 32) as u8;
            }
        }
        let fog = FogData::parse(&mut Cursor::new(data)).unwrap();

        assert_eq!(fog.factor(1, 0), 7);
        assert_eq!(fog.factor(2, 0), 14);
        assert_eq!(fog.factor(123, 0), ((123 * 7) % 32) as u8);
        assert_eq!(fog.factor(123, 511), ((123 * 7 + 511) % 32) as u8);
        assert_eq!(fog.factor(65, 256), ((65 * 7 + 256) % 32) as u8);
    }

    #[test]
    fn test_round_trip_is_byte_identical() {
        let mut data = vec![0u8; EXPECTED_LEN];
        for (i, b) in data.iter_mut().enumerate() {
            *b = (i % 251) as u8;
        }
        let fog = FogData::parse(&mut Cursor::new(data.clone())).unwrap();
        let mut out = Vec::new();
        fog.to_writer(&mut out).unwrap();
        assert_eq!(out, data);
    }

    #[test]
    fn test_set_factor_valid_write_and_rejections() {
        let mut fog = FogData::from_raw(vec![0u8; EXPECTED_LEN]);

        fog.set_factor(4, 10, 17).unwrap();
        assert_eq!(fog.factor(4, 10), 17);

        assert_eq!(
            fog.set_factor(1, 0, 32),
            Err(SetFactorError::ValueOutOfRange(32))
        );
        assert_eq!(
            fog.set_factor(0, 0, 1),
            Err(SetFactorError::LevelOutOfRange(0))
        );
        assert_eq!(
            fog.set_factor(124, 0, 1),
            Err(SetFactorError::LevelOutOfRange(124))
        );
        assert_eq!(
            fog.set_factor(1, 512, 1),
            Err(SetFactorError::PairOutOfRange(512))
        );
    }

    #[test]
    fn test_get_factor_range_checks() {
        let fog = FogData::from_raw(vec![7u8; EXPECTED_LEN]);
        assert_eq!(fog.get_factor(1, 0), Some(7));
        assert_eq!(fog.get_factor(123, 511), Some(7));
        assert_eq!(fog.get_factor(0, 0), None);
        assert_eq!(fog.get_factor(124, 0), None);
        assert_eq!(fog.get_factor(1, 512), None);
    }

    #[test]
    fn test_row_matches_factor_samples() {
        let fog = FogData::from_raw(vec![3u8; EXPECTED_LEN]);
        let row = fog.row(5);
        assert_eq!(row.len(), ROW_LEN);
        for p in [0usize, 1, 255, 256, 511] {
            assert_eq!(row[p], fog.factor(5, p));
        }
    }

    #[test]
    fn test_save_to_db_round_trips_fixture() {
        let fixture_dir = Path::new("fixtures/Dispel");
        if !fixture_dir.join("ExtraInGame/fogdata.dat").exists() {
            eprintln!("Skipping test_save_to_db_round_trips_fixture: fixture not found");
            return;
        }

        let fog = FogData::load(fixture_dir).expect("loading fogdata.dat fixture");
        let mut conn = rusqlite::Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(include_str!("../queries/create_table_fog_factors.sql"))
            .expect("creating fog_factors table");

        save_to_db(&mut conn, &fog).expect("save_to_db");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM fog_factors", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count as usize, EXPECTED_LEN);

        // Known values from the shipped file (see render.rs tests).
        let factor_at = |level: i32, pair: i32| -> i32 {
            conn.query_row(
                "SELECT factor FROM fog_factors WHERE level = ?1 AND pair_index = ?2",
                rusqlite::params![level, pair],
                |row| row.get(0),
            )
            .unwrap()
        };
        assert_eq!(factor_at(1, 0), 2);
        assert_eq!(factor_at(65, 256), 31);

        // Sampled rows must equal the in-memory table exactly.
        for (k, p) in [(1u32, 0usize), (2, 1), (65, 256), (100, 511), (123, 511)] {
            let stored = factor_at(k as i32, p as i32);
            assert_eq!(stored as u8, fog.row(k)[p], "level {k} pair {p}");
        }
    }
}
