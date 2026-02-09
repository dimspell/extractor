use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Message {
    pub id: i32,
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub line3: Option<String>,
}

pub fn read_messages(path: &Path) -> std::io::Result<Vec<Message>> {
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
