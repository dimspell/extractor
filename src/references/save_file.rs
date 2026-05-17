// Save file extraction and parsing for Dispel RPG
//
// This module provides comprehensive parsing of Dispel RPG save files (.sav)
// following the binary format documented in SAVE_FILE_RESEARCH.md

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::WINDOWS_1250;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom, Write};

use super::extractor::read_null_terminated_windows_1250;
use super::extractor::Extractor;

/// Item type identifiers for inventory items
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SaveItemType {
    /// Weapon item (attack > 0, defense = 0)
    #[default]
    Weapon = 0,
    /// Armor item (defense > 0, attack = 0)
    Armor = 1,
    /// Healing item (potions, antidotes)
    Heal = 2,
    /// Miscellaneous item (coins, keys, gems)
    Misc = 3,
    /// Edit item (scrolls, books, modifiable items)
    Edit = 4,
    /// Event-specific item (quest items)
    Event = 5,
}

impl SaveItemType {
    /// Convert from u8 with validation
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(SaveItemType::Weapon),
            1 => Some(SaveItemType::Armor),
            2 => Some(SaveItemType::Heal),
            3 => Some(SaveItemType::Misc),
            4 => Some(SaveItemType::Edit),
            5 => Some(SaveItemType::Event),
            _ => None,
        }
    }

    /// Get the numeric value
    pub fn value(&self) -> u8 {
        *self as u8
    }
}

/// Player attributes block from save file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerAttributes {
    pub strength: u16,
    pub dexterity: u16,
    pub wisdom: u16,
    pub constitution: u16,
    pub unknown_stat: u16, // Likely luck or agility
    pub hp_current: u16,
    pub hp_maximum: u16,
    pub mp_current: u16,
    pub mp_maximum: u16,
    pub xp_current: u32,
    pub level: u16,
    pub gold: u32,
}

impl PlayerAttributes {
    /// Parse player attributes from save file data
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut reader = std::io::Cursor::new(data);

        Ok(PlayerAttributes {
            strength: reader.read_u16::<LittleEndian>()?,
            dexterity: reader.read_u16::<LittleEndian>()?,
            wisdom: reader.read_u16::<LittleEndian>()?,
            constitution: reader.read_u16::<LittleEndian>()?,
            unknown_stat: reader.read_u16::<LittleEndian>()?,
            hp_current: reader.read_u16::<LittleEndian>()?,
            hp_maximum: reader.read_u16::<LittleEndian>()?,
            mp_current: reader.read_u16::<LittleEndian>()?,
            mp_maximum: reader.read_u16::<LittleEndian>()?,
            xp_current: reader.read_u32::<LittleEndian>()?,
            level: reader.read_u16::<LittleEndian>()?,
            gold: reader.read_u32::<LittleEndian>()?,
        })
    }

    /// Serialize player attributes to binary
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_u16::<LittleEndian>(self.strength)?;
        writer.write_u16::<LittleEndian>(self.dexterity)?;
        writer.write_u16::<LittleEndian>(self.wisdom)?;
        writer.write_u16::<LittleEndian>(self.constitution)?;
        writer.write_u16::<LittleEndian>(self.unknown_stat)?;
        writer.write_u16::<LittleEndian>(self.hp_current)?;
        writer.write_u16::<LittleEndian>(self.hp_maximum)?;
        writer.write_u16::<LittleEndian>(self.mp_current)?;
        writer.write_u16::<LittleEndian>(self.mp_maximum)?;
        writer.write_u32::<LittleEndian>(self.xp_current)?;
        writer.write_u16::<LittleEndian>(self.level)?;
        writer.write_u32::<LittleEndian>(self.gold)?;
        Ok(())
    }
}

/// Inventory item record from save file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryItem {
    /// Item type identifier
    pub item_type: SaveItemType,
    /// Item subtype/index (maps to game database)
    pub item_id: u32,
    /// Quantity of items in stack
    pub quantity: u16,
    /// Item name (decoded from WINDOWS-1250)
    pub name: String,
    /// Item description (decoded from WINDOWS-1250)
    pub description: String,
    /// Associated quest name (empty if no quest)
    pub quest_name: String,
}

impl InventoryItem {
    /// Parse inventory item from save file data
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut reader = std::io::Cursor::new(data);

        // Parse 10-byte header
        let field_a = reader.read_u32::<LittleEndian>()?;
        let field_b = reader.read_u32::<LittleEndian>()?;
        let quantity = reader.read_u16::<LittleEndian>()?;

        // Extract item type from Field A (bits 0-7)
        let item_type_id = (field_a & 0xFF) as u8;
        let item_type = SaveItemType::from_u8(item_type_id).unwrap_or(SaveItemType::Misc);

        // Item ID from Field B
        let item_id = field_b;

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

        // Read description (null-terminated WINDOWS-1250)
        let mut desc_bytes = Vec::new();
        loop {
            let byte = reader.read_u8()?;
            if byte == 0 {
                break;
            }
            desc_bytes.push(byte);
        }
        let (description, _, _) = WINDOWS_1250.decode(&desc_bytes);

        Ok(InventoryItem {
            item_type,
            item_id,
            quantity,
            name: name.to_string(),
            description: description.to_string(),
            quest_name: String::new(),
        })
    }

    /// Serialize inventory item to binary
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        // Write 10-byte header
        let field_a = self.item_type.value() as u32;
        writer.write_u32::<LittleEndian>(field_a)?;
        writer.write_u32::<LittleEndian>(self.item_id)?;
        writer.write_u16::<LittleEndian>(self.quantity)?;

        // Pad header to 10 bytes
        writer.write_all(&[0u8; 2])?;

        // Write name (null-terminated)
        writer.write_all(self.name.as_bytes())?;
        writer.write_u8(0)?;

        // Pad to align description (fill remaining space in 256-byte record)
        let name_len = self.name.len() + 1;
        let header_len = 10;
        let used = header_len + name_len;
        let remaining = 256 - used;

        if remaining > 0 {
            writer.write_all(&vec![0u8; remaining])?;
        }

        // Write description (null-terminated)
        writer.write_all(self.description.as_bytes())?;
        writer.write_u8(0)?;

        // Pad to 256 bytes - calculate padding needed
        let description_len = self.description.len() + 1;
        let total_written = header_len + name_len + description_len;
        let padding_needed = 256 - (total_written % 256);

        if padding_needed < 256 {
            writer.write_all(&vec![0u8; padding_needed])?;
        }

        Ok(())
    }
}

/// Potion belt slot (6 dedicated slots for healing items)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PotionSlot {
    pub item_type: SaveItemType,
    pub item_id: u32,
    pub quantity: u16,
    pub name: String,
}

impl PotionSlot {
    /// Parse potion slot from 256-byte record
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut reader = std::io::Cursor::new(data);

        let field_a = reader.read_u32::<LittleEndian>()?;
        let field_b = reader.read_u32::<LittleEndian>()?;
        let quantity = reader.read_u16::<LittleEndian>()?;

        // Skip padding
        reader.read_u16::<LittleEndian>()?;

        let item_type_id = (field_a & 0xFF) as u8;
        let item_type = SaveItemType::from_u8(item_type_id).unwrap_or(SaveItemType::Heal);

        let mut name_bytes = Vec::new();
        loop {
            let byte = reader.read_u8()?;
            if byte == 0 {
                break;
            }
            name_bytes.push(byte);
        }
        let (name, _, _) = WINDOWS_1250.decode(&name_bytes);

        Ok(PotionSlot {
            item_type,
            item_id: field_b,
            quantity,
            name: name.to_string(),
        })
    }
}

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
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut reader = std::io::Cursor::new(data);

        let signature_a = reader.read_u32::<LittleEndian>()?;
        let record_index = reader.read_u32::<LittleEndian>()?;
        let signature_b = reader.read_u32::<LittleEndian>()?;

        // Name: 24 bytes null-terminated WINDOWS-1250
        let name = {
            let mut name_bytes = Vec::new();
            for _ in 0..24 {
                let byte = reader.read_u8()?;
                if byte == 0 {
                    break;
                }
                name_bytes.push(byte);
            }
            let (name, _, _) = WINDOWS_1250.decode(&name_bytes);
            name.to_string()
        };

        // Stats block (293 bytes)
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

            // Name: 24 bytes null-terminated WINDOWS-1250
            name,

            // Stats block (293 bytes)
            hp_current: hp_current,
            hp_maximum: hp_maximum,
            state: state,
            tile_x: tile_x,
            tile_y: tile_y,
            pixel_x: pixel_x,
            pixel_y: pixel_y,
            facing_direction: facing_direction,

            experience_value: experience_value,
            attack_power: attack_power,
            defense: defense,
            magic_attack: magic_attack,
            magic_defense: magic_defense,
            agility: agility,
            luck: luck,
        })
    }
}

/// NPC record from save file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NpcRecord {
    pub counter1: u32,
    pub counter2: u32,
    pub counter3: u32,
    pub counter4: u32,
    pub name: String,
    pub role_description: String,
}

impl NpcRecord {
    /// Parse NPC record from 349-byte data
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut reader = std::io::Cursor::new(data);

        let counter1 = reader.read_u32::<LittleEndian>()?;
        let counter2 = reader.read_u32::<LittleEndian>()?;
        let counter3 = reader.read_u32::<LittleEndian>()?;
        let counter4 = reader.read_u32::<LittleEndian>()?;

        // Skip 32 bytes padding
        reader.read_u32::<LittleEndian>()?;
        reader.read_u32::<LittleEndian>()?;
        reader.read_u32::<LittleEndian>()?;
        reader.read_u32::<LittleEndian>()?;

        let name = {
            let mut name_bytes = Vec::new();
            for _ in 0..32 {
                let byte = reader.read_u8()?;
                if byte == 0 {
                    break;
                }
                name_bytes.push(byte);
            }
            let (name, _, _) = WINDOWS_1250.decode(&name_bytes);
            name.to_string()
        };

        // Skip 40 bytes unknown
        reader.read_u32::<LittleEndian>()?;
        reader.read_u32::<LittleEndian>()?;
        reader.read_u32::<LittleEndian>()?;
        reader.read_u32::<LittleEndian>()?;

        let role_description = {
            let mut desc_bytes = Vec::new();
            for _ in 0..40 {
                let byte = reader.read_u8()?;
                if byte == 0 {
                    break;
                }
                desc_bytes.push(byte);
            }
            let (desc, _, _) = WINDOWS_1250.decode(&desc_bytes);
            desc.to_string()
        };

        Ok(NpcRecord {
            counter1,
            counter2,
            counter3,
            counter4,

            // Name: 32 bytes null-terminated WINDOWS-1250
            name,

            // Role/description: 40 bytes null-terminated WINDOWS-1250
            role_description,
        })
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
        reader.seek(SeekFrom::Start(prefix_offset as u64))?;
        let prefix = reader.read_u8()?;

        // Skip to name
        reader.seek(SeekFrom::Start(name_offset as u64))?;

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

/// Event script record
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventScript {
    pub state: u8,
    pub script_name: String,
}

impl EventScript {
    /// Parse event script from 32-byte record
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut reader = std::io::Cursor::new(data);

        let state = reader.read_u8()?;

        // Skip 3 bytes padding
        reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        // Read script name (28 bytes null-terminated ASCII)
        let mut name_bytes = Vec::new();
        for _ in 0..28 {
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

/// Complete save file structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveFile {
    pub file_header: Vec<u8>,
    pub surface_monsters: Vec<MonsterRecord>,
    pub npcs: Vec<NpcRecord>,
    pub surface_objects: Vec<ExtraObjectRecord>,
    pub event_items: Vec<InventoryItem>,
    pub dungeon_map_id: u16,
    pub dungeon_monsters: Vec<MonsterRecord>,
    pub dungeon_objects: Vec<ExtraObjectRecord>,
    pub sprite_paths: Vec<String>,
    pub player_name: String,
    pub player_attributes: PlayerAttributes,
    pub inventory: Vec<InventoryItem>,
    pub potion_belt: Vec<PotionSlot>,
    pub event_scripts: Vec<EventScript>,
    pub quest_data: Vec<String>,
}

impl SaveFile {
    /// Parse complete save file from binary data
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut save = SaveFile::default();
        let mut reader = std::io::Cursor::new(data);

        // File header (12 bytes unknown + u32 count at 0x0C)
        save.file_header = data[0..12].to_vec();
        reader.seek(SeekFrom::Start(12))?;

        eprintln!("file_header: {:?}", save.file_header); // [202, 56, 2, 0, 2, 0, 0, 0, 0, 0, 0, 0] // i32: 145610 | 2 | 0 | 0, i16: 14538 | 2 | 2 | 0...0

        // Parse surface monsters (111 records × 329 bytes)
        let surface_monster_count = reader.read_u32::<LittleEndian>()? as usize; // 16-20th byte is a number of monsters
        eprintln!("surface_monster_count: {}", surface_monster_count); // 111

        save.surface_monsters = Vec::with_capacity(surface_monster_count);
        for _ in 0..surface_monster_count {
            let mut record = [0u8; 329];
            reader.read_exact(&mut record)?;
            save.surface_monsters.push(MonsterRecord::parse(&record)?);
        }
        // NPC separator (u32 = 146)
        let npc_count = reader.read_u32::<LittleEndian>()? as usize;
        eprintln!("npc_count: {}", npc_count);

        // Parse NPCs (146 records × 349 bytes)
        save.npcs = Vec::with_capacity(npc_count);
        for _ in 0..npc_count {
            let mut record = [0u8; 349];
            reader.read_exact(&mut record)?;
            save.npcs.push(NpcRecord::parse(&record)?);
            // 64 name
            // 64 role description
            // 221 bytes rest
        }

        let unknown = reader.read_u32::<LittleEndian>()?;
        eprintln!("unknown_u32: {}", unknown);

        // Surface object separator (u32 = 86)
        let surface_object_count = reader.read_u32::<LittleEndian>()? as usize;
        eprintln!("surface_object_count: {}", surface_object_count);

        let cur = reader.seek(SeekFrom::Current(0))?;
        eprintln!("cur: {}", cur); // 87493

        // Parse surface objects (86 records × 200 bytes)
        save.surface_objects = Vec::with_capacity(surface_object_count);
        for _ in 0..surface_object_count {
            let mut record = [0u8; 200];
            reader.read_exact(&mut record)?;
            save.surface_objects
                .push(ExtraObjectRecord::parse(&record, false)?);
        }

        let cur = reader.seek(SeekFrom::Current(0))?;
        eprintln!("cur: {}", cur); // 104701

        let mut unknown_bytes = [0u8; 19]; // 1
        reader.read_exact(&mut unknown_bytes)?;
        eprintln!("unknown_bytes: {:?}", unknown_bytes);

        let draw_item_count = reader.read_u16::<LittleEndian>()? as usize;
        eprintln!("draw_item_count: {}", draw_item_count);

        let cur = reader.seek(SeekFrom::Current(0))?;
        eprintln!("cur: {}", cur); // 104706 - now:104722

        for _ in 0..draw_item_count {
            // 30 = name
            // 202 = description
            // 4 = base price i32
            // 56 = unknown
            // 1 = item id u8
            // 1 = item type u8

            // KR save = 268 for monety - 36 + name 30 + decription 202
            // reader.read_u32::<LittleEndian>()?; // unknown
            // reader.read_u32::<LittleEndian>()?; // X coordinate
            // reader.read_u32::<LittleEndian>()?; // Y coordinate
            // reader.read_u8()?; // item_id
            // reader.read_u8()?; // item type
            // reader.read_u16::<LittleEndian>()?; // unknown
            // let mut name = [0u8; 30];
            // reader.read_exact(&mut name)?;
            // let mut description = [0u8; 202];
            // reader.read_exact(&mut description)?;

            let mut record = [0u8; 252];
            reader.read_exact(&mut record)?;
            // 16 bytes unknown 236
            // -- unknown i32
            // -- X i32
            // -- Y i32
            // -- item type i32 (1025 for kamień numy)
            // 30 bytes name 206
            // null terminator (1 byte)
            // 205 bytes description
            // null terminator (1 byte)
            // eprintln!("record: {:?}", record);
        }

        let cur = reader.seek(SeekFrom::Current(0))?;
        eprintln!("cur: {}", cur); // 105210 - now:105226

        // let unknown = reader.read_u32::<LittleEndian>()?;
        // eprintln!("unknown_u32: {}", unknown); // 11
        // let unknown = reader.read_u32::<LittleEndian>()?;
        // eprintln!("unknown_u32: {}", unknown); // 423
        // let unknown = reader.read_u32::<LittleEndian>()?;
        // eprintln!("unknown_u32: {}", unknown); // 176
        // let unknown = reader.read_u32::<LittleEndian>()?;
        // eprintln!("unknown_u32: {}", unknown); // 1036
        let unknown = reader.read_u32::<LittleEndian>()?;
        eprintln!("unknown_u32: {}", unknown); // 0

        let unknown = reader.read_u32::<LittleEndian>()?;
        eprintln!("unknown_u32: {}", unknown); // 7 - MAP ID Goblin's Cave

        let dungeon_monster_count = reader.read_u32::<LittleEndian>()? as usize;
        eprintln!("dungeon_monster_count: {}", dungeon_monster_count); // 107

        let unknown = reader.read_u32::<LittleEndian>()?;
        eprintln!("unknown_u32: {}", unknown); // 8
        let unknown = reader.read_u32::<LittleEndian>()?;
        eprintln!("unknown_u32: {}", unknown); // 2
                                               // let unknown = reader.read_u32::<LittleEndian>()?;
                                               // eprintln!("unknown_u32: {}", unknown); // 8

        for _ in 0..dungeon_monster_count {
            let mut record = [0u8; 329];
            reader.read_exact(&mut record)?;
            // eprintln!("record: {:?}", record);
        }

        let cur = reader.seek(SeekFrom::Current(0))?;
        eprintln!("cur: {}", cur); // 140449

        let dungeon_object_count = reader.read_u32::<LittleEndian>()? as usize;
        eprintln!("dungeon_object_count: {}", dungeon_object_count); // 23

        for _ in 0..dungeon_object_count {
            let mut record = [0u8; 200];
            reader.read_exact(&mut record)?;
            // eprintln!("record: {:?}", record);
        }

        let cur = reader.seek(SeekFrom::Current(0))?;
        eprintln!("cur: {}", cur); // 145053

        let unknown = reader.read_u32::<LittleEndian>()?;
        eprintln!("unknown_u32: {}", unknown); // 17

        // Unknown bytes
        let cur = reader.seek(SeekFrom::Start(cur + 569))?;
        eprintln!("cur: {}", cur); // 146026

        let block_20bytes_count: usize = (10180) / 20;
        eprintln!("block_20bytes_count: {}", block_20bytes_count); // 509
        for _ in 0..block_20bytes_count {
            let mut record = [0u8; 20];
            reader.read_exact(&mut record)?;
            eprint!("{} ", record[0]);
        }
        eprintln!("");

        let cur = reader.seek(SeekFrom::Current(0))?;
        eprintln!("cur 20b blocks: {}", cur); // 155806

        let unknown = reader.read_u32::<LittleEndian>()?;
        eprintln!("unknown_u32: {}", unknown); // 7

        let _ui_body_sprite_path = {
            let mut record = [0u8; 60];
            reader.read_exact(&mut record)?;
            record
        };
        let _ui_hair_sprite_path = {
            let mut record = [0u8; 60];
            reader.read_exact(&mut record)?;
            record
        }; // Path to inter/main .spr file
        let _game_body_sprite_path = {
            let mut record = [0u8; 60];
            reader.read_exact(&mut record)?;
            record
        };
        let _game_hair_sprite_path = {
            let mut record = [0u8; 60];
            reader.read_exact(&mut record)?;
            record
        }; // Path to CharacterInGame/... spr file

        let cur = reader.seek(SeekFrom::Current(0))?;
        eprintln!("cur: {}", cur); // 156046

        let unknown_character_details = {
            let mut record = [0u8; 40];
            reader.read_exact(&mut record)?;
            record
        };
        eprintln!("unknown_character_details {:?}", unknown_character_details);
        // [0, 0, 0, 0, 0, 0, 0, 0, 232, 0, 153, 1, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2, 48, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0,
        // 25, 0, 0, 0]

        let stat_strength = reader.read_u16::<LittleEndian>()?;
        eprintln!("stat_strength: {}", stat_strength); // 65
        let stat_agility = reader.read_u16::<LittleEndian>()?;
        eprintln!("stat_agility: {}", stat_agility); // 11
        let stat_wisdom = reader.read_u16::<LittleEndian>()?;
        eprintln!("stat_wisdom: {}", stat_wisdom); // 7
        let stat_constitution = reader.read_u16::<LittleEndian>()?;
        eprintln!("stat_constitution: {}", stat_constitution); // 21
        let stat_unknown = reader.read_u16::<LittleEndian>()?;
        eprintln!("stat_unknown: {}", stat_unknown); // 10 (probably: morale)
        let stat_current_hp = reader.read_u16::<LittleEndian>()?;
        eprintln!("stat_current_hp: {}", stat_current_hp); // 12
        let stat_max_hp = reader.read_u16::<LittleEndian>()?;
        eprintln!("stat_max_hp: {}", stat_max_hp); // 42
        let stat_current_mana = reader.read_u16::<LittleEndian>()?;
        eprintln!("stat_current_mana: {}", stat_current_mana); // 14
        let stat_max_mana = reader.read_u16::<LittleEndian>()?;
        eprintln!("stat_max_mana: {}", stat_max_mana); // 14
        let stat_current_xp = reader.read_u32::<LittleEndian>()?;
        eprintln!("stat_current_xp: {}", stat_current_xp); // 729
        let stat_current_level = reader.read_u16::<LittleEndian>()?;
        eprintln!("stat_current_level: {}", stat_current_level); // 5
        let stat_current_gold = reader.read_u32::<LittleEndian>()?;
        eprintln!("stat_current_gold: {}", stat_current_gold); // 1181

        let stat_unknown = reader.read_u16::<LittleEndian>()?;
        eprintln!("stat_unknown: {}", stat_unknown); // 102
        let stat_unknown = reader.read_u16::<LittleEndian>()?;
        eprintln!("stat_unknown: {}", stat_unknown); // 26

        let unknown_character_details = {
            let mut record = [0u8; 42];
            reader.read_exact(&mut record)?;
            record
        };
        eprintln!("unknown_character_details {:?}", unknown_character_details);
        // [ 12, 30, 9, 0, // 7692 or 597519
        // 0, 0, 0, 0,
        // 0, 0,
        // 1, 0,
        // 0, 4, // 1024
        // 59, 0,
        // 1, 0, // 1
        // 0, 1, // 256
        // 0, 0,
        // 1, 0,
        // 0, 1,
        // 0, 0,
        // 1, 0,
        // 0, 0,
        // 0, 0,
        // 0, 0,
        // 0, 0,
        // 0, 1,
        // 1, 0]

        // INVENTORY ---

        let cur = reader.seek(SeekFrom::Start(cur + 13982))?;
        eprintln!("cur: {}", cur); // 170028

        // (13982+4)/(7*9*3) = 74

        // 256 bytes per inventory record (not true - it is very inconsistent)
        // 246, 264, 266, 272 ... 272, 274, 292 ... 292, 256 ... 256,
        // 0x1C = 12
        // 294 bytes = Weapon (topór) - after it there is "0x1E" (i16) = 30 - maybe a third page
        // 292 bytes = Weapon (lekkie buty, skorzane spodnie, toga archanioła, sztylet)
        // 256 bytes = Healing
        for _ in 0..0 {
            let mut item_name = [0u8; 30];
            reader.read_exact(&mut item_name)?;
            let item_name = read_null_terminated_windows_1250(item_name.as_mut_slice());
            eprintln!("item_name: {:?}", item_name);

            let mut item_description = [0u8; 202];
            reader.read_exact(&mut item_description)?;
            let item_description =
                read_null_terminated_windows_1250(item_description.as_mut_slice());
            eprintln!("item_description: {:?}", item_description);

            let base_price = reader.read_i32::<LittleEndian>()?;
            eprintln!("base_price: {}", base_price); // 350 for Bloody Moss

            let mut unknown = [0u8; 56];
            reader.read_exact(&mut unknown)?;
        }

        // CHARACTER DETAILS ----

        let cur = reader.seek(SeekFrom::Current(0))?; // unknown bytes
        eprintln!("cur: {}", cur); // 170124

        let mut unknown = [0u8; 96]; // unknown bytes
        reader.read_exact(&mut unknown)?;
        eprintln!("unknown: {:?}", unknown); // 170124

        let mut character_name = [0u8; 11];
        reader.read_exact(&mut character_name)?;
        let character_name = read_null_terminated_windows_1250(character_name.as_mut_slice());
        eprintln!("character: {:?}", character_name);

        let character_class = reader.read_i16::<LittleEndian>()?;
        eprintln!("character_class: {}", character_class);

        let mut character_name = [0u8; 11];
        reader.read_exact(&mut character_name)?;
        let character_name = read_null_terminated_windows_1250(character_name.as_mut_slice());
        eprintln!("character: {:?}", character_name);

        let cur = reader.seek(SeekFrom::Current(0))?;
        eprintln!("cur: {}", cur); // 170148

        let cur = reader.seek(SeekFrom::Start(cur + 4040))?; // unknown bytes
        eprintln!("cur: {}", cur); // 174188

        // EVENTS ---

        let event_count: usize = 1 + 1593 + 657; // 2250

        // it starts with a `null` event
        for _ in 0..event_count {
            // 284 bytes for an event
            reader.read_u32::<LittleEndian>()?; // event ID (numeric ID => event0003 becomes just 3
            reader.read_u32::<LittleEndian>()?; // some unknown state (always zero?)
            reader.read_u32::<LittleEndian>()?; // the state (values: 2, 5...)
            let mut event_name_file = [0u8; 272];
            reader.read_exact(&mut event_name_file)?;
        }

        let cur = reader.seek(SeekFrom::Current(0))?;
        eprintln!("cur: {}", cur); // 813472

        let mut unknown = [0u8; 114];
        reader.read_exact(&mut unknown)?;

        // JOURNAL ---

        let journal_entries = 100;

        // JOURNAL - MAIN QUESTS

        // 37 bytes for quests (u8 + 36)
        for _ in 0..journal_entries {
            let _counter = reader.read_u8();

            let mut quest_name = [0u8; 32];
            reader.read_exact(&mut quest_name)?;
            let _quest_name = read_null_terminated_windows_1250(quest_name.as_mut_slice());
            // eprintln!("quest_name: {:?}", quest_name);

            let mut unknown_flags = [0u8; 4];
            reader.read_exact(&mut unknown_flags)?;
        }

        // JOURNAL - SIDE QUESTS

        // 37 bytes for quests (u8 + 36)
        for _ in 0..journal_entries {
            let _counter = reader.read_u8();

            let mut quest_name = [0u8; 32];
            reader.read_exact(&mut quest_name)?;
            let _quest_name = read_null_terminated_windows_1250(quest_name.as_mut_slice());
            // eprintln!("quest_name: {:?}", quest_name);

            let mut unknown_flags = [0u8; 4];
            reader.read_exact(&mut unknown_flags)?;
        }

        // JOURNAL - TRADERS / JOBS

        // 37 bytes for quests (u8 + 36)
        for _ in 0..journal_entries {
            let _counter = reader.read_u8();

            let mut quest_name = [0u8; 32];
            reader.read_exact(&mut quest_name)?;
            let _quest_name = read_null_terminated_windows_1250(quest_name.as_mut_slice());
            // eprintln!("quest_name: {:?}", quest_name);

            let mut unknown_flags = [0u8; 4];
            reader.read_exact(&mut unknown_flags)?;
        }

        let cur = reader.seek(SeekFrom::Current(0))?;
        eprintln!("cur: {}", cur); // 824686

        // ------

        // // Event item section (~533 bytes)
        // save.event_items = Vec::new();
        // let event_item_end = 0x19B12 as usize;
        // let event_item_start = 0x198FD as usize;
        // let event_item_size = event_item_end - event_item_start;

        // if event_item_size > 0 {
        //     let mut pos = event_item_start;
        //     while pos + 24 <= event_item_end {
        //         let record = &data[pos..pos + 24];
        //         if let Ok(item) = InventoryItem::parse(record) {
        //             save.event_items.push(item);
        //         }
        //         pos += 24;
        //     }
        // }

        // // Dungeon map ID (u16 at 0x19B0E)
        // reader.seek(SeekFrom::Start(0x19B0E))?;
        // save.dungeon_map_id = reader.read_u16::<LittleEndian>()?;

        // // Dungeon monster separator (u32 = 107)
        // let _dungeon_monster_count = reader.read_u32::<LittleEndian>()?;

        // // Parse dungeon monsters (107 records × 329 bytes)
        // save.dungeon_monsters = Vec::with_capacity(107);
        // for _ in 0..107 {
        //     let mut record = [0u8; 329];
        //     reader.read_exact(&mut record)?;
        //     save.dungeon_monsters.push(MonsterRecord::parse(&record)?);
        // }

        // // Dungeon object separator (u32 = 23)
        // let _dungeon_object_count = reader.read_u32::<LittleEndian>()?;

        // // Parse dungeon objects (23 records × 200 bytes)
        // save.dungeon_objects = Vec::with_capacity(23);
        // for _ in 0..23 {
        //     let mut record = [0u8; 200];
        //     reader.read_exact(&mut record)?;
        //     save.dungeon_objects
        //         .push(ExtraObjectRecord::parse(&record, true)?);
        // }

        // // Reserved/padding space (skip)
        // reader.seek(SeekFrom::Start(0x026076))?;

        // // Sprite paths (4 paths, null-terminated ASCII)
        // save.sprite_paths = Vec::with_capacity(4);
        // for _ in 0..4 {
        //     let mut path_bytes = Vec::new();
        //     loop {
        //         let byte = reader.read_u8()?;
        //         if byte == 0 {
        //             break;
        //         }
        //         path_bytes.push(byte);
        //     }
        //     save.sprite_paths
        //         .push(String::from_utf8(path_bytes).unwrap_or_else(|_| String::from("unknown")));
        // }

        // // Player name (8 bytes at 0x0261AE)
        // reader.seek(SeekFrom::Start(0x0261AE))?;
        // let mut name_bytes = [0u8; 8];
        // reader.read_exact(&mut name_bytes)?;
        // let (name, _, _) = WINDOWS_1250.decode(&name_bytes);
        // save.player_name = name.trim().to_string();

        // // Player attributes (32 bytes at 0x0261B6)
        // let attributes_data = &data[0x0261B6..0x0261D6];
        // save.player_attributes = PlayerAttributes::parse(attributes_data)?;

        // // Inventory section (0x026200 - 0x02A993)
        // let inventory_start = 0x026200;
        // let inventory_end = 0x02A993;

        // save.inventory = Vec::new();
        // let mut pos = inventory_start;
        // while pos + 502 <= inventory_end {
        //     // Quest name record (246 bytes)
        //     let quest_record = &data[pos..pos + 246];
        //     let quest_name = read_null_terminated_windows_1250(quest_record).unwrap_or_default();

        //     // Item data record (256 bytes)
        //     let item_record = &data[pos + 246..pos + 502];
        //     if let Ok(mut item) = InventoryItem::parse(item_record) {
        //         item.quest_name = quest_name;
        //         save.inventory.push(item);
        //     }

        //     pos += 502;
        // }

        // // Potion belt (6 slots × 256 bytes, likely after attributes)
        // save.potion_belt = Vec::with_capacity(6);
        // let potion_start = 0x0261D6; // After attributes (0x0261B6) + 32 bytes
        // for slot in 0..6 {
        //     let slot_start = potion_start + slot * 256;
        //     if slot_start + 256 <= data.len() {
        //         let slot_data = &data[slot_start..slot_start + 256];
        //         if let Ok(potion) = PotionSlot::parse(slot_data) {
        //             save.potion_belt.push(potion);
        //         }
        //     }
        // }

        // // Event scripts (32 bytes each, starting at 0x02A994)
        // save.event_scripts = Vec::new();
        // let script_start = 0x02A994;
        // let script_end = data.len().min(script_start + 1000); // Limit search

        // let mut pos = script_start;
        // while pos + 32 <= script_end {
        //     let record = &data[pos..pos + 32];
        //     if let Ok(script) = EventScript::parse(record) {
        //         save.event_scripts.push(script);
        //     }
        //     pos += 32;
        // }

        // // Quest data (near end of file)
        // save.quest_data = Vec::new();
        // // Simple heuristic: find WINDOWS-1250 strings near the end
        // let quest_start = (data.len() - 1000).max(script_end);
        // let quest_end = data.len();

        // let mut pos = quest_start;
        // while pos + 10 < quest_end {
        //     if data[pos] == 0 {
        //         pos += 1;
        //         continue;
        //     }
        //     let (decoded, _, had_errors) = WINDOWS_1250.decode(&data[pos..quest_end]);
        //     if !had_errors && decoded.len() > 3 {
        //         save.quest_data.push(decoded.to_string());
        //         pos += decoded.len();
        //     } else {
        //         pos += 1;
        //     }
        // }

        Ok(save)
    }
}

impl Extractor for SaveFile {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let save = SaveFile::parse(&data)?;
        Ok(vec![save])
    }

    fn to_writer<W: Write + Seek>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        if records.len() != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "SaveFile can only serialize one record at a time",
            ));
        }

        let save = &records[0];

        // Write file header
        writer.write_all(&save.file_header)?;

        // Write surface monsters count
        writer.write_u32::<LittleEndian>(save.surface_monsters.len() as u32)?;

        // Write surface monsters
        for _monster in &save.surface_monsters {
            // Write monster record (simplified - would need full serialization)
            writer.write_all(&[0u8; 329])?; // Placeholder
        }

        // Write NPC count separator
        writer.write_u32::<LittleEndian>(save.npcs.len() as u32)?;

        // Write NPCs
        for _npc in &save.npcs {
            writer.write_all(&[0u8; 349])?; // Placeholder
        }

        // Write surface object count separator
        writer.write_u32::<LittleEndian>(save.surface_objects.len() as u32)?;

        // Write surface objects
        for _obj in &save.surface_objects {
            writer.write_all(&[0u8; 200])?; // Placeholder
        }

        // Write event items
        for _item in &save.event_items {
            // Write 24-byte event item record
            writer.write_all(&[0u8; 24])?; // Placeholder
        }

        // Write dungeon map ID
        writer.write_u16::<LittleEndian>(save.dungeon_map_id)?;

        // Write dungeon monster count separator
        writer.write_u32::<LittleEndian>(save.dungeon_monsters.len() as u32)?;

        // Write dungeon monsters
        for _monster in &save.dungeon_monsters {
            writer.write_all(&[0u8; 329])?; // Placeholder
        }

        // Write dungeon object count separator
        writer.write_u32::<LittleEndian>(save.dungeon_objects.len() as u32)?;

        // Write dungeon objects
        for _obj in &save.dungeon_objects {
            writer.write_all(&[0u8; 200])?; // Placeholder
        }

        // Write sprite paths
        for path in &save.sprite_paths {
            writer.write_all(path.as_bytes())?;
            writer.write_u8(0)?;
        }

        // Write padding to reach 0x026076 (155,702 bytes)
        // We need to write enough zeros to reach position 0x026076
        // Since we can't seek on generic W, we'll write the padding directly
        // The exact amount depends on how much we've written so far
        // For now, write a conservative estimate
        let padding_size =
            (0x026076 as usize).saturating_sub(0x19B0E + 2 + save.sprite_paths.len() * 9);
        if padding_size > 0 {
            writer.write_all(&vec![0u8; padding_size])?;
        }

        // Write player name
        let (encoded, _, _) = WINDOWS_1250.encode(&save.player_name);
        writer.write_all(&encoded[..std::cmp::min(8, encoded.len())])?;
        if encoded.len() < 8 {
            writer.write_all(&vec![0u8; 8 - encoded.len()])?;
        }

        // Write player attributes
        save.player_attributes.write(writer)?;

        // Write inventory
        for item in &save.inventory {
            // Write quest name record
            let (encoded, _, _) = WINDOWS_1250.encode(&item.quest_name);
            writer.write_all(&encoded[..std::cmp::min(246, encoded.len())])?;
            if encoded.len() < 246 {
                writer.write_all(&vec![0u8; 246 - encoded.len()])?;
            }

            // Write item data record
            item.write(writer)?;
        }

        // Write event scripts
        for script in &save.event_scripts {
            writer.write_u8(script.state)?;
            writer.write_all(&[0u8; 3])?; // Padding
            writer.write_all(script.script_name.as_bytes())?;
            writer.write_u8(0)?;
            writer.write_all(&vec![0u8; 28 - script.script_name.len() - 1])?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_attributes_parse() {
        let data = [
            0x41, 0x00, // STR = 65
            0x0B, 0x00, // DEX = 11
            0x07, 0x00, // WIS = 7
            0x15, 0x00, // CON = 21
            0x0A, 0x00, // Unknown = 10
            0x0C, 0x00, // HP cur = 12
            0x2A, 0x00, // HP max = 42
            0x0E, 0x00, // MP cur = 14
            0x0E, 0x00, // MP max = 14
            0xD9, 0x02, 0x00, 0x00, // XP = 729
            0x05, 0x00, // Level = 5
            0x9D, 0x04, 0x00, 0x00, // Gold = 1181
        ];

        let attrs = PlayerAttributes::parse(&data).unwrap();
        assert_eq!(attrs.strength, 65);
        assert_eq!(attrs.dexterity, 11);
        assert_eq!(attrs.wisdom, 7);
        assert_eq!(attrs.constitution, 21);
        assert_eq!(attrs.unknown_stat, 10);
        assert_eq!(attrs.hp_current, 12);
        assert_eq!(attrs.hp_maximum, 42);
        assert_eq!(attrs.mp_current, 14);
        assert_eq!(attrs.mp_maximum, 14);
        assert_eq!(attrs.xp_current, 729);
        assert_eq!(attrs.level, 5);
        assert_eq!(attrs.gold, 1181);
    }

    #[test]
    fn test_inventory_item_parse() {
        // Simplified test - actual parsing would need full 256-byte record
        let data = [
            0x02, 0x00, 0x00, 0x00, // Field A: type=2 (Heal)
            0x04, 0x00, 0x00, 0x00, // Field B: item_id=4
            0x02, 0x00, // Quantity=2
            0x00, 0x00, // Padding
            b'w', b'y', b't', b'r', b'y', b'c', b'h',
            0, // "wytrych"
               // Rest would be description and padding
        ];

        let result = InventoryItem::parse(&data);
        assert!(result.is_ok());
        let item = result.unwrap();
        assert_eq!(item.item_type, SaveItemType::Heal);
        assert_eq!(item.item_id, 4);
        assert_eq!(item.quantity, 2);
        assert_eq!(item.name, "wytrych");
    }

    #[test]
    fn test_save_item_type_conversion() {
        assert_eq!(SaveItemType::from_u8(0), Some(SaveItemType::Weapon));
        assert_eq!(SaveItemType::from_u8(1), Some(SaveItemType::Armor));
        assert_eq!(SaveItemType::from_u8(2), Some(SaveItemType::Heal));
        assert_eq!(SaveItemType::from_u8(3), Some(SaveItemType::Misc));
        assert_eq!(SaveItemType::from_u8(4), Some(SaveItemType::Edit));
        assert_eq!(SaveItemType::from_u8(5), Some(SaveItemType::Event));
        assert_eq!(SaveItemType::from_u8(99), None);
    }
}
