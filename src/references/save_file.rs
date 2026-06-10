// Save file extraction and parsing for Dispel RPG
//
// This module provides comprehensive parsing of Dispel RPG save files (.sav)
// following the binary format documented in SAVE_FILE_RESEARCH.md

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::WINDOWS_1250;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};

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

        // Parse 10-byte header + 2-byte padding
        let field_a = reader.read_u32::<LittleEndian>()?;
        let field_b = reader.read_u32::<LittleEndian>()?;
        let quantity = reader.read_u16::<LittleEndian>()?;

        // Skip 2-byte padding after header
        reader.read_u16::<LittleEndian>()?;

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
    pub counter1: u32,
    pub counter2: u32,
    pub counter3: u32,
    pub counter4: u32,
    pub name: String,
    pub role_description: String,
}

impl NpcRecord {
    /// Parse NPC record from 349-byte data
    ///
    /// Record layout: 4×u32 (16 bytes) + 32-byte padding + 32-byte name +
    /// 40-byte padding + 40-byte role + 189 bytes trailing
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut reader = std::io::Cursor::new(data);

        let counter1 = reader.read_u32::<LittleEndian>()?;
        let counter2 = reader.read_u32::<LittleEndian>()?;
        let counter3 = reader.read_u32::<LittleEndian>()?;
        let counter4 = reader.read_u32::<LittleEndian>()?;

        // Skip 32 bytes padding (8 × u32)
        for _ in 0..8 {
            reader.read_u32::<LittleEndian>()?;
        }

        // Name: 32 bytes fixed-size field, null-terminated WINDOWS-1250
        let mut name_raw = [0u8; 32];
        reader.read_exact(&mut name_raw)?;
        let name_len = name_raw.iter().position(|&b| b == 0).unwrap_or(32);
        let (name, _, _) = WINDOWS_1250.decode(&name_raw[..name_len]);
        let name = name.to_string();

        // Skip 40 bytes unknown (10 × u32)
        for _ in 0..10 {
            reader.read_u32::<LittleEndian>()?;
        }

        // Role description: 40 bytes fixed-size field, null-terminated WINDOWS-1250
        let mut role_raw = [0u8; 40];
        reader.read_exact(&mut role_raw)?;
        let role_len = role_raw.iter().position(|&b| b == 0).unwrap_or(40);
        let (role_desc, _, _) = WINDOWS_1250.decode(&role_raw[..role_len]);
        let role_description = role_desc.to_string();

        Ok(NpcRecord {
            counter1,
            counter2,
            counter3,
            counter4,
            name,
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
    pub counter: u8,
    pub name: String,   // 32 bytes null-terminated WINDOWS-1250
    pub flags: u32,
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

        let counter = data[0];

        // Name: 32 bytes null-terminated WINDOWS-1250
        let name = {
            let mut name_bytes = Vec::new();
            for &byte in data[1..33].iter() {
                if byte == 0 {
                    break;
                }
                name_bytes.push(byte);
            }
            let (name, _, _) = WINDOWS_1250.decode(&name_bytes);
            name.to_string()
        };

        // Flags: u32 at offset 33
        let mut flag_bytes = [0u8; 4];
        flag_bytes.copy_from_slice(&data[33..37]);
        let flags = u32::from_le_bytes(flag_bytes);

        Ok(JournalEntry {
            counter,
            name,
            flags,
        })
    }

    /// Serialize journal entry to binary
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_u8(self.counter)?;

        // Write name (32 bytes, null-padded WINDOWS-1250)
        let (encoded, _, _) = WINDOWS_1250.encode(&self.name);
        let name_bytes = encoded.as_ref();
        let name_len = name_bytes.len().min(31); // Leave room for null terminator
        writer.write_all(&name_bytes[..name_len])?;
        // Pad with zeros to 32 bytes
        if name_len < 32 {
            let padding = 32 - name_len;
            writer.write_all(&vec![0u8; padding])?;
        }

        writer.write_u32::<LittleEndian>(self.flags)?;
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

/// Complete save file structure preserving all binary data for round-trip.
///
/// The authoritative binary data is stored in raw `Vec<u8>` sections.
/// Structured fields are derived on a best-effort basis and may be empty
/// for save files whose layout hasn't been mapped yet.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveFile {
    // ── Raw binary sections (authoritative for round-trip) ──
    pub header: [u8; 12],
    /// Raw bytes of all surface monsters (count × 329), parsed below
    pub surface_monsters_data: Vec<u8>,
    /// Raw bytes of all NPCs (count × 349)
    pub npcs_data: Vec<u8>,
    /// Raw bytes of all surface objects (count × 200)
    pub surface_objects_data: Vec<u8>,

    /// Everything from after surface_objects to EOF.
    pub remaining_data: Vec<u8>,

    // ── Parsed structured fields (best-effort from raw sections) ──
    pub surface_monsters: Vec<MonsterRecord>,
    pub npcs: Vec<NpcRecord>,
    pub surface_objects: Vec<ExtraObjectRecord>,

    // ── Parsed structured fields (best-effort from remaining_data) ──
    pub draw_items_data: Vec<u8>,
    pub dungeon_header_data: Vec<u8>,
    pub dungeon_map_id: u32,
    pub dungeon_monsters: Vec<MonsterRecord>,
    pub dungeon_objects: Vec<ExtraObjectRecord>,

    pub sprite_paths: Vec<String>,
    pub character_details: Vec<u8>,          // 40 bytes
    pub player_attributes: PlayerAttributes,
    pub extra_character_data: Vec<u8>,       // 46 bytes (2 u16s + 42 bytes)
    pub character_unknown_block: Vec<u8>,    // 96 bytes before character name
    pub player_name: String,
    pub player_class_id: i16,
    pub player_class_name: String,

    pub events: Vec<EventScript>,
    pub journal_main: Vec<JournalEntry>,   // 100 entries
    pub journal_side: Vec<JournalEntry>,   // 100 entries
    pub journal_trade: Vec<JournalEntry>,  // 100 entries
}

impl SaveFile {
    /// Parse complete save file from binary data
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        let mut reader = std::io::Cursor::new(data);

        // ── 1. HEADER (12 bytes) ──
        let mut header = [0u8; 12];
        reader.read_exact(&mut header)?;

        // ── 2. SURFACE MONSTERS ──
        let surface_monster_count = reader.read_u32::<LittleEndian>()? as usize;
        let mut surface_monsters_data = vec![0u8; surface_monster_count * 329];
        reader.read_exact(&mut surface_monsters_data)?;

        let mut surface_monsters = Vec::with_capacity(surface_monster_count);
        for chunk in surface_monsters_data.chunks_exact(329) {
            surface_monsters.push(MonsterRecord::parse(chunk)?);
        }

        // ── 3. NPCS ──
        let npc_count = reader.read_u32::<LittleEndian>()? as usize;
        let mut npcs_data = vec![0u8; npc_count * 349];
        reader.read_exact(&mut npcs_data)?;

        let mut npcs = Vec::with_capacity(npc_count);
        for chunk in npcs_data.chunks_exact(349) {
            npcs.push(NpcRecord::parse(chunk)?);
        }

        // ── 4. SURFACE OBJECTS ──
        let _unknown_separator = reader.read_u32::<LittleEndian>()?; // always 0
        let surface_object_count = reader.read_u32::<LittleEndian>()? as usize;
        let mut surface_objects_data = vec![0u8; surface_object_count * 200];
        reader.read_exact(&mut surface_objects_data)?;

        let mut surface_objects = Vec::with_capacity(surface_object_count);
        for chunk in surface_objects_data.chunks_exact(200) {
            surface_objects.push(ExtraObjectRecord::parse(chunk, false)?);
        }

        // ── 5. REMAINING DATA (everything after surface objects to EOF) ──
        let remaining_start = reader.position() as usize;
        let mut remaining_data = vec![0u8; data.len() - remaining_start];
        reader.read_exact(&mut remaining_data)?;

        // ── Best-effort structured field extraction ──
        // These are derived from remaining_data. If extraction fails for any
        // save file variant, the fields stay at their defaults.
        let mut character_details = Vec::new();
        let mut player_attributes = PlayerAttributes::default();
        let mut extra_character_data = Vec::new();
        let mut character_unknown_block = Vec::new();
        let mut player_name = String::new();
        let mut player_class_id: i16 = 0;
        let mut player_class_name = String::new();
        let mut events = Vec::new();
        let mut journal_main = Vec::new();
        let mut journal_side = Vec::new();
        let mut journal_trade = Vec::new();

        // Try to extract player identity (name + class) and tail sections.
        // This is best-effort — failures silently leave fields at defaults.
        Self::extract_player_identity(&remaining_data, &mut player_name,
            &mut player_class_id, &mut player_class_name);
        Self::extract_tail_sections(&remaining_data,
            &mut events, &mut journal_main, &mut journal_side, &mut journal_trade,
            &mut character_details, &mut player_attributes, &mut extra_character_data,
            &mut character_unknown_block);

        let draw_items_data = Vec::new();
        let dungeon_header_data = Vec::new();
        let dungeon_map_id = 0u32;
        let dungeon_monsters: Vec<MonsterRecord> = Vec::new();
        let dungeon_objects: Vec<ExtraObjectRecord> = Vec::new();
        let sprite_paths: Vec<String> = Vec::new();

        Ok(SaveFile {
            header,
            surface_monsters_data,
            npcs_data,
            surface_objects_data,
            remaining_data,
            surface_monsters,
            npcs,
            surface_objects,
            draw_items_data,
            dungeon_header_data,
            dungeon_map_id,
            dungeon_monsters,
            dungeon_objects,
            sprite_paths,
            character_details,
            player_attributes,
            extra_character_data,
            character_unknown_block,
            player_name,
            player_class_id,
            player_class_name,
            events,
            journal_main,
            journal_side,
            journal_trade,
        })
    }

    /// Best-effort: scan remaining_data for the 96-byte block + player name pattern.
    ///
    /// Layout: `[96-byte block (mostly zero)][name 11B][class_id i16][class_name 11B]`
    /// The 96-byte block typically has ~70+ zero bytes. We scan the entire
    /// remaining_data for this pattern.
    fn extract_player_identity(data: &[u8], name: &mut String, class_id: &mut i16, class_name: &mut String) {
        if data.len() < 150 {
            return;
        }
        let scan_end = data.len().saturating_sub(120);
        let mut offset = 0usize;
        while offset < scan_end {
            // Look for 96 bytes where ≥70 are zero
            let zero_count = data[offset..offset + 96].iter().filter(|&&b| b == 0).count();
            if zero_count < 70 {
                offset += 1;
                continue;
            }
            let after_block = &data[offset + 96..];
            if after_block.len() < 24 {
                offset += 1;
                continue;
            }
            // 11-byte name field (null-terminated WINDOWS-1250)
            let name_raw = &after_block[..11];
            let name_len = name_raw.iter().position(|&b| b == 0).unwrap_or(11);
            if name_len < 2 || name_len > 10 {
                offset += 1;
                continue;
            }
            // Name chars must be printable ASCII or extended Latin
            if !name_raw[..name_len].iter().all(|&b| b >= 0x20 && b <= 0x7E || b >= 0x80) {
                offset += 1;
                continue;
            }
            // class_id: i16 at offset 11
            let cid = i16::from_le_bytes([after_block[11], after_block[12]]);
            if cid < 1 || cid > 12 {
                offset += 1;
                continue;
            }
            // 11-byte class name field at offset 13
            let cls_raw = &after_block[13..24];
            let cls_len = cls_raw.iter().position(|&b| b == 0).unwrap_or(11);
            if cls_len < 3 || cls_len > 10 {
                offset += 1;
                continue;
            }
            // Class name chars must also be printable
            if !cls_raw[..cls_len].iter().all(|&b| b >= 0x20 && b <= 0x7E || b >= 0x80) {
                offset += 1;
                continue;
            }
            // Found! Decode and populate.
            let (decoded_name, _, _) = WINDOWS_1250.decode(&name_raw[..name_len]);
            let (decoded_cls, _, _) = WINDOWS_1250.decode(&cls_raw[..cls_len]);
            *name = decoded_name.to_string();
            *class_id = cid;
            *class_name = decoded_cls.to_string();
            return;
        }
    }

    /// Best-effort: find character_data_start in pre-events area.
    ///
    /// Character data starts after sprite paths (244 bytes). We scan backward
    /// from events_start for "inter\\" (start of first sprite path) to locate
    /// the sprite paths block, then character_data starts right after it.
    fn find_character_data_start(data: &[u8], events_start: usize) -> Option<usize> {
        let sprite_marker = b"inter\\";
        // Scan backward from events_start - 244 (minimum) up to ~50K before
        let scan_end = events_start.saturating_sub(50000);
        let scan_begin = events_start.saturating_sub(244);
        let mut pos = scan_begin.wrapping_sub(1); // start at scan_begin-1, go backward
        while pos > scan_end {
            if pos + 6 <= data.len() && &data[pos..pos + 6] == sprite_marker {
                // Found "inter\\" — sprite paths are 244 bytes: u32(separator) + 4×60B
                // The u32 separator is 4 bytes before the first sprite path
                // So character_data starts at pos - 4 + 244 = pos + 240
                let cd_start = pos + 240;
                if cd_start <= events_start && cd_start + 116 <= data.len() {
                    return Some(cd_start);
                }
            }
            pos = pos.wrapping_sub(1);
        }
        None
    }

    /// Best-effort: parse character data from remaining_data into structured fields.
    ///
    /// Layout: `[4B pad][40B details][26B attr][46B extra][inventory var][96B zero block][11B name]...`
    fn extract_character_data(data: &[u8], start: usize,
        character_details: &mut Vec<u8>,
        player_attributes: &mut PlayerAttributes,
        extra_character_data: &mut Vec<u8>,
        character_unknown_block: &mut Vec<u8>)
    {
        if start + 116 > data.len() {
            return;
        }
        // 4 bytes padding (skip)
        // 40 bytes character details
        let mut details = vec![0u8; 40];
        details.copy_from_slice(&data[start + 4..start + 44]);
        *character_details = details;
        // 26 bytes player attributes
        let mut attr_buf = [0u8; 26];
        attr_buf.copy_from_slice(&data[start + 44..start + 70]);
        if let Ok(pa) = PlayerAttributes::parse(&attr_buf) {
            *player_attributes = pa;
        }
        // 46 bytes extra character data
        let mut extra = vec![0u8; 46];
        extra.copy_from_slice(&data[start + 70..start + 116]);
        *extra_character_data = extra;
        // character_unknown_block (96 bytes after inventory) — we scan for it
        // the 96-byte block is preceded by inventory of unknown size,
        // but followed by the name+class block that extract_player_identity finds.
        // For now, leave it as best-effort via the scan above.
        let _ = character_unknown_block;
    }

    /// Best-effort: extract events, journal, and character data from remaining_data.
    ///
    /// Layout at end of remaining_data:
    /// `[pre-events var][events: N×284B][events_unknown: 114B][journal: 3×100×37 = 11100B]`
    ///
    /// The pre-events area has: `[section_table var][sprite_paths 244B][character_data var]`
    /// where character_data begins with:
    ///   `[4B padding][40B details][26B attributes][46B extra][inventory var][96B zero][name+class]`
    fn extract_tail_sections(data: &[u8],
        events: &mut Vec<EventScript>,
        journal_main: &mut Vec<JournalEntry>,
        journal_side: &mut Vec<JournalEntry>,
        journal_trade: &mut Vec<JournalEntry>,
        character_details: &mut Vec<u8>,
        player_attributes: &mut PlayerAttributes,
        extra_character_data: &mut Vec<u8>,
        character_unknown_block: &mut Vec<u8>)
    {
        const JOURNAL_SIZE: usize = 3 * 100 * 37; // 11,100
        const UNKNOWN_SIZE: usize = 114;
        const EVENT_SIZE: usize = 284;
        // Events are always exactly 2250 script entries + 1 null header = 2251 records.
        // The null event at index 0 has event_id=0, state=0, name="null".
        // Script events (index 1..2251) have event_id 1..2250, state=1 or 2.
        // Total events size = 2251 * 284 = 639,284
        const EVENTS_2251: usize = 2251 * EVENT_SIZE; // 639,284
        const TAIL: usize = EVENTS_2251 + UNKNOWN_SIZE + JOURNAL_SIZE; // 650,498

        if data.len() < TAIL {
            return;
        }
        // Events start at a fixed offset from the end.
        let pos = data.len() - TAIL;
        // Quick sanity: first event has event_id=0, state=0 (null event)
        if data[pos..pos + 4] != [0, 0, 0, 0] {
            return;
        }
        // Parse all 2251 events
        let mut parsed = Vec::with_capacity(2251);
        let all_ok = (0..2251).all(|i| {
            let chunk = &data[pos + i * EVENT_SIZE..pos + (i + 1) * EVENT_SIZE];
            EventScript::parse(chunk).map(|e| { parsed.push(e); true }).unwrap_or(false)
        });
        if !all_ok || parsed.is_empty() {
            return;
        }
        // Parse journal
        let journal_start = data.len() - JOURNAL_SIZE;
        let journal_raw = &data[journal_start..];
        if let (Ok(m), Ok(s), Ok(t)) = (
            Self::parse_journal_entries(&journal_raw[..3700], 100),
            Self::parse_journal_entries(&journal_raw[3700..7400], 100),
            Self::parse_journal_entries(&journal_raw[7400..], 100),
        ) {
            *events = parsed;
            *journal_main = m;
            *journal_side = s;
            *journal_trade = t;
            // Character data extraction (best-effort)
            if let Some(cd_start) = Self::find_character_data_start(data, pos) {
                Self::extract_character_data(data, cd_start,
                    character_details, player_attributes,
                    extra_character_data, character_unknown_block);
            }
        }
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

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        if records.len() != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "SaveFile can only serialize one record at a time",
            ));
        }

        let save = &records[0];

        // Write header (12 bytes)
        writer.write_all(&save.header)?;

        // Write surface monster count + raw data
        writer.write_u32::<LittleEndian>(save.surface_monsters.len() as u32)?;
        writer.write_all(&save.surface_monsters_data)?;

        // Write NPC count + raw data
        writer.write_u32::<LittleEndian>(save.npcs.len() as u32)?;
        writer.write_all(&save.npcs_data)?;

        // Write surface objects: separator (u32=0) + count + raw data
        writer.write_u32::<LittleEndian>(0)?;
        writer.write_u32::<LittleEndian>(save.surface_objects.len() as u32)?;
        writer.write_all(&save.surface_objects_data)?;

        // Write remaining data (everything from after surface objects to EOF)
        writer.write_all(&save.remaining_data)?;

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
            0, // "wytrych" null terminator
            0, // empty description (null terminator)
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

    #[test]
    fn test_journal_entry_parse() {
        let mut entry_data = vec![0u8; 37];
        entry_data[0] = 3; // counter
        let name_bytes = b"TestQuest";
        entry_data[1..1 + name_bytes.len()].copy_from_slice(name_bytes);
        entry_data[33..37].copy_from_slice(&2u32.to_le_bytes()); // flags = 2

        let entry = JournalEntry::parse(&entry_data).unwrap();
        assert_eq!(entry.counter, 3);
        assert_eq!(entry.name, "TestQuest");
        assert_eq!(entry.flags, 2);
    }

    #[test]
    fn test_journal_entry_write_round_trip() {
        let entry = JournalEntry {
            counter: 7,
            name: "FindTheOrb".to_string(),
            flags: 5,
        };

        let mut buf = Vec::new();
        entry.write(&mut buf).unwrap();
        assert_eq!(buf.len(), 37);

        let parsed = JournalEntry::parse(&buf).unwrap();
        assert_eq!(parsed.counter, 7);
        assert_eq!(parsed.name, "FindTheOrb");
        assert_eq!(parsed.flags, 5);
    }

    #[test]
    fn test_draw_item_parse() {
        let data = vec![0xABu8; 252];
        let item = DrawItem::parse(&data).unwrap();
        assert_eq!(item.data[0], 0xAB);
        assert_eq!(item.data[251], 0xAB);
    }

    #[test]
    fn test_draw_item_write() {
        let mut item_data = vec![0u8; 252];
        item_data[0] = 0x12;
        item_data[251] = 0x34;
        let item = DrawItem { data: item_data };

        let mut buf = Vec::new();
        item.write(&mut buf).unwrap();
        assert_eq!(buf.len(), 252);
        assert_eq!(buf[0], 0x12);
        assert_eq!(buf[251], 0x34);
    }

    #[test]
    fn test_event_script_parse_save_format() {
        // 284-byte event: u32 event_id, u32 unknown(0), u32 state(2), 272 bytes name
        let mut data = vec![0u8; 284];
        data[0..4].copy_from_slice(&1u32.to_le_bytes()); // event_id = 1
        data[4..8].copy_from_slice(&0u32.to_le_bytes()); // unknown = 0
        data[8..12].copy_from_slice(&2u32.to_le_bytes()); // state = 2 (completed)
        let name = b"event0003.scr";
        data[12..12 + name.len()].copy_from_slice(name);
        data[12 + name.len()] = 0; // null terminator

        let script = EventScript::parse(&data).unwrap();
        assert_eq!(script.state, 2);
        assert_eq!(script.script_name, "event0003.scr");
    }

    #[test]
    fn test_monster_record_parse() {
        // Minimal 329-byte monster record
        let mut data = vec![0u8; 329];
        data[0..4].copy_from_slice(&0xCAFEu32.to_le_bytes()); // signature_a
        data[4..8].copy_from_slice(&5u32.to_le_bytes()); // record_index
        data[8..12].copy_from_slice(&0xBEEFu32.to_le_bytes()); // signature_b
        let name = b"Goblin";
        data[12..12 + name.len()].copy_from_slice(name);
        data[12 + name.len()] = 0; // null terminator
        data[36..38].copy_from_slice(&30u16.to_le_bytes()); // hp_current = 30
        data[38..40].copy_from_slice(&50u16.to_le_bytes()); // hp_max = 50
        data[40..44].copy_from_slice(&3u32.to_le_bytes()); // state: dead(1) + poisoned(2) = 3
        data[44..46].copy_from_slice(&10u16.to_le_bytes()); // tile_x
        data[46..48].copy_from_slice(&20u16.to_le_bytes()); // tile_y

        let monster = MonsterRecord::parse(&data).unwrap();
        assert_eq!(monster.signature_a, 0xCAFE);
        assert_eq!(monster.record_index, 5);
        assert_eq!(monster.signature_b, 0xBEEF);
        assert_eq!(monster.name, "Goblin");
        assert_eq!(monster.hp_current, 30);
        assert_eq!(monster.hp_maximum, 50);
        assert!(monster.state.is_dead);
        assert!(monster.state.is_poisoned);
        assert_eq!(monster.tile_x, 10);
        assert_eq!(monster.tile_y, 20);
    }

    #[test]
    fn test_npc_record_parse() {
        let mut data = vec![0u8; 349];
        // 4 u32 counters
        data[0..4].copy_from_slice(&1u32.to_le_bytes());
        data[4..8].copy_from_slice(&2u32.to_le_bytes());
        data[8..12].copy_from_slice(&3u32.to_le_bytes());
        data[12..16].copy_from_slice(&4u32.to_le_bytes());
        // 32 bytes padding (offsets 16-47)
        // name at offset 48
        let name = b"Merchant";
        data[48..48 + name.len()].copy_from_slice(name);
        data[48 + name.len()] = 0;
        // 40 bytes padding after name (offset 80-119)
        // role at offset 120
        let role = b"Item Vendor";
        data[120..120 + role.len()].copy_from_slice(role);
        data[120 + role.len()] = 0;

        let npc = NpcRecord::parse(&data).unwrap();
        assert_eq!(npc.counter1, 1);
        assert_eq!(npc.counter2, 2);
        assert_eq!(npc.counter3, 3);
        assert_eq!(npc.counter4, 4);
        assert_eq!(npc.name, "Merchant");
        assert_eq!(npc.role_description, "Item Vendor");
    }

    #[test]
    fn test_extra_object_record_parse() {
        let mut data = vec![0u8; 200];
        // prefix at offset 14 for surface objects
        data[14] = 0xAB;
        // name at offset 15
        let name = b"Skrzynia";
        data[15..15 + name.len()].copy_from_slice(name);
        data[15 + name.len()] = 0;
        // state byte (chest state)
        data[15 + name.len() + 1] = 2; // closed

        let obj = ExtraObjectRecord::parse(&data, false).unwrap();
        assert_eq!(obj.prefix, 0xAB);
        assert_eq!(obj.name, "Skrzynia");
        assert_eq!(obj.state, 2);
    }

    #[test]
    fn test_player_attributes_write_round_trip() {
        let attrs = PlayerAttributes {
            strength: 10,
            dexterity: 12,
            wisdom: 8,
            constitution: 15,
            unknown_stat: 5,
            hp_current: 50,
            hp_maximum: 100,
            mp_current: 30,
            mp_maximum: 60,
            xp_current: 1500,
            level: 7,
            gold: 5000,
        };

        let mut buf = Vec::new();
        attrs.write(&mut buf).unwrap();
        // PlayerAttributes is 10 × u16 (20 bytes) + 2 × u32 (8 bytes) = 28 bytes
        assert_eq!(buf.len(), 28);

        let parsed = PlayerAttributes::parse(&buf).unwrap();
        assert_eq!(parsed.strength, 10);
        assert_eq!(parsed.dexterity, 12);
        assert_eq!(parsed.wisdom, 8);
        assert_eq!(parsed.constitution, 15);
        assert_eq!(parsed.unknown_stat, 5);
        assert_eq!(parsed.hp_current, 50);
        assert_eq!(parsed.hp_maximum, 100);
        assert_eq!(parsed.mp_current, 30);
        assert_eq!(parsed.mp_maximum, 60);
        assert_eq!(parsed.xp_current, 1500);
        assert_eq!(parsed.level, 7);
        assert_eq!(parsed.gold, 5000);
    }

    #[test]
    fn test_journal_entry_parse_too_short() {
        let data = [0u8; 10];
        let result = JournalEntry::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_draw_item_parse_too_short() {
        let data = [0u8; 100];
        let result = DrawItem::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_file_default() {
        let save = SaveFile::default();
        assert_eq!(save.surface_monsters.len(), 0);
        assert_eq!(save.npcs.len(), 0);
        assert_eq!(save.surface_objects.len(), 0);
        assert_eq!(save.player_name, "");
        assert_eq!(save.journal_main.len(), 0);
        assert_eq!(save.journal_side.len(), 0);
        assert_eq!(save.journal_trade.len(), 0);
    }

    // ── Round-trip tests against actual save files ──

    fn run_round_trip(path: &str) {
        let original = std::fs::read(path)
            .unwrap_or_else(|e| panic!("Failed to read {path}: {e}"));
        let save = SaveFile::parse(&original)
            .unwrap_or_else(|e| panic!("Failed to parse {path}: {e}"));

        let mut output = Vec::new();
        SaveFile::to_writer(&[save], &mut output)
            .unwrap_or_else(|e| panic!("Failed to write back {path}: {e}"));

        assert_eq!(
            original.len(),
            output.len(),
            "Size mismatch for {path}: original={} output={}",
            original.len(),
            output.len(),
        );

        if let Some((i, (a, b))) =
            original.iter().zip(output.iter()).enumerate().find(|(_, (a, b))| a != b)
        {
            panic!(
                "Byte mismatch at {i:#x} in {path}: \
                 original={a:#04x} output={b:#04x}",
            );
        }
    }

    #[test]
    fn round_trip_nuno_0_sav() {
        run_round_trip("nuno-0.sav");
    }

    #[test]
    fn round_trip_0_sav() {
        run_round_trip("0.sav");
    }

    #[test]
    fn round_trip_2_sav() {
        run_round_trip("2.sav");
    }
}
