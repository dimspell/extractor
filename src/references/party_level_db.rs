use crate::references::extractor::Extractor;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rusqlite::{params, Connection, Result as DbResult};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Result};
use std::path::Path;

// ===========================================================================
// PRTLEVEL.DB FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | PrtLevel.db - Character Progression  |
// +--------------------------------------+
// | Encoding: Binary (Little-Endian)     |
// | Record Size: 36 bytes per level      |
// | Total Size: 5760 bytes (8×20×36)      |
// +--------------------------------------+
// | [NPC 1]                              |
// | - [Level 1]                         |
// |   - sentinel: u32                   |
// |   - strength: u32                   |
// |   - constitution: u32               |
// |   - wisdom: u32                      |
// |   - health_points: u16              |
// |   - mana_points: u16               |
// |   - agility: u32                    |
// |   - attack: u32                      |
// |   - mana_recharge: u32               |
// |   - defense: u16                     |
// |   - padding: u16                     |
// | - [Level 2]                         |
// |   ... (same structure) ...           |
// +--------------------------------------+
// | [NPC 2]                              |
// | ... (20 levels) ...                  |
// +--------------------------------------+
// | ... (8 NPCs total) ...               |
// +--------------------------------------+
//
// STAT GROWTH PATTERNS:
// - strength: Physical damage output
// - constitution: Health point scaling
// - wisdom: Mana point scaling
// - agility: Evasion and speed
// - attack: Combat accuracy
// - defense: Damage resistance
//
// LEVEL RANGES:
// - Levels 1-20: Standard progression
// - Each level adds fixed stat increases
// - Growth curves vary by character class
//
// SPECIAL VALUES:
// - sentinel = 0: Standard record marker
// - Fixed 20 levels per NPC
// - 8 NPC slots (party size limit)
// - 5760-byte total file size
//
// FILE PURPOSE:
// Defines character progression statistics for
// levels 1-20. Used for level-up calculations,
// stat growth, and character development.
//
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyLevelRecord {
    /// Derived multiplier level tracking.
    pub level: u32,
    /// Scaling milestone block for strength.
    pub strength: u32,
    /// Health expansion parameters per level.
    pub constitution: u32,
    /// Mana multiplier logic.
    pub wisdom: u32,
    /// Fixed gain of base stamina.
    pub health_points: u16,
    /// Fixed gain of magical pools.
    pub mana_points: u16,
    /// Avoidance calculation matrix shift.
    pub agility: u32,
    /// Derived raw throughput bonus.
    pub attack: u32,
    /// Frequency recovery tracking matrix.
    pub mana_recharge: u32,
    /// Armor tracking expansion rating.
    pub defense: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyLevelNpc {
    pub npc_index: usize,
    pub records: Vec<PartyLevelRecord>,
}

/// Stores the experience and stat progression tables for party members per level.
///
/// Reads file: `NpcInGame/PrtLevel.db`
/// # File Format: `NpcInGame/PrtLevel.db`
///
/// Binary file, little-endian. Fixed size: `8 NPCs × 20 levels × 36 bytes = 5760 bytes`.
/// No header. Each 36-byte sub-block:
/// - 4-byte sentinel (u32)
/// - `strength`, `constitution`, `wisdom` : u32 each
/// - `health_points`, `mana_points`      : u16 each
/// - `agility`, `attack`, `mana_recharge` : u32 each
/// - `defense`                            : u16
impl Extractor for PartyLevelNpc {
    fn read_file(source_path: &Path) -> Result<Vec<Self>> {
        let file = File::open(source_path)?;
        let mut reader = BufReader::new(file);
        let mut npcs = Vec::new();

        // The file is 5760 bytes. Based on reverse engineering:
        // 8 NPCs * 720 bytes = 5760 bytes.
        // Each 720 byte block is 20 sub-blocks of 36 bytes.
        // Each 36 byte sub-block starts with a 4-byte sentinel followed by 8 u32 data fields.

        for npc_index in 0..8 {
            let mut records = Vec::new();
            for _block_idx in 0..20 {
                let _sentinel = reader.read_u32::<LittleEndian>()?;

                // Each block has 8 u32 values
                let strength = reader.read_u32::<LittleEndian>()?;
                let constitution = reader.read_u32::<LittleEndian>()?;
                let wisdom = reader.read_u32::<LittleEndian>()?;
                let health_points = reader.read_u16::<LittleEndian>()?;
                let mana_points = reader.read_u16::<LittleEndian>()?;

                let agility = reader.read_u32::<LittleEndian>()?;
                let attack = reader.read_u32::<LittleEndian>()?;
                let mana_recharge = reader.read_u32::<LittleEndian>()?;
                let defense = reader.read_u16::<LittleEndian>()?;
                let _ = reader.read_u16::<LittleEndian>()?; // Null byte (\0)

                records.push(PartyLevelRecord {
                    level: _block_idx + 1_u32,
                    strength,
                    constitution,
                    wisdom,
                    health_points,
                    mana_points,
                    agility,
                    attack,
                    mana_recharge,
                    defense,
                });
            }
            npcs.push(PartyLevelNpc { npc_index, records });
        }

        Ok(npcs)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> Result<()> {
        let file = File::create(dest_path)?;
        let mut writer = BufWriter::new(file);

        for npc in records {
            for record in &npc.records {
                writer.write_u32::<LittleEndian>(0)?; // sentinel

                writer.write_u32::<LittleEndian>(record.strength)?;
                writer.write_u32::<LittleEndian>(record.constitution)?;
                writer.write_u32::<LittleEndian>(record.wisdom)?;
                writer.write_u16::<LittleEndian>(record.health_points)?;
                writer.write_u16::<LittleEndian>(record.mana_points)?;

                writer.write_u32::<LittleEndian>(record.agility)?;
                writer.write_u32::<LittleEndian>(record.attack)?;
                writer.write_u32::<LittleEndian>(record.mana_recharge)?;
                writer.write_u16::<LittleEndian>(record.defense)?;
                writer.write_u16::<LittleEndian>(0)?; // null byte
            }
        }

        Ok(())
    }
}

pub fn read_party_level_db(source_path: &Path) -> Result<Vec<PartyLevelNpc>> {
    PartyLevelNpc::read_file(source_path)
}

pub fn save_party_levels(conn: &mut Connection, npcs: &Vec<PartyLevelNpc>) -> DbResult<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_party_level.sql"))?;
        for npc in npcs {
            for record in &npc.records {
                stmt.execute(params![
                    npc.npc_index as i32,
                    record.level as i32,
                    record.strength as i32,
                    record.constitution as i32,
                    record.wisdom as i32,
                    record.health_points as i32,
                    record.mana_points as i32,
                    record.agility as i32,
                    record.attack as i32,
                    record.mana_recharge as i32,
                    record.defense as i32,
                ])?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}
