use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::Serialize;
use crate::references::references::Extractor;

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
