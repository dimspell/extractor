//! Fixture-based tests for AllMap.ini

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::all_map_ini::Map;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_allmap_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/AllMap.ini");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| Map::read_file(p),
        |records, p| Map::save_file(records, p),
        fixture,
        "AllMap",
    )
    .unwrap();
}
