use std::io::{prelude::*, Cursor};
use std::io::{BufReader, BufWriter, Seek, SeekFrom};
use std::{fs::File, path::Path};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::WINDOWS_1250;
use serde::Serialize;

use crate::references::references::{read_mapper, read_null_terminated_windows_1250, Extractor};
use crate::references::enums::ProductType;

#[derive(Debug, Serialize)]
pub struct Store {
    /// Logical ordering of the store script.
    pub index: i32,
    /// 32-byte shopkeeper title.
    pub store_name: String,
    /// Price to rest; dictates physical structure padding in save format.
    pub inn_night_cost: i32,
    /// Modifies global economy or prices locally.
    pub some_unknown_number: i16,
    /// Ordered list of distinct item parameters available for purchase.
    pub products: Vec<StoreProduct>,
    /// 512-byte text shown on interacting with the merchant.
    pub invitation: String,
    /// 128-byte text shown on successful barter.
    pub haggle_success: String,
    /// 128-byte text shown on rejected transaction.
    pub haggle_fail: String,

}

pub type StoreProduct = (i16, ProductType, i16); // order, product_type, product_id

/// Stores store inventories, inn prices, and merchant dialogue references.
///
/// Reads file: `CharacterInGame/STORE.DB`
/// # File Format: `CharacterInGame/STORE.DB`
///
/// Binary file, little-endian.  Starts with a 4-byte i32 record count.
/// Each record layout (union on `inn_night_cost`):
/// - `name`           : 32 bytes, null-padded, WINDOWS-1250
/// - `inn_night_cost` : i32 — if > 0 this is an inn; the next 144 bytes are padding.
///   Otherwise `some_unknown_number` (i16) + 142 bytes of product list (up to 71
///   pairs of `(item_type: i16, item_id: i16)`, terminated by `item_type == 0`).
/// - `invitation`     : 512 bytes, null-padded, WINDOWS-1250
/// - `haggle_success` : 128 bytes, null-padded, WINDOWS-1250
/// - `haggle_fail`    : 128 bytes, null-padded, WINDOWS-1250
impl Extractor for Store {
    fn read_file(source_path: &Path) -> std::io::Result<Vec<Self>> {
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
            let name = read_null_terminated_windows_1250(&buffer).unwrap();

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
                    let item_type_raw = cursor.read_i16::<LittleEndian>().unwrap();
                    // 1 = Bron
                    // 2,3 = wyposazenie (3 = edibles/ 2 =modfiers?)
                    // 4 = magiczny
                    if item_type_raw == 0 {
                        break;
                    }

                    let product_type = ProductType::from_i32(item_type_raw as i32)
                        .unwrap_or(ProductType::Miscellaneous);
                    let item_id = cursor.read_i16::<LittleEndian>().unwrap();
                    products.push((i as i16, product_type, item_id));
                }
            }

            // text
            let mut buffer = [0u8; 512];
            reader.read_exact(&mut buffer)?;
            let invitation = read_null_terminated_windows_1250(&buffer).unwrap();

            // haggle_success
            let mut buffer = [0u8; 128];
            reader.read_exact(&mut buffer)?;
            let haggle_success = read_null_terminated_windows_1250(&buffer).unwrap();

            // haggle_fail
            let mut buffer = [0u8; 128];
            reader.read_exact(&mut buffer)?;
            let haggle_fail = read_null_terminated_windows_1250(&buffer).unwrap();

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

    fn save_file(records: &[Self], dest_path: &Path) -> std::io::Result<()> {
        let file = File::create(dest_path)?;
        let mut writer = BufWriter::new(file);

        let elements = records.len() as i32;
        writer.write_i32::<LittleEndian>(elements)?;

        for record in records {
            let mut name_buf = [0u8; 32];
            let (cow, _, _) = WINDOWS_1250.encode(&record.store_name);
            let len = std::cmp::min(cow.len(), 32);
            name_buf[..len].copy_from_slice(&cow[..len]);
            writer.write_all(&name_buf)?;

            writer.write_i32::<LittleEndian>(record.inn_night_cost)?;
            if record.inn_night_cost > 0 {
                writer.write_all(&[0u8; 144])?;
            } else {
                writer.write_i16::<LittleEndian>(record.some_unknown_number)?;
                let mut prod_buf = [0u8; 142];
                let mut cursor = Cursor::new(&mut prod_buf[..]);
                for prod in &record.products {
                    cursor.write_i16::<LittleEndian>(i32::from(prod.1) as i16)?; // type
                    cursor.write_i16::<LittleEndian>(prod.2)?; // id
                }
                writer.write_all(&prod_buf)?;
            }

            let write_str =
                |w: &mut BufWriter<File>, text: &str, max_len: usize| -> std::io::Result<()> {
                    let mut buf = vec![0u8; max_len];
                    let (cow, _, _) = WINDOWS_1250.encode(text);
                    let len = std::cmp::min(cow.len(), max_len);
                    buf[..len].copy_from_slice(&cow[..len]);
                    w.write_all(&buf)
                };

            write_str(&mut writer, &record.invitation, 512)?;
            write_str(&mut writer, &record.haggle_success, 128)?;
            write_str(&mut writer, &record.haggle_fail, 128)?;
        }

        Ok(())
    }
}

pub fn read_store_db(source_path: &Path) -> std::io::Result<Vec<Store>> {
    Store::read_file(source_path)
}
