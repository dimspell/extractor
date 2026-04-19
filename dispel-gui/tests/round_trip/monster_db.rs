//! Fixture-based tests for Monster.db

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::monster_db::Monster;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_monster_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/CharacterInGame/monster.db");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| Monster::read_file(p),
        |records, p| Monster::save_file(records, p),
        fixture,
        "Monster",
    )
    .unwrap();
}
