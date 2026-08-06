//! Fixture-based tests for PartyRef

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::Extractor;
use dispel_core::references::party_ref::PartyRef;
use std::path::Path;

#[test]
fn fixture_partyref_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/References/PartyRef.ref");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        PartyRef::read_file,
        PartyRef::save_file,
        fixture,
        "PartyRef",
    )
    .unwrap();
}
