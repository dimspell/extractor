//! Fixture-based tests for PrtLevel.db

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::party_level_db::PartyLevelNpc;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_partylevel_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/NpcInGame/PrtLevel.db");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| PartyLevelNpc::read_file(p),
        |records, p| PartyLevelNpc::save_file(records, p),
        fixture,
        "PartyLevelNpc",
    )
    .unwrap();
}
