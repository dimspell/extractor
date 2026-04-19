//! Fixture-based tests for Map.ini

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::map_ini::MapIni;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_map_ini_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/Map.ini");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| MapIni::read_file(p),
        |records, p| MapIni::save_file(records, p),
        fixture,
        "MapIni",
    )
    .unwrap();
}
