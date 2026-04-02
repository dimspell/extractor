use std::io::{BufRead, BufReader, Write};
use std::{fs::File, path::Path};

use crate::references::extractor::{parse_null, Extractor};
use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

// ===========================================================================
// MONSTER.INI FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | Monster.ini - Monster Animations    |
// +--------------------------------------+
// | Encoding: WINDOWS-1250              |
// | Format: CSV with comments            |
// | Record Size: Variable (text)        |
// +--------------------------------------+
// | ; Comment line                       |
// | id,name,sprite,attack,hit,death,walk,cast|
// | 1,Goblin,goblin.spr,1,2,3,4,5        |
// | 2,Orc,orc.spr,1,2,3,4,5              |
// +--------------------------------------+
//
// FIELD DEFINITIONS:
// - id: Unique monster visual type ID
// - name: Monster display name or "null"
// - sprite: SPR filename or "null"
// - attack: Animation sequence for attacking
// - hit: Animation sequence for taking damage
// - death: Animation sequence for dying
// - walk: Animation sequence for walking
// - cast: Animation sequence for spellcasting
//
// ANIMATION SEQUENCES:
// - Refer to frame indices in SPR files
// - 0 = no animation
// - 1-N = frame sequence numbers
// - Linked to sprite file structure
//
// SPECIAL VALUES:
// - "null" literal for missing name/sprite
// - Lines starting with ";" are comments
// - CSV format with comma delimiter
// - 0 for unused animation sequences
//
// FILE PURPOSE:
// Defines animation sequences for monsters, linking visual
// appearances with behavioral animations. Used for monster
// rendering during different combat states and actions.
//
// ===========================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct MonsterIni {
    /// Monster visual type identifier.
    pub id: i32,
    /// Translated name of the monster.
    pub name: Option<String>,
    /// Base sprite filename for the monster rendering.
    pub sprite_filename: Option<String>,
    /// Sprite animation sequence number for attacking.
    pub attack: i32,
    // animation sequence number
    /// Sprite animation sequence number for getting hit.
    pub hit: i32,
    // animation sequence number
    /// Sprite animation sequence number for death.
    pub death: i32,
    // animation sequence number
    /// Sprite animation sequence number for walking.
    pub walking: i32,
    // animation sequence number
    /// Sprite animation sequence number for casting spells.
    pub casting_magic: i32, // animation sequence number
}

/// Stores visual references and configuration for monsters.
///
/// Reads file: `Monster.ini`
/// # File Format: `Monster.ini`
///
/// Text file, WINDOWS-1250 encoded. One record per line, CSV format:
/// ```text
/// id,name,sprite_filename,attack_seq,hit_seq,death_seq,walking_seq,cast_seq
/// ```
/// - `name` and `sprite_filename` use literal `null` when absent.
/// - Sequence fields are animation indices into the SPR file.
impl Extractor for MonsterIni {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let f = File::open(source_path)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1250))
                .build(f),
        );
        let mut monsters: Vec<MonsterIni> = Vec::new();
        for line in reader.lines().map_while(Result::ok) {
            let trimmed = line.trim();
            if trimmed.starts_with(";") || trimmed.is_empty() {
                continue;
            }

            let parts: Vec<&str> = trimmed.split(",").collect();
            if parts.len() < 8 {
                continue;
            }

            let id = parts[0].trim().parse::<i32>().unwrap();
            let name = parse_null(parts[1].trim());
            let sprite_filename = parse_null(parts[2].trim());
            let attack = parts[3].trim().parse::<i32>().unwrap();
            let hit = parts[4].trim().parse::<i32>().unwrap();
            let death = parts[5].trim().parse::<i32>().unwrap();
            let walking = parts[6].trim().parse::<i32>().unwrap();
            let casting_magic = parts[7].trim().parse::<i32>().unwrap();

            monsters.push(MonsterIni {
                id,
                name,
                sprite_filename,
                attack,
                hit,
                death,
                walking,
                casting_magic,
            });
        }
        Ok(monsters)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            let n = record.name.as_deref().unwrap_or("null");
            let s = record.sprite_filename.as_deref().unwrap_or("null");
            let line = format!(
                "{},{},{},{},{},{},{},{}\r\n",
                record.id,
                n,
                s,
                record.attack,
                record.hit,
                record.death,
                record.walking,
                record.casting_magic
            );
            let (cow, _, _) = WINDOWS_1250.encode(&line);
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_monster_ini(source_path: &Path) -> std::io::Result<Vec<MonsterIni>> {
    MonsterIni::read_file(source_path)
}

pub fn save_monster_inis(conn: &mut Connection, monster_inis: &Vec<MonsterIni>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_monster_ini.sql"))?;
        for monster_ini in monster_inis {
            stmt.execute(params![
                monster_ini.id,
                monster_ini.name,
                monster_ini.sprite_filename,
                monster_ini.attack,
                monster_ini.hit,
                monster_ini.death,
                monster_ini.walking,
                monster_ini.casting_magic,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
