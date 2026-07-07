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

/// Save-file inventory record layout:
///   [type: u32(4B)][name: 30B fixed cstr][desc: 234B fixed cstr][price: i32(4B)] = 272B
///
/// The name buffer may contain a binary prefix before the text name
/// (e.g. id/qty bytes embedded). `extract_text()` skips leading non-printable
/// bytes to find the readable portion.
const INVENTORY_RECORD_SIZE: usize = 4 + 30 + 234 + 4; // 272

/// Inventory item record from save file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryItem {
    /// Raw 4-byte location field (meaning of bytes unknown)
    pub location_raw: [u8; 4],
    /// Whether this is a quest item (no standard header, name-only)
    pub is_quest: bool,
    /// Item name (decoded from WINDOWS-1250)
    pub name: String,
    /// Item description (decoded from WINDOWS-1250)
    pub description: String,
    /// Item price from save file
    pub price: i32,
}

impl InventoryItem {
    /// Whether this is a quest item (no standard header, name-only)
    pub fn is_quest(&self) -> bool {
        self.is_quest
    }

    /// Extract readable CP1250 text from a fixed-size buffer.
    ///
    /// The name buffer may contain a binary prefix before the actual text
    /// (e.g. id/qty bytes). This function finds the first segment of
    /// consecutive printable characters that is at least 2 bytes long and
    /// starts with an alphabetic letter (ASCII or extended Latin).
    ///
    /// Segments shorter than 2 bytes or starting with a non-alphabetic
    /// printable character are skipped — they are likely binary junk that
    /// happens to fall in the printable range (e.g. `%`, `2`, `=`).
    fn extract_text(buf: &[u8]) -> String {
        let is_text_byte =
            |&b: &u8| -> bool { b.is_ascii_graphic() || b == b' ' || b == b'\t' || b >= 0x80 };

        let i = 0;
        let mut i = i;
        while i < buf.len() {
            if is_text_byte(&buf[i]) {
                let seg_start = i;
                while i < buf.len() && is_text_byte(&buf[i]) && buf[i] != 0 {
                    i += 1;
                }
                let seg_len = i - seg_start;
                // Accept segments >= 2 chars whose first byte looks like a
                // text character (alphabetic or extended Latin).
                if seg_len >= 2 && (buf[seg_start].is_ascii_alphabetic() || buf[seg_start] >= 0x80)
                {
                    let (decoded, _, _) = WINDOWS_1250.decode(&buf[seg_start..seg_start + seg_len]);
                    return decoded.trim().to_string();
                }
            } else {
                i += 1;
            }
        }

        String::new()
    }

    /// Extract the item name, falling back to the description buffer when
    /// the name buffer contains no readable text (binary-id-only items).
    fn extract_name_or_desc(name_buf: &[u8], desc_buf: &[u8]) -> String {
        let name = Self::extract_text(name_buf);
        if !name.is_empty() {
            return name;
        }
        // Many Edit/Misc/Event items store their real name in the desc
        // buffer (first text segment).
        Self::extract_text(desc_buf)
    }
}

/// Potion belt slot (6 dedicated slots for healing items)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PotionSlot {
    pub location_raw: [u8; 4],
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

        let location_raw = field_a.to_le_bytes();

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
            location_raw,
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
    pub name: String, // 32 bytes null-terminated WINDOWS-1250
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
    /// Character sprite paths (4 × 60-byte WINDOWS-1250 strings).
    pub sprite_paths: Vec<String>,
    /// Raw belt/quick-slot data (40 bytes before character stats).
    pub unknown_before_stats: Vec<u8>,
    /// Parsed character stats (core, combat, skills, weapon skills).
    pub character_stats: CharacterStats,
    /// Unknown bytes after stats block (9 bytes).
    pub unknown_after_stats: Vec<u8>,
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
            println!(
                "jump_addr_after_maps ({:?}) != reader.position() {:?}",
                jump_addr_after_maps,
                reader.position() as usize
            );

            reader.set_position(jump_addr_after_maps as u64);
        }

        // ── 3. Unknown block ──
        // It could be some rendered image of the save preview?
        // Let's jump by hardcoded number of bytes.

        let guessed_save_slot_id = reader.read_u32::<LittleEndian>()?;
        let guessed_save_win32_date = reader.read_u32::<LittleEndian>()?;
        let unknown_number = reader.read_u32::<LittleEndian>()?; // nuno_shereg_cat_0.sav: 4
        let unknown_number = reader.read_u32::<LittleEndian>()?; // nuno_shereg_cat_0.sav: 8
        let unknown_number = reader.read_u32::<LittleEndian>()?; // nuno_shereg_cat_0.sav: 0
        let monster_block_size = reader.read_u32::<LittleEndian>()?; // nuno_shereg_cat_0.sav: 329
        let npc_block_size = reader.read_u32::<LittleEndian>()?; // nuno_shereg_cat_0.sav: 349
        let unknown_number = reader.read_u32::<LittleEndian>()?; // nuno_shereg_cat_0.sav: 0
        let extra_object_block_size = reader.read_u32::<LittleEndian>()?; // nuno_shereg_cat_0.sav: 200

        let size: usize = 4 + 4 + 12 + 4 + 4 + 4 + 4;
        let size = (10188 + 4 * number_of_visited_map as usize) - size;
        let mut unknown_block = vec![0u8; size];

        reader.read_exact(&mut unknown_block)?;
        println!(
            "reader.position(unknown_block) {:?} (len: {}, cap: {})",
            reader.position() as usize,
            unknown_block.len(),
            unknown_block.capacity()
        );

        // ── 4. Character sprite paths (4 × 60-byte WINDOWS-1250 strings) ──
        let sprite_paths = Self::parse_sprite_paths(&mut reader)?;

        // ── 5 Character stats ──
        let (unknown_before_stats, character_stats, unknown_after_stats) =
            Self::parse_character_stats(&mut reader)?;

        // ── 6. Inventory ──
        // IDs what is in the belt?
        // Let's jump by hardcoded number of bytes.

        // ── 6.1. ITEMS IN INVENTORY - Event ──
        let inventory_event_count = reader.read_u16::<LittleEndian>()? as usize;
        if inventory_event_count > 0 {
            let mut inventory_event_data = vec![0u8; inventory_event_count * 244];
            reader.read_exact(&mut inventory_event_data)?;
        }

        // ── 6.2. ITEMS IN INVENTORY - Misc ──
        let inventory_misc_count = reader.read_u16::<LittleEndian>()? as usize;
        if inventory_misc_count > 0 {
            let mut inventory_misc_data = vec![0u8; inventory_misc_count * 264];
            reader.read_exact(&mut inventory_misc_data)?;
        }

        // ── 6.3. ITEMS IN INVENTORY - Edit ──
        let inventory_edit_count = reader.read_u16::<LittleEndian>()? as usize;
        if inventory_edit_count > 0 {
            let mut inventory_edit_data = vec![0u8; inventory_edit_count * 272];
            reader.read_exact(&mut inventory_edit_data)?;
        }

        // ── 6.4. ITEMS IN INVENTORY - Weapon ──
        let inventory_weapon_count = reader.read_u16::<LittleEndian>()? as usize;
        if inventory_weapon_count > 0 {
            let mut inventory_weapon_data = vec![0u8; inventory_weapon_count * 292];
            reader.read_exact(&mut inventory_weapon_data)?;
        }

        // ── 6.5. ITEMS IN INVENTORY - Heal ──
        let inventory_heal_count = reader.read_u16::<LittleEndian>()? as usize;
        if inventory_heal_count > 0 {
            let mut inventory_heal_data = vec![0u8; inventory_heal_count * 256];
            reader.read_exact(&mut inventory_heal_data)?;
        }

        // ── 7. Character details ──

        // 7.1. Unknown block of 88b
        let mut unknown_block = vec![0u8; 96]; // 0.sav Christoforo
                                               // let mut unknown_block = vec![0u8; 88]; // 0.sav Nuno
        reader.read_exact(&mut unknown_block)?;
        println!(
            "reader.position(unknown_block) {:?} {:?}",
            reader.position() as usize,
            unknown_block,
        );

        // 7.2 Character name
        let mut name_raw = vec![0u8; 11];
        reader.read_exact(&mut name_raw)?;
        let character_name = read_null_terminated_windows_1250(name_raw.as_slice());

        // 7.3 Character class
        let character_class_id = reader.read_u16::<LittleEndian>()?;
        let mut class_raw = vec![0u8; 11];
        reader.read_exact(&mut class_raw)?;
        let character_class_name = read_null_terminated_windows_1250(class_raw.as_slice());
        println!(
            "name: {:?}, class: {:?} ({:?})",
            character_name, character_class_name, character_class_id
        );

        // 7.4. Unknown data
        let mut unknown_block = vec![0u8; 4040]; // 0.sav Christoforo, 0.sav Nuno
        reader.read_exact(&mut unknown_block)?;

        // ── 8. Events ──
        for _ in 0..2251 {
            let mut event_raw = vec![0u8; 284];
            reader.read_exact(&mut event_raw)?;
            let _event = EventScript::parse(event_raw.as_slice())?;
            // println!("event: {:?}", event.script_name);
        }

        let seek_cursor = reader.position();
        println!("seek addr: {:?} - after events", seek_cursor);

        // ── 9. Unknown block ──
        // Not recognized why it is variable in size.
        let mut unknown_block = vec![0u8; 12]; // 0.sav Christoforo
        reader.read_exact(&mut unknown_block)?;
        println!(
            "reader.position(unknown_block) {:?} {:?}",
            reader.position() as usize,
            unknown_block,
        );

        let unknown_counter = reader.read_u32::<LittleEndian>()? as usize;
        let mut unknown_block = vec![0u8; unknown_counter * 24];
        reader.read_exact(&mut unknown_block)?;
        println!(
            "reader.position(unknown_block) {:?} {:?}",
            reader.position() as usize,
            unknown_block,
        );

        let mut unknown_block = vec![0u8; 98]; // 0.sav Christoforo
        reader.read_exact(&mut unknown_block)?;
        println!(
            "reader.position(unknown_block) {:?} {:?}",
            reader.position() as usize,
            unknown_block,
        );

        // ── 10. Journal ──

        // 10.1. Main quests
        let mut journal_raw = vec![0u8; 37 * 100];
        reader.read_exact(&mut journal_raw)?;
        let journal_main = Self::parse_journal_entries(&journal_raw, 100);
        match journal_main {
            Ok(journal) => println!("journal_main: {:?}", journal[0].name),
            Err(e) => println!("journal_main: {:?}", e),
        };

        let seek_cursor = reader.position();
        println!("seek addr: {:?} - main end", seek_cursor);

        // 10.2. Side quests
        let mut journal_raw = vec![0u8; 37 * 100];
        reader.read_exact(&mut journal_raw)?;
        let _journal_side = Self::parse_journal_entries(&journal_raw, 100);

        let seek_cursor = reader.position();
        println!("seek addr: {:?} - side end", seek_cursor);

        // 10.3. Trading offers
        let mut journal_raw = vec![0u8; 37 * 100];
        reader.read_exact(&mut journal_raw)?;
        let _journal_trade = Self::parse_journal_entries(&journal_raw, 100);

        let seek_cursor = reader.position();
        println!("seek addr: {:?} - journal end", seek_cursor);

        Ok(SaveFile {
            jump_addr_after_maps: jump_addr_after_maps as u32,
            maps,
            sprite_paths,
            unknown_before_stats,
            character_stats,
            unknown_after_stats,
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
        // ── Belt data (40 bytes, purpose unknown) ──
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

    /// Best-effort: scan remaining_data for the 96-byte block + player name pattern.
    ///
    /// Layout: `[96-byte block (mostly zero)][name 11B][class_id i16][class_name 11B]`
    /// The 96-byte block typically has ~70+ zero bytes.
    ///
    /// We scan BACKWARD from the end because the player identity block is always
    /// the LAST section before events_start — the closest match to the end wins.
    fn extract_player_identity(
        data: &[u8],
        name: &mut String,
        class_id: &mut i16,
        class_name: &mut String,
    ) {
        if data.len() < 150 {
            return;
        }
        // Scan backward from (data.len() - 120) down to 0
        let max = data.len().saturating_sub(120);
        for offset in (0..=max).rev() {
            // Look for 96 bytes where ≥72 are zero (stricter than 70 to filter noise)
            let zero_count = data[offset..offset + 96]
                .iter()
                .filter(|&&b| b == 0)
                .count();
            if zero_count >= 72 {
                let after_block = &data[offset + 96..];
                // 11-byte name field (null-terminated WINDOWS-1250)
                let name_raw = &after_block[..11];
                let name_len = name_raw.iter().position(|&b| b == 0).unwrap_or(11);
                if name_len >= 3 && name_len <= 10 {
                    // Name must start with ASCII uppercase letter (A-Z)
                    if name_raw[0] >= 0x41 && name_raw[0] <= 0x5A {
                        // Name chars: printable ASCII or extended Latin
                        if name_raw[..name_len]
                            .iter()
                            .all(|&b| b >= 0x20 && b <= 0x7E || b >= 0x80)
                        {
                            // class_id: i16 at offset 11
                            let cid = i16::from_le_bytes([after_block[11], after_block[12]]);
                            if cid >= 1 && cid <= 12 {
                                // 11-byte class name field at offset 13
                                let cls_raw = &after_block[13..24];
                                let cls_len = cls_raw.iter().position(|&b| b == 0).unwrap_or(11);
                                if cls_len >= 3 && cls_len <= 10 {
                                    if cls_raw[..cls_len]
                                        .iter()
                                        .all(|&b| b >= 0x20 && b <= 0x7E || b >= 0x80)
                                    {
                                        // Found! Decode and populate.
                                        let (decoded_name, _, _) =
                                            WINDOWS_1250.decode(&name_raw[..name_len]);
                                        let (decoded_cls, _, _) =
                                            WINDOWS_1250.decode(&cls_raw[..cls_len]);
                                        *name = decoded_name.to_string();
                                        *class_id = cid;
                                        *class_name = decoded_cls.to_string();
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Best-effort: find character_data_start in pre-events area.
    ///
    /// Character data starts after the sprite paths block. The block has:
    ///   [u32(?=7)][u32(?=7)][4×60B null-terminated sprite paths] = 248 bytes
    ///
    /// The first 2 paths use "inter\\..." prefix; paths 2-3 use "CharacterInGame\\..."
    /// We scan backward from events_start for any "inter\\" occurrence, then walk
    /// backward by 60-byte intervals (the path stride) to find the first path in
    /// the block. Character data starts at first_path_offset + 240.
    fn find_character_data_start(data: &[u8], events_start: usize) -> Option<usize> {
        let sprite_marker = b"inter\\";
        let mut pos = events_start.wrapping_sub(1);
        while pos > 0 && pos + 6 <= data.len() {
            if &data[pos..pos + 6] == sprite_marker {
                // Found "inter\\" — this is one of the first two sprite paths.
                // Walk backward by 60-byte intervals to find the FIRST path.
                let mut first = pos;
                while first >= 60 {
                    let prev = first - 60;
                    if &data[prev..prev + 6] == sprite_marker {
                        first = prev;
                    } else {
                        break;
                    }
                }
                // first is the earliest "inter\\" in this consecutive block.
                // The sprite block header (8 bytes) is before first.
                // Total block = 8 + 240 = 248 bytes, so cd = first - 8 + 248 = first + 240.
                let cd_start = first + 240;
                if cd_start <= events_start && cd_start + 118 <= data.len() {
                    return Some(cd_start);
                }
            }
            pos = pos.wrapping_sub(1);
        }
        None
    }

    /// Find the 96-byte zero block that marks the end of inventory data.
    ///
    /// Scans forward from `inv_start` for a block of 96 bytes with ≥72 zeros,
    /// followed by a valid player name pattern (matching extract_player_identity).
    /// This avoids false positives from zero padding between inventory items.
    fn find_inventory_end(data: &[u8], inv_start: usize) -> Option<usize> {
        if inv_start >= data.len() {
            return None;
        }
        let mut pos = inv_start;
        while pos + 96 + 24 <= data.len() {
            let zero_count = data[pos..pos + 96].iter().filter(|&&b| b == 0).count();
            if zero_count >= 72 {
                // Validate what follows: should be player name (same logic as extract_player_identity)
                let after = &data[pos + 96..];
                let name_raw = &after[..11];
                let name_len = name_raw.iter().position(|&b| b == 0).unwrap_or(11);
                if name_len >= 3 && name_len <= 10 && name_raw[0] >= 0x41 && name_raw[0] <= 0x5A {
                    let cid = i16::from_le_bytes([after[11], after[12]]);
                    if cid >= 1 && cid <= 12 {
                        return Some(pos);
                    }
                }
            }
            pos += 1;
        }
        None
    }

    /// Best-effort: parse inventory items from the area after character data.
    ///
    /// Inventory layout in remaining_data after extra_character_data (cd_start + 114):
    ///   `[quest_items var][standard_items: N×272B][96B zero block][name+class]`
    ///
    /// Standard items use a fixed 272-byte record:
    ///   [type: u32(4B)][name: 30B cstr][desc: 234B cstr][price: i32(4B)]
    ///
    /// The name buffer may contain binary data before the readable text;
    /// `InventoryItem::extract_text()` handles this by scanning past non-printable bytes.
    ///
    /// Quest items precede the standard items and have no header — just a null-terminated name.
    ///
    /// Note: `item_id` and `quantity` are NOT stored in the save file's inventory records.
    /// The current InventoryItem struct omits them since this is a save-file parser.
    fn parse_inventory(data: &[u8], cd_start: usize) -> Vec<InventoryItem> {
        let inv_start = cd_start + 114;
        if inv_start >= data.len() {
            return Vec::new();
        }
        let inv_end = match Self::find_inventory_end(data, inv_start) {
            Some(pos) => pos,
            None => return Vec::new(),
        };
        if inv_end <= inv_start {
            return Vec::new();
        }

        let inv = &data[inv_start..inv_end];
        let mut items = Vec::new();
        let mut pos = 0;

        while pos < inv.len() {
            // Skip zero bytes (padding/alignment)
            while pos < inv.len() && inv[pos] == 0 {
                pos += 1;
            }
            if pos >= inv.len() {
                break;
            }

            // Try standard item: 272B record, 4B type (must be 1-5)
            if pos + INVENTORY_RECORD_SIZE <= inv.len() {
                let type_val = u32::from_le_bytes(inv[pos..pos + 4].try_into().unwrap());
                let item_type_id = (type_val & 0xFF) as u8;

                if (1..=5).contains(&item_type_id) {
                    let name_buf = &inv[pos + 4..pos + 4 + 30];
                    let desc_buf = &inv[pos + 4 + 30..pos + 4 + 30 + 234];
                    let name = InventoryItem::extract_name_or_desc(name_buf, desc_buf);

                    if !name.is_empty() {
                        // Extract description (stats text) from desc buffer,
                        // skipping the leading segment that we may have used as name
                        let description = InventoryItem::extract_text(desc_buf);

                        // Extract price (i32 at end of record)
                        let price_bytes: [u8; 4] = inv[pos + 4 + 30 + 234..pos + 4 + 30 + 234 + 4]
                            .try_into()
                            .unwrap();
                        let price = i32::from_le_bytes(price_bytes);

                        items.push(InventoryItem {
                            location_raw: type_val.to_le_bytes(),
                            is_quest: false,
                            name,
                            description,
                            price,
                        });

                        pos += INVENTORY_RECORD_SIZE;
                        continue;
                    }
                }
            }

            // Not a standard header — try quest item (null-terminated name only)
            let mut name_end = pos;
            while name_end < inv.len() && inv[name_end] != 0 {
                name_end += 1;
            }

            if name_end > pos && name_end < inv.len() {
                let (name, _, _) = WINDOWS_1250.decode(&inv[pos..name_end]);
                items.push(InventoryItem {
                    location_raw: [0; 4],
                    is_quest: true,
                    name: name.to_string(),
                    description: String::new(),
                    price: 0,
                });
                pos = name_end + 1;
                continue;
            }

            // Can't parse this byte — skip it
            pos += 1;
        }

        items
    }

    /// Best-effort: parse character data from remaining_data into structured fields.

    /// Layout in save files (observed for nuno-0.sav):
    ///   `[4B pad][40B details][24B save-attrs][46B extra][inventory var][96B zero block][11B name]...`
    ///
    /// The save-file attribute layout (24 bytes) uses a DIFFERENT field order
    /// than PlayerAttributes (28 bytes). Specifically the save has NO MP fields
    /// in this block and XP/LVL/GOLD are at different offsets:
    ///
    ///   Save:    STR/DEX/WIS/CON/LCK/HP_CUR/HP_MAX(7×u16=14B) + XP(u32=4B) + LVL(u16=2B) + GOLD(u32=4B) = 24B
    ///   Struct:  STR/DEX/WIS/CON/LCK/HP_CUR/HP_MAX/MP_CUR/MP_MAX(9×u16=18B) + XP(u32=4B) + LVL(u16=2B) + GOLD(u32=4B) = 28B
    ///
    /// We manually map because the struct expects MP fields between HP and XP.
    fn extract_character_data(
        data: &[u8],
        start: usize,
        character_details: &mut Vec<u8>,
        player_attributes: &mut PlayerAttributes,
        extra_character_data: &mut Vec<u8>,
        character_unknown_block: &mut Vec<u8>,
    ) {
        if start + 114 > data.len() {
            return;
        }
        *character_unknown_block = Vec::new(); // not populated yet
                                               // 4 bytes padding (skip)
                                               // 40 bytes character details
        let mut details = vec![0u8; 40];
        details.copy_from_slice(&data[start + 4..start + 44]);
        *character_details = details;
        // 24 bytes save attributes — manually map to PlayerAttributes
        let save_attrs = &data[start + 44..start + 68];
        let mut pa = PlayerAttributes::default();
        pa.strength = u16::from_le_bytes([save_attrs[0], save_attrs[1]]);
        pa.dexterity = u16::from_le_bytes([save_attrs[2], save_attrs[3]]);
        pa.wisdom = u16::from_le_bytes([save_attrs[4], save_attrs[5]]);
        pa.constitution = u16::from_le_bytes([save_attrs[6], save_attrs[7]]);
        pa.unknown_stat = u16::from_le_bytes([save_attrs[8], save_attrs[9]]);
        pa.hp_current = u16::from_le_bytes([save_attrs[10], save_attrs[11]]);
        pa.hp_maximum = u16::from_le_bytes([save_attrs[12], save_attrs[13]]);
        // Save has no MP fields; XP at bytes 14-17 as u32, LVL at bytes 18-19, GOLD at bytes 20-23 as u32
        pa.xp_current = u32::from_le_bytes([
            save_attrs[14],
            save_attrs[15],
            save_attrs[16],
            save_attrs[17],
        ]);
        pa.level = u16::from_le_bytes([save_attrs[18], save_attrs[19]]);
        pa.gold = u32::from_le_bytes([
            save_attrs[20],
            save_attrs[21],
            save_attrs[22],
            save_attrs[23],
        ]);
        // MP fields stay at 0 (not present in save attrs block)
        *player_attributes = pa;
        // 46 bytes extra character data (unknown structure)
        let mut extra = vec![0u8; 46];
        extra.copy_from_slice(&data[start + 68..start + 114]);
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
    fn extract_tail_sections(
        data: &[u8],
        events: &mut Vec<EventScript>,
        journal_main: &mut Vec<JournalEntry>,
        journal_side: &mut Vec<JournalEntry>,
        journal_trade: &mut Vec<JournalEntry>,
        character_details: &mut Vec<u8>,
        player_attributes: &mut PlayerAttributes,
        extra_character_data: &mut Vec<u8>,
        character_unknown_block: &mut Vec<u8>,
        inventory_items: &mut Vec<InventoryItem>,
    ) {
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
            EventScript::parse(chunk)
                .map(|e| {
                    parsed.push(e);
                    true
                })
                .unwrap_or(false)
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
                Self::extract_character_data(
                    data,
                    cd_start,
                    character_details,
                    player_attributes,
                    extra_character_data,
                    character_unknown_block,
                );
                // Inventory parsing (best-effort)
                *inventory_items = Self::parse_inventory(data, cd_start);
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_player_attributes_parse() {
//         let data = [
//             0x41, 0x00, // STR = 65
//             0x0B, 0x00, // DEX = 11
//             0x07, 0x00, // WIS = 7
//             0x15, 0x00, // CON = 21
//             0x0A, 0x00, // Unknown = 10
//             0x0C, 0x00, // HP cur = 12
//             0x2A, 0x00, // HP max = 42
//             0x0E, 0x00, // MP cur = 14
//             0x0E, 0x00, // MP max = 14
//             0xD9, 0x02, 0x00, 0x00, // XP = 729
//             0x05, 0x00, // Level = 5
//             0x9D, 0x04, 0x00, 0x00, // Gold = 1181
//         ];
//
//         let attrs = PlayerAttributes::parse(&data).unwrap();
//         assert_eq!(attrs.strength, 65);
//         assert_eq!(attrs.dexterity, 11);
//         assert_eq!(attrs.wisdom, 7);
//         assert_eq!(attrs.constitution, 21);
//         assert_eq!(attrs.unknown_stat, 10);
//         assert_eq!(attrs.hp_current, 12);
//         assert_eq!(attrs.hp_maximum, 42);
//         assert_eq!(attrs.mp_current, 14);
//         assert_eq!(attrs.mp_maximum, 14);
//         assert_eq!(attrs.xp_current, 729);
//         assert_eq!(attrs.level, 5);
//         assert_eq!(attrs.gold, 1181);
//     }
//
//     #[test]
//     fn test_inventory_item_extract_text() {
//         // Name with binary prefix followed by readable text
//         let mut name_buf = [0u8; 30];
//         // Simulate "wytrych" with 6 bytes of binary prefix + 22 bytes zero padding
//         name_buf[0..6].copy_from_slice(&[0x04, 0x00, 0x00, 0x00, 0x02, 0x00]);
//         name_buf[6..14].copy_from_slice(b"wytrych\0");
//         let name = InventoryItem::extract_text(&name_buf);
//         assert_eq!(name, "wytrych");
//
//         // Name starting at byte 0 (no binary prefix)
//         let mut name_buf2 = [0u8; 30];
//         name_buf2[..14].copy_from_slice(b"Kostka wladzy\0");
//         let name2 = InventoryItem::extract_text(&name_buf2);
//         assert_eq!(name2, "Kostka wladzy");
//
//         // Empty buffer
//         let empty = [0u8; 30];
//         assert_eq!(InventoryItem::extract_text(&empty), "");
//     }
//
//     #[test]
//     fn test_inventory_location_raw() {
//         // Verify that the type field bytes are preserved as-is
//         let item = InventoryItem {
//             location_raw: [1, 0, 0, 0],
//             is_quest: false,
//             name: "Test".to_string(),
//             description: String::new(),
//             price: 0,
//         };
//         assert!(!item.is_quest());
//         assert_eq!(item.location_raw[0], 1);
//     }
//
//     #[test]
//     fn test_journal_entry_parse() {
//         let mut entry_data = vec![0u8; 37];
//         entry_data[0] = 3; // counter
//         let name_bytes = b"TestQuest";
//         entry_data[1..1 + name_bytes.len()].copy_from_slice(name_bytes);
//         entry_data[33..37].copy_from_slice(&2u32.to_le_bytes()); // flags = 2
//
//         let entry = JournalEntry::parse(&entry_data).unwrap();
//         assert_eq!(entry.counter, 3);
//         assert_eq!(entry.name, "TestQuest");
//         assert_eq!(entry.flags, 2);
//     }
//
//     #[test]
//     fn test_journal_entry_write_round_trip() {
//         let entry = JournalEntry {
//             counter: 7,
//             name: "FindTheOrb".to_string(),
//             flags: 5,
//         };
//
//         let mut buf = Vec::new();
//         entry.write(&mut buf).unwrap();
//         assert_eq!(buf.len(), 37);
//
//         let parsed = JournalEntry::parse(&buf).unwrap();
//         assert_eq!(parsed.counter, 7);
//         assert_eq!(parsed.name, "FindTheOrb");
//         assert_eq!(parsed.flags, 5);
//     }
//
//     #[test]
//     fn test_draw_item_parse() {
//         let data = vec![0xABu8; 252];
//         let item = DrawItem::parse(&data).unwrap();
//         assert_eq!(item.data[0], 0xAB);
//         assert_eq!(item.data[251], 0xAB);
//     }
//
//     #[test]
//     fn test_draw_item_write() {
//         let mut item_data = vec![0u8; 252];
//         item_data[0] = 0x12;
//         item_data[251] = 0x34;
//         let item = DrawItem { data: item_data };
//
//         let mut buf = Vec::new();
//         item.write(&mut buf).unwrap();
//         assert_eq!(buf.len(), 252);
//         assert_eq!(buf[0], 0x12);
//         assert_eq!(buf[251], 0x34);
//     }
//
//     #[test]
//     fn test_event_script_parse_save_format() {
//         // 284-byte event: u32 event_id, u32 unknown(0), u32 state(2), 272 bytes name
//         let mut data = vec![0u8; 284];
//         data[0..4].copy_from_slice(&1u32.to_le_bytes()); // event_id = 1
//         data[4..8].copy_from_slice(&0u32.to_le_bytes()); // unknown = 0
//         data[8..12].copy_from_slice(&2u32.to_le_bytes()); // state = 2 (completed)
//         let name = b"event0003.scr";
//         data[12..12 + name.len()].copy_from_slice(name);
//         data[12 + name.len()] = 0; // null terminator
//
//         let script = EventScript::parse(&data).unwrap();
//         assert_eq!(script.state, 2);
//         assert_eq!(script.script_name, "event0003.scr");
//     }
//
//     #[test]
//     fn test_monster_record_parse() {
//         // Minimal 329-byte monster record
//         let mut data = vec![0u8; 329];
//         data[0..4].copy_from_slice(&0xCAFEu32.to_le_bytes()); // signature_a
//         data[4..8].copy_from_slice(&5u32.to_le_bytes()); // record_index
//         data[8..12].copy_from_slice(&0xBEEFu32.to_le_bytes()); // signature_b
//         let name = b"Goblin";
//         data[12..12 + name.len()].copy_from_slice(name);
//         data[12 + name.len()] = 0; // null terminator
//         data[36..38].copy_from_slice(&30u16.to_le_bytes()); // hp_current = 30
//         data[38..40].copy_from_slice(&50u16.to_le_bytes()); // hp_max = 50
//         data[40..44].copy_from_slice(&3u32.to_le_bytes()); // state: dead(1) + poisoned(2) = 3
//         data[44..46].copy_from_slice(&10u16.to_le_bytes()); // tile_x
//         data[46..48].copy_from_slice(&20u16.to_le_bytes()); // tile_y
//
//         let monster = MonsterRecord::parse(&data).unwrap();
//         assert_eq!(monster.signature_a, 0xCAFE);
//         assert_eq!(monster.record_index, 5);
//         assert_eq!(monster.signature_b, 0xBEEF);
//         assert_eq!(monster.name, "Goblin");
//         assert_eq!(monster.hp_current, 30);
//         assert_eq!(monster.hp_maximum, 50);
//         assert!(monster.state.is_dead);
//         assert!(monster.state.is_poisoned);
//         assert_eq!(monster.tile_x, 10);
//         assert_eq!(monster.tile_y, 20);
//     }
//
//     #[test]
//     fn test_npc_record_parse() {
//         let mut data = vec![0u8; 349];
//         // 4 u32 counters
//         data[0..4].copy_from_slice(&1u32.to_le_bytes());
//         data[4..8].copy_from_slice(&2u32.to_le_bytes());
//         data[8..12].copy_from_slice(&3u32.to_le_bytes());
//         data[12..16].copy_from_slice(&4u32.to_le_bytes());
//         // 32 bytes padding (offsets 16-47)
//         // name at offset 48
//         let name = b"Merchant";
//         data[48..48 + name.len()].copy_from_slice(name);
//         data[48 + name.len()] = 0;
//         // 40 bytes padding after name (offset 80-119)
//         // role at offset 120
//         let role = b"Item Vendor";
//         data[120..120 + role.len()].copy_from_slice(role);
//         data[120 + role.len()] = 0;
//
//         let npc = NpcRecord::parse(&data).unwrap();
//         assert_eq!(npc.counter1, 1);
//         assert_eq!(npc.counter2, 2);
//         assert_eq!(npc.counter3, 3);
//         assert_eq!(npc.counter4, 4);
//         assert_eq!(npc.name, "Merchant");
//         assert_eq!(npc.role_description, "Item Vendor");
//     }
//
//     #[test]
//     fn test_extra_object_record_parse() {
//         let mut data = vec![0u8; 200];
//         // prefix at offset 14 for surface objects
//         data[14] = 0xAB;
//         // name at offset 15
//         let name = b"Skrzynia";
//         data[15..15 + name.len()].copy_from_slice(name);
//         data[15 + name.len()] = 0;
//         // state byte (chest state)
//         data[15 + name.len() + 1] = 2; // closed
//
//         let obj = ExtraObjectRecord::parse(&data, false).unwrap();
//         assert_eq!(obj.prefix, 0xAB);
//         assert_eq!(obj.name, "Skrzynia");
//         assert_eq!(obj.state, 2);
//     }
//
//     #[test]
//     fn test_player_attributes_write_round_trip() {
//         let attrs = PlayerAttributes {
//             strength: 10,
//             dexterity: 12,
//             wisdom: 8,
//             constitution: 15,
//             unknown_stat: 5,
//             hp_current: 50,
//             hp_maximum: 100,
//             mp_current: 30,
//             mp_maximum: 60,
//             xp_current: 1500,
//             level: 7,
//             gold: 5000,
//         };
//
//         let mut buf = Vec::new();
//         attrs.write(&mut buf).unwrap();
//         // PlayerAttributes is 10 × u16 (20 bytes) + 2 × u32 (8 bytes) = 28 bytes
//         assert_eq!(buf.len(), 28);
//
//         let parsed = PlayerAttributes::parse(&buf).unwrap();
//         assert_eq!(parsed.strength, 10);
//         assert_eq!(parsed.dexterity, 12);
//         assert_eq!(parsed.wisdom, 8);
//         assert_eq!(parsed.constitution, 15);
//         assert_eq!(parsed.unknown_stat, 5);
//         assert_eq!(parsed.hp_current, 50);
//         assert_eq!(parsed.hp_maximum, 100);
//         assert_eq!(parsed.mp_current, 30);
//         assert_eq!(parsed.mp_maximum, 60);
//         assert_eq!(parsed.xp_current, 1500);
//         assert_eq!(parsed.level, 7);
//         assert_eq!(parsed.gold, 5000);
//     }
//
//     #[test]
//     fn test_journal_entry_parse_too_short() {
//         let data = [0u8; 10];
//         let result = JournalEntry::parse(&data);
//         assert!(result.is_err());
//     }
//
//     #[test]
//     fn test_draw_item_parse_too_short() {
//         let data = [0u8; 100];
//         let result = DrawItem::parse(&data);
//         assert!(result.is_err());
//     }
//
//     #[test]
//     fn test_save_file_default() {
//         let save = SaveFile::default();
//         assert_eq!(save.surface_monsters.len(), 0);
//         assert_eq!(save.npcs.len(), 0);
//         assert_eq!(save.surface_objects.len(), 0);
//         assert_eq!(save.player_name, "");
//         assert_eq!(save.journal_main.len(), 0);
//         assert_eq!(save.journal_side.len(), 0);
//         assert_eq!(save.journal_trade.len(), 0);
//     }
//
//     // ── Round-trip tests against actual save files ──
//
//     fn run_round_trip(path: &str) {
//         let original = std::fs::read(path).unwrap_or_else(|e| panic!("Failed to read {path}: {e}"));
//         let save =
//             SaveFile::parse(&original).unwrap_or_else(|e| panic!("Failed to parse {path}: {e}"));
//
//         let mut output = Vec::new();
//         SaveFile::to_writer(&[save], &mut output)
//             .unwrap_or_else(|e| panic!("Failed to write back {path}: {e}"));
//
//         assert_eq!(
//             original.len(),
//             output.len(),
//             "Size mismatch for {path}: original={} output={}",
//             original.len(),
//             output.len(),
//         );
//
//         if let Some((i, (a, b))) = original
//             .iter()
//             .zip(output.iter())
//             .enumerate()
//             .find(|(_, (a, b))| a != b)
//         {
//             panic!(
//                 "Byte mismatch at {i:#x} in {path}: \
//                  original={a:#04x} output={b:#04x}",
//             );
//         }
//     }
//
//     #[test]
//     fn round_trip_nuno_0_sav() {
//         run_round_trip("nuno-0.sav");
//     }
//
//     #[test]
//     fn round_trip_0_sav() {
//         run_round_trip("0.sav");
//     }
//
//     #[test]
//     fn round_trip_2_sav() {
//         run_round_trip("2.sav");
//     }
//
//     /// Verify that best-effort character data extraction works for all saves.
//     #[test]
//     fn test_character_extraction_all_saves() {
//         let files = [
//             (
//                 "nuno-0.sav",
//                 "Nuno ",
//                 "Wojownik",
//                 1u16,
//                 7u16,
//                 21u16,
//                 10u16,
//                 12u16,
//                 42u16,
//                 14u16,
//                 14u16,
//                 0u16,
//                 0u16,
//                 729u32,
//                 5u16,
//                 1181u32,
//             ),
//             (
//                 "0.sav",
//                 "Cristoforo",
//                 "Mag",
//                 3u16,
//                 220u16,
//                 220u16,
//                 20u16,
//                 1200u16,
//                 1200u16,
//                 670u16,
//                 700u16,
//                 0u16,
//                 0u16,
//                 123074u32,
//                 16u16,
//                 24965u32,
//             ),
//             (
//                 "2.sav",
//                 "Cristoforo",
//                 "Mag",
//                 3u16,
//                 220u16,
//                 220u16,
//                 10u16,
//                 991u16,
//                 1200u16,
//                 675u16,
//                 700u16,
//                 0u16,
//                 0u16,
//                 122266u32,
//                 16u16,
//                 24832u32,
//             ),
//             (
//                 "1.sav",
//                 "Cristoforo",
//                 "Mag",
//                 3u16,
//                 220u16,
//                 220u16,
//                 10u16,
//                 1200u16,
//                 1200u16,
//                 670u16,
//                 700u16,
//                 0u16,
//                 0u16,
//                 122974u32,
//                 16u16,
//                 24965u32,
//             ),
//         ];
//         for &(
//             path,
//             exp_name,
//             exp_class,
//             exp_cid,
//             exp_str,
//             exp_dex,
//             exp_wis,
//             exp_con,
//             exp_unk,
//             exp_hp_cur,
//             exp_hp_max,
//             exp_mp_cur,
//             exp_mp_max,
//             exp_xp,
//             exp_lvl,
//             exp_gold,
//         ) in &files
//         {
//             let data = std::fs::read(path).unwrap_or_else(|e| panic!("Failed to read {path}: {e}"));
//             let save =
//                 SaveFile::parse(&data).unwrap_or_else(|e| panic!("Failed to parse {path}: {e}"));
//
//             assert_eq!(save.player_name, exp_name, "{path}: player_name mismatch");
//             assert_eq!(
//                 save.player_class_name, exp_class,
//                 "{path}: class_name mismatch"
//             );
//             assert_eq!(
//                 save.player_class_id, exp_cid as i16,
//                 "{path}: class_id mismatch"
//             );
//
//             let pa = &save.player_attributes;
//             assert_eq!(pa.strength, exp_str, "{path}: STR mismatch");
//             assert_eq!(pa.dexterity, exp_dex, "{path}: DEX mismatch");
//             assert_eq!(pa.wisdom, exp_wis, "{path}: WIS mismatch");
//             assert_eq!(pa.constitution, exp_con, "{path}: CON mismatch");
//             assert_eq!(pa.unknown_stat, exp_unk, "{path}: unknown_stat mismatch");
//             assert_eq!(pa.hp_current, exp_hp_cur, "{path}: HP cur mismatch");
//             assert_eq!(pa.hp_maximum, exp_hp_max, "{path}: HP max mismatch");
//             assert_eq!(pa.mp_current, exp_mp_cur, "{path}: MP cur mismatch");
//             assert_eq!(pa.mp_maximum, exp_mp_max, "{path}: MP max mismatch");
//             assert_eq!(pa.xp_current, exp_xp, "{path}: XP mismatch");
//             assert_eq!(pa.level, exp_lvl, "{path}: Level mismatch");
//             assert_eq!(pa.gold, exp_gold, "{path}: Gold mismatch");
//
//             assert!(!save.events.is_empty(), "{path}: no events extracted");
//             assert_eq!(save.events.len(), 2251, "{path}: event count wrong");
//             assert_eq!(save.journal_main.len(), 100, "{path}: journal_main wrong");
//             assert_eq!(save.journal_side.len(), 100, "{path}: journal_side wrong");
//             assert_eq!(save.journal_trade.len(), 100, "{path}: journal_trade wrong");
//
//             // Inventory: verify at least some items parsed
//             assert!(
//                 !save.inventory_items.is_empty(),
//                 "{path}: no inventory items extracted"
//             );
//             // First item should be a quest item (Event type)
//             assert!(
//                 save.inventory_items[0].is_quest,
//                 "{path}: first item should be quest item"
//             );
//
//             eprintln!(
//                 "  ✓ {path}: player={} class={}({}) str={} events={} inv={}",
//                 save.player_name,
//                 save.player_class_name,
//                 save.player_class_id,
//                 pa.strength,
//                 save.events.len(),
//                 save.inventory_items.len()
//             );
//         }
//     }
// }
