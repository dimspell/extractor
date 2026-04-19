//! Fixture-based tests for EventNpcRef

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::event_npc_ref::EventNpcRef;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_eventnpc_ref_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/References/Eventnpc.ref");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| EventNpcRef::read_file(p),
        |records, p| EventNpcRef::save_file(records, p),
        fixture,
        "EventNpcRef",
    )
    .unwrap();
}
