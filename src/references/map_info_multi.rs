use std::io::{Read, Seek, Write};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

use crate::references::extractor::Extractor;

/// NPC spawn entry within the `.ifm` file.
///
/// Each entry has 12 bytes: `{u32 x, u32 y, u8 kind}` and three padding bytes.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct IfmNpcSpawn {
    /// Tile X coordinate.
    pub x: u32,
    /// Tile Y coordinate.
    pub y: u32,
    /// NPC kind or grid ID.
    pub kind: u8,
}

/// `Map/Multi/*.ifm` — map info / linkage struct (multiplayer).
///
/// The file contains one 100-byte record.
///
/// Layout (file offsets 0x00–0x63):
///
/// | Offset | Type | Field |
/// |---|---|---|
/// | +0x00 | `char[8]` | Self map name, NUL-padded |
/// | +0x08 | `u32` | Anchor tile index (=149 conventionally) |
/// | +0x0C | `u32` | Stride |
/// | +0x10 | `u32` | Row-step offset (=248) |
/// | +0x14 | `u32` | Start-pos A X |
/// | +0x18 | `u32` | Start-pos A Y |
/// | +0x1C | `u32` | Start-pos B X |
/// | +0x20 | `u32` | Start-pos B Y |
/// | +0x24 | `char[6]` + 2 B pad | Transition target name |
/// | +0x2C | `char[6]` + 2 B pad | `.mon` file name (self name in practice) |
/// | +0x34 | 4 × (`u32 x`, `u32 y`, `u8 kind`, 3 B pad) | NPC spawn table |
///
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MapInfoMulti {
    /// Zero-based logical index derived from parse order (not on disk).
    pub id: i32,
    /// Self map name (e.g. `"QS1"`, `"QD1-1"`), NUL-padded to 8 bytes.
    pub self_name: String,
    /// Anchor tile index.
    pub anchor_tile_index: u32,
    /// Map stride value.
    pub stride: u32,
    /// Row-step value.
    pub row_step: u32,
    /// First start position pair.
    pub start_pos_a_x: u32,
    pub start_pos_a_y: u32,
    /// Second start position pair.
    pub start_pos_b_x: u32,
    pub start_pos_b_y: u32,
    /// Transition target map name.
    pub transition_target: String,
    /// Monster spawn file name.
    pub mon_filename: String,
    /// NPC spawn table with four fixed entries.
    pub npc_spawns: Vec<IfmNpcSpawn>,
}

impl Extractor for MapInfoMulti {
    fn parse<R: Read + Seek>(reader: &mut R, len: u64) -> std::io::Result<Vec<Self>> {
        // .ifm files have no record-count header; the file size IS the record size
        // (100 bytes). We read exactly one record per file.
        let expected = len as usize;
        if expected != 100 {
            // Allow slightly larger reads for compatibility with existing files.
            if expected > 104 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        ".ifm file size {} does not match expected ~100 bytes",
                        expected
                    ),
                ));
            }
        }

        let mut items = Vec::with_capacity(1);
        for i in 0..1 {
            let mut self_name_buf = [0u8; 8];
            reader.read_exact(&mut self_name_buf)?;
            let name_len = self_name_buf.iter().position(|&b| b == 0).unwrap_or(8);
            let self_name = String::from_utf8_lossy(&self_name_buf[..name_len]).to_string();

            let anchor_tile_index = reader.read_u32::<LittleEndian>()?;
            let stride = reader.read_u32::<LittleEndian>()?;
            let row_step = reader.read_u32::<LittleEndian>()?;
            let start_pos_a_x = reader.read_u32::<LittleEndian>()?;
            let start_pos_a_y = reader.read_u32::<LittleEndian>()?;
            let start_pos_b_x = reader.read_u32::<LittleEndian>()?;
            let start_pos_b_y = reader.read_u32::<LittleEndian>()?;

            let mut transition_buf = [0u8; 6];
            reader.read_exact(&mut transition_buf)?;
            let trans_len = transition_buf.iter().position(|&b| b == 0).unwrap_or(6);
            let transition_target =
                String::from_utf8_lossy(&transition_buf[..trans_len]).to_string();
            let mut trans_pad = [0u8; 2];
            reader.read_exact(&mut trans_pad)?;

            let mut mon_buf = [0u8; 6];
            reader.read_exact(&mut mon_buf)?;
            let mon_len = mon_buf.iter().position(|&b| b == 0).unwrap_or(6);
            let mon_filename = String::from_utf8_lossy(&mon_buf[..mon_len]).to_string();
            let mut mon_pad = [0u8; 2];
            reader.read_exact(&mut mon_pad)?;

            // NPC spawn table: 4 entries × 12 bytes = 48 bytes (0x34..0x63)
            let mut npc_spawns = Vec::with_capacity(4);
            for _ in 0..4 {
                let x = reader.read_u32::<LittleEndian>()?;
                let y = reader.read_u32::<LittleEndian>()?;
                let kind = reader.read_u8()?;
                // 3 bytes padding to reach stride 0xC (12)
                let mut pad = [0u8; 3];
                reader.read_exact(&mut pad)?;
                npc_spawns.push(IfmNpcSpawn { x, y, kind });
            }

            items.push(MapInfoMulti {
                id: i,
                self_name,
                anchor_tile_index,
                stride,
                row_step,
                start_pos_a_x,
                start_pos_a_y,
                start_pos_b_x,
                start_pos_b_y,
                transition_target,
                mon_filename,
                npc_spawns,
            });
        }

        Ok(items)
    }

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        for record in records {
            // Self name (8 bytes, NUL-padded)
            let mut name_buf = [0u8; 8];
            let name_bytes = record.self_name.as_bytes();
            let n = std::cmp::min(name_bytes.len(), 8);
            name_buf[..n].copy_from_slice(&name_bytes[..n]);
            writer.write_all(&name_buf)?;

            writer.write_u32::<LittleEndian>(record.anchor_tile_index)?;
            writer.write_u32::<LittleEndian>(record.stride)?;
            writer.write_u32::<LittleEndian>(record.row_step)?;
            writer.write_u32::<LittleEndian>(record.start_pos_a_x)?;
            writer.write_u32::<LittleEndian>(record.start_pos_a_y)?;
            writer.write_u32::<LittleEndian>(record.start_pos_b_x)?;
            writer.write_u32::<LittleEndian>(record.start_pos_b_y)?;

            // Transition target (6 bytes + 2 pad)
            let mut trans_buf = [0u8; 6];
            let tb = record.transition_target.as_bytes();
            let n = std::cmp::min(tb.len(), 6);
            trans_buf[..n].copy_from_slice(&tb[..n]);
            writer.write_all(&trans_buf)?;
            writer.write_all(&[0u8; 2])?;

            // .mon filename (6 bytes + 2 pad)
            let mut mon_buf = [0u8; 6];
            let mb = record.mon_filename.as_bytes();
            let n = std::cmp::min(mb.len(), 6);
            mon_buf[..n].copy_from_slice(&mb[..n]);
            writer.write_all(&mon_buf)?;
            writer.write_all(&[0u8; 2])?;

            // NPC spawn table (4 entries × 12 bytes)
            for spawn in &record.npc_spawns {
                writer.write_u32::<LittleEndian>(spawn.x)?;
                writer.write_u32::<LittleEndian>(spawn.y)?;
                writer.write_u8(spawn.kind)?;
                writer.write_all(&[0u8; 3])?; // padding
            }
        }
        Ok(())
    }
}

pub fn read_map_info_multi(source_path: &Path) -> std::io::Result<Vec<MapInfoMulti>> {
    MapInfoMulti::read_file(source_path)
}

pub fn save_map_info_multi(conn: &mut Connection, info: &[MapInfoMulti]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT OR IGNORE INTO map_info_multi (id, self_name, anchor_tile_index, stride, row_step, start_pos_a_x, start_pos_a_y, start_pos_b_x, start_pos_b_y, transition_target, mon_filename, npc_spawns_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )?;
        for item in info {
            stmt.execute(params![
                item.id,
                item.self_name,
                item.anchor_tile_index,
                item.stride,
                item.row_step,
                item.start_pos_a_x,
                item.start_pos_a_y,
                item.start_pos_b_x,
                item.start_pos_b_y,
                item.transition_target,
                item.mon_filename,
                serde_json::to_string(&item.npc_spawns).unwrap_or_default(),
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

impl std::fmt::Display for MapInfoMulti {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MapInfoMulti(name={}, anchor={}, stride={}, row_step={}, transition={}, mon={}, npcs={:?})",
            self.self_name,
            self.anchor_tile_index,
            self.stride,
            self.row_step,
            self.transition_target,
            self.mon_filename,
            self.npc_spawns.len()
        )
    }
}
