use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

use crate::references::references::Extractor;

#[derive(Debug, Serialize, Deserialize)]
pub struct DialogueText {
    /// Translation line identity reference.
    pub id: i32,
    /// Actual text string projected into dialogue window UI.
    pub text: String,
    /// Developer trailing strings (`\;`) restored and preserved internally.
    pub comment: String,
    /// Internal tracking modifier block associated to dialog output.
    pub param1: i32,
    /// Logic bound execution parameters appended.
    pub param2: i32,

}

/// Stores translations, text strings, and associated comments used within dialogues.
///
/// Reads file: `NpcInGame/PartyDlg.dlg`
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
            let param2 = parts[3].trim().parse::<i32>().unwrap_or(0);

            texts.push(DialogueText {
                id,
                text,
                comment: current_comment.clone().trim_matches('|').to_string(),
                param1,
                param2,
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
            let text_val = if record.text.is_empty() { "null" } else { &record.text };
            let line = format!("{} | {} | {} | {}\r\n", record.id, text_val, record.param1, record.param2);
            let (cow, _, _) = WINDOWS_1250.encode(&line);
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_dialogue_texts(source_path: &Path) -> std::io::Result<Vec<DialogueText>> {
    DialogueText::read_file(source_path)
}
