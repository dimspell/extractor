use std::path::Path;

use crate::references::enums::{
    BooleanFlag, ExtraObjectType, ItemTypeId, SmallRange0to3, Special9999Flag, SpecialPatternFlag,
    VisibilityType,
};
use crate::references::extractor::Extractor;
use dispel_macros::Extractor;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// Stores specific placements and configurations for interactive objects (chests, signs, doors) on a map.
///
/// Reads file: `ExtraInGame/Extdun01.ref (and other map-specific .ref files)`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Extractor)]
#[extractor(property_item_size = 184)]
pub struct ExtraRef {
    /// Specific object generation ID for map tracking.
    #[extractor(id)]
    pub id: i32,
    /// Linear parsing index.
    #[extractor(primitive(type = "u8"))]
    pub number_in_file: u8,
    /// Unrecognized (always zero).
    #[extractor(padding(count = 1, type = "u8"))]
    pub unknown1: u8,
    /// Mapping ID linked backwards derived from Extra.ini.
    #[extractor(primitive(type = "u8"))]
    pub ext_id: u8,
    /// 32-byte label identifier.
    #[extractor(string(encoding = "WINDOWS-1250", size = 32))]
    pub name: String,
    /// Object type (chest, door, sign, etc.).
    #[extractor(enum_from_u8(type = "ExtraObjectType"))]
    pub object_type: ExtraObjectType,
    /// Tile mapping horizontal target.
    #[extractor(primitive(type = "i32"))]
    pub x_pos: i32,
    /// Tile mapping vertical target.
    #[extractor(primitive(type = "i32"))]
    pub y_pos: i32,
    /// Facing perspective index.
    #[extractor(primitive(type = "u8"))]
    pub rotation: u8,
    /// Unrecognized (always [205, 205, 205])
    #[extractor(vec_u8(size = 3))]
    pub unknown2: Vec<u8>,
    /// Unrecognized (always zero)
    #[extractor(primitive(type = "i32"))]
    pub unknown3: i32,
    /// Interaction status for chests (0=open, 1=closed).
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub closed: BooleanFlag,
    /// Key identifier (lower bound) to interact.
    #[extractor(primitive(type = "u8"))]
    pub required_item_id: u8,
    /// Category ID of lower bound requirement.
    #[extractor(enum_from_u8(type = "ItemTypeId"))]
    pub required_item_type_id: ItemTypeId,
    /// Unrecognized (always zero)
    #[extractor(primitive(type = "i16"))]
    pub unknown4: i16,
    /// Secondary requirement / Key upper bound.
    #[extractor(primitive(type = "u8"))]
    pub required_item_id2: u8,
    /// Category ID for upper bound.
    #[extractor(enum_from_u8(type = "ItemTypeId"))]
    pub required_item_type_id2: ItemTypeId,
    /// Unrecognized (always zero)
    #[extractor(primitive(type = "i16"))]
    pub unknown5: i16,
    /// Unrecognized (0 or 9999)
    #[extractor(enum_from_i32(type = "Special9999Flag"))]
    pub unknown6: Special9999Flag,
    /// Unrecognized (0 or 9999)
    #[extractor(enum_from_i32(type = "Special9999Flag"))]
    pub unknown7: Special9999Flag,
    /// Unrecognized (0 or 9999)
    #[extractor(enum_from_i32(type = "Special9999Flag"))]
    pub unknown8: Special9999Flag,
    /// Unrecognized (0 or 9999)
    #[extractor(enum_from_i32(type = "Special9999Flag"))]
    pub unknown9: Special9999Flag,
    /// Quantity of gold inside container.
    #[extractor(primitive(type = "i32"))]
    pub gold_amount: i32,
    /// Found static loot ID.
    #[extractor(primitive(type = "u8"))]
    pub item_id: u8,
    /// Category enum for found loot.
    #[extractor(enum_from_u8(type = "ItemTypeId"))]
    pub item_type_id: ItemTypeId,
    /// Unrecognized (always zero)
    #[extractor(primitive(type = "i16"))]
    pub unknown10: i16,
    /// Stacks contained within object.
    #[extractor(primitive(type = "i32"))]
    pub item_count: i32,
    /// Unrecognized (0, 28, 84, 258, 9999)
    #[extractor(enum_from_i32(type = "SpecialPatternFlag"))]
    pub unknown11: SpecialPatternFlag,
    /// Unrecognized (0 or 1)
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub unknown12: BooleanFlag,
    /// Unrecognized (0 or 9999)
    #[extractor(enum_from_i32(type = "Special9999Flag"))]
    pub unknown13: Special9999Flag,
    /// Unrecognized (always array of 28 zeros)
    #[extractor(vec_u8(size = 28))]
    pub unknown14: Vec<u8>,
    /// Bound logic ID executing upon interaction (from event.ini).
    #[extractor(primitive(type = "i32"))]
    pub event_id: i32,
    /// Pointer to signposts/plaques contained in Message.scr.
    #[extractor(primitive(type = "i32"))]
    pub message_id: i32,
    /// Unrecognized (0, 1, 2, 3)
    #[extractor(enum_from_i32(type = "SmallRange0to3"))]
    pub unknown15: SmallRange0to3,
    /// Unrecognized (0, 1, 2, 3)
    #[extractor(enum_from_i32(type = "SmallRange0to3"))]
    pub unknown16: SmallRange0to3,
    /// Unrecognized (always zero)
    #[extractor(primitive(type = "u8"))]
    pub unknown17: u8,
    /// Interactive element type (0, 1, 2, 3).
    #[extractor(enum_from_i32_from_u8(type = "SmallRange0to3"))]
    pub interactive_element_type: SmallRange0to3,
    /// Unrecognized (always array [205, 205])
    #[extractor(vec_u8(size = 2))]
    pub unknown18: Vec<u8>,
    /// Unrecognized (0 or 1)
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub is_quest_element: BooleanFlag,
    /// Unrecognized (0 or 1)
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub unknown20: BooleanFlag,
    /// Unrecognized (0 or 1)
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub unknown21: BooleanFlag,
    /// Unrecognized (always zero)
    #[extractor(primitive(type = "i32"))]
    pub unknown22: i32,
    /// Unrecognized (0 or 1)
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub unknown23: BooleanFlag,
    /// Determines alpha transparency on render.
    #[extractor(enum_from_i32_from_u8(type = "VisibilityType"))]
    pub visibility: VisibilityType,
    /// Unrecognized (0 or 1)
    #[extractor(enum_from_i32_from_u8(type = "BooleanFlag"))]
    pub unknown24: BooleanFlag,
    /// Unrecognized (always zero)
    #[extractor(primitive(type = "i16"))]
    pub unknown25: i16,
    /// Unrecognized (0 or 1)
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub unknown26: BooleanFlag,
    /// Unrecognized (0 or 1)
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub unknown27: BooleanFlag,
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
                file_path,                                    // 1
                extra_ref.id,                                 // 2
                extra_ref.number_in_file,                     // 3
                extra_ref.unknown1,                           // 4
                extra_ref.ext_id,                             // 5
                extra_ref.name,                               // 6
                u8::from(extra_ref.object_type),              // 7
                extra_ref.x_pos,                              // 8
                extra_ref.y_pos,                              // 9
                extra_ref.rotation,                           // 10
                extra_ref.unknown2,                           // 11
                extra_ref.unknown3,                           // 12
                i32::from(extra_ref.closed),                  // 13
                extra_ref.required_item_id,                   // 14
                u8::from(extra_ref.required_item_type_id),    // 15
                extra_ref.unknown4,                           // 16
                extra_ref.required_item_id2,                  // 17
                u8::from(extra_ref.required_item_type_id2),   // 18
                extra_ref.unknown5,                           // 19
                i32::from(extra_ref.unknown6),                // 20
                i32::from(extra_ref.unknown7),                // 21
                i32::from(extra_ref.unknown8),                // 22
                i32::from(extra_ref.unknown9),                // 23
                extra_ref.gold_amount,                        // 24
                extra_ref.item_id,                            // 25
                u8::from(extra_ref.item_type_id),             // 26
                extra_ref.unknown10,                          // 27
                extra_ref.item_count,                         // 28
                i32::from(extra_ref.unknown11),               // 29
                i32::from(extra_ref.unknown12),               // 30
                i32::from(extra_ref.unknown13),               // 31
                extra_ref.unknown14,                          // 32
                extra_ref.event_id,                           // 33
                extra_ref.message_id,                         // 34
                i32::from(extra_ref.unknown15),               // 35
                i32::from(extra_ref.unknown16),               // 36
                extra_ref.unknown17,                          // 37
                u8::from(extra_ref.interactive_element_type), // 38
                extra_ref.unknown18,                          // 39
                i32::from(extra_ref.is_quest_element),        // 40
                i32::from(extra_ref.unknown20),               // 41
                i32::from(extra_ref.unknown21),               // 42
                extra_ref.unknown22,                          // 43
                i32::from(extra_ref.unknown23),               // 44
                u8::from(extra_ref.visibility),               // 45
                i32::from(extra_ref.unknown24),               // 46
                extra_ref.unknown25,                          // 47
                i32::from(extra_ref.unknown26),               // 48
                i32::from(extra_ref.unknown27),               // 49
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::references::enums::ExtraObjectType;
    use std::io::Cursor;

    fn ref_bytes(name: &str, x_pos: i32, y_pos: i32, gold: i32) -> Vec<u8> {
        let mut rec = vec![0u8; 184];
        rec[0] = 1; // number_in_file
        rec[2] = 3; // extra_ini_entry_id
                    // name at offset 3, 32 bytes
        let nb = name.as_bytes();
        let n = nb.len().min(31);
        rec[3..3 + n].copy_from_slice(&nb[..n]);
        // object_type at offset 35: 0 = Chest
        // x_pos at offset 36
        rec[36..40].copy_from_slice(&x_pos.to_le_bytes());
        // y_pos at offset 40
        rec[40..44].copy_from_slice(&y_pos.to_le_bytes());
        // gold_amount at offset 80
        rec[80..84].copy_from_slice(&gold.to_le_bytes());
        rec
    }

    #[test]
    fn parse_single_ref() {
        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(ref_bytes("Chest1", 10, 20, 50));
        assert_eq!(data.len(), 188);

        let mut c = Cursor::new(&data[..]);
        let refs = ExtraRef::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].name, "Chest1");
        assert_eq!(refs[0].ext_id, 3);
        assert_eq!(refs[0].x_pos, 10);
        assert_eq!(refs[0].y_pos, 20);
        assert_eq!(refs[0].gold_amount, 50);
        assert_eq!(refs[0].object_type, ExtraObjectType::Chest);
    }

    #[test]
    fn serialize_round_trip() {
        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(ref_bytes("Chest1", 10, 20, 50));
        let mut c = Cursor::new(&data[..]);
        let records = ExtraRef::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        ExtraRef::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = ExtraRef::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].name, records2[0].name);
        assert_eq!(records[0].x_pos, records2[0].x_pos);
        assert_eq!(records[0].y_pos, records2[0].y_pos);
        assert_eq!(records[0].gold_amount, records2[0].gold_amount);
    }
}
