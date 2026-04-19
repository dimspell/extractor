//! Fixture-based tests for MiscItem.db

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::misc_item_db::MiscItem;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_miscitem_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/CharacterInGame/MiscItem.db");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| MiscItem::read_file(p),
        |records, p| MiscItem::save_file(records, p),
        fixture,
        "MiscItem",
    )
    .unwrap();
}
