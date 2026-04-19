//! Fixture-based tests for Message.scr

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::message_scr::Message;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_message_scr_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/Message.scr");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| Message::read_file(p),
        |records, p| Message::save_file(records, p),
        fixture,
        "Message",
    )
    .unwrap();
}
