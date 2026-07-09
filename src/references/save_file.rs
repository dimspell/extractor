// Save file extraction and parsing for Dispel RPG
//
// This module provides comprehensive parsing of Dispel RPG save files (.sav)
// following the binary format documented in SAVE_FILE_RESEARCH.md

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::WINDOWS_1250;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};
// use proptest::char::range;
use super::extractor::{read_null_terminated_windows_1250, Extractor};

/// Monster state flags
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonsterState {
    pub is_dead: bool,
    pub is_poisoned: bool,
    pub is_burning: bool,
    pub is_frozen: bool,
    pub is_stunned: bool,
    pub is_invisible: bool,
    pub is_boss: bool,
}

impl MonsterState {
    /// Parse monster state from flags field
    pub fn parse(flags: u32) -> Self {
        MonsterState {
            is_dead: flags & 1 != 0,
            is_poisoned: flags & 2 != 0,
            is_burning: flags & 4 != 0,
            is_frozen: flags & 8 != 0,
            is_stunned: flags & 16 != 0,
            is_invisible: flags & 32 != 0,
            is_boss: flags & (1 << 31) != 0,
        }
    }
}

/// Monster record from save file (surface or dungeon)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonsterRecord {
    pub signature_a: u32,
    pub record_index: u32,
    pub signature_b: u32,
    pub name: String,
    pub hp_current: u16,
    pub hp_maximum: u16,
    pub state: MonsterState,
    pub tile_x: u16,
    pub tile_y: u16,
    pub pixel_x: u16,
    pub pixel_y: u16,
    pub facing_direction: u8,
    pub experience_value: u32,
    pub attack_power: u16,
    pub defense: u16,
    pub magic_attack: u16,
    pub magic_defense: u16,
    pub agility: u16,
    pub luck: u16,
}

impl MonsterRecord {
    /// Parse monster record from 329-byte data
    ///
    /// Record layout: 3×u32 (12 bytes) + 24-byte name + stats (293 bytes)
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut reader = std::io::Cursor::new(data);

        let signature_a = reader.read_u32::<LittleEndian>()?;
        let record_index = reader.read_u32::<LittleEndian>()?;
        let signature_b = reader.read_u32::<LittleEndian>()?;

        // Name: 24 bytes fixed-size field, null-terminated WINDOWS-1250
        let mut name_raw = [0u8; 24];
        reader.read_exact(&mut name_raw)?;
        let name_len = name_raw.iter().position(|&b| b == 0).unwrap_or(24);
        let (name, _, _) = WINDOWS_1250.decode(&name_raw[..name_len]);
        let name = name.to_string();

        // Stats block (293 bytes remaining)
        let hp_current = reader.read_u16::<LittleEndian>()?;
        let hp_maximum = reader.read_u16::<LittleEndian>()?;
        let state = MonsterState::parse(reader.read_u32::<LittleEndian>()?);
        let tile_x = reader.read_u16::<LittleEndian>()?;
        let tile_y = reader.read_u16::<LittleEndian>()?;
        let pixel_x = reader.read_u16::<LittleEndian>()?;
        let pixel_y = reader.read_u16::<LittleEndian>()?;
        let facing_direction = reader.read_u8()?;

        // Skip 3 bytes padding
        let _ = reader.read_u8()?;
        let _ = reader.read_u8()?;
        let _ = reader.read_u8()?;

        let experience_value = reader.read_u32::<LittleEndian>()?;
        let attack_power = reader.read_u16::<LittleEndian>()?;
        let defense = reader.read_u16::<LittleEndian>()?;
        let magic_attack = reader.read_u16::<LittleEndian>()?;
        let magic_defense = reader.read_u16::<LittleEndian>()?;
        let agility = reader.read_u16::<LittleEndian>()?;
        let luck = reader.read_u16::<LittleEndian>()?;

        Ok(MonsterRecord {
            signature_a,
            record_index,
            signature_b,
            name,
            hp_current,
            hp_maximum,
            state,
            tile_x,
            tile_y,
            pixel_x,
            pixel_y,
            facing_direction,
            experience_value,
            attack_power,
            defense,
            magic_attack,
            magic_defense,
            agility,
            luck,
        })
    }
}

/// NPC record from save file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NpcRecord {
    pub name: String,
    pub role_description: String,
    pub unknown1: u32,
    pub unknown2: u32,
    pub unknown3: u32,
    pub unknown4: u16,
    pub unknown5: u16,
    pub unknown6: u16,
    pub unknown7: u16,
    pub unknown8: u16,
    pub unknown9: u16,
    pub unknown10: u16,
    pub unknown11: u16,
    pub unknown12: [u8; 15],
    pub npc_ini_id: u8,
    pub unknown13: [u8; 20],
    pub npc_ref_party_script_id: u16,
    pub npc_ref_show_on_event_id: u16,
    pub unknown14: u8,
    pub npc_ref_unknown_1: u8,
    pub npc_ref_waypoint1filled: u32,
    pub npc_ref_waypoint1x: u32,
    pub npc_ref_waypoint1y: u32,
    pub npc_ref_unknown_2: u32,
    pub npc_ref_look_direction: u32,
    pub npc_ref_unknown_9: u32,
    pub npc_ref_waypoint2filled: u32,
    pub npc_ref_waypoint2x: u32,
    pub npc_ref_waypoint2y: u32,
    pub npc_ref_unknown_3: u32,
    pub npc_ref_unknown_6: u32,
    pub npc_ref_unknown_10: u32,
    pub npc_ref_waypoint3filled: u32,
    pub npc_ref_waypoint3x: u32,
    pub npc_ref_waypoint3y: u32,
    pub npc_ref_unknown_4: u32,
    pub npc_ref_unknown_7: u32,
    pub npc_ref_unknown_11: u32,
    pub npc_ref_waypoint4filled: u32,
    pub npc_ref_waypoint4x: u32,
    pub npc_ref_waypoint4y: u32,
    pub npc_ref_unknown_5: u32,
    pub npc_ref_unknown_8: u32,
    pub npc_ref_unknown_12: u32,
    pub npc_ref_unknown_13: u32,
    pub npc_ref_unknown_14: u32,
    pub npc_ref_unknown_15: u32,
    pub npc_ref_unknown_16: u32,
    pub npc_ref_unknown_17: u32,
    pub unknown15: u16,
    pub npc_ref_dialog_id: u32,
    pub unknown16: [u8; 29],
}

impl NpcRecord {
    /// Parse NPC record from 349-byte data
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut reader = std::io::Cursor::new(data);

        // Name: 64 bytes fixed-size field, null-terminated WINDOWS-1250
        let mut name_raw = [0u8; 64];
        reader.read_exact(&mut name_raw)?;
        let name_len = name_raw.iter().position(|&b| b == 0).unwrap_or(32);
        let (name, _, _) = WINDOWS_1250.decode(&name_raw[..name_len]);
        let name = name.to_string();

        // Role description: 64 bytes fixed-size field, null-terminated WINDOWS-1250
        let mut role_raw = [0u8; 64];
        reader.read_exact(&mut role_raw)?;
        let role_len = role_raw.iter().position(|&b| b == 0).unwrap_or(40);
        let (role_desc, _, _) = WINDOWS_1250.decode(&role_raw[..role_len]);
        let role_description = role_desc.to_string();

        let unknown1 = reader.read_u32::<LittleEndian>()?;
        let unknown2 = reader.read_u32::<LittleEndian>()?;
        let unknown3 = reader.read_u32::<LittleEndian>()?;

        let unknown4 = reader.read_u16::<LittleEndian>()?;
        let unknown5 = reader.read_u16::<LittleEndian>()?;
        let unknown6 = reader.read_u16::<LittleEndian>()?;
        let unknown7 = reader.read_u16::<LittleEndian>()?;
        let unknown8 = reader.read_u16::<LittleEndian>()?;
        let unknown9 = reader.read_u16::<LittleEndian>()?;
        let unknown10 = reader.read_u16::<LittleEndian>()?;
        let unknown11 = reader.read_u16::<LittleEndian>()?;

        let mut unknown12 = [0u8; 15];
        reader.read_exact(&mut unknown12)?;

        let npc_ini_id = reader.read_u8()?;

        let mut unknown13 = [0u8; 20];
        reader.read_exact(&mut unknown13)?;

        let npc_ref_party_script_id = reader.read_u16::<LittleEndian>()?;
        let npc_ref_show_on_event_id = reader.read_u16::<LittleEndian>()?;

        let unknown14 = reader.read_u8()?;

        let npc_ref_unknown_1 = reader.read_u8()?;

        let npc_ref_waypoint1filled = reader.read_u32::<LittleEndian>()?;
        let npc_ref_waypoint1x = reader.read_u32::<LittleEndian>()?;
        let npc_ref_waypoint1y = reader.read_u32::<LittleEndian>()?;

        let npc_ref_unknown_2 = reader.read_u32::<LittleEndian>()?;
        let npc_ref_look_direction = reader.read_u32::<LittleEndian>()?;
        let npc_ref_unknown_9 = reader.read_u32::<LittleEndian>()?;

        let npc_ref_waypoint2filled = reader.read_u32::<LittleEndian>()?;
        let npc_ref_waypoint2x = reader.read_u32::<LittleEndian>()?;
        let npc_ref_waypoint2y = reader.read_u32::<LittleEndian>()?;

        let npc_ref_unknown_3 = reader.read_u32::<LittleEndian>()?;
        let npc_ref_unknown_6 = reader.read_u32::<LittleEndian>()?;
        let npc_ref_unknown_10 = reader.read_u32::<LittleEndian>()?;

        let npc_ref_waypoint3filled = reader.read_u32::<LittleEndian>()?;
        let npc_ref_waypoint3x = reader.read_u32::<LittleEndian>()?;
        let npc_ref_waypoint3y = reader.read_u32::<LittleEndian>()?;

        let npc_ref_unknown_4 = reader.read_u32::<LittleEndian>()?;
        let npc_ref_unknown_7 = reader.read_u32::<LittleEndian>()?;
        let npc_ref_unknown_11 = reader.read_u32::<LittleEndian>()?;

        let npc_ref_waypoint4filled = reader.read_u32::<LittleEndian>()?;
        let npc_ref_waypoint4x = reader.read_u32::<LittleEndian>()?;
        let npc_ref_waypoint4y = reader.read_u32::<LittleEndian>()?;

        let npc_ref_unknown_5 = reader.read_u32::<LittleEndian>()?;
        let npc_ref_unknown_8 = reader.read_u32::<LittleEndian>()?;
        let npc_ref_unknown_12 = reader.read_u32::<LittleEndian>()?;

        let npc_ref_unknown_13 = reader.read_u32::<LittleEndian>()?;
        let npc_ref_unknown_14 = reader.read_u32::<LittleEndian>()?;
        let npc_ref_unknown_15 = reader.read_u32::<LittleEndian>()?;
        let npc_ref_unknown_16 = reader.read_u32::<LittleEndian>()?;
        let npc_ref_unknown_17 = reader.read_u32::<LittleEndian>()?;

        let unknown15 = reader.read_u16::<LittleEndian>()?;

        let npc_ref_dialog_id = reader.read_u32::<LittleEndian>()?;

        let mut unknown16 = [0u8; 29];
        reader.read_exact(&mut unknown16)?;

        Ok(NpcRecord {
            name,
            role_description,
            unknown1,
            unknown2,
            unknown3,
            unknown4,
            unknown5,
            unknown6,
            unknown7,
            unknown8,
            unknown9,
            unknown10,
            unknown11,
            unknown12,
            npc_ini_id,
            unknown13,
            npc_ref_party_script_id,
            npc_ref_show_on_event_id,
            unknown14,
            npc_ref_unknown_1,
            npc_ref_waypoint1filled,
            npc_ref_waypoint1x,
            npc_ref_waypoint1y,
            npc_ref_unknown_2,
            npc_ref_look_direction,
            npc_ref_unknown_9,
            npc_ref_waypoint2filled,
            npc_ref_waypoint2x,
            npc_ref_waypoint2y,
            npc_ref_unknown_3,
            npc_ref_unknown_6,
            npc_ref_unknown_10,
            npc_ref_waypoint3filled,
            npc_ref_waypoint3x,
            npc_ref_waypoint3y,
            npc_ref_unknown_4,
            npc_ref_unknown_7,
            npc_ref_unknown_11,
            npc_ref_waypoint4filled,
            npc_ref_waypoint4x,
            npc_ref_waypoint4y,
            npc_ref_unknown_5,
            npc_ref_unknown_8,
            npc_ref_unknown_12,
            npc_ref_unknown_13,
            npc_ref_unknown_14,
            npc_ref_unknown_15,
            npc_ref_unknown_16,
            npc_ref_unknown_17,
            unknown15,
            npc_ref_dialog_id,
            unknown16,
        })
        // Not recognized from the NPC-Ref
        // Unknown 18:
        // Unknown Item:
        // Unknown 19:
        // Face Sprite ID:
    }
}

/// Extra object record (surface or dungeon)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtraObjectRecord {
    pub prefix: u8,
    pub name: String,
    pub state: u8, // For chests: 1=open, 2=closed, 3=locked
}

impl ExtraObjectRecord {
    /// Parse extra object record from 200-byte data
    pub fn parse(data: &[u8], is_dungeon: bool) -> std::io::Result<Self> {
        let mut reader = std::io::Cursor::new(data);

        let prefix_offset = if is_dungeon { 10 } else { 14 };
        let name_offset = if is_dungeon { 11 } else { 15 };

        // Read prefix
        reader.seek(std::io::SeekFrom::Start(prefix_offset as u64))?;
        let prefix = reader.read_u8()?;

        // Skip to name
        reader.seek(std::io::SeekFrom::Start(name_offset as u64))?;

        // Read name (null-terminated WINDOWS-1250)
        let mut name_bytes = Vec::new();
        loop {
            let byte = reader.read_u8()?;
            if byte == 0 {
                break;
            }
            name_bytes.push(byte);
        }
        let (name, _, _) = WINDOWS_1250.decode(&name_bytes);

        // Read state (for chests)
        let state = if name.trim() == "Skrzynia" {
            reader.read_u8()?
        } else {
            0
        };

        Ok(ExtraObjectRecord {
            prefix,
            name: name.to_string(),
            state,
        })
    }
}

/// Event script record (save file format: 284 bytes each)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventScript {
    pub state: u8,
    pub script_name: String,
}

impl EventScript {
    /// Parse event script from 284-byte save file record
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut reader = std::io::Cursor::new(data);

        // Skip event_id (u32) and unknown (u32)
        let _event_id = reader.read_u32::<LittleEndian>()?;
        let _unknown = reader.read_u32::<LittleEndian>()?;

        // Read state (u32, but we use u8 for compatibility)
        let state_val = reader.read_u32::<LittleEndian>()?;
        let state = if state_val >= 2 { 2 } else { state_val as u8 };

        // Read script name: 272 bytes null-terminated ASCII
        let mut name_bytes = Vec::new();
        for _ in 0..272 {
            let byte = reader.read_u8()?;
            if byte == 0 {
                break;
            }
            name_bytes.push(byte);
        }
        let name = String::from_utf8(name_bytes).unwrap_or_else(|_| String::from("unknown.scr"));

        Ok(EventScript {
            state,
            script_name: name,
        })
    }
}

/// Journal entry (37 bytes each)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JournalEntry {
    pub index: u8,     // 1 byte
    pub name: String,  // 24 bytes null-terminated WINDOWS-1250
    pub rest: Vec<u8>, // 12 bytes
}

impl JournalEntry {
    /// Parse journal entry from 37-byte data
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        if data.len() < 37 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "JournalEntry requires 37 bytes",
            ));
        }

        let index = data[0]; // 1

        // Name: 24 bytes null-terminated WINDOWS-1250
        let name = {
            let mut name_bytes = Vec::new();
            for &byte in data[1..25].iter() {
                if byte == 0 {
                    break;
                }
                name_bytes.push(byte);
            }
            let (name, _, _) = WINDOWS_1250.decode(&name_bytes);
            name.to_string()
        }; // 24

        let mut rest = vec![0u8; 12];
        rest.copy_from_slice(&data[25..37]); // 12

        Ok(JournalEntry { index, name, rest })
    }

    /// Serialize journal entry to binary
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_u8(self.index)?;

        // Write name (24 bytes, null-padded WINDOWS-1250)
        let (encoded, _, _) = WINDOWS_1250.encode(&self.name);
        let name_bytes = encoded.as_ref();
        let name_len = name_bytes.len().min(31); // Leave room for null terminator
        writer.write_all(&name_bytes[..name_len])?;
        // Pad with zeros to 32 bytes
        if name_len < 24 {
            let padding = 24 - name_len;
            writer.write_all(&vec![0u8; padding])?;
        }


        Ok(())
    }
}

/// DrawItem record from save file (ground items, 252 bytes each)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DrawItem {
    pub data: Vec<u8>,
}

impl DrawItem {
    /// Parse draw item from 252-byte data
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        if data.len() < 252 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "DrawItem requires 252 bytes",
            ));
        }
        Ok(DrawItem {
            data: data[..252].to_vec(),
        })
    }

    /// Serialize draw item to binary
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        if self.data.len() < 252 {
            writer.write_all(&self.data)?;
            let padding = 252 - self.data.len();
            if padding > 0 {
                writer.write_all(&vec![0u8; padding])?;
            }
        } else {
            writer.write_all(&self.data[..252])?;
        }
        Ok(())
    }
}

/// Data for one map section in a save file.
///
/// Each visited map records its monsters, NPCs, extra objects (chests, doors,
/// triggers), and items lying on the ground in five type-specific categories.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MapSectionData {
    /// Map index/ID referenced in AllMap.ini
    pub map_id: u32,
    /// Monsters present on this map
    pub monsters: Vec<MonsterRecord>,
    /// NPCs present on this map
    pub npcs: Vec<NpcRecord>,
    /// Extra objects (chests, triggers, etc.)
    pub extra_objects: Vec<ExtraObjectRecord>,
    /// Ground items — Weapon type (count × 296 bytes each)
    pub draw_items_weapon: Vec<u8>,
    /// Ground items — Heal type (count × 264 bytes each)
    pub draw_items_heal: Vec<u8>,
    /// Ground items — Edit type (count × 280 bytes each)
    pub draw_items_edit: Vec<u8>,
    /// Ground items — Misc type (count × 268 bytes each)
    pub draw_items_misc: Vec<u8>,
    /// Ground items — Event type (count × 252 bytes each)
    pub draw_items_event: Vec<u8>,
}

/// Parsed character stats from a save file.
///
/// Maps the binary stats block (~68 bytes of structured data) that follows
/// the belt-data section and precedes the inventory section.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterStats {
    // ── Core attributes ──
    pub strength: u16,
    pub agility: u16,
    pub wisdom: u16,
    pub constitution: u16,
    pub morale: u16,
    pub hp_current: u16,
    pub hp_maximum: u16,
    pub mp_current: u16,
    pub mp_maximum: u16,
    pub experience: u32,
    pub level: u16,
    pub gold: u32,
    // ── Combat stats ──
    pub offense: u16,
    pub defense: u16,
    pub dodge_rate: u8,
    pub hit_rate: u8,
    pub magic_power: u16,
    pub attack_modifier: u8,
    // ── Skills (5 × u8) ──
    pub thievery: u8,
    pub lockpicking: u8,
    pub haggling: u8,
    pub perception: u8,
    pub traps: u8,
    // ── Weapon skills (7 types × {level: u8, kills: u16}) ──
    pub swords_level: u8,
    pub swords_kills: u16,
    pub axes_level: u8,
    pub axes_kills: u16,
    pub archery_level: u8,
    pub archery_kills: u16,
    pub polearm_level: u8,
    pub polearm_kills: u16,
    pub magic_level: u8,
    pub magic_kills: u16,
    pub holy_magic_level: u8,
    pub holy_magic_kills: u16,
    pub dark_magic_level: u8,
    pub dark_magic_kills: u16,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryMiscItem {
    pub name: String,
    pub description: String,
    pub base_price: u32,
    pub unknown_1: Vec<u8>,
    pub unknown_2: u32,
    pub unknown_3: u16,
    pub unknown_4: u16,
    pub unknown_5: u32,
}

impl InventoryMiscItem {
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        if data.len() != 264 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "InventoryMiscItem requires 264 bytes",
            ));
        }

        let mut reader = std::io::Cursor::new(data);

        let mut name_raw = vec![0u8; 30];
        reader.read_exact(&mut name_raw)?;
        let name = read_null_terminated_windows_1250(&name_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut description_raw = vec![0u8; 202];
        reader.read_exact(&mut description_raw)?;
        let description = read_null_terminated_windows_1250(&description_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let base_price = reader.read_u32::<LittleEndian>()?;

        let mut unknown_1 = vec![0u8; 16];
        reader.read_exact(&mut unknown_1)?;

        let unknown_2 = reader.read_u32::<LittleEndian>()?;
        let unknown_3 = reader.read_u16::<LittleEndian>()?;
        let unknown_4 = reader.read_u16::<LittleEndian>()?;
        let unknown_5 = reader.read_u32::<LittleEndian>()?;

        Ok(InventoryMiscItem {
            name,
            description,
            base_price,
            unknown_1,
            unknown_2,
            unknown_3,
            unknown_4,
            unknown_5,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryEventItem {
    pub name: String,
    pub description: String,
    pub base_price: u32,
    pub unknown_1: u32,
    pub unknown_2: u32,
}

impl InventoryEventItem {
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        if data.len() != 244 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "InventoryEventItem requires 244 bytes",
            ));
        }

        let mut reader = std::io::Cursor::new(data);

        let mut name_raw = vec![0u8; 30];
        reader.read_exact(&mut name_raw)?;
        let name = read_null_terminated_windows_1250(&name_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut description_raw = vec![0u8; 202];
        reader.read_exact(&mut description_raw)?;
        let description = read_null_terminated_windows_1250(&description_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let base_price = reader.read_u32::<LittleEndian>()?;
        let unknown_1 = reader.read_u32::<LittleEndian>()?;
        let unknown_2 = reader.read_u32::<LittleEndian>()?;

        Ok(InventoryEventItem {
            name,
            description,
            base_price,
            unknown_1,
            unknown_2,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryEditItem {
    pub name: String,
    pub description: String,
    pub base_price: u32,

    pub unknown_1: u16,
    pub unknown_2: u16,
    pub health_points: i16,
    pub mana_points: i16,
    pub strength: i16,
    pub agility: i16,
    pub wisdom: i16,
    pub constitution: i16,
    pub to_dodge: i16,
    pub to_hit: i16,
    pub offense: i16,
    pub defense: i16,
    pub magical_power: i16,
    pub item_destroying_power: i16,
    pub unknown_3: u8,
    pub modifies_item: u8,
    pub additional_effect: i16,
    pub unknown_4: u16,
    pub unknown_5: u16,
}

impl InventoryEditItem {
    // 272 bytes long
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        if data.len() != 272 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "InventoryEditItem requires 272 bytes",
            ));
        }

        let mut reader = std::io::Cursor::new(data);

        let mut name_raw = vec![0u8; 30];
        reader.read_exact(&mut name_raw)?;
        let name = read_null_terminated_windows_1250(&name_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut description_raw = vec![0u8; 202];
        reader.read_exact(&mut description_raw)?;
        let description = read_null_terminated_windows_1250(&description_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let base_price = reader.read_u32::<LittleEndian>()?; // 36
        let unknown_1 = reader.read_u16::<LittleEndian>()?; // 34
        let unknown_2 = reader.read_u16::<LittleEndian>()?; // 32
        let health_points = reader.read_i16::<LittleEndian>()?; // 30
        let mana_points = reader.read_i16::<LittleEndian>()?; // 28
        let strength = reader.read_i16::<LittleEndian>()?; // 26
        let agility = reader.read_i16::<LittleEndian>()?; // 24
        let wisdom = reader.read_i16::<LittleEndian>()?; // 22
        let constitution = reader.read_i16::<LittleEndian>()?; // 20
        let to_dodge = reader.read_i16::<LittleEndian>()?; // 18
        let to_hit = reader.read_i16::<LittleEndian>()?; // 16
        let offense = reader.read_i16::<LittleEndian>()?; // 14
        let defense = reader.read_i16::<LittleEndian>()?; // 12
        let magical_power = reader.read_i16::<LittleEndian>()?; // 10
        let item_destroying_power = reader.read_i16::<LittleEndian>()?; // 8
        let unknown_3 = reader.read_u8()?; // 7
        let modifies_item = reader.read_u8()?; // 6
        let additional_effect = reader.read_i16::<LittleEndian>()?; // 4
        let unknown_4 = reader.read_u16::<LittleEndian>()?; // 2
        let unknown_5 = reader.read_u16::<LittleEndian>()?; // 0

        Ok(InventoryEditItem {
            name,
            description,
            base_price,
            unknown_1,
            unknown_2,
            health_points,
            mana_points,
            strength,
            agility,
            wisdom,
            constitution,
            to_dodge,
            to_hit,
            offense,
            defense,
            magical_power,
            item_destroying_power,
            unknown_3,
            modifies_item,
            additional_effect,
            unknown_4,
            unknown_5,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryHealItem {
    pub name: String,
    pub description: String,
    pub base_price: u32,
    pub heal_item_id: u32,
    pub health_points: i16,
    pub mana_points: i16,
    pub restore_full_health: u8,
    pub restore_full_mana: u8,
    pub poison_heal: u8,
    pub petrif_heal: u8,
    pub polimorph_heal: u8,
    pub unknown_1: u8,
    pub unknown_2: u16,
    pub unknown_3: u16, // index (from 0 to 30 for Nuno 0.sav)
    pub unknown_4: u16, // 6c 6c (108, 108) for the first row
}

impl InventoryHealItem {
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        if data.len() != 256 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "InventoryHealItem requires 256 bytes",
            ));
        }

        let mut reader = std::io::Cursor::new(data);

        let mut name_raw = vec![0u8; 30];
        reader.read_exact(&mut name_raw)?;
        let name = read_null_terminated_windows_1250(&name_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut description_raw = vec![0u8; 202];
        reader.read_exact(&mut description_raw)?;
        let description = read_null_terminated_windows_1250(&description_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let base_price = reader.read_u32::<LittleEndian>()?;
        let heal_item_id = reader.read_u32::<LittleEndian>()?;
        let health_points = reader.read_i16::<LittleEndian>()?;
        let mana_points = reader.read_i16::<LittleEndian>()?;
        let restore_full_health = reader.read_u8()?;
        let restore_full_mana = reader.read_u8()?;
        let poison_heal = reader.read_u8()?;
        let petrif_heal = reader.read_u8()?;
        let polimorph_heal = reader.read_u8()?;
        let unknown_1 = reader.read_u8()?;
        let unknown_2 = reader.read_u16::<LittleEndian>()?;
        let unknown_3 = reader.read_u16::<LittleEndian>()?;
        let unknown_4 = reader.read_u16::<LittleEndian>()?;

        Ok(InventoryHealItem {
            name,
            description,
            base_price,
            heal_item_id,
            health_points,
            mana_points,
            restore_full_health,
            restore_full_mana,
            poison_heal,
            petrif_heal,
            polimorph_heal,
            unknown_1,
            unknown_2,
            unknown_3,
            unknown_4,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryWeaponItem {
    pub name: String,
    pub description: String,
    pub base_price: u32,
    pub weapon_item_id: u32,
    pub health_points: i16,
    pub mana_points: i16,
    pub strength: i16,
    pub agility: i16,
    pub wisdom: i16,
    pub constitution: i16,
    pub to_dodge: i16,
    pub to_hit: i16,
    pub attack: i16,
    pub defense: i16,
    pub magical_strength: i16,
    pub durability: i16,
    pub padding2: i16,
    pub padding3: i16,
    pub req_strength: i16,
    pub padding4: i16,
    pub req_agility: i16,
    pub padding5: i16,
    pub req_wisdom: i16,
    pub padding6: i16,
    pub padding7: i16,
    pub padding8: i16,
    pub unknown_1: u32,
    pub unknown_2: u32,
}

impl InventoryWeaponItem {
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        if data.len() != 292 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "InventoryWeaponItem requires 292 bytes",
            ));
        }

        let mut reader = std::io::Cursor::new(data);

        let mut name_raw = vec![0u8; 30];
        reader.read_exact(&mut name_raw)?;
        let name = read_null_terminated_windows_1250(&name_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut description_raw = vec![0u8; 202];
        reader.read_exact(&mut description_raw)?;
        let description = read_null_terminated_windows_1250(&description_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let base_price = reader.read_u32::<LittleEndian>()?;
        let weapon_item_id = reader.read_u32::<LittleEndian>()?;
        let health_points = reader.read_i16::<LittleEndian>()?;
        let mana_points = reader.read_i16::<LittleEndian>()?;
        let strength = reader.read_i16::<LittleEndian>()?;
        let agility = reader.read_i16::<LittleEndian>()?;
        let wisdom = reader.read_i16::<LittleEndian>()?;
        let constitution = reader.read_i16::<LittleEndian>()?;
        let to_dodge = reader.read_i16::<LittleEndian>()?;
        let to_hit = reader.read_i16::<LittleEndian>()?;
        let attack = reader.read_i16::<LittleEndian>()?;
        let defense = reader.read_i16::<LittleEndian>()?;
        let magical_strength = reader.read_i16::<LittleEndian>()?;
        let durability = reader.read_i16::<LittleEndian>()?;
        let padding2 = reader.read_i16::<LittleEndian>()?;
        let padding3 = reader.read_i16::<LittleEndian>()?;
        let req_strength = reader.read_i16::<LittleEndian>()?;
        let padding4 = reader.read_i16::<LittleEndian>()?;
        let req_agility = reader.read_i16::<LittleEndian>()?;
        let padding5 = reader.read_i16::<LittleEndian>()?;
        let req_wisdom = reader.read_i16::<LittleEndian>()?;
        let padding6 = reader.read_i16::<LittleEndian>()?;
        let padding7 = reader.read_i16::<LittleEndian>()?;
        let padding8 = reader.read_i16::<LittleEndian>()?;
        let unknown_1 = reader.read_u32::<LittleEndian>()?;
        let unknown_2 = reader.read_u32::<LittleEndian>()?;

        Ok(InventoryWeaponItem {
            name,
            description,
            base_price,
            weapon_item_id,
            health_points,
            mana_points,
            strength,
            agility,
            wisdom,
            constitution,
            to_dodge,
            to_hit,
            attack,
            defense,
            magical_strength,
            durability,
            padding2,
            padding3,
            req_strength,
            padding4,
            req_agility,
            padding5,
            req_wisdom,
            padding6,
            padding7,
            padding8,
            unknown_1,
            unknown_2,
        })
    }
}

/// Journal data from a save file (3 sections × 100 entries).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JournalData {
    /// Main quest entries (100 × 37 bytes)
    pub main: Vec<JournalEntry>,
    /// Side quest entries (100 × 37 bytes)
    pub side: Vec<JournalEntry>,
    /// Trading offer entries (100 × 37 bytes)
    pub trade: Vec<JournalEntry>,
}

/// Character identity data (name, class, surrounding unknown blocks).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterIdentity {
    /// Unknown block before player name (96 bytes).
    pub unknown_block: Vec<u8>,
    /// Player name (11-byte WINDOWS-1250 null-terminated).
    pub player_name: String,
    /// Player class ID.
    pub player_class_id: u16,
    /// Player class name (11-byte WINDOWS-1250 null-terminated).
    pub player_class_name: String,
    /// Large unknown data block after identity (4040 bytes).
    pub unknown_data: Vec<u8>,
}

/// Unknown data block between map data and sprite paths (section 3).
///
/// Layout: `[9 × u32 header][variable-size remainder]`.
/// The remainder size is calculated as `(10188 + 4 * num_visited_maps) - 36`.
/// The header values may encode sizes of sub-sections within the remainder
/// (monster_block_size, npc_block_size, extra_object_block_size observed as 329, 349, 200).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostMapsData {
    /// Possibly the save slot index.
    pub save_slot_id: u32,
    /// Possibly a Win32 timestamp of when this save was created.
    pub save_timestamp: u32,
    /// 3 unknown u32 values (observed: 4, 8, 0).
    pub unknowns_a: [u32; 3],
    /// Possibly the size of the monster data block within the remainder.
    pub monster_block_size: u32,
    /// Possibly the size of the NPC data block within the remainder.
    pub npc_block_size: u32,
    /// Possibly the size of the extra object data block within the remainder.
    pub extra_object_block_size: u32,
    /// One more unknown u32 (observed: 0, sandwiched between npc and extra sizes).
    pub unknown_b: u32,
    /// The rest of the section after the header.
    pub unknown_block: Vec<u8>,
}

/// Unknown data block between events and journal sections.
///
/// Structure: fixed 12 bytes + counter-prefixed 24-byte records + fixed 98 bytes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PostEventsData {
    /// Unknown fixed block (12 bytes).
    pub block_a: Vec<u8>,
    /// Unknown records (counter × 24 bytes each).
    pub records: Vec<u8>,
    /// Unknown fixed block (98 bytes).
    pub block_b: Vec<u8>,
}

/// Complete save file structure.
///
/// More fields will be added in future phases.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveFile {
    /// Jump address after all map data (first 4 bytes of the file).
    /// The maps section is followed by alignment to this address.
    pub jump_addr_after_maps: u32,
    /// Per-map world state.
    pub maps: Vec<MapSectionData>,
    /// Unknown data between maps and sprite paths (header + variable-size remainder).
    pub post_maps: PostMapsData,
    /// Character sprite paths (4 × 60-byte WINDOWS-1250 strings).
    pub sprite_paths: Vec<String>,
    /// Raw belt/quick-slot data (40 bytes before character stats).
    pub unknown_before_stats: Vec<u8>,
    /// Parsed character stats (core, combat, skills, weapon skills).
    pub character_stats: CharacterStats,
    /// Unknown bytes after stats block (9 bytes).
    pub unknown_after_stats: Vec<u8>,
    /// Raw inventory data (5 item categories).
    pub inventory: InventoryData,
    /// Journal entries (main, side, trade — 100 entries each).
    pub journal: JournalData,
    /// Event scripts (2251 × 284 bytes).
    pub events: Vec<EventScript>,
    /// Character identity (name, class, unknown blocks).
    pub character_identity: CharacterIdentity,
    /// Unknown data between events and journal (3 sub-blocks).
    pub post_events: PostEventsData,
}

impl SaveFile {
    /// Parse complete save file from binary data
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut reader = std::io::Cursor::new(data);

        // ── 1. HEADER (4 bytes) ──
        let jump_addr_after_maps = reader.read_u32::<LittleEndian>()? as usize;

        // ── 2. Maps ──
        let number_of_visited_map = reader.read_u32::<LittleEndian>()?;
        let maps = Self::parse_maps_section(&mut reader, number_of_visited_map)?;

        if jump_addr_after_maps != reader.position() as usize {
            // 0.sav Christofor: 2 chests, +500 bytes
            eprintln!(
                "jump_addr_after_maps ({:?}) != reader.position() {:?}",
                jump_addr_after_maps,
                reader.position() as usize
            );

            reader.set_position(jump_addr_after_maps as u64);
        }

        // ── 3. Unknown data between maps and sprite paths ──
        let post_maps = Self::parse_post_maps_data(&mut reader, number_of_visited_map)?;

        // ── 4. Character sprite paths (4 × 60-byte WINDOWS-1250 strings) ──
        let sprite_paths = Self::parse_sprite_paths(&mut reader)?;

        // ── 5 Character stats ──
        let (unknown_before_stats, character_stats, unknown_after_stats) =
            Self::parse_character_stats(&mut reader)?;

        // ── 6. Inventory (5 categories, each count-prefixed) ──
        let inventory = Self::parse_inventory_section(&mut reader)?;

        // ── 7. Character identity (unknown block + name + class + large unknown) ──
        let character_identity = Self::parse_character_identity(&mut reader)?;

        // ── 8. Events (2251 × 284 bytes) ──
        let events = Self::parse_events_section(&mut reader)?;

        // ── 9. Unknown data between events and journal ──
        let post_events = Self::parse_post_events_data(&mut reader)?;

        // ── 10. Journal (3 sections × 100 × 37 bytes) ──
        let journal = Self::parse_journal_section(&mut reader)?;

        Ok(SaveFile {
            jump_addr_after_maps: jump_addr_after_maps as u32,
            maps,
            post_maps,
            sprite_paths,
            unknown_before_stats,
            character_stats,
            unknown_after_stats,
            inventory,
            character_identity,
            events,
            post_events,
            journal,
        })
    }

    /// Read a count-prefixed draw item section from the reader.
    ///
    /// Each draw item section in the save file is stored as:
    ///   `[count: u16][count × record_size bytes]`
    fn read_draw_item_section<R: Read>(
        reader: &mut R,
        record_size: u32,
    ) -> std::io::Result<Vec<u8>> {
        let count = reader.read_u16::<LittleEndian>()? as usize;
        let mut data = vec![0u8; count * record_size as usize];
        reader.read_exact(&mut data)?;
        Ok(data)
    }

    fn read_misc_item_section<R: Read>(reader: &mut R) -> std::io::Result<Vec<InventoryMiscItem>> {
        let count = reader.read_u16::<LittleEndian>()? as usize;
        let mut data = vec![0u8; count * 264];
        reader.read_exact(&mut data)?;

        let items = data
            .chunks_exact(264)
            .map(InventoryMiscItem::parse)
            .collect::<std::io::Result<Vec<_>>>()?;

        Ok(items)
    }

    fn read_event_item_section<R: Read>(
        reader: &mut R,
    ) -> std::io::Result<Vec<InventoryEventItem>> {
        let count = reader.read_u16::<LittleEndian>()? as usize;
        let mut data = vec![0u8; count * 244];
        reader.read_exact(&mut data)?;

        let items = data
            .chunks_exact(244)
            .map(InventoryEventItem::parse)
            .collect::<std::io::Result<Vec<_>>>()?;

        Ok(items)
    }
    fn read_edit_item_section<R: Read>(reader: &mut R) -> std::io::Result<Vec<InventoryEditItem>> {
        let count = reader.read_u16::<LittleEndian>()? as usize;
        let mut data = vec![0u8; count * 272];
        reader.read_exact(&mut data)?;

        let items = data
            .chunks_exact(272)
            .map(InventoryEditItem::parse)
            .collect::<std::io::Result<Vec<_>>>()?;

        Ok(items)
    }

    fn read_heal_item_section<R: Read>(reader: &mut R) -> std::io::Result<Vec<InventoryHealItem>> {
        let count = reader.read_u16::<LittleEndian>()? as usize;
        let mut data = vec![0u8; count * 256];
        reader.read_exact(&mut data)?;

        let items = data
            .chunks_exact(256)
            .map(InventoryHealItem::parse)
            .collect::<std::io::Result<Vec<_>>>()?;

        Ok(items)
    }

    fn read_weapon_item_section<R: Read>(reader: &mut R) -> std::io::Result<Vec<InventoryWeaponItem>> {
        let count = reader.read_u16::<LittleEndian>()? as usize;
        let mut data = vec![0u8; count * 292];
        reader.read_exact(&mut data)?;

        let items = data
            .chunks_exact(292)
            .map(InventoryWeaponItem::parse)
            .collect::<std::io::Result<Vec<_>>>()?;

        Ok(items)
    }

    /// Parse all map sections from the reader.
    ///
    /// Each map has:
    ///   `[map_id: u32][monsters][npcs][sep: u32][extra_objects][sep: 11B]
    ///    [draw_items_weapon][draw_items_heal][draw_items_edit]
    ///    [draw_items_misc][draw_items_event][end_sep: u32]`
    fn parse_maps_section<R: Read>(
        reader: &mut R,
        map_count: u32,
    ) -> std::io::Result<Vec<MapSectionData>> {
        let mut maps = Vec::with_capacity(map_count as usize);

        for _ in 0..map_count {
            let map_id = reader.read_u32::<LittleEndian>()?;

            // ── 2.1. Monsters ──
            let monster_count = reader.read_u32::<LittleEndian>()? as usize;
            let mut monsters_data = vec![0u8; monster_count * 329];
            reader.read_exact(&mut monsters_data)?;
            let monsters = monsters_data
                .chunks_exact(329)
                .map(MonsterRecord::parse)
                .collect::<std::io::Result<Vec<_>>>()?;

            // ── 2.2. NPCs ──
            let npc_count = reader.read_u32::<LittleEndian>()? as usize;
            let mut npcs_data = vec![0u8; npc_count * 349];
            reader.read_exact(&mut npcs_data)?;
            let npcs = npcs_data
                .chunks_exact(349)
                .map(NpcRecord::parse)
                .collect::<std::io::Result<Vec<_>>>()?;

            // ── 2.3. Separator (always 0) ──
            let _separator = reader.read_u32::<LittleEndian>()?;

            // ── 2.4. Extra objects ──
            let extras_count = reader.read_u32::<LittleEndian>()? as usize;
            let mut extras_data = vec![0u8; extras_count * 200];
            reader.read_exact(&mut extras_data)?;
            let extra_objects = extras_data
                .chunks_exact(200)
                .map(|chunk| ExtraObjectRecord::parse(chunk, false))
                .collect::<std::io::Result<Vec<_>>>()?;

            // ── 2.5. Separator (11 bytes, unknown meaning) ──
            let mut _separator = vec![0u8; 11];
            reader.read_exact(&mut _separator)?;

            // ── 2.6–2.10. Ground items (5 types) ──
            let draw_items_weapon = Self::read_draw_item_section(reader, 296)?;
            let draw_items_heal = Self::read_draw_item_section(reader, 264)?;
            let draw_items_edit = Self::read_draw_item_section(reader, 280)?;
            let draw_items_misc = Self::read_draw_item_section(reader, 268)?;
            let draw_items_event = Self::read_draw_item_section(reader, 252)?;

            // ── 2.11. End-of-map separator (always 0) ──
            let _separator = reader.read_u32::<LittleEndian>()?;

            maps.push(MapSectionData {
                map_id,
                monsters,
                npcs,
                extra_objects,
                draw_items_weapon,
                draw_items_heal,
                draw_items_edit,
                draw_items_misc,
                draw_items_event,
            });
        }

        Ok(maps)
    }

    /// Parse the unknown data block between maps and sprite paths.
    ///
    /// Layout: `[9 × u32 header][variable-size remainder]`
    fn parse_post_maps_data<R: Read>(
        reader: &mut R,
        num_visited_maps: u32,
    ) -> std::io::Result<PostMapsData> {
        let header = [
            reader.read_u32::<LittleEndian>()?, // 0: maybe save_slot_id
            reader.read_u32::<LittleEndian>()?, // 1: maybe save_timestamp
            reader.read_u32::<LittleEndian>()?, // 2: observed 4
            reader.read_u32::<LittleEndian>()?, // 3: observed 8
            reader.read_u32::<LittleEndian>()?, // 4: observed 0
            reader.read_u32::<LittleEndian>()?, // 5: monster_block_size (observed 329)
            reader.read_u32::<LittleEndian>()?, // 6: npc_block_size (observed 349)
            reader.read_u32::<LittleEndian>()?, // 7: observed 0
            reader.read_u32::<LittleEndian>()?, // 8: extra_object_block_size (observed 200)
        ];

        let remainder = (10188 + 4 * num_visited_maps as usize) - 36;
        let mut unknown_block = vec![0u8; remainder];
        reader.read_exact(&mut unknown_block)?;

        Ok(PostMapsData {
            save_slot_id: header[0],
            save_timestamp: header[1],
            unknowns_a: [header[2], header[3], header[4]],
            monster_block_size: header[5],
            npc_block_size: header[6],
            extra_object_block_size: header[8],
            unknown_b: header[7],
            unknown_block,
        })
    }

    /// Parse the 4 character sprite paths (4 × 60-byte fixed buffers).
    ///
    /// Each path is a null-terminated WINDOWS-1250 string, e.g.
    /// `"inter\\m_bald.spr"` or `"CharacterInGame\\m_warrior.spr"`.
    fn parse_sprite_paths<R: Read>(reader: &mut R) -> std::io::Result<Vec<String>> {
        let mut paths = Vec::with_capacity(4);
        for _ in 0..4 {
            let mut buf = [0u8; 60];
            reader.read_exact(&mut buf)?;
            paths.push(
                read_null_terminated_windows_1250(&buf)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?,
            );
        }
        Ok(paths)
    }

    /// Parse belt data, character stats, and trailing unknown bytes.
    ///
    /// Layout:
    ///   `[unknown_before_stats: 40B][strength u16][agility u16][wisdom u16][constitution u16]
    ///    [morale u16][hp_cur u16][hp_max u16][mp_cur u16][mp_max u16]
    ///    [xp u32][level u16][gold u32][offense u16][defense u16]
    ///    [dodge u8][hit u8][magic_power u16][attack_mod u8]
    ///    [thievery u8][lockpick u8][haggle u8][perception u8][traps u8]
    ///    [sword_lv u8][sword_kills u16][axe_lv u8][axe_kills u16]
    ///    [archery_lv u8][archery_kills u16][polearm_lv u8][polearm_kills u16]
    ///    [magic_lv u8][magic_kills u16][holy_lv u8][holy_kills u16]
    ///    [dark_lv u8][dark_kills u16][unknown: 9B]`
    fn parse_character_stats<R: Read>(
        reader: &mut R,
    ) -> std::io::Result<(Vec<u8>, CharacterStats, Vec<u8>)> {
        // ── Leading data (40 bytes, purpose unknown) ──
        let mut unknown_before_stats = vec![0u8; 40];
        reader.read_exact(&mut unknown_before_stats)?;

        // ── Structured stats block ──
        let character_stats = CharacterStats {
            strength: reader.read_u16::<LittleEndian>()?,
            agility: reader.read_u16::<LittleEndian>()?,
            wisdom: reader.read_u16::<LittleEndian>()?,
            constitution: reader.read_u16::<LittleEndian>()?,
            morale: reader.read_u16::<LittleEndian>()?,
            hp_current: reader.read_u16::<LittleEndian>()?,
            hp_maximum: reader.read_u16::<LittleEndian>()?,
            mp_current: reader.read_u16::<LittleEndian>()?,
            mp_maximum: reader.read_u16::<LittleEndian>()?,
            experience: reader.read_u32::<LittleEndian>()?,
            level: reader.read_u16::<LittleEndian>()?,
            gold: reader.read_u32::<LittleEndian>()?,
            offense: reader.read_u16::<LittleEndian>()?,
            defense: reader.read_u16::<LittleEndian>()?,
            dodge_rate: reader.read_u8()?,
            hit_rate: reader.read_u8()?,
            magic_power: reader.read_u16::<LittleEndian>()?,
            attack_modifier: reader.read_u8()?,
            thievery: reader.read_u8()?,
            lockpicking: reader.read_u8()?,
            haggling: reader.read_u8()?,
            perception: reader.read_u8()?,
            traps: reader.read_u8()?,
            swords_level: reader.read_u8()?,
            swords_kills: reader.read_u16::<LittleEndian>()?,
            axes_level: reader.read_u8()?,
            axes_kills: reader.read_u16::<LittleEndian>()?,
            archery_level: reader.read_u8()?,
            archery_kills: reader.read_u16::<LittleEndian>()?,
            polearm_level: reader.read_u8()?,
            polearm_kills: reader.read_u16::<LittleEndian>()?,
            magic_level: reader.read_u8()?,
            magic_kills: reader.read_u16::<LittleEndian>()?,
            holy_magic_level: reader.read_u8()?,
            holy_magic_kills: reader.read_u16::<LittleEndian>()?,
            dark_magic_level: reader.read_u8()?,
            dark_magic_kills: reader.read_u16::<LittleEndian>()?,
        };

        // ── Trailing unknown bytes ──
        let mut unknown_after_stats = vec![0u8; 9];
        reader.read_exact(&mut unknown_after_stats)?;

        Ok((unknown_before_stats, character_stats, unknown_after_stats))
    }

    /// Parse the inventory section (5 count-prefixed item categories).
    ///
    /// Record sizes: Event=244, Misc=264, Edit=272, Weapon=292, Heal=256.
    fn parse_inventory_section<R: Read>(reader: &mut R) -> std::io::Result<InventoryData> {
        Ok(InventoryData {
            event_items: Self::read_event_item_section(reader)?,
            misc_items: Self::read_misc_item_section(reader)?,
            edit_items: Self::read_edit_item_section(reader)?,
            weapon_items: Self::read_weapon_item_section(reader)?,
            heal_items: Self::read_heal_item_section(reader)?,
        })
    }

    /// Parse the journal section (3 × 100 × 37-byte entries).
    fn parse_journal_section<R: Read>(reader: &mut R) -> std::io::Result<JournalData> {
        const ENTRY_SIZE: usize = 37;
        const ENTRIES_PER_SECTION: usize = 100;
        const SECTION_SIZE: usize = ENTRY_SIZE * ENTRIES_PER_SECTION; // 3700

        let mut raw = vec![0u8; SECTION_SIZE];
        reader.read_exact(&mut raw)?;
        let main = Self::parse_journal_entries(&raw, ENTRIES_PER_SECTION)?;

        let mut raw = vec![0u8; SECTION_SIZE];
        reader.read_exact(&mut raw)?;
        let side = Self::parse_journal_entries(&raw, ENTRIES_PER_SECTION)?;

        let mut raw = vec![0u8; SECTION_SIZE];
        reader.read_exact(&mut raw)?;
        let trade = Self::parse_journal_entries(&raw, ENTRIES_PER_SECTION)?;

        Ok(JournalData { main, side, trade })
    }

    /// Parse the events section (2251 × 284-byte event records).
    fn parse_events_section<R: Read>(reader: &mut R) -> std::io::Result<Vec<EventScript>> {
        const EVENT_COUNT: usize = 2251;
        const EVENT_SIZE: usize = 284;

        let mut events = Vec::with_capacity(EVENT_COUNT);
        for _ in 0..EVENT_COUNT {
            let mut buf = [0u8; EVENT_SIZE];
            reader.read_exact(&mut buf)?;
            events.push(EventScript::parse(&buf)?);
        }
        Ok(events)
    }

    /// Parse character identity (unknown block + name + class + large unknown).
    ///
    /// Layout:
    ///   `[unknown_96B][name: 11B][class_id: u16][class_name: 11B][unknown_4040B]`
    fn parse_character_identity<R: Read>(reader: &mut R) -> std::io::Result<CharacterIdentity> {
        // ── 7.1. Unknown block (96 bytes before name) ──
        let mut unknown_block = vec![0u8; 96];
        reader.read_exact(&mut unknown_block)?;

        // ── 7.2. Player name (11-byte WINDOWS-1250 null-terminated) ──
        let mut name_raw = vec![0u8; 11];
        reader.read_exact(&mut name_raw)?;
        let player_name = read_null_terminated_windows_1250(&name_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // ── 7.3. Player class ──
        let player_class_id = reader.read_u16::<LittleEndian>()?;
        let mut class_raw = vec![0u8; 11];
        reader.read_exact(&mut class_raw)?;
        let player_class_name = read_null_terminated_windows_1250(&class_raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // ── 7.4. Large unknown data block ──
        let mut unknown_data = vec![0u8; 4040];
        reader.read_exact(&mut unknown_data)?;

        Ok(CharacterIdentity {
            unknown_block,
            player_name,
            player_class_id,
            player_class_name,
            unknown_data,
        })
    }

    /// Parse the unknown section between events and journal.
    ///
    /// Layout: `[block_a: 12B][count: u32][count × 24B records][block_b: 98B]`
    fn parse_post_events_data<R: Read>(reader: &mut R) -> std::io::Result<PostEventsData> {
        let mut block_a = vec![0u8; 12];
        reader.read_exact(&mut block_a)?;

        let count = reader.read_u32::<LittleEndian>()? as usize;
        let mut records = vec![0u8; count * 24];
        reader.read_exact(&mut records)?;

        let mut block_b = vec![0u8; 98];
        reader.read_exact(&mut block_b)?;

        Ok(PostEventsData {
            block_a,
            records,
            block_b,
        })
    }

    /// Parse journal entries from raw binary data
    fn parse_journal_entries(data: &[u8], count: usize) -> std::io::Result<Vec<JournalEntry>> {
        let expected_len = count * 37;
        if data.len() < expected_len {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Journal data too short",
            ));
        }

        let mut entries = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * 37;
            let entry = JournalEntry::parse(&data[offset..offset + 37])?;
            entries.push(entry);
        }

        Ok(entries)
    }
}

impl Extractor for SaveFile {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let save = SaveFile::parse(&data)?;
        Ok(vec![save])
    }

    fn to_writer<W: Write>(records: &[Self], _writer: &mut W) -> std::io::Result<()> {
        if records.len() != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "SaveFile can only serialize one record at a time",
            ));
        }

        // let save = &records[0];

        // // Write header (12 bytes)
        // writer.write_all(&save.header)?;
        //
        // // Write surface monster count + raw data
        // writer.write_u32::<LittleEndian>(save.surface_monsters.len() as u32)?;
        // writer.write_all(&save.monsters_data)?;
        //
        // // Write NPC count + raw data
        // writer.write_u32::<LittleEndian>(save.npcs.len() as u32)?;
        // writer.write_all(&save.npcs_data)?;
        //
        // // Write surface objects: separator (u32=0) + count + raw data
        // writer.write_u32::<LittleEndian>(0)?;
        // writer.write_u32::<LittleEndian>(save.surface_objects.len() as u32)?;
        // writer.write_all(&save.extras_data)?;
        //
        // // Write remaining data (everything from after surface objects to EOF)
        // writer.write_all(&save.remaining_data)?;

        Ok(())
    }
}
