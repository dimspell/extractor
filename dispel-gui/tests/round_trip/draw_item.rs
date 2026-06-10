//! Fixture-based tests for DrawItem

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::draw_item::DrawItem;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_drawitem_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/References/DrawItem.ref");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        DrawItem::read_file,
        DrawItem::save_file,
        fixture,
        "DrawItem",
    )
    .unwrap();
}
