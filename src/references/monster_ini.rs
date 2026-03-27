use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader, Write};

use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use crate::references::references::{parse_null, Extractor};

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
impl Extractor for MonsterIni {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let f = File::open(source_path)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1250))
                .build(f),
        );
        let mut monsters: Vec<MonsterIni> = Vec::new();
        for line in reader.lines() {
            match line {
                Ok(line) => {
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
                _ => {}
            }
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
                record.id, n, s, record.attack, record.hit, record.death, record.walking, record.casting_magic
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