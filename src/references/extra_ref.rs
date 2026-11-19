use std::path::Path;

use crate::references::enums::{
    BooleanFlag, ExtraObjectType, InventoryItem, ItemTypeId, SmallRange0to3,
};
use crate::references::extractor::Extractor;
use dispel_macros::{Extractor, Localizable, RecordLayout, RecordPatcher};
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

/// Stores specific placements and configurations for interactive objects (chests, signs, doors) on a map.
///
/// Reads file: `ExtraInGame/Extdun01.ref (and other map-specific .ref files)`
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Default,
    Extractor,
    Localizable,
    RecordPatcher,
    RecordLayout,
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
    /// Quantity of [`Self::additional_loot_2`].
    #[extractor(primitive(type = "i32"))]
    pub additional_loot_2_count: i32,
    /// Loot delivery mode: `0` = deterministic (gold, then each loot pair),
    /// `1` = roll-gated.
    #[extractor(primitive(type = "i32"))]
    pub loot_delivery_mode: i32,
    /// Trap arming flag.
    ///
    /// Locked containers do not check this flag. It is cleared in every
    /// shipped placement.
    #[extractor(primitive(type = "i32"))]
    pub is_trapped: i32,
    /// `Magic.db` spell index used when a trap triggers.
    ///
    /// Zero in all shipped placements. This is distinct from the open-sound
    /// index in [`Self::open_sfx_id`].
    #[extractor(primitive(type = "i32"))]
    pub trap_spell_id: i32,
    /// Unassigned configuration word at record `+0x78`. Preserve verbatim.
    #[extractor(primitive(type = "i32"))]
    pub interaction_config_reserved_1: i32,
    /// Unassigned configuration word at record `+0x7C`. Preserve verbatim.
    #[extractor(primitive(type = "i32"))]
    pub interaction_config_reserved_2: i32,
    /// Remaining interaction uses for levers and buttons.
    ///
    /// Defaults to `999` when the placement data is loaded.
    #[extractor(primitive(type = "i32"))]
    pub use_counter: i32,
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
    /// Wave.ini sound index played when an object is activated successfully.
    /// `0` means no sound.
    ///
    /// This is a sound index, not a trap or spell id. It was previously
    /// exported as `activation_effect_id` and parsed through an enum that
    /// rewrote every index other than `0` and `10` to `255` on save. The trap
    /// fields are [`Self::is_trapped`] and [`Self::trap_spell_id`].
    #[extractor(primitive(type = "u8"))]
    pub open_sfx_id: u8,
    /// Reserved byte after [`Self::open_sfx_id`]. Preserve it verbatim.
    #[extractor(primitive(type = "u8"))]
    pub open_sfx_reserved: u8,
    /// Padding following [`Self::open_sfx_reserved`] (normally 0).
    #[extractor(primitive(type = "i16"))]
    pub open_sfx_padding: i16,
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
        let mut stmt = tx.prepare(crate::database::insert_sql("extra_refs"))?;
        for extra_ref in extra_refs {
            stmt.execute(params![
                file_id,                 // 1
                extra_ref.record_index,  // 2
                extra_ref.map_object_id, // 3
                if extra_ref.extra_definition_id > 0 {
                    Some(extra_ref.extra_definition_id)
                } else {
                    None
                }, // 4
                extra_ref.object_name,   // 5
                u8::from(extra_ref.object_type), // 6
                extra_ref.map_x,         // 7
                extra_ref.map_y,         // 8
                extra_ref.direction,     // 9
                extra_ref.direction_padding, // 10
                extra_ref.interaction_state, // 11
                i32::from(extra_ref.requires_key), // 12
                extra_ref.required_item.item_id() as i32, // 13
                u8::from(
                    extra_ref
                        .required_item
                        .item_type()
                        .unwrap_or(ItemTypeId::Other)
                ) as i32, // 14
                extra_ref.required_item.raw(), // 15 — raw
                extra_ref.requirement_range_1_padding, // 16
                extra_ref.required_item2.item_id() as i32, // 17
                u8::from(
                    extra_ref
                        .required_item2
                        .item_type()
                        .unwrap_or(ItemTypeId::Other)
                ) as i32, // 18
                extra_ref.required_item2.raw(), // 19 — raw
                extra_ref.requirement_range_2_start, // 20
                extra_ref.requirement_range_2_end, // 21
                extra_ref.requirement_range_3_start, // 22
                extra_ref.requirement_range_3_end, // 23
                extra_ref.gold_amount,   // 24
                extra_ref.loot_item.item_id() as i32, // 25
                u8::from(extra_ref.loot_item.item_type().unwrap_or(ItemTypeId::Other)) as i32, // 26
                extra_ref.loot_item.raw(), // 27 — raw
                extra_ref.loot_item_padding, // 28
                extra_ref.loot_item_count, // 29
                extra_ref.additional_loot_1, // 30
                extra_ref.additional_loot_1_count, // 31
                extra_ref.additional_loot_2, // 32
                extra_ref.additional_loot_2_count, // 33
                extra_ref.loot_delivery_mode, // 34
                extra_ref.is_trapped,    // 35
                extra_ref.trap_spell_id, // 36
                extra_ref.interaction_config_reserved_1, // 37
                extra_ref.interaction_config_reserved_2, // 38
                extra_ref.use_counter,   // 39
                if extra_ref.interaction_event_id > 0 {
                    Some(extra_ref.interaction_event_id)
                } else {
                    None
                }, // 40
                if extra_ref.interaction_message_id > 0 {
                    Some(extra_ref.interaction_message_id)
                } else {
                    None
                }, // 41
                i32::from(extra_ref.footprint_width), // 42
                i32::from(extra_ref.footprint_height), // 43
                extra_ref.footprint_orientation, // 44
                u8::from(extra_ref.interaction_range), // 45
                extra_ref.interaction_range_padding, // 46
                i32::from(extra_ref.is_quest_element), // 47
                i32::from(extra_ref.post_activation_tile_flag), // 48
                i32::from(extra_ref.post_activation_footprint_mode), // 49
                extra_ref.preserve_final_sprite_frame, // 50
                i32::from(extra_ref.alternate_render_mode), // 51
                extra_ref.open_sfx_id,   // 52
                extra_ref.open_sfx_reserved, // 53
                extra_ref.open_sfx_padding, // 54
                i32::from(extra_ref.active_overlay_enabled), // 55
                i32::from(extra_ref.map_object_active)  // 56
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

    /// The open-SFX byte is a Wave.ini index (0..N), not an effect enum.
    /// A byte of 251 used to be rewritten to 255 by the old enum parser.
    #[test]
    fn test_open_sfx_byte_round_trip_preserved() {
        let mut rec = ref_bytes("Chest1", 10, 20, 50);
        rec[0x70..0x74].copy_from_slice(&1i32.to_le_bytes()); // is_trapped
        rec[0x74..0x78].copy_from_slice(&27i32.to_le_bytes()); // trap_spell_id
        rec[0x80..0x84].copy_from_slice(&999i32.to_le_bytes()); // use_counter
        rec[0xac] = 251; // open_sfx_id

        let mut data = 1i32.to_le_bytes().to_vec();
        data.extend(rec);

        let mut c = Cursor::new(&data[..]);
        let records = ExtraRef::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(records[0].is_trapped, 1);
        assert_eq!(records[0].trap_spell_id, 27);
        assert_eq!(records[0].use_counter, 999);
        assert_eq!(records[0].open_sfx_id, 251);

        let mut out = Vec::new();
        ExtraRef::to_writer(&records, &mut out).unwrap();
        assert_eq!(
            out, data,
            "trap fields and open-SFX byte must round-trip verbatim"
        );
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
