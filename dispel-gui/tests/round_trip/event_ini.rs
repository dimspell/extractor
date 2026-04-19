//! Fixture-based tests for Event.ini

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::event_ini::Event;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_event_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/Event.ini");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| Event::read_file(p),
        |records, p| Event::save_file(records, p),
        fixture,
        "Event",
    )
    .unwrap();
}
