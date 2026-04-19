//! Fixture-based tests for DialogueText

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::dialogue_text::DialogueText;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_dialogue_text_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/PgpMapFiles.pgp");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| DialogueText::read_file(p),
        |records, p| DialogueText::save_file(records, p),
        fixture,
        "DialogueText",
    )
    .unwrap();
}
