use std::{fs::File, path::Path};
use std::io::BufReader;
use std::io::prelude::*;

use byteorder::{LittleEndian, ReadBytesExt};
use encoding_rs::EUC_KR;
use serde::{Deserialize, Serialize};
use crate::references::references::read_mapper;

#[derive(Debug, Serialize, Deserialize)]
pub struct Monster {
    pub id: i32,
    pub name: String,
    pub health_points_max: i32,
    pub health_points_min: i32,
    pub magic_points_max: i32,
    pub magic_points_min: i32,
    pub walk_speed: i32,
    pub to_hit_max: i32,
    pub to_hit_min: i32,
    pub to_dodge_max: i32,
    pub to_dodge_min: i32,
    pub offense_max: i32,
    pub offense_min: i32,
    pub defense_max: i32,
    pub defense_min: i32,
    pub magic_attack_max: i32,
    pub magic_attack_min: i32,
    pub is_undead: i32,
    pub has_blood: i32,
    pub ai_type: i32,
    pub exp_gain_max: i32,
    pub exp_gain_min: i32,
    pub gold_drop_max: i32,
    pub gold_drop_min: i32,
    pub detection_sight_size: i32,
    pub distance_range_size: i32,
    pub known_spell_slot1: i32,
    pub known_spell_slot2: i32,
    pub known_spell_slot3: i32,
    pub is_oversize: i32,
    pub magic_level: i32,
    pub special_attack: i32,
    pub special_attack_chance: i32,
    pub special_attack_duration: i32,
    pub boldness: i32,
    pub attack_speed: i32,
}

pub fn read_monster_db(source_path: &Path) -> std::io::Result<Vec<Monster>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 0;
    const PROPERTY_ITEM_SIZE: i32 = 40 * 4;
    // const FILLER: u8 = 0x0;

    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
    let mut monsters: Vec<Monster> = Vec::with_capacity(elements as usize);

    for i in 0..elements {
        let mut buffer = [0u8; 24];
        reader.read_exact(&mut buffer)?;
        let dst = EUC_KR.decode(&buffer);
        let name = dst.0.trim_end_matches("\0").trim();

        let health_points_max = reader.read_i32::<LittleEndian>()?;
        let health_points_min = reader.read_i32::<LittleEndian>()?;
        let magic_points_max = reader.read_i32::<LittleEndian>()?;
        let magic_points_min = reader.read_i32::<LittleEndian>()?;

        let walk_speed = reader.read_i32::<LittleEndian>()?;

        let to_hit_max = reader.read_i32::<LittleEndian>()?;
        let to_hit_min = reader.read_i32::<LittleEndian>()?;

        let to_dodge_max = reader.read_i32::<LittleEndian>()?; // always = 10
        let to_dodge_min = reader.read_i32::<LittleEndian>()?; // always = 10

        let offense_max = reader.read_i32::<LittleEndian>()?;
        let offense_min = reader.read_i32::<LittleEndian>()?;

        let defense_max = reader.read_i32::<LittleEndian>()?;
        let defense_min = reader.read_i32::<LittleEndian>()?;

        let magic_attack_max = reader.read_i32::<LittleEndian>()?; // max
        let magic_attack_min = reader.read_i32::<LittleEndian>()?; // min

        let is_undead = reader.read_i32::<LittleEndian>()?; // "0 or 1"
        let has_blood = reader.read_i32::<LittleEndian>()?; // "0 or 1, golem is not alive and not undead"
        let ai_type = reader.read_i32::<LittleEndian>()?; // "goblin and chicken = 1,archers = 2, worm bot no zombie =3, deer and dog = 5"

        let exp_gain_max = reader.read_i32::<LittleEndian>()?;
        let exp_gain_min = reader.read_i32::<LittleEndian>()?;

        let gold_drop_max = reader.read_i32::<LittleEndian>()?;
        let gold_drop_min = reader.read_i32::<LittleEndian>()?;

        let detection_sight_size = reader.read_i32::<LittleEndian>()?; // "9 or 10 - only goblin king have 10"
        let distance_range_size = reader.read_i32::<LittleEndian>()?; // "1 or 6 if archer

        let known_spell_slot1 = reader.read_i32::<LittleEndian>()?;
        let known_spell_slot2 = reader.read_i32::<LittleEndian>()?;
        let known_spell_slot3 = reader.read_i32::<LittleEndian>()?;

        let is_oversize = reader.read_i32::<LittleEndian>()?; // redDragon, balrog, beholder, = 1

        let magic_level = reader.read_i32::<LittleEndian>()?; // always = 1

        let special_attack = reader.read_i32::<LittleEndian>()?; // 0 = none, 1 = bat/zombie/biteworm, 2 = basilisk
        let special_attack_chance = reader.read_i32::<LittleEndian>()?;
        let special_attack_duration = reader.read_i32::<LittleEndian>()?;

        let boldness = reader.read_i32::<LittleEndian>()?; // always = 10
        let attack_speed = reader.read_i32::<LittleEndian>()?;

        monsters.push(Monster {
            id: i,
            name: name.to_string(),
            health_points_max,
            health_points_min,
            magic_points_max,
            magic_points_min,
            walk_speed,
            to_hit_max,
            to_hit_min,
            to_dodge_max,
            to_dodge_min,
            offense_max,
            offense_min,
            defense_max,
            defense_min,
            magic_attack_max,
            magic_attack_min,
            is_undead,
            has_blood,
            ai_type,
            exp_gain_max,
            exp_gain_min,
            gold_drop_max,
            gold_drop_min,
            detection_sight_size,
            distance_range_size,
            known_spell_slot1,
            known_spell_slot2,
            known_spell_slot3,
            is_oversize,
            magic_level,
            special_attack,
            special_attack_chance,
            special_attack_duration,
            boldness,
            attack_speed,
        })
    }

    Ok(monsters)
}