use std::io::{BufRead, BufReader, Write};
use std::{fs::File, path::Path};

use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use crate::references::references::Extractor;

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
// ITEM_ID ENCODED FORMAT:
// - 32-bit integer combining multiple fields
// - Structure: [item_id, group_id, 0, 0]
// - item_id: Specific object/item ID
// - group_id: Object group/category ID
// - Examples: 1001 = chest group 1, item 1
//
// SPECIAL VALUES:
// - Lines starting with ";" are comments
// - Parenthesized CSV format
// - Coordinates use isometric tile system
//
// FILE PURPOSE:
// Defines placement of interactive and decorative objects
// on specific maps with exact coordinates. Used for populating
// game worlds with environmental elements, quest objects, and
// interactive items.
//
// ===========================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct DrawItem {
    /// Target map for placement.
    pub map_id: i32,
    /// Tile X coordinate.
    pub x_coord: i32,
    /// Tile Y coordinate.
    pub y_coord: i32,
    /// Encoded Item ID (int32 combining [item ID, group ID, 0, 0]).
    pub item_id: i32,
}

/// Stores map placement data for drawn items/objects.
///
/// Reads file: `Ref/DRAWITEM.ref`
/// # File Format: `Ref/DRAWITEM.ref`
///
/// Text file, EUC-KR encoded. One record per line, parenthesised CSV format:
/// ```text
/// (map_id,x_coord,y_coord,item_id)
/// ```
/// - `item_id` is an encoded i32 combining `[item_id, group_id, 0, 0]` bytes.
impl Extractor for DrawItem {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let f = File::open(source_path)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(EUC_KR))
                .build(f),
        );
        let mut draw_items: Vec<DrawItem> = Vec::new();
        for line in reader.lines() {
            if let Ok(line) = line {
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
                let item_id = parts[3].parse::<i32>().unwrap();

                draw_items.push(DrawItem {
                    map_id,
                    x_coord,
                    y_coord,
                    item_id,
                });
            }
        }
        Ok(draw_items)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            let line = format!(
                "({},{},{},{})\r\n",
                record.map_id, record.x_coord, record.y_coord, record.item_id
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

pub fn save_draw_items(conn: &mut Connection, draw_items: &Vec<DrawItem>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_draw_item.sql"))?;
        for draw_item in draw_items {
            stmt.execute(params![
                draw_item.map_id,
                draw_item.x_coord,
                draw_item.y_coord,
                draw_item.item_id,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
