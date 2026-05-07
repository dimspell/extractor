use std::path::Path;

use crate::references::extractor::Extractor;
use dispel_macros::{Localizable, TextExtractor, TextRecordPatcher};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// Stores multi-line messages, typically used for signposts and system text.
///
/// Reads file: `ExtraInGame/Message.scr`
///
/// Text file, WINDOWS-1250 encoded. One record per line, pipe-delimited:
/// ```text
/// id|line1|line2|line3
/// ```
///
/// Example:
/// ```text
/// ; Comment line
/// id|line1|line2|line3
/// 1|Welcome|to the|town
/// 2|Danger|Ahead|!
/// ```
///
/// FIELD DEFINITIONS:
/// - id: Unique message identifier
/// - line1: First text line (top)
/// - line2: Second text line (middle)
/// - line3: Third text line (bottom)
///
/// SPECIAL VALUES:
/// - "null" literal for empty text lines
/// - Lines starting with ";" are comments
/// - Pipe (|) delimiter between fields
/// - Maximum 3 lines per message
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, Localizable, TextExtractor, TextRecordPatcher,
)]
#[extractor(encoding = "WINDOWS_1250", delimiter = "|")]
#[patcher(filename = "Message.scr")]
pub struct Message {
    /// Mapping bound tracking referenced externally by `message_id` structs.
    #[extractor(field = 0)]
    pub id: i32,
    /// Initial rendered string row.
    #[translatable(encoding = "WINDOWS_1250", max_bytes = 1024)]
    #[extractor(field = 1, parse_null)]
    pub line1: Option<String>,
    /// Second rendered string row.
    #[translatable(encoding = "WINDOWS_1250", max_bytes = 1024)]
    #[extractor(field = 2, parse_null)]
    pub line2: Option<String>,
    /// Third rendered string row.
    #[translatable(encoding = "WINDOWS_1250", max_bytes = 1024)]
    #[extractor(field = 3, parse_null)]
    pub line3: Option<String>,
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
