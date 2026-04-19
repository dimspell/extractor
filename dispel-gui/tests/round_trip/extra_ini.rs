//! Fixture-based tests for Extra.ini

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::extra_ini::Extra;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_extra_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/Extra.ini");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| Extra::read_file(p),
        |records, p| Extra::save_file(records, p),
        fixture,
        "Extra",
    )
    .unwrap();
}
