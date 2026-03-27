use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::references::references::Extractor;
use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::Serialize;

// ===========================================================================
// MESSAGE.SCR FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | Message.scr - UI Text Messages       |
// +--------------------------------------+
// | Encoding: WINDOWS-1250              |
// | Format: Pipe-delimited              |
// | Record Size: Variable (text)        |
// +--------------------------------------+
// | ; Comment line                       |
// | id|line1|line2|line3                 |
// | 1|Welcome|to the|town                 |
// | 2|Danger|Ahead|!                      |
// | ...                                   |
// +--------------------------------------+
//
// FIELD DEFINITIONS:
// - id: Unique message identifier
// - line1: First text line (top)
// - line2: Second text line (middle)
// - line3: Third text line (bottom)
//
// SPECIAL VALUES:
// - "null" literal for empty text lines
// - Lines starting with ";" are comments
// - Pipe (|) delimiter between fields
// - Maximum 3 lines per message
//
// FILE PURPOSE:
// Stores multi-line text messages for UI elements like
// signposts, plaques, and system notifications. Used for
// environmental storytelling and player guidance.
//
// ===========================================================================

#[derive(Debug, Serialize)]
pub struct Message {
    /// Mapping bound tracking referenced externally by `message_id` structs.
    pub id: i32,
    /// Initial rendered string row.
    pub line1: Option<String>,
    /// Second rendered string row.
    pub line2: Option<String>,
    /// Third rendered string row.
    pub line3: Option<String>,
}

/// Stores multi-line messages, typically used for signposts and system text.
///
/// Reads file: `ExtraInGame/Message.scr`
/// # File Format: `ExtraInGame/Message.scr`
///
/// Text file, WINDOWS-1250 encoded. One record per line, pipe-delimited:
/// ```text
/// id|line1|line2|line3
/// ```
/// - Up to three display lines per message (used for signs/plaques).
/// - Absent lines use literal `null`.
impl Extractor for Message {
    fn read_file(path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(path)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1250))
                .build(file),
        );

        let mut messages = Vec::new();
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

            let l1 = parts[1].trim();
            let line1 = if l1 == "null" {
                None
            } else {
                Some(l1.to_string())
            };

            let l2 = parts[2].trim();
            let line2 = if l2 == "null" {
                None
            } else {
                Some(l2.to_string())
            };

            let l3 = parts[3].trim();
            let line3 = if l3 == "null" {
                None
            } else {
                Some(l3.to_string())
            };

            messages.push(Message {
                id,
                line1,
                line2,
                line3,
            });
        }

        Ok(messages)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            let l1 = record.line1.as_deref().unwrap_or("null");
            let l2 = record.line2.as_deref().unwrap_or("null");
            let l3 = record.line3.as_deref().unwrap_or("null");
            let line = format!("{} | {} | {} | {}\r\n", record.id, l1, l2, l3);
            let (cow, _, _) = WINDOWS_1250.encode(&line);
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_messages(path: &Path) -> std::io::Result<Vec<Message>> {
    Message::read_file(path)
}
