//! Fixture-based tests for ChData.db

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::chdata_db::ChData;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_chdata_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/CharacterInGame/ChData.db");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| ChData::read_file(p),
        |records, p| ChData::save_file(records, p),
        fixture,
        "ChData",
    )
    .unwrap();
}
