//! Fixture-based tests for ExtraRef

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::extra_ref::ExtraRef;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_extraref_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/References/ExtraRef.ref");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| ExtraRef::read_file(p),
        |records, p| ExtraRef::save_file(records, p),
        fixture,
        "ExtraRef",
    )
    .unwrap();
}
