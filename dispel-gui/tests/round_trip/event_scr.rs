//! Fixture-based tests for Event Script (.scr) files
//!
//! Tests a representative set of event scripts covering:
//! - Simple teleport scripts (single action)
//! - Conditional scripts with if/else/return()
//! - Complex cutscene scripts with sprite-targeted (~) syntax
//! - Scripts with all 7 sections fully populated
//! - Edge cases: empty sections, return() with no value

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::event_scr::EventScript;
use dispel_core::Extractor;
use std::path::Path;

/// A representative set of fixture files covering the full spectrum of
/// event script features. Revisit this list when adding new fixture files.
const TEST_FIXTURES: &[&str] = &[
    // Simple teleport (event 1)
    "Event0001.scr",
    // Conditional with if/else/return() (event 12)
    "Event0012.scr",
    // Complex cutscene with tilde ~ sprite targeting (event 96)
    "Event0096.scr",
    // Script with return(0) and dialog calls (event 74)
    "Event0074.scr",
    // Script with return() no value (event 1388)
    "Event1388.scr",
    // Script with execextra and extensive VAR section (event 82)
    "Event0082.scr",
    // Low-number event with simple structure (event 41)
    "Event0041.scr",
    // High-number event with dialog() calls and speaker names (event 1411)
    "Event1411.scr",
];

#[test]
fn fixture_event_scr_roundtrip() {
    // Try relative paths until we find the fixture directory.
    // Tests usually run from the crate root (dispel-gui/) or workspace root.
    let candidates = [
        Path::new("fixtures/Dispel/Ref"),
        Path::new("../fixtures/Dispel/Ref"),
    ];

    let fixture_dir = candidates
        .iter()
        .find(|d| d.join(TEST_FIXTURES[0]).exists())
        .expect("fixtures directory not found at any candidate path");

    let mut any_run = false;

    for fixture_name in TEST_FIXTURES {
        let fixture = fixture_dir.join(fixture_name);
        if !fixture.exists() {
            eprintln!("SKIP: fixture not found: {}", fixture.display());
            continue;
        }

        any_run = true;

        round_trip_from_fixture(
            |p| EventScript::read_file(p),
            |records, p| EventScript::save_file(records, p),
            &fixture,
            &format!("EventScript({})", fixture_name),
        )
        .unwrap_or_else(|e| panic!("Round-trip failed for {}: {}", fixture_name, e));
    }

    if !any_run {
        eprintln!("SKIP: no fixture files found (run from workspace root?)");
    }
}
