use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Information what is equipped, which slots the items in inventory are occupied.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventorySlots {
    /// Equipped weapon items — 12 slots × 9 bytes = 108 bytes.
    pub equipped_equipment: Vec<EquipmentSlot>,
    /// Belt item placements — 6 cells × 16 bytes = 96 bytes.
    pub belt_potions: Vec<BeltPotionSlot>,
    /// Inventory item placements — 3 pages × 7 columns × 9 cells × 20 bytes.
    pub inventory_placement: Vec<InventoryPlacementEntry>,
}

/// Equipped weapon items: 12 slots × 9 bytes = 108 bytes.
pub const EQUIPPED_ITEM_BYTES: usize = 12 * 9;

/// Belt item placements: 6 cells × 16 bytes = 96 bytes.
pub const BELT_BYTES_SIZE: usize = 6 * 16;

/// Inventory placement: 3 pages × 7 columns × 9 cells × 20 bytes = 3780 bytes.
pub const INVENTORY_BYTES_SIZE: usize = 3 * 7 * 9 * 20;
pub(super) const INVENTORY_SLOTS_SIZE: usize =
    EQUIPPED_ITEM_BYTES + BELT_BYTES_SIZE + INVENTORY_BYTES_SIZE;
pub(super) const INVENTORY_EVENT_ITEM_SIZE: usize = 244;
pub(super) const INVENTORY_MISC_ITEM_SIZE: usize = 264;
pub(super) const INVENTORY_EDIT_ITEM_SIZE: usize = 272;
pub(super) const INVENTORY_WEAPON_ITEM_SIZE: usize = 292;
pub(super) const INVENTORY_HEAL_ITEM_SIZE: usize = 256;

impl InventorySlots {
    pub(super) fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut data = [0u8; INVENTORY_SLOTS_SIZE];
        reader.read_exact(&mut data)?;
        Self::parse(&data)
    }

    pub(crate) fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut reader = std::io::Cursor::new(data);

        // Equipped weapon items: 12 slots × 9 bytes = 108 bytes.
        let mut equipment_raw = vec![0u8; 12 * 9];
        reader.read_exact(&mut equipment_raw)?;
        let equipped_equipment: Vec<EquipmentSlot> = equipment_raw
            .chunks_exact(9)
            .map(|chunk| {
                let mut c = std::io::Cursor::new(chunk);
                EquipmentSlot {
                    panel_slot_marker: c.read_u8().unwrap(),
                    weapon_catalog_index: c.read_i32::<LittleEndian>().unwrap(),
                    weapon_inventory_instance_id: c.read_i32::<LittleEndian>().unwrap(),
                }
            })
            .collect();

        // Belt item placements: 6 cells × 16 bytes = 96 bytes.
        let mut belt_raw = vec![0u8; 6 * 16];
        reader.read_exact(&mut belt_raw)?;
        let belt_potions: Vec<BeltPotionSlot> = belt_raw
            .chunks_exact(16)
            .map(|chunk| {
                let mut c = std::io::Cursor::new(chunk);
                BeltPotionSlot {
                    item_category: c.read_i32::<LittleEndian>().unwrap(),
                    item_catalog_index: c.read_i32::<LittleEndian>().unwrap(),
                    icon_x: c.read_i32::<LittleEndian>().unwrap(),
                    icon_y: c.read_i32::<LittleEndian>().unwrap(),
                }
            })
            .collect();

        // Inventory placement: 3 pages × 7 columns × 9 cells × 20 bytes.
        let mut inventory_raw = vec![0u8; 189 * 20];
        reader.read_exact(&mut inventory_raw)?;
        let inventory_placement: Vec<InventoryPlacementEntry> = inventory_raw
            .chunks_exact(20)
            .map(|chunk| {
                let mut c = std::io::Cursor::new(chunk);
                InventoryPlacementEntry {
                    item_category: c.read_i32::<LittleEndian>().unwrap(),
                    item_catalog_index: c.read_i32::<LittleEndian>().unwrap(),
                    icon_x: c.read_i32::<LittleEndian>().unwrap(),
                    icon_y: c.read_i32::<LittleEndian>().unwrap(),
                    item_instance_index: c.read_i32::<LittleEndian>().unwrap(),
                }
            })
            .collect();

        Ok(Self {
            inventory_placement,
            belt_potions,
            equipped_equipment,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        // Equipped weapon items: 12 slots × 9 bytes = 108 bytes.
        for slot in &self.equipped_equipment {
            writer.write_u8(slot.panel_slot_marker)?;
            writer.write_i32::<LittleEndian>(slot.weapon_catalog_index)?;
            writer.write_i32::<LittleEndian>(slot.weapon_inventory_instance_id)?;
        }

        // Belt item placements: 6 cells × 16 bytes = 96 bytes.
        for slot in &self.belt_potions {
            writer.write_i32::<LittleEndian>(slot.item_category)?;
            writer.write_i32::<LittleEndian>(slot.item_catalog_index)?;
            writer.write_i32::<LittleEndian>(slot.icon_x)?;
            writer.write_i32::<LittleEndian>(slot.icon_y)?;
        }

        // Inventory placement: 3 pages × 7 columns × 9 cells × 20 bytes.
        for entry in &self.inventory_placement {
            writer.write_i32::<LittleEndian>(entry.item_category)?;
            writer.write_i32::<LittleEndian>(entry.item_catalog_index)?;
            writer.write_i32::<LittleEndian>(entry.icon_x)?;
            writer.write_i32::<LittleEndian>(entry.icon_y)?;
            writer.write_i32::<LittleEndian>(entry.item_instance_index)?;
        }

        Ok(())
    }
}

/// One equipped weapon-item reference (9 bytes).
///
/// Part of the 12-slot equipment array (12 × 9 = 108 bytes total).
/// An empty entry has catalog index `100` and panel marker `0xff`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EquipmentSlot {
    /// Equipment-panel marker used by the game to restore this slot's UI state; `0xff` is empty.
    pub panel_slot_marker: u8,
    /// Zero-based index in the weapon-item catalog; `100` is empty.
    pub weapon_catalog_index: i32,
    /// `InventoryWeaponItem::inventory_instance_id` of the equipped weapon; zero is empty.
    pub weapon_inventory_instance_id: i32,
}

/// One belt item placement cell (16 bytes).
///
/// Part of the 6-cell belt array (6 × 16 = 96 bytes total). Larger items can
/// occupy consecutive cells with the same catalog index and icon position.
/// Empty cells use category `10` and catalog index `100`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BeltPotionSlot {
    /// Item category; the belt uses category `1` for an occupied item.
    pub item_category: i32,
    /// Zero-based index in that category's catalog; `100` is empty.
    pub item_catalog_index: i32,
    /// Horizontal pixel coordinate at which the belt icon is drawn.
    pub icon_x: i32,
    /// Vertical pixel coordinate at which the belt icon is drawn.
    pub icon_y: i32,
}

/// One item reference and its position in the inventory placement grid (20 bytes).
///
/// The grid is serialized as three pages, each with seven 9-cell columns:
/// `[3 pages][7 columns][9 cells]`. Empty cells use category `10` and catalog
/// index `100`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryPlacementEntry {
    /// Zero-based item category used to select an item collection; `10` marks an empty cell.
    pub item_category: i32,
    /// Zero-based index of the item's definition within `item_category`; `100` marks an empty cell.
    pub item_catalog_index: i32,
    /// Horizontal pixel coordinate at which the inventory icon is drawn.
    pub icon_x: i32,
    /// Vertical pixel coordinate at which the inventory icon is drawn.
    pub icon_y: i32,
    /// Category-local index of the instantiated inventory item represented by this placement.
    pub item_instance_index: i32,
}

/// Raw inventory data from a save file (5 item categories).
///
/// Each category stores count-prefixed raw records of a fixed size.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryData {
    /// Event-type items (count × 244 bytes each)
    pub event_items: Vec<InventoryEventItem>,
    /// Misc-type items (count × 264 bytes each)
    pub misc_items: Vec<InventoryMiscItem>,
    /// Edit-type items (count × 272 bytes each)
    pub edit_items: Vec<InventoryEditItem>,
    /// Weapon-type items (count × 292 bytes each)
    pub weapon_items: Vec<InventoryWeaponItem>,
    /// Heal-type items (count × 256 bytes each)
    pub heal_items: Vec<InventoryHealItem>,
}

impl InventoryData {
    pub(super) fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        Ok(Self {
            event_items: read_item_section(
                reader,
                INVENTORY_EVENT_ITEM_SIZE,
                InventoryEventItem::parse,
            )?,
            misc_items: read_item_section(
                reader,
                INVENTORY_MISC_ITEM_SIZE,
                InventoryMiscItem::parse,
            )?,
            edit_items: read_item_section(
                reader,
                INVENTORY_EDIT_ITEM_SIZE,
                InventoryEditItem::parse,
            )?,
            weapon_items: read_item_section(
                reader,
                INVENTORY_WEAPON_ITEM_SIZE,
                InventoryWeaponItem::parse,
            )?,
            heal_items: read_item_section(
                reader,
                INVENTORY_HEAL_ITEM_SIZE,
                InventoryHealItem::parse,
            )?,
        })
    }

    pub(super) fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        write_item_section(writer, &self.event_items, |item, writer| item.write(writer))?;
        write_item_section(writer, &self.misc_items, |item, writer| item.write(writer))?;
        write_item_section(writer, &self.edit_items, |item, writer| item.write(writer))?;
        write_item_section(writer, &self.weapon_items, |item, writer| {
            item.write(writer)
        })?;
        write_item_section(writer, &self.heal_items, |item, writer| item.write(writer))
    }
}

fn read_item_section<R: Read, T>(
    reader: &mut R,
    record_size: usize,
    parse: fn(&[u8]) -> std::io::Result<T>,
) -> std::io::Result<Vec<T>> {
    let count = reader.read_u16::<LittleEndian>()? as usize;
    let mut data = vec![0u8; count * record_size];
    reader.read_exact(&mut data)?;
    data.chunks_exact(record_size).map(parse).collect()
}

fn write_item_section<W: Write, T>(
    writer: &mut W,
    records: &[T],
    mut write: impl FnMut(&T, &mut W) -> std::io::Result<()>,
) -> std::io::Result<()> {
    writer.write_u16::<LittleEndian>(records.len() as u16)?;
    for record in records {
        write(record, writer)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct InventoryMiscItem {
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String,
    pub base_price: u32,
    /// X coordinate of a Rune Stone mark. Zero when no mark is stored.
    pub rune_mark_x: i32,
    /// Y coordinate of a Rune Stone mark. Zero when no mark is stored.
    pub rune_mark_y: i32,
    /// Map ID of a Rune Stone mark. Zero when no mark is stored.
    pub rune_mark_map_id: u32,
    /// Reserved bytes after the mark data. They are zero in known saves.
    #[binary_record(size = 4)]
    pub reserved_mark_tail_bytes: Vec<u8>,
    /// Zero-based index of the corresponding miscellaneous-item definition.
    pub misc_item_id: u32,
    /// Zero-based inventory category: `0`=weapon, `1`=heal, `2`=edit, `3`=misc, `4`=event.
    pub item_category: u16,
    /// Zero-based index of this record in the save's miscellaneous-item array.
    pub inventory_record_index: u16,
    /// Per-save item identity referenced by inventory placement cells.
    pub inventory_instance_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct InventoryEventItem {
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String, // 30
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String, // 232
    pub base_price: u32,    // 236
    pub event_item_id: u32, // 240
    /// Zero-based inventory category: `0`=weapon, `1`=heal, `2`=edit, `3`=misc, `4`=event.
    pub item_category: u8,
    /// Alignment byte following the category; observed as zero.
    pub item_category_padding: u8,
    /// Zero-based index of this record in the save's event-item array.
    pub inventory_record_index: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct InventoryEditItem {
    // 272
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String, // 30
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String, // 232
    pub base_price: u32, // 236
    /// Runtime index of the corresponding EditItem database definition.
    pub edit_item_id: u32, // 240
    pub health_points: i16, // 242
    pub mana_points: i16, // 244
    pub strength: i16,   // 246
    pub agility: i16,    // 248
    pub wisdom: i16,     // 250
    pub constitution: i16, // 252
    pub to_dodge: i16,   // 254
    pub to_hit: i16,     // 256
    pub offense: i16,    // 258
    pub defense: i16,    // 260
    pub magical_power: i16, // 262
    pub modification_resistance: i16, // 264
    /// Reserved byte; observed as zero and not used by the game.
    pub reserved_byte: u8, // 265
    pub modifies_item: u8, // 266
    pub additional_effect: i16, // 268
    /// Zero-based inventory category: `0`=weapon, `1`=heal, `2`=edit, `3`=misc, `4`=event.
    pub item_category: u8,
    /// Alignment byte following the category; observed as zero.
    pub item_category_padding: u8,
    /// Zero-based index of this record in the save's edit-item array.
    pub inventory_record_index: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct InventoryHealItem {
    // 256
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String, // 232
    pub base_price: u32,         // 236
    pub heal_item_id: u32,       // 240
    pub health_points: i16,      // 242
    pub mana_points: i16,        // 244
    pub restore_full_health: u8, // 245
    pub restore_full_mana: u8,   // 246
    pub poison_heal: u8,         // 247
    pub petrif_heal: u8,         // 248
    pub polimorph_heal: u8,      // 249
    /// First byte of the item definition's reserved trailer; normally zero.
    pub reserved_definition_byte: u8,
    /// Zero-based inventory category: `0`=weapon, `1`=heal, `2`=edit, `3`=misc, `4`=event.
    pub item_category: u16,
    /// Zero-based index of this record in the save's heal-item array.
    pub inventory_record_index: u16,
    /// Runtime scratch bytes. They are normally zero but are not initialized consistently.
    pub reserved_runtime_bytes: [u8; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct InventoryWeaponItem {
    // 292
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String, // 232
    pub base_price: u32,       // 236
    pub weapon_item_id: u32,   // 240
    pub health_points: i16,    // 242
    pub mana_points: i16,      // 244
    pub strength: i16,         // 246
    pub agility: i16,          // 248
    pub wisdom: i16,           // 250
    pub constitution: i16,     // 252
    pub to_dodge: i16,         // 254
    pub to_hit: i16,           // 256
    pub attack: i16,           // 258
    pub defense: i16,          // 260
    pub magical_strength: i16, // 262
    pub durability: i16,       // 264
    pub padding2: i16,         // 266
    pub padding3: i16,         // 268
    pub req_strength: i16,     // 270
    pub padding4: i16,         // 272
    pub req_agility: i16,      // 274
    pub padding5: i16,         // 276
    pub req_wisdom: i16,       // 278
    pub padding6: i16,         // 280
    pub padding7: i16,         // 282
    pub padding8: i16,         // 284
    /// Zero-based inventory category: `0`=weapon, `1`=heal, `2`=edit, `3`=misc, `4`=event.
    pub item_category: u32,
    /// Per-save item identity referenced by equipped slots and inventory placement cells.
    pub inventory_instance_id: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_round_trip<T>(
        bytes: &[u8],
        parse: fn(&[u8]) -> std::io::Result<T>,
        write: fn(&T, &mut Vec<u8>) -> std::io::Result<()>,
    ) {
        let record = parse(bytes).unwrap();
        let mut encoded = Vec::new();
        write(&record, &mut encoded).unwrap();
        assert_eq!(encoded, bytes);
    }

    #[test]
    fn test_inventory_runtime_fields_parse_at_their_binary_offsets() {
        let mut misc = vec![0; INVENTORY_MISC_ITEM_SIZE];
        misc[252..256].copy_from_slice(&17u32.to_le_bytes());
        misc[256..258].copy_from_slice(&3u16.to_le_bytes());
        misc[258..260].copy_from_slice(&7u16.to_le_bytes());
        misc[260..264].copy_from_slice(&0x1234_5678u32.to_le_bytes());
        let parsed_misc = InventoryMiscItem::parse(&misc).unwrap();
        assert_eq!(parsed_misc.misc_item_id, 17);
        assert_eq!(parsed_misc.item_category, 3);
        assert_eq!(parsed_misc.inventory_record_index, 7);
        assert_eq!(parsed_misc.inventory_instance_id, 0x1234_5678);
        assert_round_trip(&misc, InventoryMiscItem::parse, InventoryMiscItem::write);

        let mut event = vec![0; INVENTORY_EVENT_ITEM_SIZE];
        event[240] = 4;
        event[241] = 0xaa;
        event[242..244].copy_from_slice(&9u16.to_le_bytes());
        let parsed_event = InventoryEventItem::parse(&event).unwrap();
        assert_eq!(parsed_event.item_category, 4);
        assert_eq!(parsed_event.item_category_padding, 0xaa);
        assert_eq!(parsed_event.inventory_record_index, 9);
        assert_round_trip(&event, InventoryEventItem::parse, InventoryEventItem::write);

        let mut edit = vec![0; INVENTORY_EDIT_ITEM_SIZE];
        edit[268] = 2;
        edit[269] = 0xbb;
        edit[270..272].copy_from_slice(&11u16.to_le_bytes());
        let parsed_edit = InventoryEditItem::parse(&edit).unwrap();
        assert_eq!(parsed_edit.item_category, 2);
        assert_eq!(parsed_edit.item_category_padding, 0xbb);
        assert_eq!(parsed_edit.inventory_record_index, 11);
        assert_round_trip(&edit, InventoryEditItem::parse, InventoryEditItem::write);

        let mut heal = vec![0; INVENTORY_HEAL_ITEM_SIZE];
        heal[249] = 0xcc;
        heal[250..252].copy_from_slice(&1u16.to_le_bytes());
        heal[252..254].copy_from_slice(&13u16.to_le_bytes());
        heal[254..256].copy_from_slice(&[0x6c, 0x6c]);
        let parsed_heal = InventoryHealItem::parse(&heal).unwrap();
        assert_eq!(parsed_heal.reserved_definition_byte, 0xcc);
        assert_eq!(parsed_heal.item_category, 1);
        assert_eq!(parsed_heal.inventory_record_index, 13);
        assert_eq!(parsed_heal.reserved_runtime_bytes, [0x6c, 0x6c]);
        assert_round_trip(&heal, InventoryHealItem::parse, InventoryHealItem::write);

        let mut weapon = vec![0; INVENTORY_WEAPON_ITEM_SIZE];
        weapon[284..288].copy_from_slice(&0u32.to_le_bytes());
        weapon[288..292].copy_from_slice(&5152u32.to_le_bytes());
        let parsed_weapon = InventoryWeaponItem::parse(&weapon).unwrap();
        assert_eq!(parsed_weapon.item_category, 0);
        assert_eq!(parsed_weapon.inventory_instance_id, 5152);
        assert_round_trip(
            &weapon,
            InventoryWeaponItem::parse,
            InventoryWeaponItem::write,
        );
    }
}
