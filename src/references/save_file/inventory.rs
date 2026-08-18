use serde::{Deserialize, Serialize};
use dispel_macros::BinaryRecord;

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

