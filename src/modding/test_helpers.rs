//! Test-only fixtures shared across modding test modules.

use crate::references::layout::RecordLayout;
use crate::references::misc_item_db::MiscItem;

/// Encode a single `MiscItem` binary record with the given name and base price.
pub fn misc_bytes(name: &str, price: i32) -> Vec<u8> {
    let layout = MiscItem::LAYOUT;
    let mut data = vec![0u8; layout.footprint() as usize];
    data[..layout.header_size as usize].copy_from_slice(&1i32.to_le_bytes());
    let name_def = layout.field("name").unwrap();
    let start = layout.header_size as usize + name_def.offset as usize;
    let n = name.len().min(name_def.size as usize - 1);
    data[start..start + n].copy_from_slice(&name.as_bytes()[..n]);
    let price_def = layout.field("base_price").unwrap();
    let start = layout.header_size as usize + price_def.offset as usize;
    data[start..start + price_def.size as usize].copy_from_slice(&price.to_le_bytes());
    data
}
