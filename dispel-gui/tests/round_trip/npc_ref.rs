//! Fixture-based tests for NPCRef

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::npc_ref::NPC;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_npcref_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/References/NpcRef.ref");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        NPC::read_file,
        NPC::save_file,
        fixture,
        "NPC",
    )
    .unwrap();
}
