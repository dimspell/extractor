//! Fixture-based tests for Store.db

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::store_db::Store;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_store_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/CharacterInGame/Store.db");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| Store::read_file(p),
        |records, p| Store::save_file(records, p),
        fixture,
        "Store",
    )
    .unwrap();
}
