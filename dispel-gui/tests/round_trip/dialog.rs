//! Fixture-based tests for Dialog

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::dialogue_script::DialogueScript;
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
        DialogueScript::read_file,
        DialogueScript::save_file,
        fixture,
        "Dialog",
    )
    .unwrap();
}
