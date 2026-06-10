//! Fixture-based tests for Npc.ini

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::npc_ini::NpcIni;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_npcini_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/Npc.ini");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        NpcIni::read_file,
        NpcIni::save_file,
        fixture,
        "NpcIni",
    )
    .unwrap();
}
