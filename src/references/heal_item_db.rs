use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::{fs::File, path::Path};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::EUC_KR;
use serde::Serialize;
use encoding_rs::WINDOWS_1250;

use crate::references::references::{read_mapper, read_null_terminated_windows_1250, Extractor};

#[derive(Debug, Serialize)]
pub struct HealItem {
    /// Record index mapping internally.
    pub id: i32,
    /// Fixed array byte name for inventory viewing.
    pub name: String,
    /// Descriptive utility tooltip.
    pub description: String,
    /// Standardized merchant valuation.
    pub base_price: i16,
    pub pz: i16,
    pub pm: i16,
    pub full_pz: u8,
    pub full_pm: u8,
    pub poison_heal: u8,
    pub petrif_heal: u8,
    pub polimorph_heal: u8,

}

/// Stores definitions, stats, and prices for consumable healing items.
///
/// Reads file: `CharacterInGame/HealItem.db`
impl Extractor for HealItem {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
        let file = File::open(source_path)?;

        let metadata = file.metadata()?;
        let file_len = metadata.len();

        let mut reader = BufReader::new(file);

        const COUNTER_SIZE: u8 = 4;
        const PROPERTY_ITEM_SIZE: i32 = 63 * 4;

        let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;
        let mut items: Vec<HealItem> = Vec::with_capacity(elements as usize);

        for i in 0..elements {
            let mut buffer = [0u8; 30];
            reader.read_exact(&mut buffer)?;
            let name = read_null_terminated_windows_1250(&buffer).unwrap();

            let mut buffer = [0u8; 202];
            reader.read_exact(&mut buffer)?;
            let dst = EUC_KR.decode(&buffer);
            let description = dst.0.trim_end_matches("\0").trim();

            let base_price = reader.read_i16::<LittleEndian>()?;

            reader.read_i16::<LittleEndian>()?;
            reader.read_i16::<LittleEndian>()?;
            reader.read_i16::<LittleEndian>()?;

            let pz = reader.read_i16::<LittleEndian>()?;
            let pm = reader.read_i16::<LittleEndian>()?;
            let full_pz = reader.read_u8()?;
            let full_pm = reader.read_u8()?;
            let poison_heal = reader.read_u8()?;
            let petrif_heal = reader.read_u8()?;
            let polimorph_heal = reader.read_u8()?;

            reader.read_u8()?;
            reader.read_i16::<LittleEndian>()?;

            items.push(HealItem {
                id: i,
                name: name.to_string(),
                description: description.to_string(),
                base_price,
                pz,
                pm,
                full_pz,
                full_pm,
                poison_heal,
                petrif_heal,
                polimorph_heal,
            })
        }

        Ok(items)
    }

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let file = File::create(dest_path)?;
        let mut writer = BufWriter::new(file);

        let elements = records.len() as i32;
        writer.write_i32::<LittleEndian>(elements)?;

        for record in records {
            let mut name_buf = [0u8; 30];
            let (cow, _, _) = WINDOWS_1250.encode(&record.name);
            let len = std::cmp::min(cow.len(), 30);
            name_buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&name_buf)?;

            let mut desc_buf = [0u8; 202];
            let (cow, _, _) = EUC_KR.encode(&record.description);
            let len = std::cmp::min(cow.len(), 202);
            desc_buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&desc_buf)?;

            writer.write_i16::<LittleEndian>(record.base_price)?;

            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(0)?;
            writer.write_i16::<LittleEndian>(0)?;

            writer.write_i16::<LittleEndian>(record.pz)?;
            writer.write_i16::<LittleEndian>(record.pm)?;

            writer.write_u8(record.full_pz)?;
            writer.write_u8(record.full_pm)?;
            writer.write_u8(record.poison_heal)?;
            writer.write_u8(record.petrif_heal)?;
            writer.write_u8(record.polimorph_heal)?;

            writer.write_u8(0)?;
            writer.write_i16::<LittleEndian>(0)?;
        }
        Ok(())
    }
}

pub fn read_heal_item_db(source_path: &Path) -> std::io::Result<Vec<HealItem>> {
    HealItem::read_file(source_path)
}
