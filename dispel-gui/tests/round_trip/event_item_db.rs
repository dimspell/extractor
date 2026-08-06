//! Fixture-based tests for EventItem.db

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::Extractor;
use dispel_core::references::event_item_db::EventItem;
use std::path::Path;

#[test]
fn fixture_eventitem_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/CharacterInGame/EventItem.db");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        EventItem::read_file,
        EventItem::save_file,
        fixture,
        "EventItem",
    )
    .unwrap();
}
