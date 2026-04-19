//! Fixture-based tests for PartyRef

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::party_ref::PartyRef;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_partyref_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/References/PartyRef.ref");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| PartyRef::read_file(p),
        |records, p| PartyRef::save_file(records, p),
        fixture,
        "PartyRef",
    )
    .unwrap();
}
