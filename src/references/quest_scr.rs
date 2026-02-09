use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Quest {
    pub id: i32,
    pub type_id: i32, // 0=main, 1=side, 2=traders
    pub title: Option<String>,
    pub description: Option<String>,
}

pub fn read_quests(path: &Path) -> std::io::Result<Vec<Quest>> {
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
