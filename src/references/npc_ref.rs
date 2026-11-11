use std::path::Path;

use crate::references::enums::{
    BooleanFlag, InventoryItem, ItemTypeId, NpcInteractionMode, NpcLookingDirection,
    NpcMovementMode, NpcRoleResult,
};
use crate::references::extractor::Extractor;
use dispel_macros::{Extractor, Localizable, RecordLayout, RecordPatcher};
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

/// NPC placements (`NpcInGame/*.ref`).
///
/// Each 672-byte little-endian record contains two 260-byte Windows-1250
/// strings, followed by a 36-word configuration block. The original loader
/// copies the waypoint block, activation rectangle, movement mode, and
/// interaction fields directly into the NPC runtime object. See
/// `docs/files/NpcInGame/NpcMapFiles.ref.md` for the complete offset table.
#[derive(
    Debug,
    Clone,
    Default,
    Serialize,
    Deserialize,
    Extractor,
    Localizable,
    RecordPatcher,
    RecordLayout,
)]
#[extractor(property_item_size = 672)]
#[patcher(extension = "ref", stem_prefix = "npc")]
pub struct NPC {
    /// Internal iteration index mapped from the file array.
    #[extractor(index)]
    pub index: i32,
    /// File-local ID. The map loader identifies NPCs by record index instead.
    #[extractor(primitive(type = "i32"))]
    pub file_record_id: i32,
    /// NPC visual-archetype ID linked from `Npc.ini`.
    #[extractor(primitive(type = "i32"))]
    pub npc_ini_id: i32,
    /// Display name shown by the game.
    #[translatable(encoding = "WINDOWS_1250", max_bytes = 260)]
    #[extractor(string(encoding = "WINDOWS-1250", size = 260))]
    pub name: String,
    /// Role or descriptive text stored with the NPC.
    #[translatable(encoding = "WINDOWS_1250", max_bytes = 260)]
    #[extractor(string(encoding = "WINDOWS-1250", size = 260))]
    pub role_description: String,
    /// Role behavior selected when the NPC is interacted with.
    #[extractor(enum_from_i32(type = "NpcRoleResult"))]
    pub role_result: NpcRoleResult,
    /// Event ID condition required to spawn NPC.
    #[extractor(primitive(type = "i32"))]
    pub show_on_event: i32,
    /// Movement pattern: static, waypoint patrol, or random movement in the activation rectangle.
    #[extractor(enum_from_i32(type = "NpcMovementMode"))]
    pub movement_mode: NpcMovementMode,
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
    /// Wait time before the NPC moves from waypoint 1.
    #[extractor(primitive(type = "i32"))]
    pub waypoint1_wait_time: i32,
    /// Wait time before the NPC moves from waypoint 2.
    #[extractor(primitive(type = "i32"))]
    pub waypoint2_wait_time: i32,
    /// Wait time before the NPC moves from waypoint 3.
    #[extractor(primitive(type = "i32"))]
    pub waypoint3_wait_time: i32,
    /// Wait time before the NPC moves from waypoint 4.
    #[extractor(primitive(type = "i32"))]
    pub waypoint4_wait_time: i32,
    /// Facing direction at waypoint 1.
    #[extractor(enum_from_i32(type = "NpcLookingDirection"))]
    pub waypoint1_facing_direction: NpcLookingDirection,
    /// Facing direction at waypoint 2.
    #[extractor(enum_from_i32(type = "NpcLookingDirection"))]
    pub waypoint2_facing_direction: NpcLookingDirection,
    /// Facing direction at waypoint 3.
    #[extractor(enum_from_i32(type = "NpcLookingDirection"))]
    pub waypoint3_facing_direction: NpcLookingDirection,
    /// Facing direction at waypoint 4.
    #[extractor(enum_from_i32(type = "NpcLookingDirection"))]
    pub waypoint4_facing_direction: NpcLookingDirection,
    /// Reserved value for waypoint 1. Observed as zero.
    #[extractor(primitive(type = "i32"))]
    pub waypoint1_reserved: i32,
    /// Reserved value for waypoint 2. Observed as zero.
    #[extractor(primitive(type = "i32"))]
    pub waypoint2_reserved: i32,
    /// Reserved value for waypoint 3. Observed as zero.
    #[extractor(primitive(type = "i32"))]
    pub waypoint3_reserved: i32,
    /// Reserved value for waypoint 4. Observed as zero.
    #[extractor(primitive(type = "i32"))]
    pub waypoint4_reserved: i32,
    /// Activation rectangle X1 coordinate.
    #[extractor(primitive(type = "i32"))]
    pub activation_rect_x1: i32,
    /// Activation rectangle Y1 coordinate.
    #[extractor(primitive(type = "i32"))]
    pub activation_rect_y1: i32,
    /// Activation rectangle X2 coordinate.
    #[extractor(primitive(type = "i32"))]
    pub activation_rect_x2: i32,
    /// Activation rectangle Y2 coordinate.
    #[extractor(primitive(type = "i32"))]
    pub activation_rect_y2: i32,
    /// Selects special interaction-result behavior.
    #[extractor(enum_from_i32(type = "NpcInteractionMode"))]
    pub interaction_mode: NpcInteractionMode,
    /// Low 16 bits of the packed interaction result.
    #[extractor(inventory_item(wire_type = "i16"))]
    pub interaction_result_item: InventoryItem,
    /// High 16 bits of the packed interaction result, preserved verbatim.
    #[extractor(primitive(type = "i16"))]
    pub interaction_result_parameter: i16,
    /// Value added to one by the game for its interaction-distance comparison.
    #[extractor(primitive(type = "i32"))]
    pub interaction_range_offset: i32,
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

pub fn save_npc_refs(
    conn: &mut Connection,
    file_id: i32,
    dialog_file_id: i32,
    npc_refs: &[NPC],
) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_npc_ref.sql"))?;
        for npc in npc_refs {
            stmt.execute(params![
                file_id,
                npc.index,
                npc.file_record_id,
                if npc.npc_ini_id == 0 {
                    None
                } else {
                    Some(npc.npc_ini_id)
                },
                npc.name,
                npc.role_description,
                i32::from(npc.role_result),
                if npc.show_on_event == 0 {
                    None
                } else {
                    Some(npc.show_on_event)
                },
                i32::from(npc.movement_mode),
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
                npc.waypoint1_wait_time,
                npc.waypoint2_wait_time,
                npc.waypoint3_wait_time,
                npc.waypoint4_wait_time,
                i32::from(npc.waypoint1_facing_direction),
                i32::from(npc.waypoint2_facing_direction),
                i32::from(npc.waypoint3_facing_direction),
                i32::from(npc.waypoint4_facing_direction),
                npc.waypoint1_reserved,
                npc.waypoint2_reserved,
                npc.waypoint3_reserved,
                npc.waypoint4_reserved,
                npc.activation_rect_x1,
                npc.activation_rect_y1,
                npc.activation_rect_x2,
                npc.activation_rect_y2,
                i32::from(npc.interaction_mode),
                npc.interaction_result_item.item_id() as i32,
                u8::from(
                    npc.interaction_result_item
                        .item_type()
                        .unwrap_or(ItemTypeId::Other),
                ) as i32,
                npc.interaction_result_item.raw(),
                npc.interaction_range_offset,
                dialog_file_id,
                if npc.dialog_id == 0 {
                    None
                } else {
                    Some(npc.dialog_id)
                },
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

    fn npc_bytes(npc_ini_id: i32, name: &str, dialog_id: i32) -> Vec<u8> {
        let mut rec = vec![0u8; 672];
        // file_record_id at 0, npc_ini_id at 4
        rec[0..4].copy_from_slice(&0i32.to_le_bytes());
        rec[4..8].copy_from_slice(&npc_ini_id.to_le_bytes());
        // name at 8, 260 bytes
        let nb = name.as_bytes();
        let n = nb.len().min(259);
        rec[8..8 + n].copy_from_slice(&nb[..n]);
        // role_description at 268 (8+260), 260 bytes – stays zero.
        // dialog_id is at offset 664; dialogue_face_sprite_id is at 668.
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
        assert_eq!(npcs[0].npc_ini_id, 42);
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
    fn npc_role_result_maps_all_known_wire_values() {
        let expected = [
            NpcRoleResult::NormalDialogue,
            NpcRoleResult::PartyMemberDialogue1,
            NpcRoleResult::PartyMemberDialogue2,
            NpcRoleResult::PartyMemberDialogue3,
            NpcRoleResult::PartyMemberDialogue4,
            NpcRoleResult::PartyMemberDialogue5,
            NpcRoleResult::PartyMemberDialogue6,
            NpcRoleResult::PartyMemberDialogue7,
            NpcRoleResult::PartyMemberDialogue8,
            NpcRoleResult::WeaponShop,
            NpcRoleResult::HealMiscShop,
            NpcRoleResult::EditItemShop,
            NpcRoleResult::Inn,
        ];

        for (value, role) in expected.into_iter().enumerate() {
            assert_eq!(NpcRoleResult::from_i32(value as i32), Some(role));
            assert_eq!(i32::from(role), value as i32);
        }
        assert_eq!(NpcRoleResult::from_i32(13), None);
    }

    #[test]
    fn parse_runtime_mapped_fields_at_their_wire_offsets() {
        let mut rec = npc_bytes(42, "Inkeeper", 81);
        rec[528..532].copy_from_slice(&7i32.to_le_bytes());
        rec[532..536].copy_from_slice(&42i32.to_le_bytes());
        rec[536..540].copy_from_slice(&2i32.to_le_bytes());
        rec[540..544].copy_from_slice(&1i32.to_le_bytes());
        rec[556..560].copy_from_slice(&100i32.to_le_bytes());
        rec[572..576].copy_from_slice(&200i32.to_le_bytes());
        rec[588..592].copy_from_slice(&30i32.to_le_bytes());
        rec[604..608].copy_from_slice(&7i32.to_le_bytes());
        rec[636..640].copy_from_slice(&193i32.to_le_bytes());
        rec[640..644].copy_from_slice(&431i32.to_le_bytes());
        rec[644..648].copy_from_slice(&202i32.to_le_bytes());
        rec[648..652].copy_from_slice(&438i32.to_le_bytes());
        rec[652..656].copy_from_slice(&2i32.to_le_bytes());
        rec[656..660].copy_from_slice(&0x0010_0401i32.to_le_bytes());
        rec[660..664].copy_from_slice(&10i32.to_le_bytes());
        rec[668..672].copy_from_slice(&6i32.to_le_bytes());

        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(rec);
        let npc = NPC::parse(&mut Cursor::new(data), 676).unwrap().remove(0);

        assert_eq!(npc.role_result, NpcRoleResult::PartyMemberDialogue7);
        assert_eq!(npc.show_on_event, 42);
        assert_eq!(npc.movement_mode, NpcMovementMode::RandomInActivationRect);
        assert_eq!(npc.goto1_filled, BooleanFlag::True);
        assert_eq!((npc.goto1_x, npc.goto1_y), (100, 200));
        assert_eq!(npc.waypoint1_wait_time, 30);
        assert_eq!(npc.waypoint1_facing_direction, NpcLookingDirection::UpLeft);
        assert_eq!(
            (
                npc.activation_rect_x1,
                npc.activation_rect_y1,
                npc.activation_rect_x2,
                npc.activation_rect_y2,
            ),
            (193, 431, 202, 438),
        );
        assert_eq!(
            npc.interaction_mode,
            NpcInteractionMode::ConfiguredThenRandom
        );
        assert_eq!(npc.interaction_result_item.raw(), 0x0401);
        assert_eq!(npc.interaction_result_parameter, 0x10);
        assert_eq!(npc.interaction_range_offset, 10);
        assert_eq!(npc.dialogue_face_sprite_id, 6);
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
        assert_eq!(records[0].npc_ini_id, records2[0].npc_ini_id);
        assert_eq!(records[0].name, records2[0].name);
        assert_eq!(records[0].dialog_id, records2[0].dialog_id);
    }
}
