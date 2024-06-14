use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader};

use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DrawItem {
    pub map_id: i32,
    pub x_coord: i32,
    pub y_coord: i32,
    pub item_id: i32, // item id (int32 but [item id, group id, 0 , 0]])
}

pub fn read_draw_items(source_path: &Path) -> std::io::Result<Vec<DrawItem>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(f),
    );
    let mut draw_items: Vec<DrawItem> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }

                let parts: Vec<&str> = line
                    .trim_start_matches("(")
                    .trim_end_matches(")")
                    .split(",")
                    .collect();

                let map_id = parts[0].parse::<i32>().unwrap();
                let x_coord = parts[1].parse::<i32>().unwrap();
                let y_coord = parts[2].parse::<i32>().unwrap();
                let item_id = parts[3].parse::<i32>().unwrap();

                draw_items.push(DrawItem {
                    map_id,
                    x_coord,
                    y_coord,
                    item_id,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(draw_items)
}
