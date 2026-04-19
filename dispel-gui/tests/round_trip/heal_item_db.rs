//! Fixture-based tests for HealItem.db

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::heal_item_db::HealItem;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_healitem_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/CharacterInGame/HealItem.db");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| HealItem::read_file(p),
        |records, p| HealItem::save_file(records, p),
        fixture,
        "HealItem",
    )
    .unwrap();
}
