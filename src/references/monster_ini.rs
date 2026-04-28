use std::path::Path;

use crate::references::extractor::Extractor;
use dispel_macros::TextExtractor;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// Monster.ini - Monster Animations
///
/// Stores visual references and configuration for monsters.
///
/// Reads file: `Monster.ini`
///
/// # ASCII Structure
///
/// ```text
/// +--------------------------------------+
/// | Monster.ini - Monster Animations    |
/// +--------------------------------------+
/// | Encoding: WINDOWS-1250              |
/// | Format: CSV with comments            |
/// | Record Size: Variable (text)        |
/// +--------------------------------------+
/// | ; Comment line                       |
/// | id,name,sprite,attack,hit,death,walk,cast|
/// | 1,Goblin,goblin.spr,1,2,3,4,5        |
/// | 2,Orc,orc.spr,1,2,3,4,5              |
/// +--------------------------------------+
/// ```
///
/// # Field Definitions
///
/// - `id`: Unique monster visual type ID
/// - `name`: Monster display name or "null"
/// - `sprite`: SPR filename or "null"
/// - `attack`: Animation sequence for attacking
/// - `hit`: Animation sequence for taking damage
/// - `death`: Animation sequence for dying
/// - `walk`: Animation sequence for walking
/// - `cast`: Animation sequence for spellcasting
///
/// # Animation Sequences
///
/// - Refer to frame indices in SPR files
/// - `0` = no animation
/// - `1-N` = frame sequence numbers
/// - Linked to sprite file structure
///
/// # Special Values
///
/// - `"null"` literal for missing name/sprite
/// - Lines starting with `;` are comments
/// - CSV format with comma delimiter
/// - `0` for unused animation sequences
///
/// # File Purpose
///
/// Defines animation sequences for monsters, linking visual
/// appearances with behavioral animations. Used for monster
/// rendering during different combat states and actions.
#[derive(Debug, Clone, Default, Serialize, Deserialize, TextExtractor)]
#[extractor(encoding = "WINDOWS_1250")]
pub struct MonsterIni {
    /// Monster visual type identifier.
    #[extractor(field = 0)]
    pub id: i32,
    /// Translated name of the monster.
    #[extractor(field = 1, parse_null)]
    pub name: Option<String>,
    /// Base sprite filename for the monster rendering.
    #[extractor(field = 2, parse_null)]
    pub sprite_filename: Option<String>,
    /// Sprite animation sequence number for attacking.
    #[extractor(field = 3)]
    pub attack: i32,
    /// Sprite animation sequence number for getting hit.
    #[extractor(field = 4)]
    pub hit: i32,
    /// Sprite animation sequence number for death.
    #[extractor(field = 5)]
    pub death: i32,
    /// Sprite animation sequence number for walking.
    #[extractor(field = 6)]
    pub walking: i32,
    /// Sprite animation sequence number for casting spells.
    #[extractor(field = 7)]
    pub casting_magic: i32,
}

pub fn read_monster_ini(source_path: &Path) -> std::io::Result<Vec<MonsterIni>> {
    MonsterIni::read_file(source_path)
}

pub fn save_monster_inis(conn: &mut Connection, monster_inis: &[MonsterIni]) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_entry() {
        let data = b"1,Goblin,goblin.spr,1,2,3,4,5\n";
        let mut c = Cursor::new(data.as_ref());
        let mons = MonsterIni::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(mons.len(), 1);
        assert_eq!(mons[0].id, 1);
        assert_eq!(mons[0].name.as_deref(), Some("Goblin"));
        assert_eq!(mons[0].sprite_filename.as_deref(), Some("goblin.spr"));
        assert_eq!(mons[0].attack, 1);
        assert_eq!(mons[0].hit, 2);
        assert_eq!(mons[0].death, 3);
        assert_eq!(mons[0].walking, 4);
        assert_eq!(mons[0].casting_magic, 5);
    }

    #[test]
    fn parse_null_fields() {
        let data = b"2,null,null,0,0,0,0,0\n";
        let mut c = Cursor::new(data.as_ref());
        let mons = MonsterIni::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(mons[0].name, None);
        assert_eq!(mons[0].sprite_filename, None);
    }

    #[test]
    fn serialize_round_trip() {
        let data = b"1,Goblin,goblin.spr,1,2,3,4,5\r\n";
        let mut c = Cursor::new(data.as_ref());
        let records = MonsterIni::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        MonsterIni::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = MonsterIni::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].id, records2[0].id);
        assert_eq!(records[0].name, records2[0].name);
        assert_eq!(records[0].sprite_filename, records2[0].sprite_filename);
    }
}
