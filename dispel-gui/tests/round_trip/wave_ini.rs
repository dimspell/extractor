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

    round_trip_from_fixture(WaveIni::read_file, WaveIni::save_file, fixture, "WaveIni").unwrap();
}
