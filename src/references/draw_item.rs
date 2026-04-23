use std::io::{BufRead, BufReader, Write};
use std::{fs::File, path::Path};

use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use crate::references::enums::ItemTypeId;
use crate::references::extractor::Extractor;

// ===========================================================================
// DRAWITEM.REF FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | DRAWITEM.ref - Map Object Placements |
// +--------------------------------------+
// | Encoding: EUC-KR                     |
// | Format: Parenthesized CSV            |
// | Record Size: Variable (text)         |
// +--------------------------------------+
// | ; Comment line                       |
// | (map_id,x_coord,y_coord,item_id)     |
// | (1,5,10,1001)                        |
// | (1,6,11,1002)                        |
// | ...                                  |
// +--------------------------------------+
//
// FIELD DEFINITIONS:
// - map_id: Target map identifier
// - x_coord: Tile X coordinate (isometric)
// - y_coord: Tile Y coordinate (isometric)
// - item_id: Encoded item/object identifier
//
// SPECIAL VALUES:
// - Lines starting with ";" are comments
// - Parenthesized CSV format
// - Coordinates use isometric tile system
//
//
// STORAGE FORMATS:
// - File format: Encoded i32 (for compatibility with game files)
// - Memory (DrawItem struct): Separate fields (item_id: u8, item_type: ItemTypeId)
// - Database: Separate columns (item_id: INTEGER, item_type: INTEGER)
//
// ===========================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DrawItem {
    /// Target map for placement (a reference to the AllMap.ini).
    pub map_id: i32,
    /// Tile X coordinate.
    pub x_coord: i32,
    /// Tile Y coordinate.
    pub y_coord: i32,
    /// Object type/category.
    pub item_type: ItemTypeId,
    /// Specific object/item ID (0-255).
    pub item_id: u8,
}

/// Stores map placement data for drawn items/objects.
///
/// The struct uses decoded form with separate item_type and item_id fields,
/// while file I/O maintains compatibility with the encoded i32 format.
///
/// Reads file: `Ref/DRAWITEM.ref`
/// # File Format: `Ref/DRAWITEM.ref`
///
/// Text file, EUC-KR encoded. One record per line, parenthesised CSV format:
/// ```text
/// (map_id,x_coord,y_coord,item_id)
/// ```
/// - `item_id` is an encoded i32 combining `[item_id, item_type, 0, 0]` bytes.
/// - In memory: stored as separate `item_id: u8` and `item_type: ItemTypeId` fields
/// - In database: stored as separate `item_id: INTEGER` and `item_type: INTEGER` columns
impl Extractor for DrawItem {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let f = File::open(source_path)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(EUC_KR))
                .build(f),
        );
        let mut draw_items: Vec<DrawItem> = Vec::new();
        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with(";") {
                continue;
            }

            let parts: Vec<&str> = line
                .trim_start_matches("(")
                .trim_end_matches(")")
                .split(",")
                .collect();
            if parts.len() < 4 {
                continue;
            }

            let map_id = parts[0].parse::<i32>().unwrap();
            let x_coord = parts[1].parse::<i32>().unwrap();
            let y_coord = parts[2].parse::<i32>().unwrap();
            let encoded_item_id = parts[3].parse::<i32>().unwrap();
            let encoded_item_id: [u8; 4] = encoded_item_id.to_le_bytes();

            let item_type = ItemTypeId::from_u8(encoded_item_id[1]).unwrap_or(ItemTypeId::Other);
            let item_id = encoded_item_id[0];

            draw_items.push(DrawItem {
                map_id,
                x_coord,
                y_coord,
                item_type,
                item_id,
            });
        }
        Ok(draw_items)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            // Reconstruct the encoded item_id from item_type and item_id
            let item_type_byte: u8 = record.item_type.into();
            let encoded_item_id = i32::from_le_bytes([record.item_id, item_type_byte, 0, 0]);

            let line = format!(
                "({},{},{},{})\r\n",
                record.map_id, record.x_coord, record.y_coord, encoded_item_id
            );
            let (cow, _, _) = EUC_KR.encode(&line);
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_draw_items(source_path: &Path) -> std::io::Result<Vec<DrawItem>> {
    DrawItem::read_file(source_path)
}

pub fn save_draw_items(conn: &mut Connection, draw_items: &[DrawItem]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_draw_item.sql"))?;
        for draw_item in draw_items {
            // Store decoded form: item_id and item_type separately
            let item_type_value: u8 = draw_item.item_type.into();

            stmt.execute(params![
                draw_item.map_id,
                draw_item.x_coord,
                draw_item.y_coord,
                draw_item.item_id as i32,
                item_type_value as i32,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
