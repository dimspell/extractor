use std::io::{BufRead, BufReader, Write};
use std::{fs::File, path::Path};

use crate::references::enums::{DialogOwner, DialogType};
use crate::references::extractor::{parse_int, Extractor};
use encoding_rs::EUC_KR;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

// ===========================================================================
// DLG FILE FORMAT (Dialogue)
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | *.dlg - Dialogue/Conversation Scripts|
// +--------------------------------------+
// | Encoding: EUC-KR                     |
// | Format: CSV with comments            |
// | Record Size: Variable (text)         |
// +--------------------------------------+
// | ; Comment line                       |
// | id,prev_event,next_dlg,type,owner,dlg_id,event_id|
// | 1,100,2,0,1,1001,1000                |
// | 2,101,3,1,0,1002,1001                |
// | ...                                  |
// +--------------------------------------+
//
// FIELD DEFINITIONS:
// - id: Unique dialogue line identifier
// - prev_event: Required event ID to trigger
// - next_dlg: Next dialogue ID in chain
// - type: 0=normal, 1=choice dialog
// - owner: 0=player, 1=NPC
// - dlg_id: Reference to PGP text content
// - event_id: Event triggered by dialogue
//
// DIALOGUE TYPES:
// - 0: Normal dialogue (linear conversation)
// - 1: Choice dialogue (branching options)
//
// DIALOGUE OWNERS:
// - 0: Main character/player speaking
// - 1: NPC character speaking
//
// SPECIAL VALUES:
// - "null" literal for optional fields
// - Lines starting with ";" are comments
// - CSV format with comma delimiter
//
// FILE PURPOSE:
// Defines dialogue scripts with branching conversations, event triggers,
// and text references. Used for NPC interactions, quest dialogues,
// and story progression systems.
//
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Dialog {
    /// Unique dialog script line ID.
    pub id: i32,
    /// Required event to trigger this dialog.
    pub required_event_id: Option<i32>,
    /// Reference link to the next dialog node in the chain.
    pub next_dialog_to_check: Option<i32>,
    /// Type of dialog (normal or choice).
    /// In case of the choice, the choices are in next_dialog_id1, next_dialog_id2, next_dialog_id3 unless they are zero.
    pub dialog_type: Option<DialogType>,
    /// Indicates active speaker (player or NPC).
    pub dialog_owner: Option<DialogOwner>,
    /// Reference ID in the corresponding PGP file (paragraph ID).
    pub dialog_id: Option<i32>,
    /// Next dialog option [1] for choice dialogs or the next dialog in linear type.
    pub next_dialog_id1: Option<i32>,
    /// Next conversation option [2] for choice dialogs.
    pub next_dialog_id2: Option<i32>,
    /// Next conversation option [3] for choice dialogs.
    pub next_dialog_id3: Option<i32>,
    /// Event ID executed upon reading this dialog.
    pub triggered_event_id: Option<i32>,
}

/// Stores dialogues and conversational branches for characters.
///
/// Reads file: `NpcInGame/Dlgcat1.dlg (and other .dlg files)`
/// # File Format: `NpcInGame/Dlgcat1.dlg` (and other map `.dlg` files)
///
/// Text file, WINDOWS-1250 encoded. One record per line, pipe-delimited:
/// ```text
/// id|required_event_id|next_dialog_to_check|dialog_type|dialog_owner|dialog_id|opt0|opt1|opt2|event_id
/// ```
/// - `dialog_type`: 0 = normal, 1 = choose dialog.
/// - `dialog_owner`: 0 = main character talking, 1 = NPC talking.
/// - Optional fields use literal `null` when absent.
impl Extractor for Dialog {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let f = File::open(source_path)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(EUC_KR))
                .build(f),
        );
        let mut dlgs: Vec<Dialog> = Vec::new();
        for line in reader.lines().map_while(Result::ok) {
            let trimmed = line.trim();
            if trimmed.starts_with(';') || trimmed.is_empty() {
                continue;
            }
            let parts: Vec<&str> = trimmed.split(',').collect();
            if parts.len() < 10 {
                continue;
            }

            let id: i32 = parts[0].trim().parse::<i32>().unwrap();
            let required_event_id = parse_int(parts[1].trim());
            let next_dialog_to_check = parse_int(parts[2].trim());
            let dialog_type_id = parse_int(parts[3].trim());
            let dialog_owner_id = parse_int(parts[4].trim());
            let dialog_id = parse_int(parts[5].trim());
            let next_dialog_id1 = parse_int(parts[6].trim());
            let next_dialog_id2 = parse_int(parts[7].trim());
            let next_dialog_id3 = parse_int(parts[8].trim());
            let triggered_event_id = parse_int(parts[9].trim());

            let dialog_type = dialog_type_id.and_then(DialogType::from_i32);
            let dialog_owner = dialog_owner_id.and_then(DialogOwner::from_i32);

            dlgs.push(Dialog {
                id,
                required_event_id,
                next_dialog_to_check,
                dialog_type,
                dialog_owner,
                dialog_id,
                next_dialog_id1,
                next_dialog_id2,
                next_dialog_id3,
                triggered_event_id,
            });
        }
        Ok(dlgs)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            let prev = record
                .required_event_id
                .map_or_else(|| "0".to_string(), |v| v.to_string());
            let next = record
                .next_dialog_to_check
                .map_or_else(|| "0".to_string(), |v| v.to_string());
            let dtype = record
                .dialog_type
                .map_or_else(|| "0".to_string(), |v| v.value().to_string());
            let owner = record
                .dialog_owner
                .map_or_else(|| "0".to_string(), |v| v.value().to_string());
            let did = record
                .dialog_id
                .map_or_else(|| "0".to_string(), |v| v.to_string());
            let nd0 = record
                .next_dialog_id1
                .map_or_else(|| "0".to_string(), |v| v.to_string());
            let nd1 = record
                .next_dialog_id2
                .map_or_else(|| "0".to_string(), |v| v.to_string());
            let nd2 = record
                .next_dialog_id3
                .map_or_else(|| "0".to_string(), |v| v.to_string());
            let triggered_event_id = record
                .triggered_event_id
                .map_or_else(|| "0".to_string(), |v| v.to_string());

            let line = format!(
                "{},{},{},{},{},{},{},{},{},{}\r\n",
                record.id, prev, next, dtype, owner, did, nd0, nd1, nd2, triggered_event_id
            );
            let (cow, _, _) = EUC_KR.encode(&line);
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_dialogs(source_path: &Path) -> std::io::Result<Vec<Dialog>> {
    Dialog::read_file(source_path)
}

pub fn save_dialogs(conn: &mut Connection, dialog_file: &str, dialogs: &[Dialog]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_dialog.sql"))?;
        for dialog in dialogs {
            stmt.execute(params![
                dialog_file,
                dialog.id,
                dialog.required_event_id,
                dialog.next_dialog_to_check,
                dialog.dialog_type.map(|v| v.value()),
                dialog.dialog_owner.map(|v| v.value()),
                dialog.dialog_id,
                dialog.next_dialog_id1,
                dialog.next_dialog_id2,
                dialog.next_dialog_id3,
                dialog.triggered_event_id,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
