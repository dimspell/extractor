//! Fixture-based tests for MiscItem.db

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::Extractor;
use dispel_core::references::misc_item_db::MiscItem;
use std::path::Path;

#[test]
fn fixture_miscitem_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/CharacterInGame/MiscItem.db");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        MiscItem::read_file,
        MiscItem::save_file,
        fixture,
        "MiscItem",
    )
    .unwrap();
}
