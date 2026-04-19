//! Fixture-based tests for MagicSpell.db

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::magic_db::MagicSpell;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_magicspell_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/CharacterInGame/magicSpell.db");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| MagicSpell::read_file(p),
        |records, p| MagicSpell::save_file(records, p),
        fixture,
        "MagicSpell",
    )
    .unwrap();
}
