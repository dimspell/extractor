use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::{fs::File, path::Path};

use crate::references::enums::NpcLookingDirection;
use crate::references::extractor::{read_mapper, read_null_terminated_windows_1250, Extractor};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::WINDOWS_1250;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

// ===========================================================================
// NPCREF.REF FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | NpcRef.ref - NPC Placement Data       |
// +--------------------------------------+
// | Encoding: Binary (Little-Endian)     |
// | Text Encoding: WINDOWS-1250          |
// | Header: 4-byte record count          |
// | Record Size: 672 bytes (0x2A0)       |
// +--------------------------------------+
// | [Header]                            |
// | - record_count: i32                  |
// +--------------------------------------+
// | [Record 1]                           |
// | - id: i32                            |
// | - npc_id: i32 (NPC type ID)           |
// | - name: 260 bytes (WINDOWS-1250)     |
// | - ignored_string: 260 bytes (zeros)  |
// | - party_script_id: i32               |
// | - show_on_event: i32                 |
// | - padding: 4 bytes                  |
// | - goto1_filled: i32                 |
// | - goto2_filled: i32                 |
// | - goto3_filled: i32                 |
// | - goto4_filled: i32                 |
// | - goto1_x: i32                      |
// | - goto2_x: i32                      |
// | - goto3_x: i32                      |
// | - goto4_x: i32                      |
// | - padding: 16 bytes                 |
// | - goto1_y: i32                      |
// | - goto2_y: i32                      |
// | - goto3_y: i32                      |
// | - goto4_y: i32                      |
// | - padding: 16 bytes                 |
// | - looking_direction: i32            |
// | - padding: 56 bytes                 |
// | - dialog_id: i32                    |
// | - padding: 4 bytes                  |
// +--------------------------------------+
// | [Record 2]                           |
// | ... (same structure) ...             |
// +--------------------------------------+
//
// LOOKING DIRECTIONS:
// - 0: Up (North)
// - 1: Right (East)
// - 2: Down (South)
// - 3: Left (West)
// - Clockwise rotation
//
// WAYPOINT SYSTEM:
// - 4 waypoints per NPC
// - gotoN_filled: 0=inactive, 1=active
// - gotoN_x/gotoN_y: Tile coordinates
// - Used for patrol routes and movement
//
// SPECIAL VALUES:
// - show_on_event = 0: Always visible
// - show_on_event > 0: Event-triggered
// - dialog_id = 0: No dialogue
// - Fixed 260-byte string fields
//
// FILE PURPOSE:
// Defines NPC placements with waypoints, dialogue,
// and behavioral parameters. Used for populating
// maps with interactive characters.
//
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPC {
    /// Internal iteration index mapped from the file array.
    pub index: i32,
    /// Global identifier for this mapping instance.
    pub id: i32,
    /// Underlying archetype ID linked from npccat or prtini.
    pub npc_id: i32,
    /// Fixed 30-byte display descriptor.
    pub name: String,
    /// Reference script matching PartyRefs logic.
    pub party_script_id: i32,
    /// Event ID condition required to spawn NPC.
    pub show_on_event: i32,
    /// Waypoint 1 definition flag.
    pub goto1_filled: i32,
    /// Waypoint 2 definition flag.
    pub goto2_filled: i32,
    /// Waypoint 3 definition flag.
    pub goto3_filled: i32,
    /// Waypoint 4 definition flag.
    pub goto4_filled: i32,
    /// Waypoint 1 X target.
    pub goto1_x: i32,
    /// Waypoint 2 X target.
    pub goto2_x: i32,
    /// Waypoint 3 X target.
    pub goto3_x: i32,
    /// Waypoint 4 X target.
    pub goto4_x: i32,
    /// Waypoint 1 Y target.
    pub goto1_y: i32,
    /// Waypoint 2 Y target.
    pub goto2_y: i32,
    /// Waypoint 3 Y target.
    pub goto3_y: i32,
    /// Waypoint 4 Y target.
    pub goto4_y: i32,
    /// Compass rotation (0=up, proceeds clockwise).
    pub looking_direction: NpcLookingDirection,
    /// Pointer to `Dlgcat` or dialogue node triggering on click.
    pub dialog_id: i32,
}

/// Stores specific placements and configurations for NPCs on a given map.
///
/// Reads file: `NpcInGame/Npccat1.ref (and other map-specific .ref files)`
/// # File Format: `NpcInGame/Npccat1.ref` (and other map `.ref` files)
///
/// Binary file, little-endian.  Starts with a 4-byte i32 record count.
/// Each record is exactly `0x2a0 = 672` bytes:
/// - `npc_id`           : i32
/// - `name`             : 260 bytes, null-padded, WINDOWS-1250
/// - (ignored string)   : 260 bytes (always zeroed on write)
/// - `party_script_id`  : i32
/// - `show_on_event`    : i32
/// - goto filled flags  : 4 × i32
/// - goto X coords      : 4 × i32
/// - 16-byte padding
/// - goto Y coords      : 4 × i32
/// - 16-byte padding
/// - `looking_direction`: i32  (0=up, clockwise)
/// - 56-byte padding
/// - `dialog_id`        : i32  (also shop text reference)
/// - 4-byte padding
impl Extractor for NPC {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(source_path)?;

        let metadata = file.metadata()?;
        let file_len = metadata.len();

        let mut reader = BufReader::new(file);

        const COUNTER_SIZE: u8 = 4;
        const PROPERTY_ITEM_SIZE: i32 = 0x2a0; // 672
        const STRING_MAX_LENGTH: usize = 260;

        let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
        let mut npcs: Vec<NPC> = Vec::with_capacity(elements as usize);

        for i in 0..elements {
            let id = reader.read_i32::<LittleEndian>()?;
            let npc_id = reader.read_i32::<LittleEndian>()?;

            let mut buffer = [0u8; STRING_MAX_LENGTH];
            reader.read_exact(&mut buffer)?;
            let name = read_null_terminated_windows_1250(&buffer).unwrap();

            let mut buffer_ignored = [0u8; STRING_MAX_LENGTH];
            reader.read_exact(&mut buffer_ignored)?;

            let party_script_id = reader.read_i32::<LittleEndian>()?;
            let show_on_event = reader.read_i32::<LittleEndian>()?;

            let mut buffer_4 = [0u8; 4];
            reader.read_exact(&mut buffer_4)?;

            let goto1_filled = reader.read_i32::<LittleEndian>()?;
            let goto2_filled = reader.read_i32::<LittleEndian>()?;
            let goto3_filled = reader.read_i32::<LittleEndian>()?;
            let goto4_filled = reader.read_i32::<LittleEndian>()?;

            let goto1_x = reader.read_i32::<LittleEndian>()?;
            let goto2_x = reader.read_i32::<LittleEndian>()?;
            let goto3_x = reader.read_i32::<LittleEndian>()?;
            let goto4_x = reader.read_i32::<LittleEndian>()?;

            let goto1_y = reader.read_i32::<LittleEndian>()?;
            let goto2_y = reader.read_i32::<LittleEndian>()?;
            let goto3_y = reader.read_i32::<LittleEndian>()?;
            let goto4_y = reader.read_i32::<LittleEndian>()?;

            let mut buffer_16 = [0u8; 16];
            reader.read_exact(&mut buffer_16)?;

            let looking_direction_raw = reader.read_i32::<LittleEndian>()?; // 0 = up, clockwise

            let mut buffer_56 = [0u8; 16 + 16 + 16 + 8];
            reader.read_exact(&mut buffer_56)?;

            let dialog_id = reader.read_i32::<LittleEndian>()?; // also text for shop

            let mut buffer_last = [0u8; 4];
            reader.read_exact(&mut buffer_last)?;

            let looking_direction = NpcLookingDirection::from_i32(looking_direction_raw)
                .unwrap_or(NpcLookingDirection::Up);

            npcs.push(NPC {
                index: i,
                id,
                npc_id,
                name: name.to_string(),
                party_script_id,
                show_on_event,
                goto1_filled,
                goto2_filled,
                goto3_filled,
                goto4_filled,
                goto1_x,
                goto2_x,
                goto3_x,
                goto4_x,
                goto1_y,
                goto2_y,
                goto3_y,
                goto4_y,
                looking_direction,
                dialog_id,
            })
        }

        Ok(npcs)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let file = File::create(dest_path)?;
        let mut writer = BufWriter::new(file);

        let elements = records.len() as i32;
        writer.write_i32::<LittleEndian>(elements)?;

        for record in records {
            writer.write_i32::<LittleEndian>(record.id)?;
            writer.write_i32::<LittleEndian>(record.npc_id)?;

            let mut name_buf = [0u8; 260];
            let (cow, _, _) = WINDOWS_1250.encode(&record.name);
            let len = std::cmp::min(cow.len(), 260);
            name_buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&name_buf)?;

            writer.write_all(&[0u8; 260])?; // ignored string

            writer.write_i32::<LittleEndian>(record.party_script_id)?;
            writer.write_i32::<LittleEndian>(record.show_on_event)?;

            writer.write_all(&[0u8; 4])?;

            writer.write_i32::<LittleEndian>(record.goto1_filled)?;
            writer.write_i32::<LittleEndian>(record.goto2_filled)?;
            writer.write_i32::<LittleEndian>(record.goto3_filled)?;
            writer.write_i32::<LittleEndian>(record.goto4_filled)?;

            writer.write_i32::<LittleEndian>(record.goto1_x)?;
            writer.write_i32::<LittleEndian>(record.goto2_x)?;
            writer.write_i32::<LittleEndian>(record.goto3_x)?;
            writer.write_i32::<LittleEndian>(record.goto4_x)?;

            writer.write_i32::<LittleEndian>(record.goto1_y)?;
            writer.write_i32::<LittleEndian>(record.goto2_y)?;
            writer.write_i32::<LittleEndian>(record.goto3_y)?;
            writer.write_i32::<LittleEndian>(record.goto4_y)?;

            writer.write_all(&[0u8; 16])?;

            writer.write_i32::<LittleEndian>(i32::from(record.looking_direction))?;

            writer.write_all(&[0u8; 56])?;

            writer.write_i32::<LittleEndian>(record.dialog_id)?;

            writer.write_all(&[0u8; 4])?;
        }

        Ok(())
    }
}

pub fn read_npc_ref(source_path: &Path) -> std::io::Result<Vec<NPC>> {
    NPC::read_file(source_path)
}

pub fn save_npc_refs(conn: &mut Connection, file_path: &str, npc_refs: &[NPC]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_npc_ref.sql"))?;
        for npc in npc_refs {
            stmt.execute(params![
                file_path,
                npc.index,
                npc.id,
                npc.npc_id,
                npc.name,
                npc.party_script_id,
                npc.show_on_event,
                npc.goto1_filled,
                npc.goto2_filled,
                npc.goto3_filled,
                npc.goto4_filled,
                npc.goto1_x,
                npc.goto2_x,
                npc.goto3_x,
                npc.goto4_x,
                npc.goto1_y,
                npc.goto2_y,
                npc.goto3_y,
                npc.goto4_y,
                i32::from(npc.looking_direction),
                npc.dialog_id,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
