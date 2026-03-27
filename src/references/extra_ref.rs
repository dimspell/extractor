use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::{fs::File, path::Path};

use crate::references::references::{read_mapper, read_null_terminated_windows_1250, Extractor};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::WINDOWS_1250;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ExtraRef {
    /// Specific object generation ID for map tracking.
    pub id: i32,
    /// Linear parsing index.
    pub number_in_file: u8,
    /// Mapping ID linked backwards derived from Extra.ini.
    pub ext_id: u8,
    /// 32-byte label identifier.
    pub name: String,
    /// Enum control (7=magic, 6=interactive, 5=altar, 4=sign, 2=door, 0=chest).
    pub object_type: u8,
    /// Tile mapping horizontal target.
    pub x_pos: i32,
    /// Tile mapping vertical target.
    pub y_pos: i32,
    /// Facing perspective index.
    pub rotation: u8,
    /// Interaction status for chests (0=open, 1=closed).
    pub closed: i32,
    /// Key identifier (lower bound) to interact.
    pub required_item_id: u8,
    /// Category ID of lower bound requirement.
    pub required_item_type_id: u8,
    /// Secondary requirement / Key upper bound.
    pub required_item_id2: u8,
    /// Category ID for upper bound.
    pub required_item_type_id2: u8,
    /// Quantity of gold inside container.
    pub gold_amount: i32,
    /// Found static loot ID.
    pub item_id: u8,
    /// Category enum for found loot.
    pub item_type_id: u8,
    /// Stacks contained within object.
    pub item_count: i32,
    /// Bound logic ID executing upon interaction (from event.ini).
    pub event_id: i32,
    /// Pointer to signposts/plaques contained in Message.scr.
    pub message_id: i32,
    /// Determines alpha transparency on render.
    pub visibility: u8,

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

            reader.read_u8()?;
            let ext_id = reader.read_u8()?; // Id from Extra.ini

            let mut buffer = [0u8; 32];
            reader.read_exact(&mut buffer)?;
            let name = read_null_terminated_windows_1250(&buffer).unwrap();

            let object_type = reader.read_u8()?; // 7-magic, 6-interactive object, 5-altar, 4-sign, 2-door, 0-chest

            let x_pos = reader.read_i32::<LittleEndian>()?;
            let y_pos = reader.read_i32::<LittleEndian>()?;
            let rotation = reader.read_u8()?;

            reader.read_u8()?;
            reader.read_u8()?;
            reader.read_u8()?;

            reader.read_i32::<LittleEndian>()?;

            let closed = reader.read_i32::<LittleEndian>()?; // chest 0-open, 1-closed

            let required_item_id = reader.read_u8()?; // lower bound
            let required_item_type_id = reader.read_u8()?;
            reader.read_u8()?;
            reader.read_u8()?;

            let required_item_id2 = reader.read_u8()?; // upper bound
            let required_item_type_id2 = reader.read_u8()?;
            reader.read_u8()?;
            reader.read_u8()?;

            let mut buffer_16 = [0u8; 16];
            reader.read_exact(&mut buffer_16)?;

            let gold_amount = reader.read_i32::<LittleEndian>()?;

            let item_id = reader.read_u8()?;
            let item_type_id = reader.read_u8()?;
            reader.read_u8()?;
            reader.read_u8()?;

            let item_count = reader.read_i32::<LittleEndian>()?;

            let mut buffer_40 = [0u8; 40];
            reader.read_exact(&mut buffer_40)?;

            let event_id = reader.read_i32::<LittleEndian>()?; // id from event.ini
            let message_id = reader.read_i32::<LittleEndian>()?; // id from message.scr for signs

            let mut buffer_32 = [0u8; 32];
            reader.read_exact(&mut buffer_32)?;

            let visibility = reader.read_u8()?;

            let mut buffer_3 = [0u8; 3];
            reader.read_exact(&mut buffer_3)?;

            // 8 byte padding to reach 184 bytes total
            let mut padding = [0u8; 8];
            let _ = reader.read_exact(&mut padding);

            refs.push(ExtraRef {
                id: i,
                number_in_file,
                ext_id,
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
                visibility,
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
            writer.write_u8(0)?;
            writer.write_u8(record.ext_id)?;

            let mut name_buf = [0u8; 32];
            let (cow, _, _) = WINDOWS_1250.encode(&record.name);
            let len = std::cmp::min(cow.len(), 32);
            name_buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&name_buf)?;

            writer.write_u8(record.object_type)?;
            writer.write_i32::<LittleEndian>(record.x_pos)?;
            writer.write_i32::<LittleEndian>(record.y_pos)?;
            writer.write_u8(record.rotation)?;

            writer.write_all(&[0u8; 3])?;
            writer.write_i32::<LittleEndian>(0)?;

            writer.write_i32::<LittleEndian>(record.closed)?;

            writer.write_u8(record.required_item_id)?;
            writer.write_u8(record.required_item_type_id)?;
            writer.write_all(&[0u8; 2])?;

            writer.write_u8(record.required_item_id2)?;
            writer.write_u8(record.required_item_type_id2)?;
            writer.write_all(&[0u8; 2])?;

            writer.write_all(&[0u8; 16])?;

            writer.write_i32::<LittleEndian>(record.gold_amount)?;
            writer.write_u8(record.item_id)?;
            writer.write_u8(record.item_type_id)?;
            writer.write_all(&[0u8; 2])?;

            writer.write_i32::<LittleEndian>(record.item_count)?;
            writer.write_all(&[0u8; 40])?;

            writer.write_i32::<LittleEndian>(record.event_id)?;
            writer.write_i32::<LittleEndian>(record.message_id)?;

            writer.write_all(&[0u8; 32])?;

            writer.write_u8(record.visibility)?;
            writer.write_all(&[0u8; 3])?;

            writer.write_all(&[0u8; 8])?; // pad to 184 bytes
        }
        Ok(())
    }
}

pub fn read_extra_ref(source_path: &Path) -> std::io::Result<Vec<ExtraRef>> {
    ExtraRef::read_file(source_path)
}
