use std::{fs::File, path::Path};
use std::io::{BufRead, BufReader};

use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::{Deserialize, Serialize};
use crate::references::references::parse_int;

#[derive(Debug, Serialize, Deserialize)]
pub struct Dialog {
    // Dialogs on map (translated from Korean)
    //
    // id,
    // previous event,
    // next dialog to check,
    // dialog type 0=normal 1=choose dialog
    // dialog topic(who is talking?) 0=main character 1=NPC
    // dialog id (ID in PGP file),
    // option 0 (dialog id),
    // option 1,
    // option 2,
    // event id to execute
    pub id: i32,
    pub previous_event_id: Option<i32>,
    pub next_dialog_to_check: Option<i32>,
    pub dialog_type_id: Option<i32>,
    pub dialog_owner: Option<i32>,
    pub dialog_id: Option<i32>,
    pub event_id: Option<i32>,
}

pub fn read_dialogs(source_path: &Path) -> std::io::Result<Vec<Dialog>> {
    let f = File::open(source_path)?;
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(EUC_KR))
            .build(f),
    );
    let mut dlgs: Vec<Dialog> = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(";") {
                    continue;
                }
                let parts: Vec<&str> = line.split(",").collect();

                let id: i32 = parts[0].parse::<i32>().unwrap();
                let previous_event_id = parse_int(parts[1]);
                let next_dialog_to_check = parse_int(parts[2]);
                let dialog_type_id = parse_int(parts[3]);
                let dialog_owner = parse_int(parts[4]);
                let dialog_id = parse_int(parts[5]);
                let event_id = parse_int(parts[6]);

                dlgs.push(Dialog {
                    id,
                    previous_event_id,
                    next_dialog_to_check,
                    dialog_type_id,
                    dialog_owner,
                    dialog_id,
                    event_id,
                });
            }
            _ => {
                println!("{:?}", line);
            }
        }
    }
    Ok(dlgs)
}
