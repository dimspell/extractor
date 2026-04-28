use std::path::Path;

use crate::references::enums::{
    BooleanFlag, ItemTypeId, NpcLookingDirection, Unknown0110, Unknown012, Unknown0to7,
};
use crate::references::extractor::Extractor;
use dispel_macros::{Extractor, Localizable};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// NPC Reference (NpcInGame/Npccat1.ref) - NPC Placements on Maps
///
/// Stores specific placements and configurations for NPCs on a given map.
///
/// Reads file: `NpcInGame/Npccat1.ref` (and other map-specific `.ref` files)
///
/// # Binary Format
///
/// - **Encoding**: Little-endian for all numeric values
/// - **Text Encoding**: WINDOWS-1250 for `name` and `description` fields (260 bytes each, null-padded)
/// - **Record Size**: 672 bytes
/// - **Header**: 4-byte i32 record count, followed by records
///
/// ```text
/// +--------------------------------------+
/// | NPC Reference - NPC Placements      |
/// +--------------------------------------+
/// | Encoding: Binary (Little-Endian)     |
/// | Text Encoding: WINDOWS-1250           |
/// | Record Size: 672 bytes               |
/// | Header: 4-byte record count          |
/// +--------------------------------------+
/// | [Header]                             |
/// | - record_count: i32                  |
/// +--------------------------------------+
/// | [Record 1] - 672 bytes               |
/// | - index: i32 (auto-generated)        |
/// | - id: i32 (instance ID)             |
/// | - npc_id: i32 (-> NpcIni/NpcRef)   |
/// | - name: 260 bytes (WINDOWS-1250)    |
/// | - description: 260 bytes (WINDOWS...) |
/// | - party_script_id: i32                |
/// | - show_on_event: i32 (-> Event.ini)  |
/// | - unknown_1: i32 (Unknown012)        |
/// | - goto1-4_filled: i32 (BooleanFlag) |
/// | - goto1-4_x/y: i32 (waypoints)      |
/// | - unknown_2-5: i32 (coordinates?)    |
/// | - looking_direction: i32 (enum)      |
/// | - unknown_6-8: i32 (Unknown0to7)   |
/// | - unknown_9-12: i32 (always 0)      |
/// | - unknown_13-16: i32 (coordinates?)  |
/// | - unknown_17: i32 (Unknown012)       |
/// | - unknown_item_id: u8                 |
/// | - unknown_item_type: u8 (ItemTypeId) |
/// | - unknown_18: i16 (padding)          |
/// | - unknown_19: i32 (Unknown0110)      |
/// | - dialog_id: i32 (-> .dlg files)     |
/// | - dialogue_face_sprite_id: i32         |
/// +--------------------------------------+
/// | [Record 2]                           |
/// | ... (same structure) ...             |
/// +--------------------------------------+
/// ```
///
/// # Field Categories
///
/// - **Identification**: `id` (instance ID), `npc_id` (links to `NpcIni` or `NpcRef`)
/// - **Localization**: `name` (260 bytes), `description` (260 bytes), both WINDOWS-1250
/// - **Event Link**: `show_on_event` (required event to spawn NPC)
/// - **Party Script**: `party_script_id` (links to `PartyRef` logic)
/// - **Waypoints**: 4 waypoint slots (`goto1-4_filled`, `goto1-4_x`, `goto1-4_y`)
/// - **Dialogue**: `dialog_id` (links to `.dlg` files), `dialogue_face_sprite_id` (face sprite)
/// - **Appearance**: `looking_direction` (compass direction)
/// - **Unknown**: `unknown_1` through `unknown_19` (need investigation)
///
/// # Special Values
///
/// - `unknown_1/unknown_17`: Enum = 0, 1, or 2
/// - `goto1-4_filled`: 0 = waypoint not defined, 1 = waypoint defined
/// - `looking_direction`: 0 = up, proceeds clockwise (1=right, 2=down, 3=left)
/// - `unknown_6-8`: Enum = 0-7
/// - `unknown_9-12`: Always observed as 0
/// - `unknown_19`: Enum = 0, 1, or 10
/// - `unknown_item_id/type`: Unknown item reference
///
/// # File Purpose
///
/// Defines NPC placements on specific maps with waypoints,
/// dialogue triggers, and visual configurations. Used for populating
/// maps with interactive characters and quest givers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Extractor, Localizable)]
#[extractor(property_item_size = 672)]
pub struct NPC {
    /// Internal iteration index mapped from the file array.
    #[extractor(index)]
    pub index: i32,
    /// Global identifier for this mapping instance.
    #[extractor(primitive(type = "i32"))]
    pub id: i32,
    /// Underlying archetype ID linked from npccat or prtini.
    #[extractor(primitive(type = "i32"))]
    pub npc_id: i32,
    /// Fixed 30-byte display descriptor.
    #[translatable(encoding = "WINDOWS_1250", max_bytes = 260)]
    #[extractor(string(encoding = "WINDOWS-1250", size = 260))]
    pub name: String,
    /// Description of the NPC, usually a role of the NPC (e.g. "guard", "king").
    #[translatable(encoding = "WINDOWS_1250", max_bytes = 260)]
    #[extractor(string(encoding = "WINDOWS-1250", size = 260))]
    pub description: String,
    /// Reference script matching PartyRefs logic.
    #[extractor(primitive(type = "i32"))]
    pub party_script_id: i32,
    /// Event ID condition required to spawn NPC.
    #[extractor(primitive(type = "i32"))]
    pub show_on_event: i32,
    /// Unknown. Enum = 0, 1 or 2.
    #[extractor(enum_from_i32(type = "Unknown012"))]
    pub unknown_1: Unknown012,
    /// Waypoint 1 definition flag. Enum = 0 or 1.
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub goto1_filled: BooleanFlag,
    /// Waypoint 2 definition flag. Enum = 0 or 1.
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub goto2_filled: BooleanFlag,
    /// Waypoint 3 definition flag. Enum = 0 or 1.
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub goto3_filled: BooleanFlag,
    /// Waypoint 4 definition flag. Enum = 0 or 1.
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub goto4_filled: BooleanFlag,
    /// Waypoint 1 X target.
    #[extractor(primitive(type = "i32"))]
    pub goto1_x: i32,
    /// Waypoint 2 X target.
    #[extractor(primitive(type = "i32"))]
    pub goto2_x: i32,
    /// Waypoint 3 X target.
    #[extractor(primitive(type = "i32"))]
    pub goto3_x: i32,
    /// Waypoint 4 X target.
    #[extractor(primitive(type = "i32"))]
    pub goto4_x: i32,
    /// Waypoint 1 Y target.
    #[extractor(primitive(type = "i32"))]
    pub goto1_y: i32,
    /// Waypoint 2 Y target.
    #[extractor(primitive(type = "i32"))]
    pub goto2_y: i32,
    /// Waypoint 3 Y target.
    #[extractor(primitive(type = "i32"))]
    pub goto3_y: i32,
    /// Waypoint 4 Y target.
    #[extractor(primitive(type = "i32"))]
    pub goto4_y: i32,
    /// Unknown coordinate (X).
    #[extractor(primitive(type = "i32"))]
    pub unknown_2: i32,
    /// Unknown coordinate (Y).
    #[extractor(primitive(type = "i32"))]
    pub unknown_3: i32,
    /// Unknown coordinate (X).
    #[extractor(primitive(type = "i32"))]
    pub unknown_4: i32,
    /// Unknown coordinate (Y).
    #[extractor(primitive(type = "i32"))]
    pub unknown_5: i32,
    /// Compass rotation (0=up, proceeds clockwise).
    #[extractor(enum_from_i32(type = "NpcLookingDirection"))]
    pub looking_direction: NpcLookingDirection,
    /// Unknown. Enum = 0, 1, 2, 3, 4, 5, 6 or 7.
    #[extractor(enum_from_i32(type = "Unknown0to7"))]
    pub unknown_6: Unknown0to7,
    /// Unknown. Enum = 0, 1, 2, 3, 4, 5, 6 or 7.
    #[extractor(enum_from_i32(type = "Unknown0to7"))]
    pub unknown_7: Unknown0to7,
    /// Unknown. Enum = 0, 1, 2, 3, 4, 5, 6 or 7.
    #[extractor(enum_from_i32(type = "Unknown0to7"))]
    pub unknown_8: Unknown0to7,
    /// Unknown. Always zero (0).
    #[extractor(primitive(type = "i32"))]
    pub unknown_9: i32,
    /// Unknown. Always zero (0).
    #[extractor(primitive(type = "i32"))]
    pub unknown_10: i32,
    /// Unknown. Always zero (0).
    #[extractor(primitive(type = "i32"))]
    pub unknown_11: i32,
    /// Unknown. Always zero (0).
    #[extractor(primitive(type = "i32"))]
    pub unknown_12: i32,
    /// Unknown coordinate (X).
    #[extractor(primitive(type = "i32"))]
    pub unknown_13: i32,
    /// Unknown coordinate (Y).
    #[extractor(primitive(type = "i32"))]
    pub unknown_14: i32,
    /// Unknown coordinate (X).
    #[extractor(primitive(type = "i32"))]
    pub unknown_15: i32,
    /// Unknown coordinate (Y).
    #[extractor(primitive(type = "i32"))]
    pub unknown_16: i32,
    /// Unknown. Enum = 0, 1 or 2.
    #[extractor(enum_from_i32(type = "Unknown012"))]
    pub unknown_17: Unknown012,
    /// Unknown item reference.
    #[extractor(primitive(type = "u8"))]
    pub unknown_item_id: u8,
    /// Unknown item reference.
    #[extractor(enum_from_u8(type = "ItemTypeId"))]
    pub unknown_item_type: ItemTypeId,
    // Padding
    #[extractor(primitive(type = "i16"))]
    pub unknown_18: i16,
    /// Unknown. Enum = 0, 1, 10.
    #[extractor(enum_from_i32(type = "Unknown0110"))]
    pub unknown_19: Unknown0110,
    /// Pointer to `Dlgcat` or dialogue node triggering on click.
    #[extractor(primitive(type = "i32"))]
    pub dialog_id: i32,
    /// Sprite ID for the character's portrait/face displayed in dialogue windows.
    /// Used to construct sprite paths: "Dispel/NpcInGame/face%d.spr" or "Dispel/NpcInGame/Face%d.spr",
    /// where %d is replaced with this field's value (e.g., value 5 => "face5.spr").
    #[extractor(primitive(type = "i32"))]
    pub dialogue_face_sprite_id: i32,
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
                npc.unknown_item_id,
                u8::from(npc.unknown_item_type),
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
