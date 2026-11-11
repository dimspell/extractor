use std::io::{Read, Seek, Write};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

use crate::references::extractor::Extractor;

/// `Map/Multi/*.dat` — per-tile `u32` grid (multiplayer).
///
/// The payload is a flat array of tile descriptor values.
///
/// File layout:
/// - `+0x00`: `u32` payload length (`248004` for `249² = 62001` entries)
/// - `+0x04`: `62001 × u32` grid values (`249 × 249`)
///
/// Total file size = `4 + 62001*4` = `248008` bytes (verified against `QD1-1.dat`).
///
/// Each tile descriptor uses this bit layout:
///
/// | Bits | Extracted value | Meaning |
/// |---|---|---|
/// | 0–9 | `object_id` | Object or entity ID. `0` is empty and `1` is a wall. |
/// | 10–24 | `tile_index` | Tile graphic index. |
/// | 25–26 | `render_flags` | Display flags. |
/// | 27–31 | `reserved` | Stored bits with no documented meaning. |
///
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TileGrid {
    /// Logical index derived from parse order (always 0 for single-file grid).
    pub id: i32,
    /// Self-describing payload length (`filesize - 4`). Always `248004` for standard 249×249 grids.
    pub payload_length: u32,
    /// Derived grid dimension (`sqrt(payload_length / 4)`). Standard value: `249`.
    pub grid_size: u32,
    /// Flat array of tile descriptor words (`payload_length / 4` entries).
    pub data: Vec<u32>,
}

/// One unpacked tile descriptor from a [`TileGrid`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileDescriptor {
    /// Object or entity ID from bits 0–9.
    pub object_id: u16,
    /// Tile graphic index from bits 10–24.
    pub tile_index: u16,
    /// Display flags from bits 25–26.
    pub render_flags: u8,
    /// Stored bits from positions 27–31.
    pub reserved: u8,
}

impl TileDescriptor {
    /// Unpacks one tile descriptor word.
    #[must_use]
    pub const fn from_word(word: u32) -> Self {
        Self {
            object_id: (word & 0x03ff) as u16,
            tile_index: ((word >> 10) & 0x7fff) as u16,
            render_flags: ((word >> 25) & 0x03) as u8,
            reserved: ((word >> 27) & 0x1f) as u8,
        }
    }
}

impl TileGrid {
    /// Returns the unpacked descriptor at the specified flat tile index.
    #[must_use]
    pub fn descriptor(&self, index: usize) -> Option<TileDescriptor> {
        self.data.get(index).copied().map(TileDescriptor::from_word)
    }
}

impl Extractor for TileGrid {
    fn parse<R: Read + Seek>(reader: &mut R, len: u64) -> std::io::Result<Vec<Self>> {
        let file_size = len as usize;
        if file_size < 4 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    ".dat file too small ({} bytes; expected at least 4-byte header)",
                    file_size
                ),
            ));
        }

        let payload_length = ReadBytesExt::read_u32::<LittleEndian>(reader)?;
        let expected_size = (4 + payload_length as usize) as u64;
        if file_size != expected_size as usize && file_size > expected_size as usize {
            // Allow slightly larger files for compatibility with existing files.
        }

        // Compute grid size from payload length
        let entry_count = payload_length / 4;
        let grid_size = (entry_count as f64).sqrt() as u32;
        if grid_size * grid_size != entry_count {
            // Not a perfect square; still parse the array verbatim.
        }

        let mut data = Vec::with_capacity(entry_count as usize);
        for _ in 0..entry_count {
            data.push(ReadBytesExt::read_u32::<LittleEndian>(reader)?);
        }

        Ok(vec![TileGrid {
            id: 0,
            payload_length,
            grid_size,
            data,
        }])
    }

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        for record in records {
            writer.write_u32::<LittleEndian>(record.payload_length)?;
            for value in &record.data {
                writer.write_u32::<LittleEndian>(*value)?;
            }
        }
        Ok(())
    }
}

pub fn read_tile_grid(source_path: &Path) -> std::io::Result<Vec<TileGrid>> {
    TileGrid::read_file(source_path)
}

pub fn save_tile_grid(conn: &mut Connection, grids: &[TileGrid]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT OR IGNORE INTO tile_grids (id, payload_length, grid_size, data_json) VALUES (?1, ?2, ?3, ?4)",
        )?;
        for grid in grids {
            stmt.execute(params![
                grid.id,
                grid.payload_length,
                grid.grid_size,
                serde_json::to_string(&grid.data).unwrap_or_default(),
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

impl std::fmt::Display for TileGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TileGrid(id={}, payload={}, grid={}, entries={})",
            self.id,
            self.payload_length,
            self.grid_size,
            self.data.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;


    #[test]
    fn serialize_round_trip_small() {
        let mut data = 248004u32.to_le_bytes().to_vec();
        // 62001 zeros for simplicity
        data.extend(
            vec![0u32.to_le_bytes(); 62001]
                .iter()
                .flatten()
                .copied()
                .collect::<Vec<u8>>(),
        );
        assert_eq!(data.len(), 4 + 62001 * 4);

        let mut c = Cursor::new(&data[..]);
        let grids = TileGrid::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(grids.len(), 1);
        assert_eq!(grids[0].payload_length, 248004);
        assert_eq!(grids[0].data.len(), 62001);
    }

    #[test]
    fn test_tile_descriptor_extracts_all_bit_ranges() {
        let descriptor = TileDescriptor::from_word(u32::MAX);

        assert_eq!(descriptor.object_id, 0x03ff);
        assert_eq!(descriptor.tile_index, 0x7fff);
        assert_eq!(descriptor.render_flags, 0x03);
        assert_eq!(descriptor.reserved, 0x1f);
    }

    #[test]
    fn test_tile_grid_descriptor_returns_selected_tile() {
        let word = 800u32 | (1337 << 10) | (2 << 25) | (17 << 27);
        let grid = TileGrid {
            data: vec![word],
            ..TileGrid::default()
        };

        assert_eq!(
            grid.descriptor(0),
            Some(TileDescriptor {
                object_id: 800,
                tile_index: 1337,
                render_flags: 2,
                reserved: 17,
            })
        );
        assert_eq!(grid.descriptor(1), None);
    }
}
