use std::{fs::File, path::Path};
use std::io::{Cursor, prelude::*};
use std::io::{BufReader, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};
use encoding_rs::WINDOWS_1250;

use crate::references::references::read_mapper;

#[derive(Debug)]
pub struct Store {
    pub index: i32,
    pub store_name: String,
    pub inn_night_cost: i32,
    pub some_unknown_number: i16,
    pub products: Vec<StoreProduct>,
    pub invitation: String,
    pub haggle_success: String,
    pub haggle_fail: String,
}

pub type StoreProduct = (i16, i16, i16); // order, product_type, product_id

pub fn read_store_db(source_path: &Path) -> std::io::Result<Vec<Store>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    const COUNTER_SIZE: u8 = 4;
    const PROPERTY_ITEM_SIZE: i32 = 237 * 4;
    let elements = read_mapper(&mut reader, file_len, COUNTER_SIZE, PROPERTY_ITEM_SIZE)?;

    let mut store: Vec<Store> = vec![];
    for i in 0..elements as usize {
        // name
        let mut buffer = [0u8; 32];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let name = dst.0.trim_end_matches("\0").trim();

        let inn_night_cost = reader.read_i32::<LittleEndian>()?;
        let mut some_unknown_number = 0;
        let mut products: Vec<StoreProduct> = vec![];

        if inn_night_cost > 0 {
            reader.seek(SeekFrom::Current(144))?;
        } else {
            some_unknown_number = reader.read_i16::<LittleEndian>()?; // price modifier?

            let mut buffer = [0u8; 142];
            reader.read_exact(&mut buffer)?;
            let mut cursor = Cursor::new(&buffer);

            for i in 0..buffer.len() / 2 {
                let item_type = cursor.read_i16::<LittleEndian>().unwrap();
                // 1 = Bron
                // 2,3 = wyposazenie (3 = edibles/ 2 =modfiers?)
                // 4 = magiczny
                if item_type == 0 {
                    break;
                }

                let item_id = cursor.read_i16::<LittleEndian>().unwrap();
                products.push((i as i16, item_type, item_id));
            }
        }

        // text
        let mut buffer = [0u8; 512];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let invitation = dst.0.trim_end_matches("\0").trim();

        // haggle_success
        let mut buffer = [0u8; 128];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let haggle_success = dst.0.trim_end_matches("\0").trim();

        // haggle_fail
        let mut buffer = [0u8; 128];
        reader.read_exact(&mut buffer)?;
        let dst = WINDOWS_1250.decode(&buffer);
        let haggle_fail = dst.0.trim_end_matches("\0").trim();

        let item = Store {
            index: i as i32,
            store_name: name.to_string(),
            inn_night_cost,
            some_unknown_number,
            products,
            invitation: invitation.to_string(),
            haggle_success: haggle_success.to_string(),
            haggle_fail: haggle_fail.to_string(),
        };

        store.push(item);
    }

    Ok(store)
}
