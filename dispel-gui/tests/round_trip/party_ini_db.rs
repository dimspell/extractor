//! Fixture-based tests for PrtIni.db

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::party_ini_db::PartyIniNpc;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_partyini_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/NpcInGame/PrtIni.db");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| PartyIniNpc::read_file(p),
        |records, p| PartyIniNpc::save_file(records, p),
        fixture,
        "PartyIniNpc",
    )
    .unwrap();
}
