use std::io::{BufRead, BufReader, Write};
use std::{fs::File, path::Path};

use crate::references::enums::GhostFaceId;
use crate::references::references::{parse_null, Extractor};
use encoding_rs::WINDOWS_1250;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

// ===========================================================================
// PARTYREF.REF FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | PartyRef.ref - Party Characters     |
// +--------------------------------------+
// | Encoding: WINDOWS-1250              |
// | Format: CSV with comments            |
// | Record Size: Variable (text)        |
// +--------------------------------------+
// | ; Comment line                      |
// | id,name,job,map_id,npc_id,dlg_out,dlg_in,ghost|
// | 1,Hero,null,1,1,100,101,1           |
// | 2,Warrior,Fighter,1,2,102,103,2     |
// | ...                                 |
// +--------------------------------------+
//
// FIELD DEFINITIONS:
// - id: Unique character identifier
// - name: Character display name or "null"
// - job: Character class/job or "null"
// - map_id: Origin map ID where character is found
// - npc_id: Linked NPC record ID
// - dlg_out: Dialog ID when not in party
// - dlg_in: Dialog ID when in party
// - ghost: Ghost face/sprite ID for UI
//
// SPECIAL VALUES:
// - "null" literal for missing name/job fields
// - Lines starting with ";" are comments
// - CSV format with comma delimiter
//
//
// FILE PURPOSE:
// Defines all party characters with their names, classes, origin locations,
// dialog references, and visual representations. Used for party management,
// recruitment, and character interaction systems.
//
// ===========================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct PartyRef {
    /// Party member identifier.
    pub id: i32,
    /// Display name of the party character.
    pub full_name: Option<String>,
    /// Character class or job title.
    pub job_name: Option<String>,
    /// Origin map identifier where the character is found.
    pub root_map_id: i32,
    /// NPC record ID this character is linked to.
    pub npc_id: i32,
    /// Dialog topic when the character is roaming/not recruited.
    pub dlg_when_not_in_party: i32,
    /// Dialog topic when the character is actively grouped.
    pub dlg_when_in_party: i32,
    /// Sprite ID for their UI portrait or ghost form.
    pub ghost_face_id: GhostFaceId,
}

/// Stores character definitions and references for the party.
///
/// Reads file: `Ref/PartyRef.ref`
/// # File Format: `Ref/PartyRef.ref`
///
/// Text file, WINDOWS-1250 encoded. One record per line, CSV format:
/// ```text
/// id,full_name,job_name,root_map_id,npc_id,dlg_not_in_party,dlg_in_party,ghost_face_id
/// ```
/// - `full_name` and `job_name` use literal `null` when absent.
impl Extractor for PartyRef {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let f = File::open(source_path)?;
        let reader = BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1250))
                .build(f),
        );
        let mut party_refs: Vec<PartyRef> = Vec::new();
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.starts_with(";") || trimmed.is_empty() {
                        continue;
                    }

                    let parts: Vec<&str> = trimmed.split(",").collect();
                    if parts.len() < 8 {
                        continue;
                    }

                    let id = parts[0].trim().parse::<i32>().unwrap();
                    let full_name = parse_null(parts[1].trim());
                    let job_name = parse_null(parts[2].trim());
                    let root_map_id = parts[3].trim().parse::<i32>().unwrap();
                    let npc_id = parts[4].trim().parse::<i32>().unwrap();
                    let dlg_when_not_in_party = parts[5].trim().parse::<i32>().unwrap();
                    let dlg_when_in_party = parts[6].trim().parse::<i32>().unwrap();
                    let ghost_face_id_raw = parts[7].trim().parse::<i32>().unwrap();

                    let ghost_face_id =
                        GhostFaceId::from_i32(ghost_face_id_raw).unwrap_or(GhostFaceId::None);

                    party_refs.push(PartyRef {
                        id,
                        full_name,
                        job_name,
                        root_map_id,
                        npc_id,
                        dlg_when_not_in_party,
                        dlg_when_in_party,
                        ghost_face_id,
                    });
                }
                _ => {}
            }
        }
        Ok(party_refs)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let mut file = File::create(dest_path)?;
        for record in records {
            let full_name = record.full_name.as_deref().unwrap_or("null");
            let job_name = record.job_name.as_deref().unwrap_or("null");

            let line = format!(
                "{},{},{},{},{},{},{},{}\r\n",
                record.id,
                full_name,
                job_name,
                record.root_map_id,
                record.npc_id,
                record.dlg_when_not_in_party,
                record.dlg_when_in_party,
                i32::from(record.ghost_face_id)
            );
            let (cow, _, _) = WINDOWS_1250.encode(&line);
            file.write_all(&cow)?;
        }
        Ok(())
    }
}

pub fn read_part_refs(source_path: &Path) -> std::io::Result<Vec<PartyRef>> {
    PartyRef::read_file(source_path)
}

pub fn save_party_refs(conn: &mut Connection, party_refs: &Vec<PartyRef>) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(include_str!("../queries/insert_party_ref.sql"))?;
        for party_ref in party_refs {
            stmt.execute(params![
                party_ref.id,
                party_ref.full_name,
                party_ref.job_name,
                party_ref.root_map_id,
                party_ref.npc_id,
                party_ref.dlg_when_not_in_party,
                party_ref.dlg_when_in_party,
                i32::from(party_ref.ghost_face_id),
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
