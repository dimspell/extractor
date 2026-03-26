use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::{fs::File, path::Path};

use crate::references::references::{read_mapper, read_null_terminated_windows_1250, Extractor};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::WINDOWS_1250;
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

impl Extractor for NPC {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(source_path)?;

        let metadata = file.metadata()?;
        let file_len = metadata.len();

        let mut reader = BufReader::new(file);

        const COUNTER_SIZE: u8 = 4;
        const PROPERTY_ITEM_SIZE: i32 = 0x2a0; // 672
        const STRING_MAX_LENGTH: usize = 260;

        let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
        let mut npcs: Vec<NPC> = Vec::with_capacity(elements as usize);

        for i in 0..elements {
            let id = reader.read_i32::<LittleEndian>()?;
            let npc_id = reader.read_i32::<LittleEndian>()?;

            let mut buffer = [0u8; STRING_MAX_LENGTH];
            reader.read_exact(&mut buffer)?;
            let name = read_null_terminated_windows_1250(&buffer).unwrap();

            let mut buffer_ignored = [0u8; STRING_MAX_LENGTH];
            reader.read_exact(&mut buffer_ignored)?;

            let party_script_id = reader.read_i32::<LittleEndian>()?;
            let show_on_event = reader.read_i32::<LittleEndian>()?;

            let mut buffer_4 = [0u8; 4];
            reader.read_exact(&mut buffer_4)?;

            let goto1_filled = reader.read_i32::<LittleEndian>()?;
            let goto2_filled = reader.read_i32::<LittleEndian>()?;
            let goto3_filled = reader.read_i32::<LittleEndian>()?;
            let goto4_filled = reader.read_i32::<LittleEndian>()?;

            let goto1_x = reader.read_i32::<LittleEndian>()?;
            let goto2_x = reader.read_i32::<LittleEndian>()?;
            let goto3_x = reader.read_i32::<LittleEndian>()?;
            let goto4_x = reader.read_i32::<LittleEndian>()?;

            let goto1_y = reader.read_i32::<LittleEndian>()?;
            let goto2_y = reader.read_i32::<LittleEndian>()?;
            let goto3_y = reader.read_i32::<LittleEndian>()?;
            let goto4_y = reader.read_i32::<LittleEndian>()?;

            let mut buffer_16 = [0u8; 16];
            reader.read_exact(&mut buffer_16)?;

            let looking_direction = reader.read_i32::<LittleEndian>()?; // 0 = up, clockwise

            let mut buffer_56 = [0u8; 16 + 16 + 16 + 8];
            reader.read_exact(&mut buffer_56)?;

            let dialog_id = reader.read_i32::<LittleEndian>()?; // also text for shop

            let mut buffer_last = [0u8; 4];
            reader.read_exact(&mut buffer_last)?;

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

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let file = File::create(dest_path)?;
        let mut writer = BufWriter::new(file);

        let elements = records.len() as i32;
        writer.write_i32::<LittleEndian>(elements)?;

        for record in records {
            writer.write_i32::<LittleEndian>(record.id)?;
            writer.write_i32::<LittleEndian>(record.npc_id)?;

            let mut name_buf = [0u8; 260];
            let (cow, _, _) = WINDOWS_1250.encode(&record.name);
            let len = std::cmp::min(cow.len(), 260);
            name_buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&name_buf)?;

            writer.write_all(&[0u8; 260])?; // ignored string

            writer.write_i32::<LittleEndian>(record.party_script_id)?;
            writer.write_i32::<LittleEndian>(record.show_on_event)?;

            writer.write_all(&[0u8; 4])?;

            writer.write_i32::<LittleEndian>(record.goto1_filled)?;
            writer.write_i32::<LittleEndian>(record.goto2_filled)?;
            writer.write_i32::<LittleEndian>(record.goto3_filled)?;
            writer.write_i32::<LittleEndian>(record.goto4_filled)?;

            writer.write_i32::<LittleEndian>(record.goto1_x)?;
            writer.write_i32::<LittleEndian>(record.goto2_x)?;
            writer.write_i32::<LittleEndian>(record.goto3_x)?;
            writer.write_i32::<LittleEndian>(record.goto4_x)?;

            writer.write_i32::<LittleEndian>(record.goto1_y)?;
            writer.write_i32::<LittleEndian>(record.goto2_y)?;
            writer.write_i32::<LittleEndian>(record.goto3_y)?;
            writer.write_i32::<LittleEndian>(record.goto4_y)?;

            writer.write_all(&[0u8; 16])?;

            writer.write_i32::<LittleEndian>(record.looking_direction)?;

            writer.write_all(&[0u8; 56])?;

            writer.write_i32::<LittleEndian>(record.dialog_id)?;

            writer.write_all(&[0u8; 4])?;
        }

        Ok(())
    }
}

pub fn read_npc_ref(source_path: &Path) -> std::io::Result<Vec<NPC>> {
    NPC::read_file(source_path)
}
