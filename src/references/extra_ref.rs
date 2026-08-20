use std::path::Path;

use crate::references::enums::{
    ActivationEffectId, BooleanFlag, ExtraObjectType, InventoryItem, ItemTypeId, SmallRange0to3,
};
use crate::references::extractor::Extractor;
use dispel_macros::{Extractor, Localizable, RecordPatcher};
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

/// Stores specific placements and configurations for interactive objects (chests, signs, doors) on a map.
///
/// Reads file: `ExtraInGame/Extdun01.ref (and other map-specific .ref files)`
#[derive(
    Debug, Clone, PartialEq, Serialize, Deserialize, Default, Extractor, Localizable, RecordPatcher,
)]
#[extractor(property_item_size = 184)]
#[patcher(extension = "ref", stem_prefix = "ext")]
pub struct ExtraRef {
    /// Zero-based record position, derived by the parser rather than stored on disk.
    #[extractor(id)]
    pub record_index: i32,
    /// Map-local object ID.
    ///
    /// The engine exposes this object to its tile/object system as
    /// `700 + map_object_id`; it is not merely the record's parse order.
    #[extractor(primitive(type = "u16"))]
    pub map_object_id: u16,
    /// ID of the visual/behavior definition in `Extra.ini`.
    #[extractor(primitive(type = "u8"))]
    pub extra_definition_id: u8,
    /// Author-facing, null-terminated object label.
    #[extractor(string(encoding = "WINDOWS-1250", size = 32))]
    #[translatable(encoding = "WINDOWS-1250", max_bytes = 32)]
    pub object_name: String,
    /// Object type (chest, door, sign, etc.).
    #[extractor(enum_from_u8(type = "ExtraObjectType"))]
    pub object_type: ExtraObjectType,
    /// Object X coordinate in the map's tile grid.
    #[extractor(primitive(type = "i32"))]
    pub map_x: i32,
    /// Object Y coordinate in the map's tile grid.
    #[extractor(primitive(type = "i32"))]
    pub map_y: i32,
    /// Sprite-facing direction/frame index.
    #[extractor(primitive(type = "u8"))]
    pub direction: u8,
    /// Padding after [`Self::direction`] (normally `[0xCD; 3]`).
    #[extractor(vec_u8(size = 3))]
    pub direction_padding: Vec<u8>,
    /// Mutable interaction state: `0` before use and `1` after activation/opening.
    ///
    /// The game preserves this field while reloading the reference record and
    /// changes it in every object-type interaction handler. It selects the
    /// corresponding sprite/map state.
    #[extractor(primitive(type = "i32"))]
    pub interaction_state: i32,
    /// Enables the key/requirement check for chest and door-like interactions.
    ///
    /// This is configuration, not the current open/closed state; that state is
    /// stored in [`Self::interaction_state`].
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub requires_key: BooleanFlag,
    /// First accepted key/item identifier (inclusive).
    #[extractor(inventory_item(wire_type = "i16"))]
    pub required_item: InventoryItem,
    /// Padding after the packed first requirement item identifier.
    #[extractor(primitive(type = "i16"))]
    pub requirement_range_1_padding: i16,
    /// Last accepted key/item identifier for the first requirement range (inclusive).
    #[extractor(inventory_item(wire_type = "i32"))]
    pub required_item2: InventoryItem,
    /// First accepted identifier in the second requirement range (inclusive).
    /// `9999` marks an unused range.
    #[extractor(primitive(type = "i32"))]
    pub requirement_range_2_start: i32,
    /// Last accepted identifier in the second requirement range (inclusive).
    #[extractor(primitive(type = "i32"))]
    pub requirement_range_2_end: i32,
    /// First accepted identifier in the third requirement range (inclusive).
    #[extractor(primitive(type = "i32"))]
    pub requirement_range_3_start: i32,
    /// Last accepted identifier in the third requirement range (inclusive).
    #[extractor(primitive(type = "i32"))]
    pub requirement_range_3_end: i32,
    /// Gold awarded when the object is successfully opened or activated.
    #[extractor(primitive(type = "i32"))]
    pub gold_amount: i32,
    /// First static loot item.
    #[extractor(inventory_item(wire_type = "i16"))]
    pub loot_item: InventoryItem,
    /// Padding after the packed first loot item identifier.
    #[extractor(primitive(type = "i16"))]
    pub loot_item_padding: i16,
    /// Quantity of [`Self::loot_item`].
    #[extractor(primitive(type = "i32"))]
    pub loot_item_count: i32,
    /// Second loot item identifier. `9999` marks an unused slot.
    #[extractor(primitive(type = "i32"))]
    pub additional_loot_1: i32,
    /// Quantity of [`Self::additional_loot_1`].
    #[extractor(primitive(type = "i32"))]
    pub additional_loot_1_count: i32,
    /// Third loot item identifier. `9999` marks an unused slot.
    #[extractor(primitive(type = "i32"))]
    pub additional_loot_2: i32,
    /// The third loot quantity followed by object-specific interaction configuration.
    ///
    /// The first `i32` is read as the quantity for `additional_loot_2`; the
    /// remaining 24 bytes are not yet assigned a stable meaning.
    #[extractor(vec_u8(size = 28))]
    pub additional_loot_2_count_and_config: Vec<u8>,
    /// `Event.ini` logic ID executed after interaction.
    #[extractor(primitive(type = "i32"))]
    pub interaction_event_id: i32,
    /// `Message.scr` entry displayed by message/sign-style objects.
    #[extractor(primitive(type = "i32"))]
    pub interaction_message_id: i32,
    /// Occupied-footprint width in map cells.
    #[extractor(enum_from_i32(type = "SmallRange0to3"))]
    pub footprint_width: SmallRange0to3,
    /// Occupied-footprint height in map cells.
    #[extractor(enum_from_i32(type = "SmallRange0to3"))]
    pub footprint_height: SmallRange0to3,
    /// Footprint traversal orientation. Zero selects the normal direction.
    #[extractor(primitive(type = "u8"))]
    pub footprint_orientation: u8,
    /// Maximum distance at which the object can be activated.
    #[extractor(enum_from_i32_from_u8(type = "SmallRange0to3"))]
    pub interaction_range: SmallRange0to3,
    /// Padding after [`Self::interaction_range`] (normally `[0xCD; 2]`).
    #[extractor(vec_u8(size = 2))]
    pub interaction_range_padding: Vec<u8>,
    /// Requests a quest-state refresh after a successful requirement check.
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub is_quest_element: BooleanFlag,
    /// Enables the post-activation tile flag used by the map-object grid.
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub post_activation_tile_flag: BooleanFlag,
    /// Selects the post-activation footprint update mode in the map-object grid.
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub post_activation_footprint_mode: BooleanFlag,
    /// Keeps the object's terminal sprite frame instead of resetting it after interaction.
    #[extractor(primitive(type = "i32"))]
    pub preserve_final_sprite_frame: i32,
    /// Selects the alternate renderer used for this object.
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub alternate_render_mode: BooleanFlag,
    /// Activation-effect index passed to the engine's effect dispatcher.
    #[extractor(enum_from_i32_from_u8(type = "ActivationEffectId"))]
    pub activation_effect_id: ActivationEffectId,
    /// Reserved flag adjacent to [`Self::activation_effect_id`]. Preserve it verbatim.
    #[extractor(enum_from_i32_from_u8(type = "BooleanFlag"))]
    pub activation_effect_reserved_flag: BooleanFlag,
    /// Padding following the activation-effect fields.
    #[extractor(primitive(type = "i16"))]
    pub activation_effect_padding: i16,
    /// Enables the active-object overlay render path.
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub active_overlay_enabled: BooleanFlag,
    /// Whether this object is active in the map-object grid and update loop.
    #[extractor(enum_from_i32(type = "BooleanFlag"))]
    pub map_object_active: BooleanFlag,
}

pub fn read_extra_ref(source_path: &Path) -> std::io::Result<Vec<ExtraRef>> {
    ExtraRef::read_file(source_path)
}

pub fn save_extra_refs(conn: &mut Connection, file_id: i32, extra_refs: &[ExtraRef]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_extra_ref.sql"))?;
        for extra_ref in extra_refs {
            stmt.execute(params![
                file_id,                 // 1
                extra_ref.record_index,  // 2
                extra_ref.map_object_id, // 3
                if extra_ref.extra_definition_id > 0 {
                    Some(extra_ref.extra_definition_id)
                } else {
                    None
                }, // 5
                extra_ref.object_name,   // 6
                u8::from(extra_ref.object_type), // 7
                extra_ref.map_x,         // 8
                extra_ref.map_y,         // 9
                extra_ref.direction,     // 10
                extra_ref.direction_padding, // 11
                extra_ref.interaction_state, // 12
                i32::from(extra_ref.requires_key), // 13
                extra_ref.required_item.item_id() as i32, // 14
                u8::from(
                    extra_ref
                        .required_item
                        .item_type()
                        .unwrap_or(ItemTypeId::Other)
                ) as i32, // 15
                extra_ref.required_item.raw(), // 16 — raw
                extra_ref.requirement_range_1_padding, // 17
                extra_ref.required_item2.item_id() as i32, // 18
                u8::from(
                    extra_ref
                        .required_item2
                        .item_type()
                        .unwrap_or(ItemTypeId::Other)
                ) as i32, // 19
                extra_ref.required_item2.raw(), // 20 — raw
                // extra_ref.unknown5,       // 21
                extra_ref.requirement_range_2_start,  // 22
                extra_ref.requirement_range_2_end,    // 23
                extra_ref.requirement_range_3_start,  // 24
                extra_ref.requirement_range_3_end,    // 25
                extra_ref.gold_amount,                // 26
                extra_ref.loot_item.item_id() as i32, // 27
                u8::from(extra_ref.loot_item.item_type().unwrap_or(ItemTypeId::Other)) as i32, // 28
                extra_ref.loot_item.raw(),            // 29 — raw
                extra_ref.loot_item_padding,          // 30
                extra_ref.loot_item_count,            // 31
                extra_ref.additional_loot_1,          // 32
                extra_ref.additional_loot_1_count,    // 33
                extra_ref.additional_loot_2,          // 34
                extra_ref.additional_loot_2_count_and_config, // 35
                if extra_ref.interaction_event_id > 0 {
                    Some(extra_ref.interaction_event_id)
                } else {
                    None
                }, // 36
                if extra_ref.interaction_message_id > 0 {
                    Some(extra_ref.interaction_message_id)
                } else {
                    None
                }, // 37
                i32::from(extra_ref.footprint_width), // 38
                i32::from(extra_ref.footprint_height), // 39
                extra_ref.footprint_orientation,      // 40
                u8::from(extra_ref.interaction_range), // 41
                extra_ref.interaction_range_padding,  // 42
                i32::from(extra_ref.is_quest_element), // 43
                i32::from(extra_ref.post_activation_tile_flag), // 44
                i32::from(extra_ref.post_activation_footprint_mode), // 45
                extra_ref.preserve_final_sprite_frame, // 46
                i32::from(extra_ref.alternate_render_mode), // 47
                u8::from(extra_ref.activation_effect_id), // 48
                i32::from(extra_ref.activation_effect_reserved_flag), // 49
                extra_ref.activation_effect_padding,  // 50
                i32::from(extra_ref.active_overlay_enabled), // 51
                i32::from(extra_ref.map_object_active), // 52
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
        assert_eq!(refs[0].object_name, "Chest1");
        assert_eq!(refs[0].extra_definition_id, 3);
        assert_eq!(refs[0].map_x, 10);
        assert_eq!(refs[0].map_y, 20);
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
        assert_eq!(records[0].object_name, records2[0].object_name);
        assert_eq!(records[0].map_x, records2[0].map_x);
        assert_eq!(records[0].map_y, records2[0].map_y);
        assert_eq!(records[0].gold_amount, records2[0].gold_amount);
    }
}
