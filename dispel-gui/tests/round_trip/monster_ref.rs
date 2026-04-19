//! Fixture-based tests for MonsterRef

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::monster_ref::MonsterRef;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_monsterref_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/References/MonsterRef.ref");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| MonsterRef::read_file(p),
        |records, p| MonsterRef::save_file(records, p),
        fixture,
        "MonsterRef",
    )
    .unwrap();
}
