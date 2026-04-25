use std::io::{BufRead, BufReader, Read, Seek, Write};
use std::path::Path;

use crate::references::extractor::Extractor;
use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

// ===========================================================================
// QUEST.SCR FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | Quest.scr - Quest Journal Entries    |
// +--------------------------------------+
// | Encoding: WINDOWS-1250              |
// | Format: Pipe-delimited              |
// | Record Size: Variable (text)        |
// +--------------------------------------+
// | ; Comment line                       |
// | id|type|title|description            |
// | 1|0|Main Quest|Defeat the Dark Lord   |
// | 2|1|Side Quest|Find the Lost Artifact|
// | ...                                  |
// +--------------------------------------+
//
// FIELD DEFINITIONS:
// - id: Unique quest identifier
// - type: Quest category (0=main, 1=side, 2=traders)
// - title: Quest title/name
// - description: Quest description/text
//
// QUEST TYPES:
// - 0: Main quests
// - 1: Side quests
// - 2: Traders journal
//
// SPECIAL VALUES:
// - "null" literal for missing title/description
// - Lines starting with ";" are comments
// - Pipe (|) delimiter between fields
// - Empty lines ignored
//
// FILE PURPOSE:
// Defines all quests with categories, titles, and descriptions.
// Used for quest journal system, quest tracking, and player
// progression. Linked to event system for quest completion.
//
// ===========================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Quest {
    /// Quest table database pointer index.
    pub id: i32,
    /// Grouping context enum (0=Main, 1=Side, 2=Traders).
    pub type_id: i32, // 0=main, 1=side, 2=traders
    /// Journal summary topic literal.
    pub title: Option<String>,
    /// Journal paragraph body text literal.
    pub description: Option<String>,
}

/// Stores quest diary entries, including main quests, side quests, and trader journals.
///
/// Reads file: `ExtraInGame/Quest.scr`
/// # File Format: `ExtraInGame/Quest.scr`
///
/// Text file, WINDOWS-1250 encoded. One record per line, pipe-delimited:
/// ```text
/// id|type_id|title|description
/// ```
/// - `type_id`: 0 = main quest, 1 = side quest, 2 = traders journal.
/// - `title` and `description` use literal `null` when absent.
impl Extractor for Quest {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        let decoded = DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1250))
            .build(reader.by_ref());
        let buf_reader = BufReader::new(decoded);

        let mut quests = Vec::new();
        for line in buf_reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with(';') {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 4 {
                continue;
            }

            let id = parts[0].trim().parse::<i32>().unwrap_or(0);
            let type_id = parts[1].trim().parse::<i32>().unwrap_or(0);

            let title_str = parts[2].trim();
            let title = if title_str == "null" {
                None
            } else {
                Some(title_str.to_string())
            };

            let desc_str = parts[3].trim();
            let description = if desc_str == "null" {
                None
            } else {
                Some(desc_str.to_string())
            };

            quests.push(Quest {
                id,
                type_id,
                title,
                description,
            });
        }

        Ok(quests)
    }

    fn serialize<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        for record in records {
            let title = record.title.as_deref().unwrap_or("null");
            let desc = record.description.as_deref().unwrap_or("null");

            let line = format!("{}|{}|{}|{}\r\n", record.id, record.type_id, title, desc);
            let (cow, _, _) = WINDOWS_1250.encode(&line);
            writer.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_quests(path: &Path) -> std::io::Result<Vec<Quest>> {
    Quest::read_file(path)
}

pub fn save_quests(conn: &mut Connection, quests: &[Quest]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_quest.sql"))?;
        for quest in quests {
            stmt.execute(params![
                quest.id,
                quest.type_id,
                quest.title,
                quest.description,
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
    fn parse_quests() {
        let data = b"1|0|Main Quest|Kill the dragon\n2|1|null|null\n";
        let mut c = Cursor::new(data.as_ref());
        let quests = Quest::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(quests.len(), 2);
        assert_eq!(quests[0].id, 1);
        assert_eq!(quests[0].type_id, 0);
        assert_eq!(quests[0].title.as_deref(), Some("Main Quest"));
        assert_eq!(quests[0].description.as_deref(), Some("Kill the dragon"));
        assert_eq!(quests[1].type_id, 1);
        assert!(quests[1].title.is_none());
        assert!(quests[1].description.is_none());
    }

    #[test]
    fn parse_skips_comments_and_short_lines() {
        let data = b"; intro\n1|0|Title|Desc\n";
        let mut c = Cursor::new(data.as_ref());
        let quests = Quest::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(quests.len(), 1);
    }
}
