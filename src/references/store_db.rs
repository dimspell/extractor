use std::io::{prelude::*, Cursor};
use std::io::{BufReader, BufWriter, Seek, SeekFrom};
use std::{fs::File, path::Path};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use encoding_rs::WINDOWS_1250;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use crate::references::enums::ProductType;
use crate::references::extractor::{read_mapper, read_null_terminated_windows_1250, Extractor};

// ===========================================================================
// STORE.DB FILE FORMAT
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | STORE.DB - Shop & Inn Database      |
// +--------------------------------------+
// | Encoding: Binary (Little-Endian)     |
// | Text Encoding: WINDOWS-1250          |
// | Header: 4-byte record count          |
// | Record Size: 948 bytes (237 × i32)   |
// +--------------------------------------+
// | [Header]                            |
// | - record_count: i32                  |
// +--------------------------------------+
// | [Record 1]                           |
// | - name: 32 bytes (WINDOWS-1250)     |
// | - inn_night_cost: i32                |
// | - IF inn_night_cost > 0:            |
// |   - 144 bytes padding (inn only)     |
// | - ELSE:                              |
// |   - some_unknown_number: i16         |
// |   - products: 71 × (i16, i16)         |
// | - invitation: 512 bytes (WINDOWS-1250)|
// | - haggle_success: 128 bytes (WINDOWS-1250)|
// | - haggle_fail: 128 bytes (WINDOWS-1250)|
// +--------------------------------------+
// | [Record 2]                           |
// | ... (same structure) ...             |
// +--------------------------------------+
//
// STORE TYPES:
// - inn_night_cost > 0: Inn (no products)
// - inn_night_cost = 0: Shop (with products)
//
// PRODUCT TYPES:
// - 1: Bronze/Weapons
// - 2: Equipment
// - 3: Edibles/Consumables
// - 4: Magical Items
//
// PRODUCT STRUCTURE:
// - (order: i16, type: i16, item_id: i16)
// - Terminated by type = 0
// - Max 71 products per shop
//
// FILE PURPOSE:
// Defines all shops and inns with inventories, prices, dialogue,
// and restocking behavior. Used for economy system, shopping
// interface, and NPC merchant interactions.
//
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
                    if item_type_raw == 0 {
                        break;
                    }

                    let product_type = ProductType::from_i32(item_type_raw as i32)
                        .unwrap_or(ProductType::MiscItem);

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

pub fn save_stores(conn: &mut Connection, stores: &[Store]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt_store = tx.prepare(include_str!("../queries/insert_store.sql"))?;
        let mut stmt_product = tx.prepare(include_str!("../queries/insert_store_product.sql"))?;

        for store in stores {
            stmt_store.execute(params![
                store.index,
                store.store_name,
                store.inn_night_cost,
                store.some_unknown_number,
                store.invitation,
                store.haggle_success,
                store.haggle_fail,
            ])?;

            for product in &store.products {
                stmt_product.execute(params![
                    store.index,
                    product.0,
                    i32::from(product.1) as i16,
                    product.2,
                ])?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}
