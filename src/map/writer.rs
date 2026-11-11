use std::fs;
use std::io;
use std::path::Path;

use super::MapData;
use super::types::Coords;

/// Write map data back to a .map file by patching only the 3 end grids
/// (event grid, tile & access grid, access-ref grid) in-place. All
/// header/sprite/object bytes are preserved unchanged — including the typed
/// tiled-bundle tree (`MapData::tiled_bundles`), which therefore survives a
/// write → re-parse cycle byte-identically.
pub fn write_map_to_path(path: &Path, data: &MapData) -> io::Result<()> {
    let w = data.model.tiled_map_width;
    let h = data.model.tiled_map_height;
    let block_size = (w * h * 4) as u64;

    // Read entire file
    let mut bytes = fs::read(path)?;
    let file_len = bytes.len() as u64;
    let needed = block_size * 3;
    if file_len < needed {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "File too small: {} bytes, need {} for {w}×{h} end blocks",
                file_len, needed
            ),
        ));
    }

    // Event block offset (from start)
    let events_offset = file_len - block_size * 3;
    for y in 0..h {
        for x in 0..w {
            let coords: Coords = (x, y);
            let base = (events_offset + (y as u64 * w as u64 + x as u64) * 4) as usize;

            let word = data.events.get(&coords).map(|e| e.word).unwrap_or(0);
            bytes[base..base + 4].copy_from_slice(&word.to_le_bytes());
        }
    }

    // Tile & access block offset
    let tiles_offset = file_len - block_size * 2;
    for y in 0..h {
        for x in 0..w {
            let coords: Coords = (x, y);
            let base = (tiles_offset + (y as u64 * w as u64 + x as u64) * 4) as usize;

            let gtl_id = data.gtl_tiles.get(&coords).copied().unwrap_or(0);
            let collision = data.collisions.get(&coords).copied().unwrap_or(false);
            let object_id = data.object_ids.get(&coords).copied().unwrap_or(0);
            let packed =
                ((gtl_id & 0x7FFF) << 10) | ((object_id & 0x1FF) << 1) | (collision as i32);
            bytes[base..base + 4].copy_from_slice(&(packed as i32).to_le_bytes());
        }
    }

    // Access-ref ("roof") block offset — write the raw packed words back
    // (preserving shadow levels and light flags), patching only the overlay
    // ref bits from any btl_tiles edits.
    let access_ref_offset = file_len - block_size;
    for y in 0..h {
        for x in 0..w {
            let coords: Coords = (x, y);
            let base = (access_ref_offset + (y as u64 * w as u64 + x as u64) * 4) as usize;

            let mut word = data.access_ref_words.get(&coords).copied().unwrap_or(0);
            if let Some(&id) = data.btl_tiles.get(&coords) {
                word = (word & !0x7FFF) | ((id as u32) & 0x7FFF);
            }
            bytes[base..base + 4].copy_from_slice(&word.to_le_bytes());
        }
    }

    fs::write(path, &bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::BufReader;

    use crate::map::read_map_data;

    /// Create a minimal synthetic .map file, write + read back, verify 3 blocks match.
    #[test]
    fn round_trip_3x3_map() {
        let fixture = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/Dispel/Map/cat1.map"
        ));
        if !fixture.exists() {
            eprintln!("Skipping round_trip test: fixture not found");
            return;
        }

        // Read original
        let file = fs::File::open(fixture).unwrap();
        let mut reader = BufReader::new(file);
        let original = read_map_data(&mut reader).unwrap();
        let w = original.model.tiled_map_width;
        let h = original.model.tiled_map_height;

        // Write to temp
        let tmp = std::env::temp_dir().join("test_round_trip_cat1.map");
        fs::copy(fixture, &tmp).unwrap();
        write_map_to_path(&tmp, &original).unwrap();

        // Re-read
        let file = fs::File::open(&tmp).unwrap();
        let mut reader = BufReader::new(file);
        let reloaded = read_map_data(&mut reader).unwrap();

        // Compare the 3 end-block fields
        assert_eq!(original.gtl_tiles, reloaded.gtl_tiles, "GTL tiles differ");
        assert_eq!(
            original.collisions, reloaded.collisions,
            "Collisions differ"
        );
        assert_eq!(original.btl_tiles, reloaded.btl_tiles, "BTL tiles differ");
        assert_eq!(
            original.access_ref_words, reloaded.access_ref_words,
            "Access-ref words (shadow/light bits) differ"
        );
        assert_eq!(
            original.object_ids, reloaded.object_ids,
            "Object IDs differ"
        );
        for y in 0..h {
            for x in 0..w {
                let o_ev = original.events.get(&(x, y));
                let r_ev = reloaded.events.get(&(x, y));
                match (o_ev, r_ev) {
                    (Some(a), Some(b)) => {
                        assert_eq!(a.word, b.word, "event word mismatch at ({x},{y})");
                    }
                    (None, None) => {}
                    _ => panic!("Event presence differs at ({x},{y})"),
                }
            }
        }

        // Check we can toggle a collision
        let mut modified = original;
        modified.collisions.insert(
            (5, 5),
            !modified.collisions.get(&(5, 5)).copied().unwrap_or(false),
        );
        write_map_to_path(&tmp, &modified).unwrap();
        let file = fs::File::open(&tmp).unwrap();
        let mut reader = BufReader::new(file);
        let reloaded2 = read_map_data(&mut reader).unwrap();
        assert_eq!(
            modified.collisions, reloaded2.collisions,
            "Modified collision not reflected"
        );

        // Check we can change an event_id
        let mut modified2 = reloaded2;
        if let Some(ev) = modified2.events.get_mut(&(5, 5)) {
            ev.set_event_id(42);
        }
        write_map_to_path(&tmp, &modified2).unwrap();
        let file = fs::File::open(&tmp).unwrap();
        let mut reader = BufReader::new(file);
        let reloaded3 = read_map_data(&mut reader).unwrap();
        assert_eq!(
            reloaded3.events.get(&(5, 5)).map(|e| e.event_id()),
            Some(42),
            "Modified event_id not reflected"
        );

        // Cleanup
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn too_small_file_errors() {
        let tmp = std::env::temp_dir().join("test_too_small.map");
        fs::write(&tmp, b"toosmall").unwrap();

        // Create a dummy MapData with large dimensions that won't fit
        use crate::map::model::MapModel;
        let data = MapData {
            model: MapModel {
                tiled_map_width: 100,
                tiled_map_height: 100,
                ..Default::default()
            },
            gtl_tiles: HashMap::new(),
            btl_tiles: HashMap::new(),
            overlay_modes: Vec::new(),
            access_ref_words: HashMap::new(),
            collisions: HashMap::new(),
            events: HashMap::new(),
            object_ids: HashMap::new(),
            tiled_infos: Vec::new(),
            internal_sprites: Vec::new(),
            sprite_blocks: Vec::new(),
            internal_sprite_stamps: Vec::new(),
            tiled_object_refs: Vec::new(),
            tiled_bundles: Vec::new(),
        };

        let result = write_map_to_path(&tmp, &data);
        assert!(result.is_err(), "Should error on too-small file");
        let _ = fs::remove_file(&tmp);
    }
}
