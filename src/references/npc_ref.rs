use std::io::{Read, Seek, Write};
use std::path::Path;

use crate::references::enums::{
    BooleanFlag, ItemTypeId, NpcLookingDirection, Unknown0110, Unknown012, Unknown0to7,
};
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
// | Record Size: 672 bytes               |
// +--------------------------------------+
// | [Header]                            |
// | - record_count: i32                  |
// +--------------------------------------+
// | [Record 1]                           |
// | - id: i32                            |
// | - npc_id: i32 (NPC type ID)           |
// | - name: 260 bytes (WINDOWS-1250)     |
// | - description: 260 bytes (WINDOWS-1250) |
// | - party_script_id: i32               |
// | - show_on_event: i32                 |
// | - unknown_1: i32                    |
// | - goto1_filled: i32                 |
// | - goto2_filled: i32                 |
// | - goto3_filled: i32                 |
// | - goto4_filled: i32                 |
// | - goto1_x: i32                      |
// | - goto2_x: i32                      |
// | - goto3_x: i32                      |
// | - goto4_x: i32                      |
// | - goto1_y: i32                      |
// | - goto2_y: i32                      |
// | - goto3_y: i32                      |
// | - goto4_y: i32                      |
// | - unknown_2: i32                    |
// | - unknown_3: i32                    |
// | - unknown_4: i32                    |
// | - unknown_5: i32                    |
// | - looking_direction: i32            |
// | - unknown_6: i32                    |
// | - unknown_7: i32                    |
// | - unknown_8: i32                    |
// | - unknown_9: i32                    |
// | - unknown_10: i32                   |
// | - unknown_11: i32                   |
// | - unknown_12: i32                   |
// | - unknown_13: i32                   |
// | - unknown_14: i32                   |
// | - unknown_15: i32                   |
// | - unknown_16: i32                   |
// | - unknown_17: i32                   |
// | - unknown_18: i32 (encoded as [item_id, item_type, 0, 0]) |
// | - unknown_19: i32                   |
// | - dialog_id: i32                    |
// | - dialogue_face_sprite_id: i32      |
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
// - dialogue_face_sprite_id: Sprite ID for the character's portrait/face displayed in dialogue windows.
//
// FILE PURPOSE:
// Defines NPC placements with waypoints, dialogue,
// and behavioral parameters. Used for populating
// maps with interactive characters.
//
// ===========================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NPC {
    /// Internal iteration index mapped from the file array.
    pub index: i32,
    /// Global identifier for this mapping instance.
    pub id: i32,
    /// Underlying archetype ID linked from npccat or prtini.
    pub npc_id: i32,
    /// Fixed 30-byte display descriptor.
    pub name: String,
    /// Description of the NPC, usually a role of the NPC (e.g. "guard", "king").
    pub description: String,
    /// Reference script matching PartyRefs logic.
    pub party_script_id: i32,
    /// Event ID condition required to spawn NPC.
    pub show_on_event: i32,
    /// Unknown. Enum = 0, 1 or 2.
    pub unknown_1: Unknown012,
    /// Waypoint 1 definition flag. Enum = 0 or 1.
    pub goto1_filled: BooleanFlag,
    /// Waypoint 2 definition flag. Enum = 0 or 1.
    pub goto2_filled: BooleanFlag,
    /// Waypoint 3 definition flag. Enum = 0 or 1.
    pub goto3_filled: BooleanFlag,
    /// Waypoint 4 definition flag. Enum = 0 or 1.
    pub goto4_filled: BooleanFlag,
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
    /// Unknown coordinate (X).
    pub unknown_2: i32,
    /// Unknown coordinate (Y).
    pub unknown_3: i32,
    /// Unknown coordinate (X).
    pub unknown_4: i32,
    /// Unknown coordinate (Y).
    pub unknown_5: i32,
    /// Compass rotation (0=up, proceeds clockwise).
    pub looking_direction: NpcLookingDirection,
    /// Unknown. Enum = 0, 1, 2, 3, 4, 5, 6 or 7.
    pub unknown_6: Unknown0to7,
    /// Unknown. Enum = 0, 1, 2, 3, 4, 5, 6 or 7.
    pub unknown_7: Unknown0to7,
    /// Unknown. Enum = 0, 1, 2, 3, 4, 5, 6 or 7.
    pub unknown_8: Unknown0to7,
    /// Unknown. Always zero (0).
    pub unknown_9: i32,
    /// Unknown. Always zero (0).
    pub unknown_10: i32,
    /// Unknown. Always zero (0).
    pub unknown_11: i32,
    /// Unknown. Always zero (0).
    pub unknown_12: i32,
    /// Unknown coordinate (X).
    pub unknown_13: i32,
    /// Unknown coordinate (Y).
    pub unknown_14: i32,
    /// Unknown coordinate (X).
    pub unknown_15: i32,
    /// Unknown coordinate (Y).
    pub unknown_16: i32,
    /// Unknown. Enum = 0, 1 or 2.
    pub unknown_17: Unknown012,
    /// Unknown item reference.
    pub unknown_item_type: ItemTypeId,
    pub unknown_item_id: u8,
    /// Unknown. Enum = 0, 1, 10.
    pub unknown_19: Unknown0110,
    /// Pointer to `Dlgcat` or dialogue node triggering on click.
    pub dialog_id: i32,
    /// Sprite ID for the character's portrait/face displayed in dialogue windows.
    /// Used to construct sprite paths: "Dispel/NpcInGame/face%d.spr" or "Dispel/NpcInGame/Face%d.spr"
    /// where %d is replaced with this field's value (e.g., value 5 => "face5.spr").
    pub dialogue_face_sprite_id: i32,
}

/// Stores specific placements and configurations for NPCs on a given map.
///
/// Reads file: `NpcInGame/Npccat1.ref (and other map-specific .ref files)`
///
/// Binary file, little-endian.  Starts with a 4-byte i32 record count.
/// Each record is exactly `672` bytes
impl Extractor for NPC {
    fn parse<R: Read + Seek>(reader: &mut R, len: u64) -> std::io::Result<Vec<Self>> {
        const COUNTER_SIZE: u8 = 4_u8;
        const PROPERTY_ITEM_SIZE: i32 = 672_i32;
        const STRING_MAX_LENGTH: usize = 260_usize;

        let elements = read_mapper(reader, len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
        let mut npcs: Vec<NPC> = Vec::with_capacity(elements as usize);

        for i in 0..elements {
            let id = reader.read_i32::<LittleEndian>()?;
            let npc_id = reader.read_i32::<LittleEndian>()?;

            let mut buffer = [0u8; STRING_MAX_LENGTH];
            reader.read_exact(&mut buffer)?;
            let name = read_null_terminated_windows_1250(&buffer).unwrap();

            let mut buffer = [0u8; STRING_MAX_LENGTH];
            reader.read_exact(&mut buffer)?;
            let description = read_null_terminated_windows_1250(&buffer).unwrap(); // after null the rest is 205 byte array.

            let party_script_id = reader.read_i32::<LittleEndian>()?;
            let show_on_event = reader.read_i32::<LittleEndian>()?;

            let unknown_1 = Unknown012::from_i32(reader.read_i32::<LittleEndian>()?)
                .unwrap_or(Unknown012::Value0);

            let goto1_filled = BooleanFlag::from_i32(reader.read_i32::<LittleEndian>()?)
                .unwrap_or(BooleanFlag::False);
            let goto2_filled = BooleanFlag::from_i32(reader.read_i32::<LittleEndian>()?)
                .unwrap_or(BooleanFlag::False);
            let goto3_filled = BooleanFlag::from_i32(reader.read_i32::<LittleEndian>()?)
                .unwrap_or(BooleanFlag::False);
            let goto4_filled = BooleanFlag::from_i32(reader.read_i32::<LittleEndian>()?)
                .unwrap_or(BooleanFlag::False);

            let goto1_x = reader.read_i32::<LittleEndian>()?;
            let goto2_x = reader.read_i32::<LittleEndian>()?;
            let goto3_x = reader.read_i32::<LittleEndian>()?;
            let goto4_x = reader.read_i32::<LittleEndian>()?;

            let goto1_y = reader.read_i32::<LittleEndian>()?;
            let goto2_y = reader.read_i32::<LittleEndian>()?;
            let goto3_y = reader.read_i32::<LittleEndian>()?;
            let goto4_y = reader.read_i32::<LittleEndian>()?;

            let unknown_2 = reader.read_i32::<LittleEndian>()?;
            let unknown_3 = reader.read_i32::<LittleEndian>()?;
            let unknown_4 = reader.read_i32::<LittleEndian>()?;
            let unknown_5 = reader.read_i32::<LittleEndian>()?;

            let looking_direction_raw = reader.read_i32::<LittleEndian>()?; // 0 = up, clockwise

            let unknown_6 = Unknown0to7::from_i32(reader.read_i32::<LittleEndian>()?)
                .unwrap_or(Unknown0to7::Value0);
            let unknown_7 = Unknown0to7::from_i32(reader.read_i32::<LittleEndian>()?)
                .unwrap_or(Unknown0to7::Value0);
            let unknown_8 = Unknown0to7::from_i32(reader.read_i32::<LittleEndian>()?)
                .unwrap_or(Unknown0to7::Value0);
            let unknown_9 = reader.read_i32::<LittleEndian>()?;
            let unknown_10 = reader.read_i32::<LittleEndian>()?;
            let unknown_11 = reader.read_i32::<LittleEndian>()?;
            let unknown_12 = reader.read_i32::<LittleEndian>()?;

            let unknown_13 = reader.read_i32::<LittleEndian>()?;
            let unknown_14 = reader.read_i32::<LittleEndian>()?;
            let unknown_15 = reader.read_i32::<LittleEndian>()?;
            let unknown_16 = reader.read_i32::<LittleEndian>()?;
            let unknown_17 = Unknown012::from_i32(reader.read_i32::<LittleEndian>()?)
                .unwrap_or(Unknown012::Value0);
            let unknown_18 = reader.read_i32::<LittleEndian>()?;
            let unknown_18_bytes = unknown_18.to_le_bytes();
            let unknown_item_id = unknown_18_bytes[0];
            let unknown_item_type =
                ItemTypeId::from_u8(unknown_18_bytes[1]).unwrap_or(ItemTypeId::Other);
            let unknown_19 = Unknown0110::from_i32(reader.read_i32::<LittleEndian>()?)
                .unwrap_or(Unknown0110::Value0);

            let dialog_id = reader.read_i32::<LittleEndian>()?; // also text for shop
            let dialogue_face_sprite_id = reader.read_i32::<LittleEndian>()?;

            let looking_direction = NpcLookingDirection::from_i32(looking_direction_raw)
                .unwrap_or(NpcLookingDirection::Up);

            npcs.push(NPC {
                index: i,
                id,
                npc_id,
                name: name.to_string(),
                description: description.to_string(),
                party_script_id,
                show_on_event,
                unknown_1,
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
                unknown_2,
                unknown_3,
                unknown_4,
                unknown_5,
                looking_direction,
                unknown_6,
                unknown_7,
                unknown_8,
                unknown_9,
                unknown_10,
                unknown_11,
                unknown_12,
                unknown_13,
                unknown_14,
                unknown_15,
                unknown_16,
                unknown_17,
                unknown_item_type,
                unknown_item_id,
                unknown_19,
                dialog_id,
                dialogue_face_sprite_id,
            })
        }

        Ok(npcs)
    }

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
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

            let mut desc_buf = [0u8; 260];
            let (cow, _, _) = WINDOWS_1250.encode(&record.description);
            let len = std::cmp::min(cow.len(), 260);
            desc_buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&desc_buf)?;

            writer.write_i32::<LittleEndian>(record.party_script_id)?;
            writer.write_i32::<LittleEndian>(record.show_on_event)?;

            writer.write_i32::<LittleEndian>(i32::from(record.unknown_1))?;
            writer.write_i32::<LittleEndian>(i32::from(record.goto1_filled))?;
            writer.write_i32::<LittleEndian>(i32::from(record.goto2_filled))?;
            writer.write_i32::<LittleEndian>(i32::from(record.goto3_filled))?;
            writer.write_i32::<LittleEndian>(i32::from(record.goto4_filled))?;

            writer.write_i32::<LittleEndian>(record.goto1_x)?;
            writer.write_i32::<LittleEndian>(record.goto2_x)?;
            writer.write_i32::<LittleEndian>(record.goto3_x)?;
            writer.write_i32::<LittleEndian>(record.goto4_x)?;

            writer.write_i32::<LittleEndian>(record.goto1_y)?;
            writer.write_i32::<LittleEndian>(record.goto2_y)?;
            writer.write_i32::<LittleEndian>(record.goto3_y)?;
            writer.write_i32::<LittleEndian>(record.goto4_y)?;

            writer.write_i32::<LittleEndian>(record.unknown_2)?;
            writer.write_i32::<LittleEndian>(record.unknown_3)?;
            writer.write_i32::<LittleEndian>(record.unknown_4)?;
            writer.write_i32::<LittleEndian>(record.unknown_5)?;

            writer.write_i32::<LittleEndian>(i32::from(record.looking_direction))?;

            writer.write_i32::<LittleEndian>(i32::from(record.unknown_6))?;
            writer.write_i32::<LittleEndian>(i32::from(record.unknown_7))?;
            writer.write_i32::<LittleEndian>(i32::from(record.unknown_8))?;
            writer.write_i32::<LittleEndian>(record.unknown_9)?;
            writer.write_i32::<LittleEndian>(record.unknown_10)?;
            writer.write_i32::<LittleEndian>(record.unknown_11)?;
            writer.write_i32::<LittleEndian>(record.unknown_12)?;
            writer.write_i32::<LittleEndian>(record.unknown_13)?;
            writer.write_i32::<LittleEndian>(record.unknown_14)?;
            writer.write_i32::<LittleEndian>(record.unknown_15)?;
            writer.write_i32::<LittleEndian>(record.unknown_16)?;
            writer.write_i32::<LittleEndian>(i32::from(record.unknown_17))?;
            // Reconstruct unknown_18 from separate fields
            let unknown_item_type_byte: u8 = record.unknown_item_type.into();
            let unknown_18_encoded =
                i32::from_le_bytes([record.unknown_item_id, unknown_item_type_byte, 0, 0]);
            writer.write_i32::<LittleEndian>(unknown_18_encoded)?;
            writer.write_i32::<LittleEndian>(i32::from(record.unknown_19))?;

            writer.write_i32::<LittleEndian>(record.dialog_id)?;

            writer.write_i32::<LittleEndian>(record.dialogue_face_sprite_id)?;
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
                npc.description,
                npc.party_script_id,
                npc.show_on_event,
                i32::from(npc.unknown_1),
                i32::from(npc.goto1_filled),
                i32::from(npc.goto2_filled),
                i32::from(npc.goto3_filled),
                i32::from(npc.goto4_filled),
                npc.goto1_x,
                npc.goto2_x,
                npc.goto3_x,
                npc.goto4_x,
                npc.goto1_y,
                npc.goto2_y,
                npc.goto3_y,
                npc.goto4_y,
                npc.unknown_2,
                npc.unknown_3,
                npc.unknown_4,
                npc.unknown_5,
                i32::from(npc.looking_direction),
                i32::from(npc.unknown_6),
                i32::from(npc.unknown_7),
                i32::from(npc.unknown_8),
                npc.unknown_9,
                npc.unknown_10,
                npc.unknown_11,
                npc.unknown_12,
                npc.unknown_13,
                npc.unknown_14,
                npc.unknown_15,
                npc.unknown_16,
                i32::from(npc.unknown_17),
                npc.unknown_item_id as i32,
                npc.unknown_item_type as i32,
                i32::from(npc.unknown_19),
                npc.dialog_id,
                npc.dialogue_face_sprite_id,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn npc_bytes(npc_id: i32, name: &str, dialog_id: i32) -> Vec<u8> {
        let mut rec = vec![0u8; 672];
        // id at 0, npc_id at 4
        rec[0..4].copy_from_slice(&0i32.to_le_bytes());
        rec[4..8].copy_from_slice(&npc_id.to_le_bytes());
        // name at 8, 260 bytes
        let nb = name.as_bytes();
        let n = nb.len().min(259);
        rec[8..8 + n].copy_from_slice(&nb[..n]);
        // description at 268 (8+260), 260 bytes – stays zero
        // dialog_id at offset 664 (672 - 8 = 664? let me compute)
        // Total: id(4)+npc_id(4)+name(260)+desc(260)+rest until dialog_id
        // party_script_id at 528, show_on_event at 532, unknown_1 at 536,
        // 4 goto_filled at 540-555, 4 goto_x at 556-571, 4 goto_y at 572-587
        // unknown_2..5 at 588-603, looking_direction at 604
        // unknown_6..8 at 608-619, unknown_9..12 at 620-635, unknown_13..16 at 636-651
        // unknown_17 at 652, unknown_18 at 656, unknown_19 at 660, dialog_id at 664
        rec[664..668].copy_from_slice(&dialog_id.to_le_bytes());
        rec
    }

    #[test]
    fn parse_single_npc() {
        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(npc_bytes(42, "Innkeeper", 500));
        assert_eq!(data.len(), 676);

        let mut c = Cursor::new(&data[..]);
        let npcs = NPC::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(npcs.len(), 1);
        assert_eq!(npcs[0].npc_id, 42);
        assert_eq!(npcs[0].name, "Innkeeper");
        assert_eq!(npcs[0].dialog_id, 500);
    }

    #[test]
    fn parse_two_npcs() {
        let mut data = 2i32.to_le_bytes().to_vec();
        data.extend(npc_bytes(1, "Guard", 10));
        data.extend(npc_bytes(2, "Mage", 20));

        let mut c = Cursor::new(&data[..]);
        let npcs = NPC::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(npcs.len(), 2);
        assert_eq!(npcs[0].name, "Guard");
        assert_eq!(npcs[1].name, "Mage");
    }

    #[test]
    fn serialize_round_trip() {
        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(npc_bytes(42, "Innkeeper", 500));
        let mut c = Cursor::new(&data[..]);
        let records = NPC::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        NPC::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = NPC::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].npc_id, records2[0].npc_id);
        assert_eq!(records[0].name, records2[0].name);
        assert_eq!(records[0].dialog_id, records2[0].dialog_id);
    }
}
