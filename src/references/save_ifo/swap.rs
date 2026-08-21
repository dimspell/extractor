// Save-slot swap and summary operations for Dispel RPG.

use super::{SLOT_COUNT, SaveIfo, SaveSlotInfo};
use crate::references::extractor::Extractor;
#[cfg(test)]
use byteorder::WriteBytesExt;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::io::Write;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Size of the global tail block embedded at the end of each `.sav` payload.
const TAIL_SIZE: usize = 32;

/// The 32-byte global tail embedded in a `.sav` file (same layout as the
/// `Save.ifo` tail).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SavTail {
    pub game_version: f32,
    pub game_tmp_key: u32,
    pub map_id: u32,
    pub reserved: u32,
    pub payload_counts: [u32; 4],
}

impl SavTail {
    /// Peek a `.sav` file: read the u32 LE blob size, then seek to
    /// `4 + blob_size` and read exactly 32 bytes of tail.
    pub fn peek<R: Read + Seek>(reader: &mut R) -> std::io::Result<Self> {
        let blob_size = reader.read_u32::<LittleEndian>()?;
        let tail_offset = 4u64.wrapping_add(blob_size as u64);
        reader.seek(SeekFrom::Start(tail_offset))?;
        let mut buf = [0u8; TAIL_SIZE];
        reader.read_exact(&mut buf)?;

        let mut cursor = std::io::Cursor::new(&buf[..]);
        let game_version = cursor.read_f32::<LittleEndian>()?;
        let game_tmp_key = cursor.read_u32::<LittleEndian>()?;
        let map_id = cursor.read_u32::<LittleEndian>()?;
        let reserved = cursor.read_u32::<LittleEndian>()?;
        let mut payload_counts = [0u32; 4];
        for count in payload_counts.iter_mut() {
            *count = cursor.read_u32::<LittleEndian>()?;
        }
        Ok(SavTail {
            game_version,
            game_tmp_key,
            map_id,
            reserved,
            payload_counts,
        })
    }

    #[cfg(test)]
    fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_f32::<LittleEndian>(self.game_version)?;
        writer.write_u32::<LittleEndian>(self.game_tmp_key)?;
        writer.write_u32::<LittleEndian>(self.map_id)?;
        writer.write_u32::<LittleEndian>(self.reserved)?;
        for count in &self.payload_counts {
            writer.write_u32::<LittleEndian>(*count)?;
        }
        Ok(())
    }
}

/// Per-slot summary combining `Save.ifo` metadata with `.sav` presence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotSummary {
    pub index: usize,
    pub occupied: bool,
    pub sav_present: bool,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    /// None when the `.sav` is missing or unreadable.
    pub game_tmp_key: Option<u32>,
    /// None when the `.sav` is missing or unreadable.
    pub map_id: Option<u32>,
}

fn ifo_path(game_root: &Path) -> std::path::PathBuf {
    game_root.join("Save.ifo")
}

fn sav_path(game_root: &Path, index: usize) -> std::path::PathBuf {
    game_root.join(format!("{}.sav", index))
}

fn read_ifo(game_root: &Path) -> std::io::Result<SaveIfo> {
    let mut records = SaveIfo::read_file(&ifo_path(game_root))?;
    if records.len() != 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Save.ifo must contain exactly one record, got {}",
                records.len()
            ),
        ));
    }
    Ok(records.remove(0))
}

/// Summarize all six save slots: `Save.ifo` metadata plus a peek into each
/// slot's `.sav` payload (when present and readable).
pub fn summarize_slots(game_root: &Path) -> std::io::Result<Vec<SlotSummary>> {
    let ifo = read_ifo(game_root)?;
    let mut summaries = Vec::with_capacity(SLOT_COUNT);
    for (index, slot) in ifo.slots.iter().enumerate() {
        let path = sav_path(game_root, index);
        let (sav_present, game_tmp_key, map_id) = match std::fs::File::open(&path) {
            Err(_) => (false, None, None),
            Ok(mut file) => match SavTail::peek(&mut file) {
                Ok(tail) => (true, Some(tail.game_tmp_key), Some(tail.map_id)),
                // Present but corrupt: report presence without tail values.
                Err(_) => (true, None, None),
            },
        };
        summaries.push(SlotSummary {
            index,
            occupied: slot.is_occupied(),
            sav_present,
            month: slot.month,
            day: slot.day,
            hour: slot.hour,
            minute: slot.minute,
            game_tmp_key,
            map_id,
        });
    }
    Ok(summaries)
}

/// Swap two save slots: exchanges both the whole `.sav` files and their
/// 32-byte records inside `Save.ifo`. The global tail (current-session state)
/// is never touched.
///
/// File exchange rules:
/// - both exist → contents exchanged crosswise via temp files
/// - only one exists → content moves to the other position; the source record
///   in `Save.ifo` is cleared
/// - neither exists → record swap only
pub fn swap_slots(game_root: &Path, a: usize, b: usize) -> std::io::Result<()> {
    if a == b {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Cannot swap slot {} with itself", a),
        ));
    }
    if a >= SLOT_COUNT || b >= SLOT_COUNT {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Slot indices must be below {} (got {}, {})",
                SLOT_COUNT, a, b
            ),
        ));
    }

    let mut ifo = read_ifo(game_root)?;

    // Fail-fast: read BOTH .sav files fully before any mutation.
    let path_a = sav_path(game_root, a);
    let path_b = sav_path(game_root, b);
    let data_a = read_sav_if_exists(&path_a)?;
    let data_b = read_sav_if_exists(&path_b)?;

    // Validate embedded tails of existing files (fail-fast on corruption).
    for (index, data) in [(a, &data_a), (b, &data_b)] {
        if let Some(bytes) = data {
            let mut cursor = std::io::Cursor::new(bytes);
            if let Err(e) = SavTail::peek(&mut cursor) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Corrupt {}.sav: {}", index, e),
                ));
            }
        }
    }

    // Swap Save.ifo records (full records incl. reserved + flags).
    let tmp_record = ifo.slots[a].clone();
    ifo.slots[a] = ifo.slots[b].clone();
    ifo.slots[b] = tmp_record;

    // Exchange files.
    match (&data_a, &data_b) {
        (Some(bytes_a), Some(bytes_b)) => {
            exchange_files(&path_a, bytes_a, &path_b, bytes_b)?;
        }
        (Some(bytes_a), None) => {
            // Only a exists: move content to b, clear record a.
            move_file(&path_a, bytes_a, &path_b)?;
            ifo.slots[a] = SaveSlotInfo {
                reserved: [0; 12],
                month: 0,
                day: 0,
                hour: 0,
                minute: 0,
                flags: [0; 4],
            };
        }
        (None, Some(bytes_b)) => {
            // Only b exists: move content to a, clear record b.
            move_file(&path_b, bytes_b, &path_a)?;
            ifo.slots[b] = SaveSlotInfo {
                reserved: [0; 12],
                month: 0,
                day: 0,
                hour: 0,
                minute: 0,
                flags: [0; 4],
            };
        }
        (None, None) => {}
    }

    Extractor::save_file(&[ifo], &ifo_path(game_root))
}

/// Read a `.sav` fully; missing file → None, any other error propagates.
fn read_sav_if_exists(path: &Path) -> std::io::Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(data) => Ok(Some(data)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// Crosswise file exchange via temp files with best-effort rollback from the
/// in-memory buffers on failure.
fn exchange_files(
    path_a: &Path,
    bytes_a: &[u8],
    path_b: &Path,
    bytes_b: &[u8],
) -> std::io::Result<()> {
    let tmp_a = path_a.with_extension("sav.tmp");
    let tmp_b = path_b.with_extension("sav.tmp");

    std::fs::write(&tmp_a, bytes_a)?;
    if let Err(e) = std::fs::write(&tmp_b, bytes_b) {
        let _ = std::fs::remove_file(&tmp_a);
        return Err(e);
    }

    if let Err(e) = std::fs::rename(&tmp_a, path_b) {
        restore(path_a, bytes_a);
        restore(path_b, bytes_b);
        // Neither rename happened; both temps are still on disk.
        let _ = std::fs::remove_file(&tmp_a);
        let _ = std::fs::remove_file(&tmp_b);
        return Err(e);
    }
    if let Err(e) = std::fs::rename(&tmp_b, path_a) {
        restore(path_a, bytes_a);
        restore(path_b, bytes_b);
        // tmp_a was already renamed onto path_b; only tmp_b leaks here.
        let _ = std::fs::remove_file(&tmp_b);
        return Err(e);
    }
    Ok(())
}

/// Move `bytes` from `src` to `dst` via a temp file + rename, then remove the
/// source. Restores from the in-memory buffer on failure.
fn move_file(src: &Path, bytes: &[u8], dst: &Path) -> std::io::Result<()> {
    let tmp = dst.with_extension("sav.tmp");
    std::fs::write(&tmp, bytes)?;
    if let Err(e) = std::fs::rename(&tmp, dst) {
        restore(dst, bytes);
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    if let Err(e) = std::fs::remove_file(src) {
        restore(src, bytes);
        return Err(e);
    }
    Ok(())
}

/// Best-effort restore of a file's original contents. Never panics; logs to
/// stderr when the restore itself fails so data loss is at least visible.
fn restore(path: &Path, bytes: &[u8]) {
    if let Err(e) = std::fs::write(path, bytes) {
        eprintln!("save_ifo swap: failed to restore {}: {e}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::references::save_ifo::SaveIfo;
    use std::path::PathBuf;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "dispel_swap_test_{}_{}_{}",
                tag,
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Build synthetic Save.ifo bytes with per-slot timestamps.
    fn write_test_ifo(root: &Path, slots: [(u32, u32); SLOT_COUNT]) {
        let mut ifo = SaveIfo {
            slots: vec![SaveSlotInfo::default(); SLOT_COUNT],
            ..Default::default()
        };
        for (i, &(month, day)) in slots.iter().enumerate() {
            ifo.slots[i] = SaveSlotInfo {
                reserved: [0; 12],
                month,
                day,
                hour: i as u32,
                minute: 10 + i as u32,
                flags: [1, 0, 0, 0],
            };
        }
        let mut bytes = Vec::new();
        ifo.write_to(&mut bytes).unwrap();
        std::fs::write(ifo_path(root), bytes).unwrap();
    }

    /// Synthetic `.sav`: `[u32 blob_size=0][32-byte tail]` with distinct values.
    fn sav_bytes(key: u32, map: u32) -> Vec<u8> {
        let mut out = Vec::new();
        out.write_u32::<LittleEndian>(0).unwrap(); // blob size
        SavTail {
            game_version: 1.4,
            game_tmp_key: key,
            map_id: map,
            reserved: 0,
            payload_counts: [key, key * 2, 0, key * 3],
        }
        .write_to(&mut out)
        .unwrap();
        out
    }

    fn write_sav(root: &Path, index: usize, bytes: &[u8]) {
        std::fs::write(sav_path(root, index), bytes).unwrap();
    }

    fn read_tail(root: &Path, index: usize) -> SavTail {
        let mut file = std::fs::File::open(sav_path(root, index)).unwrap();
        SavTail::peek(&mut file).unwrap()
    }

    fn read_ifo_bytes(root: &Path) -> SaveIfo {
        read_ifo(root).unwrap()
    }

    #[test]
    fn swap_both_exist() {
        let dir = TempDir::new("both");
        let root = dir.path();
        write_test_ifo(root, [(2, 10), (3, 11), (4, 12), (5, 13), (6, 14), (7, 15)]);
        write_sav(root, 0, &sav_bytes(100, 1));
        write_sav(root, 2, &sav_bytes(200, 2));

        swap_slots(root, 0, 2).unwrap();

        let ifo = read_ifo_bytes(root);
        assert_eq!(ifo.slots[0].month, 4);
        assert_eq!(ifo.slots[0].day, 12);
        assert_eq!(ifo.slots[2].month, 2);
        assert_eq!(ifo.slots[2].day, 10);

        // File contents exchanged.
        let tail0 = read_tail(root, 0);
        assert_eq!(tail0.game_tmp_key, 200);
        assert_eq!(tail0.map_id, 2);
        let tail2 = read_tail(root, 2);
        assert_eq!(tail2.game_tmp_key, 100);
        assert_eq!(tail2.map_id, 1);

        // Global tail untouched.
        assert_eq!(ifo.game_version, 0.0);
        assert_eq!(ifo.game_tmp_key, 0);
        assert_eq!(ifo.payload_counts, [0; 4]);
    }

    #[test]
    fn move_single_save() {
        let dir = TempDir::new("single");
        let root = dir.path();
        write_test_ifo(root, [(2, 10), (3, 11), (4, 12), (5, 13), (6, 14), (7, 15)]);
        write_sav(root, 3, &sav_bytes(300, 9));

        swap_slots(root, 1, 3).unwrap();

        assert!(!sav_path(root, 3).exists());
        let tail1 = read_tail(root, 1);
        assert_eq!(tail1.game_tmp_key, 300);
        assert_eq!(tail1.map_id, 9);

        let ifo = read_ifo_bytes(root);
        // Record 3 (source of the moved content) cleared.
        assert!(!ifo.slots[3].is_occupied());
        assert_eq!(ifo.slots[3].month, 0);
        assert_eq!(ifo.slots[3].flags, [0; 4]);
        // Record 1 (content destination) got old record 3.
        assert_eq!(ifo.slots[1].month, 5);
        assert_eq!(ifo.slots[1].day, 13);
        assert!(ifo.slots[1].is_occupied());
    }

    #[test]
    fn swap_empty_slots() {
        let dir = TempDir::new("empty");
        let root = dir.path();
        write_test_ifo(root, [(2, 10), (3, 11), (4, 12), (5, 13), (6, 14), (7, 15)]);

        swap_slots(root, 4, 5).unwrap();

        let ifo = read_ifo_bytes(root);
        assert_eq!(ifo.slots[4].month, 7);
        assert_eq!(ifo.slots[5].month, 6);
        for i in 0..SLOT_COUNT {
            assert!(!sav_path(root, i).exists(), "no .sav should be created");
        }
    }

    #[test]
    fn invalid_indices_rejected() {
        let dir = TempDir::new("invalid");
        let root = dir.path();
        write_test_ifo(root, [(2, 10), (3, 11), (4, 12), (5, 13), (6, 14), (7, 15)]);
        let before = std::fs::read(ifo_path(root)).unwrap();

        assert!(swap_slots(root, 0, 6).is_err());
        assert!(swap_slots(root, 2, 2).is_err());

        let after = std::fs::read(ifo_path(root)).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn corrupt_sav_aborts_without_mutation() {
        let dir = TempDir::new("corrupt");
        let root = dir.path();
        write_test_ifo(root, [(2, 10), (3, 11), (4, 12), (5, 13), (6, 14), (7, 15)]);
        write_sav(root, 0, &[0u8; 10]); // truncated (< 36 bytes)
        write_sav(root, 2, &sav_bytes(200, 2));

        let ifo_before = std::fs::read(ifo_path(root)).unwrap();
        let sav2_before = std::fs::read(sav_path(root, 2)).unwrap();

        assert!(swap_slots(root, 0, 2).is_err());

        assert_eq!(std::fs::read(ifo_path(root)).unwrap(), ifo_before);
        assert_eq!(std::fs::read(sav_path(root, 2)).unwrap(), sav2_before);
    }

    #[test]
    fn summarize_reports_missing_sav() {
        let dir = TempDir::new("summary");
        let root = dir.path();
        write_test_ifo(root, [(2, 10), (3, 11), (4, 12), (5, 13), (6, 14), (7, 15)]);
        write_sav(root, 0, &sav_bytes(100, 1));
        write_sav(root, 2, &sav_bytes(200, 2));
        // Slots 1, 3, 4, 5 have no .sav.

        let summaries = summarize_slots(root).unwrap();
        assert_eq!(summaries.len(), SLOT_COUNT);

        let s0 = &summaries[0];
        assert!(s0.occupied && s0.sav_present);
        assert_eq!(s0.month, 2);
        assert_eq!(s0.game_tmp_key, Some(100));
        assert_eq!(s0.map_id, Some(1));

        let s2 = &summaries[2];
        assert!(s2.sav_present);
        assert_eq!(s2.game_tmp_key, Some(200));

        for &i in &[1usize, 3, 4, 5] {
            let s = &summaries[i];
            assert!(s.occupied);
            assert!(!s.sav_present);
            assert_eq!(s.game_tmp_key, None);
            assert_eq!(s.map_id, None);
        }
    }

    #[test]
    fn summarize_reports_corrupt_sav() {
        let dir = TempDir::new("corrupt_summary");
        let root = dir.path();
        write_test_ifo(root, [(2, 10), (3, 11), (4, 12), (5, 13), (6, 14), (7, 15)]);
        write_sav(root, 1, &[0u8; 10]); // truncated (< 36 bytes)
        write_sav(root, 2, &sav_bytes(200, 2));

        let summaries = summarize_slots(root).unwrap();

        // Corrupt slot: present but no readable tail values.
        let s1 = &summaries[1];
        assert!(s1.occupied);
        assert!(s1.sav_present);
        assert_eq!(s1.game_tmp_key, None);
        assert_eq!(s1.map_id, None);

        // Other slots unaffected.
        assert_eq!(summaries[2].game_tmp_key, Some(200));
        assert_eq!(summaries[2].map_id, Some(2));
        for &i in &[0usize, 3, 4, 5] {
            assert!(!summaries[i].sav_present);
            assert_eq!(summaries[i].game_tmp_key, None);
        }
    }

    #[test]
    fn summarize_real_fixture() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/Dispel");
        let summaries = summarize_slots(&root).unwrap();
        assert_eq!(summaries.len(), SLOT_COUNT);
        for s in &summaries {
            assert!(s.occupied);
        }
        for s in &summaries[0..3] {
            assert!(s.sav_present, "slot {} should have a .sav", s.index);
            assert!(s.game_tmp_key.is_some());
            assert!(s.map_id.is_some());
        }
        for s in &summaries[3..] {
            assert!(!s.sav_present, "slot {} should have no .sav", s.index);
            assert!(s.game_tmp_key.is_none());
        }
    }
}
