//! Fixture-based tests for Monster.db

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::Extractor;
use dispel_core::references::monster_db::Monster;
use std::path::Path;

#[test]
fn fixture_monster_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/CharacterInGame/monster.db");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(Monster::read_file, Monster::save_file, fixture, "Monster").unwrap();
}
