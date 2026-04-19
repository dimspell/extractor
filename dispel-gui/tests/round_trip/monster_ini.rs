//! Fixture-based tests for Monster.ini

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::monster_ini::MonsterIni;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_monsterini_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/Monster.ini");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| MonsterIni::read_file(p),
        |records, p| MonsterIni::save_file(records, p),
        fixture,
        "MonsterIni",
    )
    .unwrap();
}
