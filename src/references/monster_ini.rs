use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader};

use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use crate::references::references::parse_null;

#[derive(Debug, Serialize, Deserialize)]
pub struct MonsterIni {
    pub id: i32,
    pub name: Option<String>,
    pub sprite_filename: Option<String>,
    pub attack: i32,
    // animation sequence number
    pub hit: i32,
    // animation sequence number
    pub death: i32,
    // animation sequence number
    pub walking: i32,
    // animation sequence number
    pub casting_magic: i32, // animation sequence number
}


pub fn read_monster_ini(source_path: &Path) -> std::io::Result<Vec<MonsterIni>> {
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
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line.split(",").collect();

                let id = parts[0].parse::<i32>().unwrap();
                let name = parse_null(parts[1]);
                let sprite_filename = parse_null(parts[2]);
                let attack = parts[3].parse::<i32>().unwrap();
                let hit = parts[4].parse::<i32>().unwrap();
                let death = parts[5].parse::<i32>().unwrap();
                let walking = parts[6].parse::<i32>().unwrap();
                let casting_magic = parts[7].parse::<i32>().unwrap();

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
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(monsters)
}