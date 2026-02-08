use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DialogueText {
    pub id: i32,
    pub text: String,
    pub comment: String,
    pub param1: i32,
    pub param2: i32,
}

pub fn read_dialogue_texts(source_path: &Path) -> std::io::Result<Vec<DialogueText>> {
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

        let text = parts[1].to_string();
        let param1 = parts[2].trim().parse::<i32>().unwrap_or(0);
        let param2 = parts[3].trim().parse::<i32>().unwrap_or(0);

        texts.push(DialogueText {
            id,
            text,
            comment: current_comment.clone(),
            param1,
            param2,
        });

        last_was_comment = false;
    }
    Ok(texts)
}
