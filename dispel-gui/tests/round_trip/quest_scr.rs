//! Fixture-based tests for Quest.scr

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::quest_scr::Quest;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_quest_scr_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/Quest.scr");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| Quest::read_file(p),
        |records, p| Quest::save_file(records, p),
        fixture,
        "Quest",
    )
    .unwrap();
}
