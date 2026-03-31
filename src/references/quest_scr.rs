use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::references::extractor::Extractor;
use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::Serialize;

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

#[derive(Debug, Serialize)]
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
    fn read_file(path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(path)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1250))
                .build(file),
        );

        let mut quests = Vec::new();
        for line in reader.lines() {
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

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            let title = record.title.as_deref().unwrap_or("null");
            let desc = record.description.as_deref().unwrap_or("null");

            let line = format!("{}|{}|{}|{}\r\n", record.id, record.type_id, title, desc);
            let (cow, _, _) = WINDOWS_1250.encode(&line);
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_quests(path: &Path) -> std::io::Result<Vec<Quest>> {
    Quest::read_file(path)
}

pub fn save_quests(conn: &mut Connection, quests: &Vec<Quest>) -> Result<()> {
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
