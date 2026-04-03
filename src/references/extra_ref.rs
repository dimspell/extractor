use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::{fs::File, path::Path};

use crate::references::enums::{ExtraObjectType, ItemTypeId, VisibilityType};
use crate::references::extractor::{read_mapper, read_null_terminated_windows_1250, Extractor};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::WINDOWS_1250;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

// ===========================================================================
// EXTRAREF.REF FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | ExtraRef.ref - Object Placements     |
// +--------------------------------------+
// | Encoding: Binary (Little-Endian)     |
// | Text Encoding: WINDOWS-1250          |
// | Header: 4-byte record count          |
// | Record Size: 184 bytes (46 × i32)      |
// +--------------------------------------+
// | [Header]                            |
// | - record_count: i32                  |
// +--------------------------------------+
// | [Record 1]                           |
// | - number_in_file: u8                 |
// | - padding: u8                       |
// | - ext_id: u8 (links to Extra.ini)    |
// | - name: 32 bytes (WINDOWS-1250)      |
// | - object_type: u8                   |
// | - x_pos: i32                        |
// | - y_pos: i32                        |
// | - rotation: u8                      |
// | - padding: 3 bytes + i32            |
// | - closed: i32 (0=open, 1=closed)     |
// | - required_item_id: u8               |
// | - required_item_type_id: u8          |
// | - padding: 2 bytes                  |
// | - required_item_id2: u8              |
// | - required_item_type_id2: u8         |
// | - padding: 2 bytes + 16 bytes        |
// | - gold_amount: i32                  |
// | - item_id: u8                       |
// | - item_type_id: u8                  |
// | - padding: 2 bytes                  |
// | - item_count: i32                   |
// | - padding: 40 bytes                 |
// | - event_id: i32                     |
// | - message_id: i32                   |
// | - padding: 32 bytes                 |
// | - visibility: u8                    |
// | - padding: 3 bytes + 8 bytes         |
// +--------------------------------------+
// | [Record 2]                           |
// | ... (same structure) ...             |
// +--------------------------------------+
//
// FILE PURPOSE:
// Defines interactive object placements with exact
// coordinates, requirements, contents, and behaviors.
// Used for populating maps with chests, doors, signs,
// and other interactive elements.
//
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtraRef {
    /// Specific object generation ID for map tracking.
    pub id: i32,
    /// Linear parsing index.
    pub number_in_file: u8,
    /// Unrecognized (always zero).
    pub unknown1: u8,
    /// Mapping ID linked backwards derived from Extra.ini.
    pub ext_id: u8,
    /// 32-byte label identifier.
    pub name: String,
    /// Object type (chest, door, sign, etc.).
    pub object_type: ExtraObjectType,
    /// Tile mapping horizontal target.
    pub x_pos: i32,
    /// Tile mapping vertical target.
    pub y_pos: i32,
    /// Facing perspective index.
    pub rotation: u8,
    /// Unrecognized (always [205, 205, 205])
    pub unknown2: Vec<u8>,
    /// Unrecognized (always zero)
    pub unknown3: i32,
    /// Interaction status for chests (0=open, 1=closed).
    pub closed: i32,
    /// Key identifier (lower bound) to interact.
    pub required_item_id: u8,
    /// Category ID of lower bound requirement.
    pub required_item_type_id: ItemTypeId,
    /// Unrecognized (always zero)
    pub unknown4: i16,
    /// Secondary requirement / Key upper bound.
    pub required_item_id2: u8,
    /// Category ID for upper bound.
    pub required_item_type_id2: ItemTypeId,
    /// Unrecognized (always zero)
    pub unknown5: i16,
    /// Unrecognized (0 or 9999)
    pub unknown6: i32,
    /// Unrecognized (0 or 9999)
    pub unknown7: i32,
    /// Unrecognized (0 or 9999)
    pub unknown8: i32,
    /// Unrecognized (0 or 9999)
    pub unknown9: i32,
    /// Quantity of gold inside container.
    pub gold_amount: i32,
    /// Found static loot ID.
    pub item_id: u8,
    /// Category enum for found loot.
    pub item_type_id: ItemTypeId,
    /// Unrecognized (always zero)
    pub unknown10: i16,
    /// Stacks contained within object.
    pub item_count: i32,
    /// Unrecognized (0, 28, 84, 258, 9999)
    pub unknown11: i32,
    /// Unrecognized (0 or 1)
    pub unknown12: i32,
    /// Unrecognized (0 or 9999)
    pub unknown13: i32,
    /// Unrecognized (always array of 28 zeros)
    pub unknown14: Vec<u8>,
    /// Bound logic ID executing upon interaction (from event.ini).
    pub event_id: i32,
    /// Pointer to signposts/plaques contained in Message.scr.
    pub message_id: i32,
    /// Unrecognized (0, 1, 2, 3)
    pub unknown15: i32,
    /// Unrecognized (0, 1, 2, 3)
    pub unknown16: i32,
    /// Unrecognized (always zero)
    pub unknown17: u8,
    /// Interactive element type (0, 1, 2, 3).
    pub interactive_element_type: u8,
    /// Unrecognized (always array [205, 205])
    pub unknown18: Vec<u8>,
    /// Unrecognized (0 or 1)
    pub is_quest_element: i32,
    /// Unrecognized (0 or 1)
    pub unknown20: i32,
    /// Unrecognized (0 or 1)
    pub unknown21: i32,
    /// Unrecognized (always zero)
    pub unknown22: i32,
    /// Unrecognized (0 or 1)
    pub unknown23: i32,
    /// Determines alpha transparency on render.
    pub visibility: VisibilityType,
    /// Unrecognized (0 or 1)
    pub unknown24: u8,
    /// Unrecognized (always zero)
    pub unknown25: i16,
    /// Unrecognized (0 or 1)
    pub unknown26: i32,
    /// Unrecognized (0 or 1)
    pub unknown27: i32,
}

/// Stores specific placements and configurations for interactive objects (chests, signs, doors) on a map.
///
/// Reads file: `ExtraInGame/Extdun01.ref (and other map-specific .ref files)`
/// # File Format: `ExtraInGame/Extdun01.ref` (and other map `.ref` files)
///
/// Binary file, little-endian.  Starts with a 4-byte i32 record count.
/// Each record is exactly `46 × 4 = 184` bytes:
/// - `number_in_file`       : u8
/// - 1 byte padding
/// - `ext_id`               : u8  (links to `Extra.ini`)
/// - `name`                 : 32 bytes, null-padded, WINDOWS-1250
/// - `object_type`          : u8  (7=magic, 6=interactive, 5=altar, 4=sign, 2=door, 0=chest)
/// - `x_pos`, `y_pos`       : i32
/// - `rotation`             : u8
/// - 3 bytes + i32 padding
/// - `closed`               : i32  (0=open, 1=closed)
/// - `required_item_id`     : u8  (lower key bound)
/// - `required_item_type_id`: u8
/// - 2 bytes padding
/// - `required_item_id2`    : u8  (upper key bound)
/// - `required_item_type_id2`: u8
/// - 2 bytes + 16 bytes padding
/// - `gold_amount`          : i32
/// - `item_id` / `item_type_id`: u8, u8
/// - 2 bytes padding
/// - `item_count`           : i32
/// - 40 bytes padding
/// - `event_id`             : i32  (from `Event.ini`)
/// - `message_id`           : i32  (from `Message.scr`)
/// - 32 bytes padding
/// - `visibility`           : u8
/// - 3 bytes + 8 bytes padding
impl Extractor for ExtraRef {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(source_path)?;

        let metadata = file.metadata()?;
        let file_len = metadata.len();

        let mut reader = BufReader::new(file);

        const COUNTER_SIZE: u8 = 4;
        const PROPERTY_ITEM_SIZE: i32 = 46 * 4;

        let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
        let mut refs: Vec<ExtraRef> = Vec::with_capacity(elements as usize);

        for i in 0..elements {
            let number_in_file = reader.read_u8()?;

            let unknown1 = reader.read_u8()?;
            let extra_ini_entry_id = reader.read_u8()?; // Id from Extra.ini

            let mut buffer = [0u8; 32];
            reader.read_exact(&mut buffer)?;
            let name = read_null_terminated_windows_1250(&buffer).unwrap();

            let object_type_raw = reader.read_u8()?;
            let object_type =
                ExtraObjectType::from_u8(object_type_raw).unwrap_or(ExtraObjectType::Unknown);

            let x_pos = reader.read_i32::<LittleEndian>()?;
            let y_pos = reader.read_i32::<LittleEndian>()?;
            let rotation = reader.read_u8()?;

            let mut unknown2 = vec![0u8; 3];
            reader.read_exact(&mut unknown2)?;

            let unknown3 = reader.read_i32::<LittleEndian>()?;

            let closed = reader.read_i32::<LittleEndian>()?; // chest 0-open, 1-closed

            let required_item_id = reader.read_u8()?; // lower bound
            let required_item_type_id_raw = reader.read_u8()?;
            let unknown4 = reader.read_i16::<LittleEndian>()?;

            let required_item_id2 = reader.read_u8()?;
            let required_item_type_id2_raw = reader.read_u8()?;
            let unknown5 = reader.read_i16::<LittleEndian>()?;

            let unknown6 = reader.read_i32::<LittleEndian>()?;
            let unknown7 = reader.read_i32::<LittleEndian>()?;
            let unknown8 = reader.read_i32::<LittleEndian>()?;
            let unknown9 = reader.read_i32::<LittleEndian>()?;

            let gold_amount = reader.read_i32::<LittleEndian>()?;

            let item_id = reader.read_u8()?;
            let item_type_id_raw = reader.read_u8()?;
            let unknown10 = reader.read_i16::<LittleEndian>()?;

            let item_count = reader.read_i32::<LittleEndian>()?;

            let unknown11 = reader.read_i32::<LittleEndian>()?;
            let unknown12 = reader.read_i32::<LittleEndian>()?;
            let unknown13 = reader.read_i32::<LittleEndian>()?;

            let mut unknown14 = vec![0u8; 28];
            reader.read_exact(&mut unknown14)?;

            let event_id = reader.read_i32::<LittleEndian>()?; // id from event.ini
            let message_id = reader.read_i32::<LittleEndian>()?; // id from message.scr for signs

            let unknown15 = reader.read_i32::<LittleEndian>()?;
            let unknown16 = reader.read_i32::<LittleEndian>()?;
            let unknown17 = reader.read_u8()?;

            // 0 = pillars in Gods garden
            // 3 = Vera altar
            // otherwise = 1
            let interactive_element_type = reader.read_u8()?;

            let mut unknown18 = vec![0u8; 2];
            reader.read_exact(&mut unknown18)?;

            // Door, Vera altar, resurrection altar = 1, otherwise = 0
            let is_quest_element = reader.read_i32::<LittleEndian>()?;

            let unknown20 = reader.read_i32::<LittleEndian>()?;
            let unknown21 = reader.read_i32::<LittleEndian>()?;
            let unknown22 = reader.read_i32::<LittleEndian>()?;
            let unknown23 = reader.read_i32::<LittleEndian>()?;

            let visibility_raw = reader.read_u8()?;
            let visibility =
                VisibilityType::from_u8(visibility_raw).unwrap_or(VisibilityType::Unknown);

            let unknown24 = reader.read_u8()?;
            let unknown25 = reader.read_i16::<LittleEndian>()?;

            // last 8 bytes to reach 184 bytes total
            let unknown26 = reader.read_i32::<LittleEndian>()?;
            let unknown27 = reader.read_i32::<LittleEndian>()?;

            let required_item_type_id =
                ItemTypeId::from_u8(required_item_type_id_raw).unwrap_or(ItemTypeId::Weapon);
            let required_item_type_id2 =
                ItemTypeId::from_u8(required_item_type_id2_raw).unwrap_or(ItemTypeId::Weapon);
            let item_type_id = ItemTypeId::from_u8(item_type_id_raw).unwrap_or(ItemTypeId::Weapon);

            refs.push(ExtraRef {
                id: i,
                number_in_file,
                ext_id: extra_ini_entry_id,
                name: name.to_string(),
                object_type,
                x_pos,
                y_pos,
                rotation,
                closed,
                required_item_id,
                required_item_type_id,
                required_item_id2,
                required_item_type_id2,
                gold_amount,
                item_id,
                item_type_id,
                item_count,
                event_id,
                message_id,
                is_quest_element,
                interactive_element_type,
                visibility,
                unknown1,
                unknown2,
                unknown3,
                unknown4,
                unknown5,
                unknown6,
                unknown7,
                unknown8,
                unknown9,
                unknown10,
                unknown11,
                unknown12,
                unknown13,
                unknown14,
                unknown15,
                unknown16,
                unknown17,
                unknown18,
                unknown20,
                unknown21,
                unknown22,
                unknown23,
                unknown24,
                unknown25,
                unknown26,
                unknown27,
            })
        }

        Ok(refs)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let file = File::create(dest_path)?;
        let mut writer = BufWriter::new(file);

        let elements = records.len() as i32;
        writer.write_i32::<LittleEndian>(elements)?;

        for record in records {
            writer.write_u8(record.number_in_file)?;
            writer.write_u8(record.unknown1)?;
            writer.write_u8(record.ext_id)?;

            let mut name_buf = [0u8; 32];
            let (cow, _, _) = WINDOWS_1250.encode(&record.name);
            let len = std::cmp::min(cow.len(), 32);
            name_buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&name_buf)?;

            writer.write_u8(u8::from(record.object_type))?;
            writer.write_i32::<LittleEndian>(record.x_pos)?;
            writer.write_i32::<LittleEndian>(record.y_pos)?;
            writer.write_u8(record.rotation)?;

            writer.write_all(&record.unknown2)?;
            writer.write_i32::<LittleEndian>(record.unknown3)?;

            writer.write_i32::<LittleEndian>(record.closed)?;

            writer.write_u8(record.required_item_id)?;
            writer.write_u8(u8::from(record.required_item_type_id))?;
            writer.write_i16::<LittleEndian>(record.unknown4)?;

            writer.write_u8(record.required_item_id2)?;
            writer.write_u8(u8::from(record.required_item_type_id2))?;
            writer.write_i16::<LittleEndian>(record.unknown5)?;

            writer.write_i32::<LittleEndian>(record.unknown6)?;
            writer.write_i32::<LittleEndian>(record.unknown7)?;
            writer.write_i32::<LittleEndian>(record.unknown8)?;
            writer.write_i32::<LittleEndian>(record.unknown9)?;

            writer.write_i32::<LittleEndian>(record.gold_amount)?;

            writer.write_u8(record.item_id)?;
            writer.write_u8(u8::from(record.item_type_id))?;
            writer.write_i16::<LittleEndian>(record.unknown10)?;

            writer.write_i32::<LittleEndian>(record.item_count)?;

            writer.write_i32::<LittleEndian>(record.unknown11)?;
            writer.write_i32::<LittleEndian>(record.unknown12)?;
            writer.write_i32::<LittleEndian>(record.unknown13)?;

            writer.write_all(&record.unknown14)?;

            writer.write_i32::<LittleEndian>(record.event_id)?;
            writer.write_i32::<LittleEndian>(record.message_id)?;

            writer.write_i32::<LittleEndian>(record.unknown15)?;
            writer.write_i32::<LittleEndian>(record.unknown16)?;
            writer.write_u8(record.unknown17)?;

            writer.write_u8(record.interactive_element_type)?;
            writer.write_all(&record.unknown18)?;

            writer.write_i32::<LittleEndian>(record.is_quest_element)?;

            writer.write_i32::<LittleEndian>(record.unknown20)?;
            writer.write_i32::<LittleEndian>(record.unknown21)?;
            writer.write_i32::<LittleEndian>(record.unknown22)?;
            writer.write_i32::<LittleEndian>(record.unknown23)?;

            writer.write_u8(u8::from(record.visibility))?;
            writer.write_u8(record.unknown24)?;
            writer.write_i16::<LittleEndian>(record.unknown25)?;

            writer.write_i32::<LittleEndian>(record.unknown26)?;
            writer.write_i32::<LittleEndian>(record.unknown27)?;
        }
        Ok(())
    }
}

pub fn read_extra_ref(source_path: &Path) -> std::io::Result<Vec<ExtraRef>> {
    ExtraRef::read_file(source_path)
}

pub fn save_extra_refs(
    conn: &mut Connection,
    file_path: &str,
    extra_refs: &[ExtraRef],
) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_extra_ref.sql"))?;
        for extra_ref in extra_refs {
            stmt.execute(params![
                file_path,                                  // 1
                extra_ref.id,                               // 2
                extra_ref.number_in_file,                   // 3
                extra_ref.unknown1,                         // 4
                extra_ref.ext_id,                           // 5
                extra_ref.name,                             // 6
                u8::from(extra_ref.object_type),            // 7
                extra_ref.x_pos,                            // 8
                extra_ref.y_pos,                            // 9
                extra_ref.rotation,                         // 10
                extra_ref.unknown2,                         // 11
                extra_ref.unknown3,                         // 12
                extra_ref.closed,                           // 13
                extra_ref.required_item_id,                 // 14
                u8::from(extra_ref.required_item_type_id),  // 15
                extra_ref.unknown4,                         // 16
                extra_ref.required_item_id2,                // 17
                u8::from(extra_ref.required_item_type_id2), // 18
                extra_ref.unknown5,                         // 19
                extra_ref.unknown6,                         // 20
                extra_ref.unknown7,                         // 21
                extra_ref.unknown8,                         // 22
                extra_ref.unknown9,                         // 23
                extra_ref.gold_amount,                      // 24
                extra_ref.item_id,                          // 25
                u8::from(extra_ref.item_type_id),           // 26
                extra_ref.unknown10,                        // 27
                extra_ref.item_count,                       // 28
                extra_ref.unknown11,                        // 29
                extra_ref.unknown12,                        // 30
                extra_ref.unknown13,                        // 31
                extra_ref.unknown14,                        // 32
                extra_ref.event_id,                         // 33
                extra_ref.message_id,                       // 34
                extra_ref.unknown15,                        // 35
                extra_ref.unknown16,                        // 36
                extra_ref.unknown17,                        // 37
                extra_ref.interactive_element_type,         // 38
                extra_ref.unknown18,                        // 39
                extra_ref.is_quest_element,                 // 40
                extra_ref.unknown20,                        // 41
                extra_ref.unknown21,                        // 42
                extra_ref.unknown22,                        // 43
                extra_ref.unknown23,                        // 44
                u8::from(extra_ref.visibility),             // 45
                extra_ref.unknown24,                        // 46
                extra_ref.unknown25,                        // 47
                extra_ref.unknown26,                        // 48
                extra_ref.unknown27,                        // 49
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
