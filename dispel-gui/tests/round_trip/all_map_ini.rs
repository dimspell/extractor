//! Fixture-based tests for AllMap.ini

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::Extractor;
use dispel_core::references::all_map_ini::Map;
use std::path::Path;

#[test]
fn fixture_allmap_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/AllMap.ini");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(Map::read_file, Map::save_file, fixture, "AllMap").unwrap();
}
