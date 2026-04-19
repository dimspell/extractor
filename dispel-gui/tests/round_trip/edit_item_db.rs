//! Fixture-based tests for EditItem.db

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::edit_item_db::EditItem;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_edititem_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/CharacterInGame/EditItem.db");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| EditItem::read_file(p),
        |records, p| EditItem::save_file(records, p),
        fixture,
        "EditItem",
    )
    .unwrap();
}
