//! Fixture-based tests for EventItem.db

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::event_item_db::EventItem;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_eventitem_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/CharacterInGame/EventItem.db");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| EventItem::read_file(p),
        |records, p| EventItem::save_file(records, p),
        fixture,
        "EventItem",
    )
    .unwrap();
}
