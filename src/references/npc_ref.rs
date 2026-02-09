use std::io::prelude::*;
use std::io::BufReader;
use std::{fs::File, path::Path};

use crate::references::references::{read_mapper, read_null_terminated_windows_1250};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct NPC {
    pub index: i32,
    pub id: i32,
    pub npc_id: i32,
    pub name: String,
    pub party_script_id: i32,
    pub show_on_event: i32,
    pub goto1_filled: i32,
    pub goto2_filled: i32,
    pub goto3_filled: i32,
    pub goto4_filled: i32,
    pub goto1_x: i32,
    pub goto2_x: i32,
    pub goto3_x: i32,
    pub goto4_x: i32,
    pub goto1_y: i32,
    pub goto2_y: i32,
    pub goto3_y: i32,
    pub goto4_y: i32,
    pub looking_direction: i32,
    pub dialog_id: i32,
}

//     todo!(); // Eventnpc.ref

//     // NPCs used only in events
//     //
//     // id
//     // sprite id
//     //     ?
//     //     ?
//     //     ?
//     //     ?
//     // x coordinate,
//     // y coordinate,
//     // 30 times ?

//     todo!(); // Eventnpc.ref

// fixtures/Dispel/NpcInGame/Npccat1.ref fixtures/Dispel/NpcInGame/Npccat2.ref fixtures/Dispel/NpcInGame/Npccat3.ref fixtures/Dispel/NpcInGame/Npccatp.ref fixtures/Dispel/NpcInGame/npcdun08.ref fixtures/Dispel/NpcInGame/npcdun19.ref fixtures/Dispel/NpcInGame/Npcmap1.ref fixtures/Dispel/NpcInGame/Npcmap2.ref fixtures/Dispel/NpcInGame/Npcmap3.ref

pub fn read_npc_ref(source_path: &Path) -> std::io::Result<Vec<NPC>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 0x2a0;
    // const FILLER: u8 = 205; // 0xCD
    const STRING_MAX_LENGTH: usize = 260;

    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
    let mut npcs: Vec<NPC> = Vec::with_capacity(elements as usize);

    for i in 0..elements {
        let id = reader.read_i32::<LittleEndian>()?;
        let npc_id = reader.read_i32::<LittleEndian>()?;

        let mut buffer = [0u8; STRING_MAX_LENGTH];
        reader.read_exact(&mut buffer)?;
        let name = read_null_terminated_windows_1250(&buffer).unwrap();

        let mut buffer = [0u8; STRING_MAX_LENGTH];
        reader.read_exact(&mut buffer)?;
        // let dst = WINDOWS_1250.decode(&buffer);
        // let test = dst.0.trim_end_matches("\0").trim();

        let party_script_id = reader.read_i32::<LittleEndian>()?;
        let show_on_event = reader.read_i32::<LittleEndian>()?;

        let mut buffer = [0u8; 4];
        reader.read_exact(&mut buffer)?;
        // let cursor = Cursor::new(&buffer);
        // println!("4: {:?}", cursor);

        let goto1_filled = reader.read_i32::<LittleEndian>()?;
        let goto2_filled = reader.read_i32::<LittleEndian>()?;
        let goto3_filled = reader.read_i32::<LittleEndian>()?;
        let goto4_filled = reader.read_i32::<LittleEndian>()?; // "when goto4 not filled its 1, idk why

        let goto1_x = reader.read_i32::<LittleEndian>()?;
        let goto2_x = reader.read_i32::<LittleEndian>()?;
        let goto3_x = reader.read_i32::<LittleEndian>()?;
        let goto4_x = reader.read_i32::<LittleEndian>()?;

        let goto1_y = reader.read_i32::<LittleEndian>()?;
        let goto2_y = reader.read_i32::<LittleEndian>()?;
        let goto3_y = reader.read_i32::<LittleEndian>()?;
        let goto4_y = reader.read_i32::<LittleEndian>()?;

        let mut buffer = [0u8; 16];
        reader.read_exact(&mut buffer)?;
        // let cursor = Cursor::new(&buffer);
        // println!("16: {:?}", cursor);

        let looking_direction = reader.read_i32::<LittleEndian>()?; // 0 = up, clockwise

        let mut buffer = [0u8; 16 + 16 + 16 + 8];
        reader.read_exact(&mut buffer)?;
        // let cursor = Cursor::new(&buffer);
        // println!("56: {:?}", cursor);

        let dialog_id = reader.read_i32::<LittleEndian>()?; // also text for shop

        let mut buffer = [0u8; 4];
        reader.read_exact(&mut buffer)?;
        // let cursor = Cursor::new(&buffer);
        // println!("Last: {:?}", cursor);

        npcs.push(NPC {
            index: i,
            id,
            npc_id,
            name: name.to_string(),
            party_script_id,
            show_on_event,
            goto1_filled,
            goto2_filled,
            goto3_filled,
            goto4_filled,
            goto1_x,
            goto2_x,
            goto3_x,
            goto4_x,
            goto1_y,
            goto2_y,
            goto3_y,
            goto4_y,
            looking_direction,
            dialog_id,
        })
    }

    Ok(npcs)
}
