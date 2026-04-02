use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

use crate::references::extractor::Extractor;

// ===========================================================================
// DIALOGUE TEXT FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | *.pgp - Dialogue Text         |
// +--------------------------------------+
// | Encoding: WINDOWS-1250               |
// | Format: Commented pipe-delimited     |
// | Record Size: Variable (text)         |
// +--------------------------------------+
// | ; Comment line 1                     |
// | ; Comment line 2                     |
// | id|text|param1|param2                |
// | 1|Hello|0|0                          |
// | ; Quest dialogue                     |
// | 2|Find the artifact|100|5            |
// | ...                                  |
// +--------------------------------------+
//
// FIELD DEFINITIONS:
// - id: Unique dialogue text identifier
// - text: Display text content
// - param1: Logic parameter 1
// - param2: Logic parameter 2
// - comment: Developer notes (multi-line)
//
// PARAMETER USAGE:
// - param1: Dialogue branch conditions
// - param2: Event triggers or requirements
// - Special values: 0 = no condition
//
// TEXT FORMATTING:
// - "null" literal for empty text
// - "$" literal interpretet as a line-break in game
// - Pipe (|) delimiter between fields
// - Semicolon (;) for comment lines
// - Multi-line comments supported
//
// SPECIAL VALUES:
// - param1 = 0: Unconditional dialogue
// - param2 = 0: No event trigger
// - Empty text: "null" literal
// - Comment lines preserved with ";" prefix
//
// FILE PURPOSE:
// Stores dialogue text content with developer comments
// and logical parameters. Used for displaying conversation
// text, branching dialogue, and triggering game events.
//
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueText {
    /// Translation line identity reference.
    pub id: i32,
    /// Actual text string projected into dialogue window UI.
    pub text: String,
    /// Developer trailing strings (`\;`) restored and preserved internally.
    pub comment: String,
    /// Internal tracking modifier block associated to dialog output.
    pub param1: i32,
    /// ID of the sound from the wave.ini file. Played at start of dialogue.
    pub wave_ini_entry_id: i32,
}

/// Stores translations, text strings, and associated comments used within dialogues.
///
/// Reads file: `NpcInGame/*.pgp`
/// # File Format: `NpcInGame/*.pgp`
///
/// Text file, WINDOWS-1250 encoded. Lines starting with `;` are comments
/// and are associated with the next data record.
/// Data lines are pipe-delimited:
/// ```text
/// id|text|param1|param2
/// ```
/// - `text` uses literal `null` for empty strings.
/// - `param1` / `param2` are integer logic parameters.
/// - Comment lines (`;`) preceding a record are stored in `comment` using ` | ` as separator.
impl Extractor for DialogueText {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let f = File::open(source_path)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1250))
                .build(f),
        );

        let mut texts: Vec<DialogueText> = Vec::new();
        let mut current_comment = String::new();
        let mut last_was_comment = false;

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with(';') {
                if !last_was_comment {
                    current_comment.clear();
                }
                let c = trimmed.trim_start_matches(';').trim();
                if !c.is_empty() {
                    if !current_comment.is_empty() {
                        current_comment.push_str(" | ");
                    }
                    current_comment.push_str(c);
                }
                last_was_comment = true;
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 4 {
                continue;
            }

            let id = match parts[0].trim().parse::<i32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let text = parts[1]
                .to_string()
                .trim_matches('|')
                .replace("null", "")
                .to_string();
            let param1 = parts[2].trim().parse::<i32>().unwrap_or(0);
            let wave_ini_entry_id = parts[3].trim().parse::<i32>().unwrap_or(0);

            texts.push(DialogueText {
                id,
                text,
                comment: current_comment.clone().trim_matches('|').to_string(),
                param1,
                wave_ini_entry_id,
            });

            last_was_comment = false;
        }
        Ok(texts)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            if !record.comment.is_empty() {
                for c in record.comment.split(" | ") {
                    let line = format!("; {}\r\n", c);
                    let (cow, _, _) = WINDOWS_1250.encode(&line);
                    file.write_all(&cow)?;
                }
            }
            let text_val = if record.text.is_empty() {
                "null"
            } else {
                &record.text
            };
            let line = format!(
                "{} | {} | {} | {}\r\n",
                record.id, text_val, record.param1, record.wave_ini_entry_id
            );
            let (cow, _, _) = WINDOWS_1250.encode(&line);
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_dialogue_texts(source_path: &Path) -> std::io::Result<Vec<DialogueText>> {
    DialogueText::read_file(source_path)
}

pub fn save_dialogue_texts(
    conn: &mut Connection,
    file_name: &str,
    texts: &Vec<DialogueText>,
) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_dialogue_text.sql"))?;
        for text in texts {
            stmt.execute(params![
                file_name,
                text.id,
                text.text,
                text.comment,
                text.param1,
                text.wave_ini_entry_id,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
