//! Fixture-based tests for PrtLevel.db

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::Extractor;
use dispel_core::references::party_level_db::PartyLevelNpc;
use std::path::Path;

#[test]
fn fixture_partylevel_roundtrip() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/Dispel/NpcInGame/PrtLevel.db");
    assert!(fixture.is_file(), "missing fixture: {}", fixture.display());

    round_trip_from_fixture(
        PartyLevelNpc::read_file,
        PartyLevelNpc::save_file,
        &fixture,
        "PartyLevelNpc",
    )
    .unwrap();
}
