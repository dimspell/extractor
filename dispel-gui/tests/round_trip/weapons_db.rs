//! Fixture-based tests for WeaponItem.db

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::weapons_db::WeaponItem;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_weaponitem_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/CharacterInGame/weaponItem.db");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| WeaponItem::read_file(p),
        |records, p| WeaponItem::save_file(records, p),
        fixture,
        "WeaponItem",
    )
    .unwrap();
}
