//! Fixture-based tests for Monster.ini

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::Extractor;
use dispel_core::references::monster_ini::MonsterIni;
use std::path::Path;

#[test]
fn fixture_monsterini_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/Monster.ini");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        MonsterIni::read_file,
        MonsterIni::save_file,
        fixture,
        "MonsterIni",
    )
    .unwrap();
}
