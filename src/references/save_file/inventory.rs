use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Stores information what has been equipped and which slots the items occupies in the inventory.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryPlacements {
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

impl InventoryPlacements {
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

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct InventoryMiscItem {
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String,
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String,
    pub base_price: u32,
    #[binary_record(size = 16)]
    pub unknown_1: Vec<u8>,
    pub misc_item_id: u32, // misc_item_id
    pub item_type_id: u16,
    pub unknown_4: u16, // 260
    pub unknown_5: u8,  // inventory position
    pub unknown_6: u8,  // inventory position
    pub unknown_7: u16, // 264
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct InventoryEventItem {
    #[binary_record(string(encoding = "WINDOWS-1250", size = 30))]
    pub name: String, // 30
    #[binary_record(string(encoding = "WINDOWS-1250", size = 202))]
    pub description: String, // 232
    pub base_price: u32,    // 236
    pub event_item_id: u32, // 240
    pub item_type_id: u8,   // inventory position 241
    pub unknown_3: u8,      // inventory position 242
    pub unknown_4: u16,     // 244
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
    pub item_type_id: u8, // inventory position 269
    pub unknown_5: u8,   // inventory position 270
    pub unknown_6: u16,  // 272
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
    pub unknown_1: u8,           // 250
    pub item_type_id: u16,       // 252
    pub position_index: u16,     // inventory position 254
    pub unknown_4: u8,           // inventory position 255
    pub unknown_5: u8,           // 6c 6c (108, 108) for the first row 256
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
    /// Per-save identity of this weapon inventory record, referenced by equipped slots.
    pub inventory_instance_id: u32, // 288
    pub unknown_2: u8,         // inventory position 289
    pub unknown_3: u8,         // inventory position 290
    pub unknown_4: u16,        // 292
}
