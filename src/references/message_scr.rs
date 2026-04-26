use std::io::{BufRead, BufReader, Read, Seek, Write};
use std::path::Path;

use crate::references::extractor::Extractor;
use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        let decoded = DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1250))
            .build(reader.by_ref());
        let buf_reader = BufReader::new(decoded);

        let mut messages = Vec::new();
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

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        for record in records {
            let l1 = record.line1.as_deref().unwrap_or("null");
            let l2 = record.line2.as_deref().unwrap_or("null");
            let l3 = record.line3.as_deref().unwrap_or("null");
            let line = format!("{} | {} | {} | {}\r\n", record.id, l1, l2, l3);
            let (cow, _, _) = WINDOWS_1250.encode(&line);
            writer.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_messages(path: &Path) -> std::io::Result<Vec<Message>> {
    Message::read_file(path)
}

pub fn save_messages(conn: &mut Connection, messages: &[Message]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_message.sql"))?;
        for message in messages {
            stmt.execute(params![
                message.id,
                message.line1,
                message.line2,
                message.line3,
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
    fn parse_messages() {
        let data = b"1|Hello|World|Bye\n2|null|null|null\n";
        let mut c = Cursor::new(data.as_ref());
        let msgs = Message::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].id, 1);
        assert_eq!(msgs[0].line1.as_deref(), Some("Hello"));
        assert_eq!(msgs[0].line2.as_deref(), Some("World"));
        assert_eq!(msgs[0].line3.as_deref(), Some("Bye"));
        assert!(msgs[1].line1.is_none());
        assert!(msgs[1].line2.is_none());
        assert!(msgs[1].line3.is_none());
    }

    #[test]
    fn parse_skips_short_lines() {
        let data = b"1|only two\n2|a|b|c\n";
        let mut c = Cursor::new(data.as_ref());
        let msgs = Message::parse(&mut c, data.len() as u64).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].id, 2);
    }

    #[test]
    fn serialize_round_trip() {
        let data = b"1|Hello|World|Bye\r\n2|null|null|null\r\n";
        let mut c = Cursor::new(data.as_ref());
        let records = Message::parse(&mut c, data.len() as u64).unwrap();
        let mut out = Vec::new();
        Message::to_writer(&records, &mut out).unwrap();
        let mut c2 = Cursor::new(out.as_slice());
        let records2 = Message::parse(&mut c2, out.len() as u64).unwrap();
        assert_eq!(records.len(), records2.len());
        assert_eq!(records[0].id, records2[0].id);
        assert_eq!(records[1].id, records2[1].id);
    }
}
