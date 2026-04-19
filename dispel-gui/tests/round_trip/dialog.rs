//! Fixture-based tests for Dialog

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::dialog::Dialog;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_dialog_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/DlgMapFiles.dlg");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| Dialog::read_file(p),
        |records, p| Dialog::save_file(records, p),
        fixture,
        "Dialog",
    )
    .unwrap();
}
