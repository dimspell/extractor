//! Fixture-based tests for wave.ini

use super::round_trip_utils::round_trip_from_fixture;
use dispel_core::references::wave_ini::WaveIni;
use dispel_core::Extractor;
use std::path::Path;

#[test]
fn fixture_waveini_roundtrip() {
    let fixture = Path::new("fixtures/Dispel/wave.ini");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found: {}", fixture.display());
        return;
    }

    round_trip_from_fixture(
        |p| WaveIni::read_file(p),
        |records, p| WaveIni::save_file(records, p),
        fixture,
        "WaveIni",
    )
    .unwrap();
}
